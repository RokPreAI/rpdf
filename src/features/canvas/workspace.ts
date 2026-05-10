import type {
  CanvasBackgroundPattern,
  CanvasDocument,
  CanvasImagePlacementDocument,
  WorkspaceController,
  WorkspaceDocumentSnapshot,
} from "../../app/types";

type Point = {
  x: number;
  y: number;
};

type StrokePoint = Point & {
  pressure: number;
};

type Stroke = {
  points: StrokePoint[];
  color: string;
  baseWidth: number;
};

type Camera = {
  x: number;
  y: number;
  scale: number;
};

type CanvasImage = {
  id: string;
  assetPath: string;
  image: HTMLImageElement;
  x: number;
  y: number;
  width: number;
  height: number;
};

type SelectionTarget =
  | {
    kind: "stroke";
    index: number;
  }
  | {
    kind: "image";
    index: number;
  };

type Tool = "pen" | "select" | "pan" | "eraser";
type BackgroundPattern = "dotted" | "vlines" | "hlines" | "grid" | "none";

export function mountCanvasWorkspace(container: HTMLElement): WorkspaceController {
  container.innerHTML = `
    <div class="canvas-workspace">
      <canvas class="canvas-surface"></canvas>

      <div class="canvas-toolbar">
        Left drag: draw/select | Right/Middle/Space drag: pan | Wheel: zoom | Ctrl+Z: undo | C: clear
        <div id="selected-tool"></div>
        <div id="selected-color"></div>
      </div>

      <div class="canvas-settings">
        <label class="stroke-width-control" for="stroke-width">
          <span>Stroke width</span>
          <input id="stroke-width" type="range" min="1" max="24" step="1" value="3" />
          <output id="stroke-width-value">3px</output>
        </label>
      </div>

      <div class="canvas-pickers">
        <button class="tool-picker active" data-tool="pen" type="button">P</button>
        <button class="tool-picker" data-tool="select" type="button">S</button>
        <button class="tool-picker" data-tool="pan" type="button">M</button>
        <button class="tool-picker" data-tool="eraser" type="button">E</button>

        <button id="color-picker-fg" class="color-picker active" type="button"></button>
        <button id="color-picker-blue" class="color-picker" type="button"></button>
        <button id="color-picker-cyan" class="color-picker" type="button"></button>
        <button id="color-picker-green" class="color-picker" type="button"></button>
        <button id="color-picker-yellow" class="color-picker" type="button"></button>
        <button id="color-picker-orange" class="color-picker" type="button"></button>
        <button id="color-picker-red" class="color-picker" type="button"></button>
        <button id="color-picker-magenta" class="color-picker" type="button"></button>
        <button id="color-picker-purple" class="color-picker" type="button"></button>
      </div>
    </div>
  `;

  const canvas = requireElement<HTMLCanvasElement>(container, ".canvas-surface");

  const context = canvas.getContext("2d");

  if (!context) {
    throw new Error("Could not get 2D canvas context.");
  }

  const ctx = context;
  const strokes: Stroke[] = [];
  const images: CanvasImage[] = [];
  const backgroundColor = "#1a1b26";
  const gridColor = "#292e42";
  const backgroundPattern: BackgroundPattern = "dotted";
  let documentId: string = crypto.randomUUID();

  const camera: Camera = {
    x: container.clientWidth / 2,
    y: container.clientHeight / 2,
    scale: 1,
  };

  let selectedTool: Tool = "pen";
  let strokeColor = readCssVariable("--fg") || "#c0caf5";
  let currentStroke: Stroke | null = null;
  let selectedItem: SelectionTarget | null = null;
  let moveAnchorPoint: Point | null = null;
  let isPanning = false;
  let isSpaceDown = false;
  let devicePixelRatioValue = window.devicePixelRatio || 1;
  let baseStrokeWidth = 3;

  const colorVariableByButtonId: Record<string, string> = {
    "color-picker-fg": "--fg",
    "color-picker-blue": "--blue",
    "color-picker-cyan": "--cyan",
    "color-picker-green": "--green",
    "color-picker-yellow": "--yellow",
    "color-picker-orange": "--orange",
    "color-picker-red": "--red",
    "color-picker-magenta": "--magenta",
    "color-picker-purple": "--purple",
  };

  function readCssVariable(name: string) {
    return getComputedStyle(document.documentElement)
      .getPropertyValue(name)
      .trim();
  }

  function getCanvasSize() {
    return {
      width: canvas.width / devicePixelRatioValue,
      height: canvas.height / devicePixelRatioValue,
    };
  }

  function screenToWorld(clientX: number, clientY: number): Point {
    const rect = canvas.getBoundingClientRect();
    const screenX = clientX - rect.left;
    const screenY = clientY - rect.top;

    return {
      x: (screenX - camera.x) / camera.scale,
      y: (screenY - camera.y) / camera.scale,
    };
  }

  function screenPointToWorld(screenX: number, screenY: number): Point {
    return {
      x: (screenX - camera.x) / camera.scale,
      y: (screenY - camera.y) / camera.scale,
    };
  }

  function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
  }

  function updateCursor() {
    if (isSpaceDown) {
      canvas.style.cursor = "grab";
      return;
    }

    if (selectedTool === "pen") {
      canvas.style.cursor = "crosshair";
    } else if (selectedTool === "select") {
      canvas.style.cursor = selectedItem ? "move" : "default";
    } else if (selectedTool === "pan") {
      canvas.style.cursor = "grab";
    } else {
      canvas.style.cursor = "cell";
    }
  }

  function updateToolIndicator(tool: Tool) {
    const toolIndicator = container.querySelector<HTMLElement>("#selected-tool");

    if (toolIndicator) {
      toolIndicator.textContent = tool;
    }
  }

  function updateColorIndicator(color: string) {
    const colorIndicator = container.querySelector<HTMLElement>("#selected-color");

    if (colorIndicator) {
      colorIndicator.textContent = color;
    }
  }

  function setActiveToolButton(toolButtons: NodeListOf<HTMLButtonElement>, tool: Tool) {
    for (const button of toolButtons) {
      button.classList.toggle("active", button.dataset.tool === tool);
    }
  }

  function setupPickers() {
    const colorButtons = container.querySelectorAll<HTMLButtonElement>(".color-picker");
    const toolButtons = container.querySelectorAll<HTMLButtonElement>(".tool-picker");
    const strokeWidthInput = requireElement<HTMLInputElement>(container, "#stroke-width");
    const strokeWidthValue = requireElement<HTMLOutputElement>(container, "#stroke-width-value");

    for (const button of colorButtons) {
      button.addEventListener("pointerdown", (event) => {
        event.stopPropagation();
      });

      button.addEventListener("click", (event) => {
        event.stopPropagation();

        const cssVariable = colorVariableByButtonId[button.id];

        if (!cssVariable) {
          return;
        }

        strokeColor = readCssVariable(cssVariable);
        selectedTool = "pen";

        updateColorIndicator(strokeColor);
        updateToolIndicator(selectedTool);
        setActiveToolButton(toolButtons, selectedTool);

        for (const colorButton of colorButtons) {
          colorButton.classList.toggle("active", colorButton === button);
        }

        updateCursor();
      });
    }

    for (const button of toolButtons) {
      button.addEventListener("pointerdown", (event) => {
        event.stopPropagation();
      });

      button.addEventListener("click", (event) => {
        event.stopPropagation();

        const tool = button.dataset.tool;

        if (tool !== "pen" && tool !== "select" && tool !== "pan" && tool !== "eraser") {
          return;
        }

        selectedTool = tool;
        updateToolIndicator(selectedTool);
        setActiveToolButton(toolButtons, selectedTool);
        updateCursor();
      });
    }

    updateToolIndicator(selectedTool);
    updateColorIndicator(strokeColor);
    setActiveToolButton(toolButtons, selectedTool);
    updateCursor();

    strokeWidthInput.addEventListener("input", () => {
      baseStrokeWidth = Number(strokeWidthInput.value);
      strokeWidthValue.textContent = `${baseStrokeWidth}px`;
    });
  }

  function resizeCanvas() {
    devicePixelRatioValue = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();

    canvas.width = Math.floor(rect.width * devicePixelRatioValue);
    canvas.height = Math.floor(rect.height * devicePixelRatioValue);

    redraw();
  }

  function drawDottedGrid(startX: number, startY: number, right: number, bottom: number, gridStep: number) {
    const columnCount = Math.floor((right - startX) / gridStep) + 1;
    const rowCount = Math.floor((bottom - startY) / gridStep) + 1;
    const dotCount = columnCount * rowCount;

    if (dotCount > 5000) {
      return;
    }

    const dotSize = 4 / camera.scale;
    ctx.fillStyle = gridColor;

    for (let x = startX; x <= right; x += gridStep) {
      for (let y = startY; y <= bottom; y += gridStep) {
        ctx.fillRect(x - dotSize / 2, y - dotSize / 2, dotSize, dotSize);
      }
    }
  }

  function drawLineGrid(
    startX: number,
    startY: number,
    left: number,
    top: number,
    right: number,
    bottom: number,
    gridStep: number,
  ) {
    const verticalLineCount = Math.floor((right - startX) / gridStep) + 1;
    const horizontalLineCount = Math.floor((bottom - startY) / gridStep) + 1;

    if (verticalLineCount + horizontalLineCount > 1000) {
      return;
    }

    ctx.beginPath();
    ctx.lineWidth = 1 / camera.scale;
    ctx.strokeStyle = gridColor;

    if (backgroundPattern === "vlines" || backgroundPattern === "grid") {
      for (let x = startX; x <= right; x += gridStep) {
        ctx.moveTo(x, top);
        ctx.lineTo(x, bottom);
      }
    }

    if (backgroundPattern === "hlines" || backgroundPattern === "grid") {
      for (let y = startY; y <= bottom; y += gridStep) {
        ctx.moveTo(left, y);
        ctx.lineTo(right, y);
      }
    }

    ctx.stroke();
  }

  function drawGrid() {
    if (backgroundPattern === "none") {
      return;
    }

    const { width, height } = getCanvasSize();
    const left = -camera.x / camera.scale;
    const top = -camera.y / camera.scale;
    const right = (width - camera.x) / camera.scale;
    const bottom = (height - camera.y) / camera.scale;
    const gridStep = 50;
    const startX = Math.floor(left / gridStep) * gridStep;
    const startY = Math.floor(top / gridStep) * gridStep;

    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);

    if (backgroundPattern === "dotted") {
      drawDottedGrid(startX, startY, right, bottom, gridStep);
    } else {
      drawLineGrid(startX, startY, left, top, right, bottom, gridStep);
    }

    ctx.restore();
  }

  function drawStroke(stroke: Stroke) {
    if (stroke.points.length === 0) {
      return;
    }

    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);
    ctx.strokeStyle = stroke.color;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    if (stroke.points.length === 1) {
      const point = stroke.points[0];
      const radius = calculateRenderedStrokeWidth(stroke.baseWidth, point.pressure) / 2;

      ctx.beginPath();
      ctx.fillStyle = stroke.color;
      ctx.arc(point.x, point.y, radius, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
      return;
    }

    for (let index = 1; index < stroke.points.length; index += 1) {
      const previousPoint = stroke.points[index - 1];
      const currentPoint = stroke.points[index];
      const segmentPressure = (previousPoint.pressure + currentPoint.pressure) / 2;

      ctx.beginPath();
      ctx.lineWidth = calculateRenderedStrokeWidth(stroke.baseWidth, segmentPressure) / camera.scale;
      ctx.moveTo(previousPoint.x, previousPoint.y);
      ctx.lineTo(currentPoint.x, currentPoint.y);
      ctx.stroke();
    }

    ctx.restore();
  }

  function calculateRenderedStrokeWidth(strokeWidth: number, pressure: number) {
    return Math.max(1, strokeWidth * pressure);
  }

  function drawImages() {
    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);

    for (const canvasImage of images) {
      ctx.drawImage(
        canvasImage.image,
        canvasImage.x,
        canvasImage.y,
        canvasImage.width,
        canvasImage.height,
      );
    }

    ctx.restore();
  }

  function drawSelectionOverlay() {
    if (!selectedItem) {
      return;
    }

    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);
    ctx.setLineDash([10 / camera.scale, 8 / camera.scale]);
    ctx.lineWidth = 2 / camera.scale;
    ctx.strokeStyle = readCssVariable("--yellow") || "#e0af68";

    if (selectedItem.kind === "image") {
      const image = images[selectedItem.index];

      if (image) {
        ctx.strokeRect(image.x, image.y, image.width, image.height);
      }
    } else {
      const stroke = strokes[selectedItem.index];

      if (stroke) {
        const bounds = getStrokeBounds(stroke);

        if (bounds) {
          ctx.strokeRect(bounds.x, bounds.y, bounds.width, bounds.height);
        }
      }
    }

    ctx.restore();
  }

  function redraw() {
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = backgroundColor;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    drawGrid();
    drawImages();

    for (const stroke of strokes) {
      drawStroke(stroke);
    }

    drawSelectionOverlay();
  }

  function eraseAtPoint(point: Point) {
    const eraserRadius = 12 / camera.scale;

    for (let strokeIndex = strokes.length - 1; strokeIndex >= 0; strokeIndex -= 1) {
      const stroke = strokes[strokeIndex];

      if (stroke.points.some((strokePoint) => distanceBetweenPoints(point, strokePoint) <= eraserRadius)) {
        strokes.splice(strokeIndex, 1);
      }
    }
  }

  function distanceBetweenPoints(firstPoint: Point, secondPoint: Point) {
    return Math.hypot(firstPoint.x - secondPoint.x, firstPoint.y - secondPoint.y);
  }

  function getStrokeBounds(stroke: Stroke) {
    if (stroke.points.length === 0) {
      return null;
    }

    let minX = stroke.points[0].x;
    let minY = stroke.points[0].y;
    let maxX = stroke.points[0].x;
    let maxY = stroke.points[0].y;

    for (const point of stroke.points) {
      minX = Math.min(minX, point.x);
      minY = Math.min(minY, point.y);
      maxX = Math.max(maxX, point.x);
      maxY = Math.max(maxY, point.y);
    }

    const padding = stroke.baseWidth;

    return {
      x: minX - padding,
      y: minY - padding,
      width: Math.max(1, maxX - minX + padding * 2),
      height: Math.max(1, maxY - minY + padding * 2),
    };
  }

  function hitTestSelection(point: Point): SelectionTarget | null {
    for (let imageIndex = images.length - 1; imageIndex >= 0; imageIndex -= 1) {
      const image = images[imageIndex];

      if (
        point.x >= image.x
        && point.x <= image.x + image.width
        && point.y >= image.y
        && point.y <= image.y + image.height
      ) {
        return {
          kind: "image",
          index: imageIndex,
        };
      }
    }

    for (let strokeIndex = strokes.length - 1; strokeIndex >= 0; strokeIndex -= 1) {
      const stroke = strokes[strokeIndex];
      const bounds = getStrokeBounds(stroke);

      if (!bounds) {
        continue;
      }

      const withinBounds = point.x >= bounds.x
        && point.x <= bounds.x + bounds.width
        && point.y >= bounds.y
        && point.y <= bounds.y + bounds.height;

      if (!withinBounds) {
        continue;
      }

      if (stroke.points.some((strokePoint) => distanceBetweenPoints(point, strokePoint) <= stroke.baseWidth * 2)) {
        return {
          kind: "stroke",
          index: strokeIndex,
        };
      }
    }

    return null;
  }

  function moveSelectedItem(deltaX: number, deltaY: number) {
    if (!selectedItem) {
      return;
    }

    if (selectedItem.kind === "image") {
      const image = images[selectedItem.index];

      if (!image) {
        return;
      }

      image.x += deltaX;
      image.y += deltaY;
      return;
    }

    const stroke = strokes[selectedItem.index];

    if (!stroke) {
      return;
    }

    for (const point of stroke.points) {
      point.x += deltaX;
      point.y += deltaY;
    }
  }

  function addLoadedImage(image: HTMLImageElement, id: string, assetPath: string) {
    const { width, height } = getCanvasSize();
    const centerWorld = screenPointToWorld(width / 2, height / 2);
    const maxImageScreenWidth = width * 0.5;
    const scale = Math.min(1, maxImageScreenWidth / image.naturalWidth);
    const imageWidth = (image.naturalWidth * scale) / camera.scale;
    const imageHeight = (image.naturalHeight * scale) / camera.scale;

    images.push({
      id,
      assetPath,
      image,
      x: centerWorld.x - imageWidth / 2,
      y: centerWorld.y - imageHeight / 2,
      width: imageWidth,
      height: imageHeight,
    });

    redraw();
  }

  function readFileAsDataUrl(file: File) {
    return new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        if (typeof reader.result === "string") {
          resolve(reader.result);
          return;
        }

        reject(new Error("Could not read pasted image as a data URL."));
      };
      reader.onerror = () => {
        reject(reader.error ?? new Error("Failed to read pasted image file."));
      };
      reader.readAsDataURL(file);
    });
  }

  function loadImageFromSource(source: string) {
    return new Promise<HTMLImageElement>((resolve, reject) => {
      const image = new Image();

      image.onload = () => {
        resolve(image);
      };

      image.onerror = () => {
        reject(new Error(`Failed to load image source: ${source}`));
      };

      image.src = source;
    });
  }

  async function addImageFromFile(file: File) {
    try {
      const source = await readFileAsDataUrl(file);
      const image = await loadImageFromSource(source);
      addLoadedImage(image, file.name || crypto.randomUUID(), source);
    } catch (error) {
      console.error("Failed to load pasted image file:", file, error);
    }
  }

  function addImageFromSource(source: string) {
    loadImageFromSource(source)
      .then((image) => {
        addLoadedImage(image, crypto.randomUUID(), source);
      })
      .catch((error) => {
        console.error("Failed to load image source:", source, error);
      });
  }

  function looksLikeImageSource(source: string) {
    return source.startsWith("data:image/")
      || source.startsWith("blob:")
      || source.startsWith("file://")
      || /^https?:\/\//i.test(source);
  }

  function getImageSourceFromHtml(html: string): string | null {
    const parsedDocument = new DOMParser().parseFromString(html, "text/html");
    const image = parsedDocument.querySelector("img");

    if (!image) {
      return null;
    }

    const src = image.getAttribute("src");

    if (src && looksLikeImageSource(src)) {
      return src;
    }

    const srcset = image.getAttribute("srcset");

    if (!srcset) {
      return null;
    }

    const firstSrcsetEntry = srcset.split(",")[0]?.trim();
    const firstSrcsetUrl = firstSrcsetEntry?.split(/\s+/)[0];

    if (firstSrcsetUrl && looksLikeImageSource(firstSrcsetUrl)) {
      return firstSrcsetUrl;
    }

    return null;
  }

  const onPaste = (event: ClipboardEvent) => {
    const clipboardData = event.clipboardData;

    if (!clipboardData) {
      return;
    }

    for (const item of clipboardData.items) {
      if (!item.type.startsWith("image/")) {
        continue;
      }

      const file = item.getAsFile();

      if (!file) {
        continue;
      }

      addImageFromFile(file);
      event.preventDefault();
      return;
    }

    const html = clipboardData.getData("text/html");

    if (html) {
      const imageSource = getImageSourceFromHtml(html);

      if (imageSource) {
        addImageFromSource(imageSource);
        event.preventDefault();
        return;
      }
    }

    const text = clipboardData.getData("text/plain");

    if (text && looksLikeImageSource(text.trim())) {
      addImageFromSource(text.trim());
      event.preventDefault();
    }
  };

  const onPointerDown = (event: PointerEvent) => {
    const point = screenToWorld(event.clientX, event.clientY);
    const pressure = getPointerPressure(event);

    if (selectedTool === "pan" || isSpaceDown || event.button === 1 || event.button === 2) {
      isPanning = true;
      canvas.setPointerCapture(event.pointerId);
      canvas.style.cursor = "grabbing";
      return;
    }

    if (selectedTool === "eraser") {
      eraseAtPoint(point);
      selectedItem = null;
      redraw();
      return;
    }

    if (selectedTool === "select") {
      selectedItem = hitTestSelection(point);
      moveAnchorPoint = selectedItem ? point : null;
      canvas.setPointerCapture(event.pointerId);
      updateCursor();
      redraw();
      return;
    }

    if (selectedTool !== "pen") {
      return;
    }

    currentStroke = {
      points: [{
        ...point,
        pressure,
      }],
      color: strokeColor,
      baseWidth: baseStrokeWidth,
    };

    strokes.push(currentStroke);
    redraw();
  };

  const onPointerMove = (event: PointerEvent) => {
    if (isPanning) {
      camera.x += event.movementX;
      camera.y += event.movementY;
      redraw();
      return;
    }

    const point = screenToWorld(event.clientX, event.clientY);
    const pressure = getPointerPressure(event);

    if (selectedTool === "eraser") {
      if (event.buttons === 1) {
        eraseAtPoint(point);
        selectedItem = null;
        redraw();
      }

      return;
    }

    if (selectedTool === "select") {
      if (!selectedItem || !moveAnchorPoint || (event.buttons & 1) === 0) {
        return;
      }

      const deltaX = point.x - moveAnchorPoint.x;
      const deltaY = point.y - moveAnchorPoint.y;
      moveSelectedItem(deltaX, deltaY);
      moveAnchorPoint = point;
      redraw();
      return;
    }

    if (selectedTool !== "pen" || !currentStroke) {
      return;
    }

    currentStroke.points.push({
      ...point,
      pressure,
    });
    redraw();
  };

  const onPointerUp = (event: PointerEvent) => {
    currentStroke = null;
    moveAnchorPoint = null;
    isPanning = false;
    updateCursor();

    if (canvas.hasPointerCapture(event.pointerId)) {
      canvas.releasePointerCapture(event.pointerId);
    }
  };

  const onPointerCancel = () => {
    currentStroke = null;
    moveAnchorPoint = null;
    isPanning = false;
    updateCursor();
  };

  const onWheel = (event: WheelEvent) => {
    event.preventDefault();

    const rect = canvas.getBoundingClientRect();
    const mouseScreenX = event.clientX - rect.left;
    const mouseScreenY = event.clientY - rect.top;
    const worldBeforeZoom = screenToWorld(event.clientX, event.clientY);
    const zoomFactor = Math.exp(-event.deltaY * 0.001);

    camera.scale = clamp(camera.scale * zoomFactor, 0.1, 10);
    camera.x = mouseScreenX - worldBeforeZoom.x * camera.scale;
    camera.y = mouseScreenY - worldBeforeZoom.y * camera.scale;

    redraw();
  };

  const onContextMenu = (event: Event) => {
    event.preventDefault();
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.code === "Space") {
      isSpaceDown = true;
      updateCursor();
      event.preventDefault();
    }

    if (event.ctrlKey && event.key.toLowerCase() === "z") {
      event.preventDefault();
      strokes.pop();
      if (selectedItem?.kind === "stroke" && selectedItem.index >= strokes.length) {
        selectedItem = null;
      }
      redraw();
    }

    if (!event.ctrlKey && event.key.toLowerCase() === "c") {
      strokes.length = 0;
      selectedItem = null;
      redraw();
    }
  };

  const onKeyUp = (event: KeyboardEvent) => {
    if (event.code === "Space") {
      isSpaceDown = false;
      updateCursor();
    }
  };

  setupPickers();
  resizeCanvas();
  window.addEventListener("paste", onPaste);
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);
  window.addEventListener("resize", resizeCanvas);
  canvas.addEventListener("pointerdown", onPointerDown);
  canvas.addEventListener("pointermove", onPointerMove);
  canvas.addEventListener("pointerup", onPointerUp);
  canvas.addEventListener("pointercancel", onPointerCancel);
  canvas.addEventListener("wheel", onWheel);
  canvas.addEventListener("contextmenu", onContextMenu);
  updateCursor();

  function toCanvasBackgroundPattern(pattern: BackgroundPattern): CanvasBackgroundPattern {
    if (pattern === "dotted") {
      return "dots";
    }

    if (pattern === "grid") {
      return "squares";
    }

    if (pattern === "hlines" || pattern === "vlines") {
      return "lines";
    }

    return "none";
  }

  async function importImages(imagePlacements: CanvasImagePlacementDocument[]) {
    const loadedImages = await Promise.all(
      imagePlacements.map(async (placement) => {
        const image = await loadImageFromSource(placement.assetPath);

        return {
          id: placement.id,
          assetPath: placement.assetPath,
          image,
          x: placement.x,
          y: placement.y,
          width: placement.width,
          height: placement.height,
        } satisfies CanvasImage;
      }),
    );

    images.length = 0;
    images.push(...loadedImages);
  }

  return {
    exportDocument(): WorkspaceDocumentSnapshot {
      const document: CanvasDocument = {
        version: {
          major: 1,
          minor: 0,
        },
        id: documentId,
        backgroundPattern: toCanvasBackgroundPattern(backgroundPattern),
        strokes: strokes.map((stroke) => ({
          color: stroke.color,
          width: stroke.baseWidth,
          points: stroke.points.map((point) => ({
            x: point.x,
            y: point.y,
            pressure: point.pressure,
          })),
        })),
        images: images.map((image) => ({
          id: image.id,
          assetPath: image.assetPath,
          x: image.x,
          y: image.y,
          width: image.width,
          height: image.height,
        })),
      };

      return {
        kind: "canvas",
        document,
      };
    },
    async importDocument(snapshot) {
      if (snapshot.kind !== "canvas") {
        throw new Error("Canvas workspace cannot load a non-canvas document.");
      }

      documentId = snapshot.document.id;
      selectedItem = null;
      moveAnchorPoint = null;
      strokes.length = 0;
      strokes.push(
        ...snapshot.document.strokes.map((stroke) => ({
          color: stroke.color,
          baseWidth: stroke.width,
          points: stroke.points.map((point) => ({
            x: point.x,
            y: point.y,
            pressure: point.pressure,
          })),
        })),
      );
      await importImages(snapshot.document.images);
      redraw();
    },
    destroy() {
      window.removeEventListener("paste", onPaste);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      window.removeEventListener("resize", resizeCanvas);
      canvas.removeEventListener("pointerdown", onPointerDown);
      canvas.removeEventListener("pointermove", onPointerMove);
      canvas.removeEventListener("pointerup", onPointerUp);
      canvas.removeEventListener("pointercancel", onPointerCancel);
      canvas.removeEventListener("wheel", onWheel);
      canvas.removeEventListener("contextmenu", onContextMenu);
      container.replaceChildren();
    },
  };
}

function getPointerPressure(event: PointerEvent) {
  if (event.pointerType === "mouse") {
    return 1;
  }

  if (event.pressure <= 0) {
    return 0.25;
  }

  return Math.max(0.2, Math.min(event.pressure, 1));
}

function requireElement<T extends HTMLElement>(root: ParentNode, selector: string) {
  const element = root.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Required element "${selector}" was not found.`);
  }

  return element;
}
