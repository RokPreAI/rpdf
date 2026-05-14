import { readImage } from "@tauri-apps/plugin-clipboard-manager";
import { invoke } from "@tauri-apps/api/core";

import type {
  CanvasBackgroundPattern,
  CanvasDocument,
  CanvasImagePlacementDocument,
  CanvasPdfPagePlacementDocument,
  CanvasSelectionTargetDocument,
  CanvasSelectionDocument,
  PdfRecolorSettingsDocument,
  CanvasShapeDocument,
  CanvasShapeKindDocument,
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
  id: string;
  color: string;
  baseWidth: number;
  order: number;
  points: StrokePoint[];
};

type ShapeKind = CanvasShapeKindDocument;

type Shape = {
  id: string;
  kind: ShapeKind;
  color: string;
  baseWidth: number;
  order: number;
  start: Point;
  end: Point;
};

type VectorItem = Stroke | Shape;

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

type CanvasPdfPage = {
  id: string;
  sourcePdfPath: string;
  pageIndex: number;
  assetPath: string;
  image: HTMLImageElement;
  x: number;
  y: number;
  width: number;
  height: number;
  recolor: PdfRecolorSettingsDocument;
};

type PendingPdfPageImport = {
  sourcePdfPath: string;
  pageIndex: number;
  assetPath: string;
  width: number;
  height: number;
  recolor: PdfRecolorSettingsDocument;
};

type SelectionTarget =
  | {
    kind: "stroke";
    index: number;
  }
  | {
    kind: "shape";
    index: number;
  }
  | {
    kind: "image";
    index: number;
  }
  | {
    kind: "pdf_page";
    index: number;
  };

type ResizeHandle = "nw" | "ne" | "se" | "sw";

type Bounds = {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
};

type ResizeSession = {
  handle: ResizeHandle;
  originalBounds: Bounds;
  pointerOffset: Point;
  targets: Array<{
    target: SelectionTarget;
    originalStroke?: {
      baseWidth: number;
      points: StrokePoint[];
    };
    originalShapePoints?: {
      start: Point;
      end: Point;
    };
    originalBox?: {
      x: number;
      y: number;
      width: number;
      height: number;
    };
  }>;
};

type MarqueeSession = {
  origin: Point;
  current: Point;
  additive: boolean;
};

type Tool = "pen" | "rectangle" | "ellipse" | "line" | "arrow" | "select" | "pan" | "eraser";
type BackgroundPattern = "dotted" | "vlines" | "hlines" | "grid" | "none";
const PREFERENCES_STORAGE_KEY = "rpdf.preferences.v1";

export function mountCanvasWorkspace(container: HTMLElement): WorkspaceController {
  container.innerHTML = `
    <div class="canvas-workspace">
      <canvas class="canvas-surface"></canvas>

      <div class="canvas-toolbar">
        Shortcuts: V select, H pan, P pen, R rectangle, O ellipse, L line, A arrow, E eraser, 1-9 colors | Shift/Ctrl click: multi-select | Right/Middle/Space drag: pan | Wheel: zoom | Ctrl+Z: undo
      </div>

      <div class="stroke-width-control">
        <label class="stroke-control-field" for="stroke-width">
          <span>Stroke width</span>
          <input id="stroke-width" type="range" min="1" max="24" step="1" value="3" />
          <output id="stroke-width-value">3px</output>
        </label>
        <label class="stroke-control-field" for="input-quality">
          <span>Input quality</span>
          <input id="input-quality" type="range" min="1" max="5" step="1" value="3" />
          <output id="input-quality-value">3/5</output>
        </label>
      </div>

      <div class="canvas-pickers">
        <button class="tool-picker active" data-tool="pen" type="button" title="Pen (P)" aria-label="Pen (P)">✎</button>
        <button class="tool-picker" data-tool="rectangle" type="button" title="Rectangle (R)" aria-label="Rectangle (R)">▭</button>
        <button class="tool-picker" data-tool="ellipse" type="button" title="Ellipse (O)" aria-label="Ellipse (O)">◯</button>
        <button class="tool-picker" data-tool="line" type="button" title="Line (L)" aria-label="Line (L)">／</button>
        <button class="tool-picker" data-tool="arrow" type="button" title="Arrow (A)" aria-label="Arrow (A)">↗</button>
        <button class="tool-picker" data-tool="select" type="button" title="Select (V)" aria-label="Select (V)">⌖</button>
        <button class="tool-picker" data-tool="pan" type="button" title="Pan (H or Space)" aria-label="Pan (H or Space)">✥</button>
        <button class="tool-picker" data-tool="eraser" type="button" title="Eraser (E)" aria-label="Eraser (E)">⌫</button>

        <button id="color-picker-fg" class="color-picker active" type="button" title="Foreground color (1)" aria-label="Foreground color (1)"></button>
        <button id="color-picker-blue" class="color-picker" type="button" title="Blue (2)" aria-label="Blue (2)"></button>
        <button id="color-picker-cyan" class="color-picker" type="button" title="Cyan (3)" aria-label="Cyan (3)"></button>
        <button id="color-picker-green" class="color-picker" type="button" title="Green (4)" aria-label="Green (4)"></button>
        <button id="color-picker-yellow" class="color-picker" type="button" title="Yellow (5)" aria-label="Yellow (5)"></button>
        <button id="color-picker-orange" class="color-picker" type="button" title="Orange (6)" aria-label="Orange (6)"></button>
        <button id="color-picker-red" class="color-picker" type="button" title="Red (7)" aria-label="Red (7)"></button>
        <button id="color-picker-magenta" class="color-picker" type="button" title="Magenta (8)" aria-label="Magenta (8)"></button>
        <button id="color-picker-purple" class="color-picker" type="button" title="Purple (9)" aria-label="Purple (9)"></button>
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
  const shapes: Shape[] = [];
  const images: CanvasImage[] = [];
  const pdfPages: CanvasPdfPage[] = [];
  const backgroundColor = "#1a1b26";
  const gridColor = "#292e42";
  const backgroundPattern: BackgroundPattern = "dotted";
  let documentId: string = crypto.randomUUID();
  let nextVectorOrder = 1;

  const camera: Camera = {
    x: container.clientWidth / 2,
    y: container.clientHeight / 2,
    scale: 1,
  };

  const preferences = readPreferences();
  let selectedTool: Tool = "pen";
  let selectedShapeKind: ShapeKind = preferences.defaultShapeKind;
  let strokeColor = preferences.defaultCanvasColor;
  let currentStroke: Stroke | null = null;
  let currentShape: Shape | null = null;
  let selectedItems: SelectionTarget[] = [];
  let moveAnchorPoint: Point | null = null;
  let activeResizeHandle: ResizeHandle | null = null;
  let resizeSession: ResizeSession | null = null;
  let marqueeSession: MarqueeSession | null = null;
  let isPanning = false;
  let isSpaceDown = false;
  let devicePixelRatioValue = window.devicePixelRatio || 1;
  let baseStrokeWidth = preferences.defaultStrokeWidth;
  let inputQuality = preferences.defaultInputQuality;

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
  const toolShortcutByKey: Partial<Record<string, Tool>> = {
    v: "select",
    h: "pan",
    p: "pen",
    r: "rectangle",
    o: "ellipse",
    l: "line",
    a: "arrow",
    e: "eraser",
  };
  const colorButtonIdByShortcutDigit: Record<string, string> = {
    "1": "color-picker-fg",
    "2": "color-picker-blue",
    "3": "color-picker-cyan",
    "4": "color-picker-green",
    "5": "color-picker-yellow",
    "6": "color-picker-orange",
    "7": "color-picker-red",
    "8": "color-picker-magenta",
    "9": "color-picker-purple",
  };

  function clampInputQuality(value: number) {
    if (!Number.isFinite(value)) {
      return 3;
    }

    return clamp(Math.round(value), 1, 5);
  }

  function isShapeTool(tool: Tool) {
    return tool === "rectangle" || tool === "ellipse" || tool === "line" || tool === "arrow";
  }

  function readCssVariable(name: string) {
    return getComputedStyle(document.documentElement)
      .getPropertyValue(name)
      .trim();
  }

  function readPreferences() {
    const fallbackColor = readCssVariable("--fg") || "#c0caf5";
    const rawValue = window.localStorage.getItem(PREFERENCES_STORAGE_KEY);

    if (!rawValue) {
      return {
        defaultStrokeWidth: 3,
        defaultInputQuality: 3,
        defaultShapeKind: "rectangle" as ShapeKind,
        defaultCanvasColor: fallbackColor,
      };
    }

    try {
      const parsed = JSON.parse(rawValue) as Partial<{
        defaultStrokeWidth: number;
        defaultInputQuality: number;
        defaultShapeKind: ShapeKind;
        defaultCanvasColor: string;
      }>;

      return {
        defaultStrokeWidth: typeof parsed.defaultStrokeWidth === "number" ? parsed.defaultStrokeWidth : 3,
        defaultInputQuality: clampInputQuality(parsed.defaultInputQuality ?? 3),
        defaultShapeKind: parsed.defaultShapeKind ?? "rectangle",
        defaultCanvasColor: parsed.defaultCanvasColor ?? fallbackColor,
      };
    } catch (error) {
      console.error("Could not parse canvas preferences:", error);
      return {
        defaultStrokeWidth: 3,
        defaultInputQuality: 3,
        defaultShapeKind: "rectangle" as ShapeKind,
        defaultCanvasColor: fallbackColor,
      };
    }
  }

  function writePreferences(update: Partial<{
    defaultStrokeWidth: number;
    defaultInputQuality: number;
    defaultShapeKind: ShapeKind;
    defaultCanvasColor: string;
  }>) {
    const nextPreferences = {
      ...readPreferences(),
      ...update,
      defaultInputQuality: clampInputQuality(update.defaultInputQuality ?? readPreferences().defaultInputQuality),
    };

    window.localStorage.setItem(PREFERENCES_STORAGE_KEY, JSON.stringify(nextPreferences));
  }

  function getCanvasSize() {
    return {
      width: canvas.width / devicePixelRatioValue,
      height: canvas.height / devicePixelRatioValue,
    };
  }

  function screenToWorld(clientX: number, clientY: number): Point {
    const rect = canvas.getBoundingClientRect();

    return {
      x: (clientX - rect.left - camera.x) / camera.scale,
      y: (clientY - rect.top - camera.y) / camera.scale,
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

    if (selectedTool === "pen" || isShapeTool(selectedTool)) {
      canvas.style.cursor = "crosshair";
    } else if (selectedTool === "select") {
      if (resizeSession?.handle || activeResizeHandle) {
        const handle = resizeSession?.handle ?? activeResizeHandle;
        canvas.style.cursor = handle === "nw" || handle === "se" ? "nwse-resize" : "nesw-resize";
        return;
      }

      canvas.style.cursor = selectedItems.length > 0 ? "move" : "default";
    } else if (selectedTool === "pan") {
      canvas.style.cursor = "grab";
    } else {
      canvas.style.cursor = "cell";
    }
  }

  function setActiveToolButton(toolButtons: NodeListOf<HTMLButtonElement>, tool: Tool) {
    for (const button of toolButtons) {
      button.classList.toggle("active", button.dataset.tool === tool);
    }
  }

  function setActiveTool(toolButtons: NodeListOf<HTMLButtonElement>, tool: Tool) {
    selectedTool = tool;

    if (isShapeTool(tool)) {
      selectedShapeKind = tool;
    }

    setActiveToolButton(toolButtons, selectedTool);
    updateCursor();
  }

  function selectedVectorItems() {
    return selectedItems
      .map((target) => {
        if (target.kind === "stroke") {
          return strokes[target.index] ?? null;
        }

        if (target.kind === "shape") {
          return shapes[target.index] ?? null;
        }

        return null;
      })
      .filter((item): item is VectorItem => Boolean(item));
  }

  function currentSelectionColor() {
    const vectorItems = selectedVectorItems();

    if (vectorItems.length === 0) {
      return null;
    }

    const [firstItem] = vectorItems;

    return vectorItems.every((item) => item.color === firstItem.color)
      ? firstItem.color
      : null;
  }

  function currentSelectionStrokeWidth() {
    const vectorItems = selectedVectorItems();

    if (vectorItems.length === 0) {
      return null;
    }

    const [firstItem] = vectorItems;

    return vectorItems.every((item) => item.baseWidth === firstItem.baseWidth)
      ? firstItem.baseWidth
      : null;
  }

  function applySelectionColor(color: string) {
    const vectorItems = selectedVectorItems();

    if (vectorItems.length === 0) {
      return false;
    }

    for (const item of vectorItems) {
      item.color = color;
    }

    return true;
  }

  function applySelectionStrokeWidth(width: number) {
    const normalizedWidth = Number.isFinite(width) ? Math.max(1, width) : 1;
    const vectorItems = selectedVectorItems();

    if (vectorItems.length === 0) {
      return false;
    }

    for (const item of vectorItems) {
      item.baseWidth = normalizedWidth;
    }

    return true;
  }

  function activateColor(buttonId: string, toolButtons: NodeListOf<HTMLButtonElement>) {
    const cssVariable = colorVariableByButtonId[buttonId];

    if (!cssVariable) {
      return false;
    }

    strokeColor = readCssVariable(cssVariable);

    if (selectedItems.length > 0) {
      applySelectionColor(strokeColor);
      redraw();
    } else {
      setActiveTool(toolButtons, "pen");
    }

    syncStyleControls();
    updateCursor();
    return true;
  }

  function syncStyleControls() {
    const colorButtons = container.querySelectorAll<HTMLButtonElement>(".color-picker");
    const strokeWidthInput = container.querySelector<HTMLInputElement>("#stroke-width");
    const strokeWidthValue = container.querySelector<HTMLOutputElement>("#stroke-width-value");
    const selectionColor = currentSelectionColor();
    const selectionStrokeWidth = currentSelectionStrokeWidth();
    const activeColor = selectionColor ?? strokeColor;
    const activeWidth = selectionStrokeWidth ?? baseStrokeWidth;

    for (const colorButton of colorButtons) {
      const cssVariable = colorVariableByButtonId[colorButton.id];
      const buttonColor = cssVariable ? readCssVariable(cssVariable) : "";
      const isActive = selectionColor
        ? buttonColor === selectionColor
        : buttonColor === activeColor;

      colorButton.classList.toggle("active", isActive);
    }

    if (strokeWidthInput && strokeWidthValue) {
      strokeWidthInput.value = String(Math.max(1, Math.round(activeWidth)));
      strokeWidthValue.textContent = selectionStrokeWidth === null && selectedVectorItems().length > 1
        ? "Mixed"
        : `${Math.max(1, Math.round(activeWidth))}px`;
    }
  }

  function inputQualityLabel(value: number) {
    return `${clampInputQuality(value)}/5`;
  }

  function syncInputQualityControl() {
    const inputQualityInput = container.querySelector<HTMLInputElement>("#input-quality");
    const inputQualityValue = container.querySelector<HTMLOutputElement>("#input-quality-value");

    if (inputQualityInput && inputQualityValue) {
      inputQualityInput.value = String(clampInputQuality(inputQuality));
      inputQualityValue.textContent = inputQualityLabel(inputQuality);
    }
  }

  function strokeSampleSpacing() {
    return (
      inputQuality >= 5 ? 1.5
        : inputQuality === 4 ? 2.5
          : inputQuality === 3 ? 4
            : inputQuality === 2 ? 6
              : 8
    ) / camera.scale;
  }

  function appendPointToCurrentStroke(point: Point, pressure: number) {
    if (!currentStroke) {
      return;
    }

    const lastPoint = currentStroke.points[currentStroke.points.length - 1];

    if (!lastPoint) {
      currentStroke.points.push({
        ...point,
        pressure,
      });
      return;
    }

    const deltaX = point.x - lastPoint.x;
    const deltaY = point.y - lastPoint.y;
    const distance = Math.hypot(deltaX, deltaY);
    const sampleSpacing = strokeSampleSpacing();

    if (distance < sampleSpacing * 0.45) {
      currentStroke.points[currentStroke.points.length - 1] = {
        ...point,
        pressure,
      };
      return;
    }

    const stepCount = Math.max(1, Math.ceil(distance / sampleSpacing));

    for (let stepIndex = 1; stepIndex <= stepCount; stepIndex += 1) {
      const ratio = stepIndex / stepCount;

      currentStroke.points.push({
        x: lastPoint.x + deltaX * ratio,
        y: lastPoint.y + deltaY * ratio,
        pressure: lastPoint.pressure + (pressure - lastPoint.pressure) * ratio,
      });
    }
  }

  function pointerSamples(event: PointerEvent) {
    if (inputQuality < 3 || typeof event.getCoalescedEvents !== "function") {
      return [event];
    }

    const coalescedEvents = event.getCoalescedEvents();

    return coalescedEvents.length > 0 ? coalescedEvents : [event];
  }

  function setupPickers() {
    const colorButtons = container.querySelectorAll<HTMLButtonElement>(".color-picker");
    const toolButtons = container.querySelectorAll<HTMLButtonElement>(".tool-picker");
    const strokeWidthInput = requireElement<HTMLInputElement>(container, "#stroke-width");
    const strokeWidthValue = requireElement<HTMLOutputElement>(container, "#stroke-width-value");
    const inputQualityInput = requireElement<HTMLInputElement>(container, "#input-quality");
    const inputQualityValue = requireElement<HTMLOutputElement>(container, "#input-quality-value");

    for (const button of colorButtons) {
      button.addEventListener("pointerdown", (event) => {
        event.stopPropagation();
      });

      button.addEventListener("click", (event) => {
        event.stopPropagation();
        activateColor(button.id, toolButtons);
      });
    }

    for (const button of toolButtons) {
      button.addEventListener("pointerdown", (event) => {
        event.stopPropagation();
      });

      button.addEventListener("click", (event) => {
        event.stopPropagation();

        const tool = button.dataset.tool;

        if (tool !== "pen" && tool !== "rectangle" && tool !== "ellipse" && tool !== "line" && tool !== "arrow" && tool !== "select" && tool !== "pan" && tool !== "eraser") {
          return;
        }

        setActiveTool(toolButtons, tool);
      });
    }

    strokeWidthInput.value = String(baseStrokeWidth);
    strokeWidthValue.textContent = `${baseStrokeWidth}px`;
    inputQualityInput.value = String(inputQuality);
    inputQualityValue.textContent = inputQualityLabel(inputQuality);
    setActiveToolButton(toolButtons, selectedTool);
    syncStyleControls();
    syncInputQualityControl();
    updateCursor();

    strokeWidthInput.addEventListener("input", () => {
      baseStrokeWidth = Math.max(1, Number(strokeWidthInput.value));
      writePreferences({
        defaultStrokeWidth: baseStrokeWidth,
      });

      if (selectedItems.length > 0) {
        applySelectionStrokeWidth(baseStrokeWidth);
        redraw();
      }

      syncStyleControls();
    });

    inputQualityInput.addEventListener("input", () => {
      inputQuality = clampInputQuality(Number(inputQualityInput.value));
      writePreferences({
        defaultInputQuality: inputQuality,
      });
      syncInputQualityControl();
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

    if (columnCount * rowCount > 5000) {
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

  function calculateRenderedStrokeWidth(strokeWidth: number, pressure: number) {
    return Math.max(1, strokeWidth * pressure);
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
      ctx.lineWidth = calculateRenderedStrokeWidth(stroke.baseWidth, segmentPressure);
      ctx.moveTo(previousPoint.x, previousPoint.y);
      ctx.lineTo(currentPoint.x, currentPoint.y);
      ctx.stroke();
    }

    ctx.restore();
  }

  function shapeBounds(shape: Shape) {
    const relevantPoints = shape.kind === "arrow"
      ? [shape.start, shape.end, ...arrowHeadPoints(shape)]
      : [shape.start, shape.end];
    const minX = Math.min(...relevantPoints.map((point) => point.x));
    const minY = Math.min(...relevantPoints.map((point) => point.y));
    const maxX = Math.max(...relevantPoints.map((point) => point.x));
    const maxY = Math.max(...relevantPoints.map((point) => point.y));
    const padding = shape.baseWidth;

    return {
      x: minX - padding,
      y: minY - padding,
      width: Math.max(1, maxX - minX + padding * 2),
      height: Math.max(1, maxY - minY + padding * 2),
      minX,
      minY,
      maxX,
      maxY,
    };
  }

  function arrowHeadPoints(shape: Shape) {
    const deltaX = shape.end.x - shape.start.x;
    const deltaY = shape.end.y - shape.start.y;
    const length = Math.hypot(deltaX, deltaY);

    if (length === 0) {
      return [shape.end, shape.end];
    }

    const unitX = deltaX / length;
    const unitY = deltaY / length;
    const headLength = Math.max(shape.baseWidth * 4, 18);
    const headWidth = Math.max(shape.baseWidth * 2.4, 10);
    const baseX = shape.end.x - unitX * headLength;
    const baseY = shape.end.y - unitY * headLength;
    const perpendicularX = -unitY;
    const perpendicularY = unitX;

    return [
      {
        x: baseX + perpendicularX * headWidth,
        y: baseY + perpendicularY * headWidth,
      },
      {
        x: baseX - perpendicularX * headWidth,
        y: baseY - perpendicularY * headWidth,
      },
    ] satisfies [Point, Point];
  }

  function drawShape(shape: Shape) {
    const bounds = shapeBounds(shape);

    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);
    ctx.strokeStyle = shape.color;
    ctx.lineWidth = shape.baseWidth;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.beginPath();

    if (shape.kind === "line") {
      ctx.moveTo(shape.start.x, shape.start.y);
      ctx.lineTo(shape.end.x, shape.end.y);
    } else if (shape.kind === "arrow") {
      const [leftHeadPoint, rightHeadPoint] = arrowHeadPoints(shape);

      ctx.moveTo(shape.start.x, shape.start.y);
      ctx.lineTo(shape.end.x, shape.end.y);
      ctx.moveTo(shape.end.x, shape.end.y);
      ctx.lineTo(leftHeadPoint.x, leftHeadPoint.y);
      ctx.moveTo(shape.end.x, shape.end.y);
      ctx.lineTo(rightHeadPoint.x, rightHeadPoint.y);
    } else if (shape.kind === "rectangle") {
      ctx.rect(
        bounds.minX,
        bounds.minY,
        Math.max(1, bounds.maxX - bounds.minX),
        Math.max(1, bounds.maxY - bounds.minY),
      );
    } else {
      const centerX = (shape.start.x + shape.end.x) / 2;
      const centerY = (shape.start.y + shape.end.y) / 2;
      const radiusX = Math.max(0.5, Math.abs(shape.end.x - shape.start.x) / 2);
      const radiusY = Math.max(0.5, Math.abs(shape.end.y - shape.start.y) / 2);

      ctx.ellipse(centerX, centerY, radiusX, radiusY, 0, 0, Math.PI * 2);
    }

    ctx.stroke();
    ctx.restore();
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

  function drawPdfPages() {
    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);

    for (const pdfPage of pdfPages) {
      ctx.drawImage(
        pdfPage.image,
        pdfPage.x,
        pdfPage.y,
        pdfPage.width,
        pdfPage.height,
      );
    }

    ctx.restore();
  }

  function orderedVectorItems() {
    return [...strokes, ...shapes].sort((firstItem, secondItem) => firstItem.order - secondItem.order);
  }

  function normalizeBounds(bounds: Bounds) {
    return {
      minX: Math.min(bounds.minX, bounds.maxX),
      minY: Math.min(bounds.minY, bounds.maxY),
      maxX: Math.max(bounds.minX, bounds.maxX),
      maxY: Math.max(bounds.minY, bounds.maxY),
    } satisfies Bounds;
  }

  function shapeGeometryBounds(shape: Shape) {
    return normalizeBounds({
      minX: shape.start.x,
      minY: shape.start.y,
      maxX: shape.end.x,
      maxY: shape.end.y,
    });
  }

  function targetBounds(target: SelectionTarget): Bounds | null {
    if (target.kind === "image") {
      const image = images[target.index];

      return image ? normalizeBounds({
        minX: image.x,
        minY: image.y,
        maxX: image.x + image.width,
        maxY: image.y + image.height,
      }) : null;
    }

    if (target.kind === "pdf_page") {
      const pdfPage = pdfPages[target.index];

      return pdfPage ? normalizeBounds({
        minX: pdfPage.x,
        minY: pdfPage.y,
        maxX: pdfPage.x + pdfPage.width,
        maxY: pdfPage.y + pdfPage.height,
      }) : null;
    }

    if (target.kind === "shape") {
      const shape = shapes[target.index];
      return shape ? shapeGeometryBounds(shape) : null;
    }

    const stroke = strokes[target.index];
    const bounds = stroke ? getStrokeBounds(stroke) : null;

    return bounds ? normalizeBounds({
      minX: bounds.x,
      minY: bounds.y,
      maxX: bounds.x + bounds.width,
      maxY: bounds.y + bounds.height,
    }) : null;
  }

  function unionBounds(boundsList: Bounds[]) {
    return normalizeBounds({
      minX: Math.min(...boundsList.map((bounds) => bounds.minX)),
      minY: Math.min(...boundsList.map((bounds) => bounds.minY)),
      maxX: Math.max(...boundsList.map((bounds) => bounds.maxX)),
      maxY: Math.max(...boundsList.map((bounds) => bounds.maxY)),
    });
  }

  function currentSelectionBounds() {
    const boundsList = selectedItems
      .map((target) => targetBounds(target))
      .filter((bounds): bounds is Bounds => Boolean(bounds));

    if (boundsList.length === 0) {
      return null;
    }

    return unionBounds(boundsList);
  }

  function isResizableTarget(target: SelectionTarget | null) {
    return target?.kind === "stroke" || target?.kind === "shape" || target?.kind === "image" || target?.kind === "pdf_page";
  }

  function currentResizableSelectionTargets() {
    if (selectedItems.length === 0) {
      return [];
    }

    return selectedItems.every((target) => isResizableTarget(target))
      ? selectedItems
      : [];
  }

  function currentResizeBounds() {
    const resizeTargets = currentResizableSelectionTargets();

    if (resizeTargets.length === 0) {
      return null;
    }

    const boundsList = resizeTargets
      .map((target) => targetBounds(target))
      .filter((bounds): bounds is Bounds => Boolean(bounds));

    if (boundsList.length === 0) {
      return null;
    }

    return unionBounds(boundsList);
  }

  function resizeHandlePositions(bounds: Bounds) {
    return {
      nw: { x: bounds.minX, y: bounds.minY },
      ne: { x: bounds.maxX, y: bounds.minY },
      se: { x: bounds.maxX, y: bounds.maxY },
      sw: { x: bounds.minX, y: bounds.maxY },
    } satisfies Record<ResizeHandle, Point>;
  }

  function handlePoint(bounds: Bounds, handle: ResizeHandle) {
    return resizeHandlePositions(bounds)[handle];
  }

  function hitTestResizeHandle(point: Point) {
    const bounds = currentResizeBounds();

    if (!bounds) {
      return null;
    }

    const handleRadius = Math.max(8 / camera.scale, 6);
    const handles = resizeHandlePositions(bounds);

    for (const handle of ["nw", "ne", "se", "sw"] as ResizeHandle[]) {
      if (distanceBetweenPoints(point, handles[handle]) <= handleRadius) {
        return handle;
      }
    }

    return null;
  }

  function drawSelectionOverlay() {
    if (selectedItems.length === 0) {
      return;
    }

    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);
    ctx.setLineDash([10 / camera.scale, 8 / camera.scale]);
    ctx.lineWidth = 2 / camera.scale;
    ctx.strokeStyle = readCssVariable("--yellow") || "#e0af68";

    for (const target of selectedItems) {
      if (target.kind === "stroke") {
        const stroke = strokes[target.index];

        if (stroke) {
          ctx.save();
          ctx.setLineDash([]);
          ctx.strokeStyle = readCssVariable("--cyan") || "#7dcfff";
          ctx.lineCap = "round";
          ctx.lineJoin = "round";

          if (stroke.points.length === 1) {
            const point = stroke.points[0];
            const radius = Math.max(stroke.baseWidth * 0.9, 6 / camera.scale);

            ctx.beginPath();
            ctx.arc(point.x, point.y, radius, 0, Math.PI * 2);
            ctx.stroke();
          } else {
            ctx.beginPath();
            ctx.lineWidth = Math.max(stroke.baseWidth + 4 / camera.scale, 6 / camera.scale);

            for (let index = 0; index < stroke.points.length; index += 1) {
              const strokePoint = stroke.points[index];

              if (index === 0) {
                ctx.moveTo(strokePoint.x, strokePoint.y);
              } else {
                ctx.lineTo(strokePoint.x, strokePoint.y);
              }
            }

            ctx.stroke();
          }

          ctx.restore();
        }
      }

      const bounds = targetBounds(target);

      if (!bounds) {
        continue;
      }

      ctx.strokeRect(
        bounds.minX,
        bounds.minY,
        Math.max(1, bounds.maxX - bounds.minX),
        Math.max(1, bounds.maxY - bounds.minY),
      );
    }

    if (selectedItems.length > 1) {
      const groupedBounds = currentSelectionBounds();

      if (groupedBounds) {
        ctx.save();
        ctx.setLineDash([18 / camera.scale, 12 / camera.scale]);
        ctx.lineWidth = 3 / camera.scale;
        ctx.strokeStyle = readCssVariable("--orange") || "#ff9e64";
        ctx.strokeRect(
          groupedBounds.minX,
          groupedBounds.minY,
          Math.max(1, groupedBounds.maxX - groupedBounds.minX),
          Math.max(1, groupedBounds.maxY - groupedBounds.minY),
        );
        ctx.restore();
      }
    }

    const resizeBounds = currentResizeBounds();

    if (resizeBounds) {
      const handleSize = Math.max(10 / camera.scale, 8);
      const handleHalf = handleSize / 2;
      const handles = resizeHandlePositions(resizeBounds);

      ctx.setLineDash([]);
      ctx.fillStyle = readCssVariable("--bg") || "#1a1b26";
      ctx.strokeStyle = readCssVariable("--yellow") || "#e0af68";

      for (const handle of ["nw", "ne", "se", "sw"] as ResizeHandle[]) {
        const position = handles[handle];
        ctx.fillRect(position.x - handleHalf, position.y - handleHalf, handleSize, handleSize);
        ctx.strokeRect(position.x - handleHalf, position.y - handleHalf, handleSize, handleSize);
      }
    }

    ctx.restore();
  }

  function drawMarqueeOverlay() {
    if (!marqueeSession) {
      return;
    }

    const bounds = normalizeBounds({
      minX: marqueeSession.origin.x,
      minY: marqueeSession.origin.y,
      maxX: marqueeSession.current.x,
      maxY: marqueeSession.current.y,
    });

    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.scale, camera.scale);
    ctx.setLineDash([12 / camera.scale, 8 / camera.scale]);
    ctx.lineWidth = 2 / camera.scale;
    ctx.strokeStyle = readCssVariable("--cyan") || "#7dcfff";
    ctx.fillStyle = "rgba(125, 207, 255, 0.12)";
    ctx.fillRect(
      bounds.minX,
      bounds.minY,
      Math.max(1, bounds.maxX - bounds.minX),
      Math.max(1, bounds.maxY - bounds.minY),
    );
    ctx.strokeRect(
      bounds.minX,
      bounds.minY,
      Math.max(1, bounds.maxX - bounds.minX),
      Math.max(1, bounds.maxY - bounds.minY),
    );
    ctx.restore();
  }

  function currentSvgExportState() {
    if (selectedItems.length > 0) {
      const vectors: VectorItem[] = [];

      for (const target of selectedItems) {
        if (target.kind === "image") {
          return {
            eligible: false,
            message: "SVG export is disabled for raster image selections.",
            vectors: [] as VectorItem[],
          };
        }

        if (target.kind === "pdf_page") {
          return {
            eligible: false,
            message: "SVG export is disabled for imported PDF page selections.",
            vectors: [] as VectorItem[],
          };
        }

        if (target.kind === "stroke") {
          const stroke = strokes[target.index];

          if (!stroke) {
            return {
              eligible: false,
              message: "A selected stroke is no longer available.",
              vectors: [] as VectorItem[],
            };
          }

          vectors.push(stroke);
          continue;
        }

        const shape = shapes[target.index];

        if (!shape) {
          return {
            eligible: false,
            message: "A selected shape is no longer available.",
            vectors: [] as VectorItem[],
          };
        }

        vectors.push(shape);
      }

      return {
        eligible: vectors.length > 0,
        message: vectors.length === 1
          ? "SVG export will include the selected vector item only."
          : "SVG export will include the selected vector items only.",
        vectors,
      };
    }

    if (images.length > 0 || pdfPages.length > 0) {
      return {
        eligible: false,
        message: "SVG export is disabled while the canvas contains raster images or PDF page imports. Select a vector item or remove raster content.",
        vectors: [] as VectorItem[],
      };
    }

    const vectors = orderedVectorItems();

    if (vectors.length === 0) {
      return {
        eligible: false,
        message: "Draw at least one stroke or shape before exporting SVG.",
        vectors: [] as VectorItem[],
      };
    }

    return {
      eligible: true,
      message: "SVG export will include the full vector canvas.",
      vectors,
    };
  }

  function updateSvgExportState() {
    updateSvgExportStateMessage();
  }

  function updateSvgExportStateMessage(messageOverride?: string) {
    const exportState = currentSvgExportState();
    window.dispatchEvent(new CustomEvent("rpdf:canvas-svg-export-state", {
      detail: {
        eligible: exportState.eligible,
        message: messageOverride ?? exportState.message,
      },
    }));
  }

  function redraw() {
    normalizeSelection();
    updateCursor();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = backgroundColor;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    drawGrid();
    drawImages();
    drawPdfPages();

    for (const vectorItem of orderedVectorItems()) {
      if ("points" in vectorItem) {
        drawStroke(vectorItem);
      } else {
        drawShape(vectorItem);
      }
    }

    drawSelectionOverlay();
    drawMarqueeOverlay();
    updateSvgExportState();
  }

  function distanceBetweenPoints(firstPoint: Point, secondPoint: Point) {
    return Math.hypot(firstPoint.x - secondPoint.x, firstPoint.y - secondPoint.y);
  }

  function pointToSegmentDistance(point: Point, segmentStart: Point, segmentEnd: Point) {
    const segmentLengthSquared = (segmentEnd.x - segmentStart.x) ** 2 + (segmentEnd.y - segmentStart.y) ** 2;

    if (segmentLengthSquared === 0) {
      return distanceBetweenPoints(point, segmentStart);
    }

    const projection = (
      ((point.x - segmentStart.x) * (segmentEnd.x - segmentStart.x))
      + ((point.y - segmentStart.y) * (segmentEnd.y - segmentStart.y))
    ) / segmentLengthSquared;
    const clampedProjection = clamp(projection, 0, 1);

    return distanceBetweenPoints(point, {
      x: segmentStart.x + (segmentEnd.x - segmentStart.x) * clampedProjection,
      y: segmentStart.y + (segmentEnd.y - segmentStart.y) * clampedProjection,
    });
  }

  function selectionToleranceForWidth(baseWidth: number) {
    return Math.max(baseWidth * 0.9, 5 / camera.scale);
  }

  function pointNearRectangleOutline(bounds: ReturnType<typeof shapeBounds>, point: Point, tolerance: number) {
    const withinOuterBounds = pointInsideBounds(point, {
      minX: bounds.minX,
      minY: bounds.minY,
      maxX: bounds.maxX,
      maxY: bounds.maxY,
    }, tolerance);

    if (!withinOuterBounds) {
      return false;
    }

    const innerMinX = bounds.minX + tolerance;
    const innerMinY = bounds.minY + tolerance;
    const innerMaxX = bounds.maxX - tolerance;
    const innerMaxY = bounds.maxY - tolerance;
    const hasInnerArea = innerMinX < innerMaxX && innerMinY < innerMaxY;

    if (!hasInnerArea) {
      return true;
    }

    return !pointInsideBounds(point, {
      minX: innerMinX,
      minY: innerMinY,
      maxX: innerMaxX,
      maxY: innerMaxY,
    });
  }

  function pointNearEllipseOutline(shape: Shape, point: Point, tolerance: number) {
    const centerX = (shape.start.x + shape.end.x) / 2;
    const centerY = (shape.start.y + shape.end.y) / 2;
    const radiusX = Math.max(0.5, Math.abs(shape.end.x - shape.start.x) / 2);
    const radiusY = Math.max(0.5, Math.abs(shape.end.y - shape.start.y) / 2);
    const outerRadiusX = radiusX + tolerance;
    const outerRadiusY = radiusY + tolerance;
    const outerDistance = (((point.x - centerX) / outerRadiusX) ** 2)
      + (((point.y - centerY) / outerRadiusY) ** 2);

    if (outerDistance > 1) {
      return false;
    }

    const innerRadiusX = radiusX - tolerance;
    const innerRadiusY = radiusY - tolerance;

    if (innerRadiusX <= 0 || innerRadiusY <= 0) {
      return true;
    }

    const innerDistance = (((point.x - centerX) / innerRadiusX) ** 2)
      + (((point.y - centerY) / innerRadiusY) ** 2);

    return innerDistance >= 1;
  }

  function shapeContainsPoint(shape: Shape, point: Point, tolerance: number) {
    if (shape.kind === "line") {
      return pointToSegmentDistance(point, shape.start, shape.end) <= tolerance;
    }

    if (shape.kind === "arrow") {
      const [leftHeadPoint, rightHeadPoint] = arrowHeadPoints(shape);
      return pointToSegmentDistance(point, shape.start, shape.end) <= tolerance
        || pointToSegmentDistance(point, shape.end, leftHeadPoint) <= tolerance
        || pointToSegmentDistance(point, shape.end, rightHeadPoint) <= tolerance;
    }

    if (shape.kind === "rectangle") {
      const bounds = shapeBounds(shape);
      return pointNearRectangleOutline(bounds, point, tolerance);
    }

    return pointNearEllipseOutline(shape, point, tolerance);
  }

  function strokeContainsPoint(stroke: Stroke, point: Point, tolerance: number) {
    if (stroke.points.length === 0) {
      return false;
    }

    if (stroke.points.length === 1) {
      return distanceBetweenPoints(point, stroke.points[0]) <= tolerance;
    }

    for (let index = 1; index < stroke.points.length; index += 1) {
      if (pointToSegmentDistance(point, stroke.points[index - 1], stroke.points[index]) <= tolerance) {
        return true;
      }
    }

    return false;
  }

  function eraseAtPoint(point: Point) {
    const eraserRadius = 12 / camera.scale;

    for (let pdfPageIndex = pdfPages.length - 1; pdfPageIndex >= 0; pdfPageIndex -= 1) {
      const pdfPage = pdfPages[pdfPageIndex];
      const withinPdfPage = point.x >= pdfPage.x - eraserRadius
        && point.x <= pdfPage.x + pdfPage.width + eraserRadius
        && point.y >= pdfPage.y - eraserRadius
        && point.y <= pdfPage.y + pdfPage.height + eraserRadius;

      if (withinPdfPage) {
        pdfPages.splice(pdfPageIndex, 1);
      }
    }

    for (let shapeIndex = shapes.length - 1; shapeIndex >= 0; shapeIndex -= 1) {
      if (shapeContainsPoint(shapes[shapeIndex], point, eraserRadius)) {
        shapes.splice(shapeIndex, 1);
      }
    }

    for (let strokeIndex = strokes.length - 1; strokeIndex >= 0; strokeIndex -= 1) {
      const stroke = strokes[strokeIndex];

      if (stroke.points.some((strokePoint) => distanceBetweenPoints(point, strokePoint) <= eraserRadius)) {
        strokes.splice(strokeIndex, 1);
      }
    }
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

  function strokeExportBounds(stroke: Stroke) {
    const bounds = getStrokeBounds(stroke);

    if (!bounds) {
      return null;
    }

    return {
      minX: bounds.x,
      minY: bounds.y,
      maxX: bounds.x + bounds.width,
      maxY: bounds.y + bounds.height,
    };
  }

  function vectorItemBounds(vectorItem: VectorItem) {
    if ("points" in vectorItem) {
      return strokeExportBounds(vectorItem);
    }

    const bounds = shapeBounds(vectorItem);

    return {
      minX: bounds.x,
      minY: bounds.y,
      maxX: bounds.x + bounds.width,
      maxY: bounds.y + bounds.height,
    };
  }

  function boundsContainBounds(containerBounds: Bounds, targetBounds: Bounds, tolerance = 0) {
    return targetBounds.minX >= containerBounds.minX - tolerance
      && targetBounds.maxX <= containerBounds.maxX + tolerance
      && targetBounds.minY >= containerBounds.minY - tolerance
      && targetBounds.maxY <= containerBounds.maxY + tolerance;
  }

  function pointInsideBounds(point: Point, bounds: Bounds, padding = 0) {
    return point.x >= bounds.minX - padding
      && point.x <= bounds.maxX + padding
      && point.y >= bounds.minY - padding
      && point.y <= bounds.maxY + padding;
  }

  function collectTargetsInBounds(bounds: Bounds) {
    const targets: SelectionTarget[] = [];
    const containmentTolerance = Math.max(2 / camera.scale, 1);

    for (let pdfPageIndex = 0; pdfPageIndex < pdfPages.length; pdfPageIndex += 1) {
      const target: SelectionTarget = {
        kind: "pdf_page",
        index: pdfPageIndex,
      };
      const targetSelectionBounds = targetBounds(target);

      if (targetSelectionBounds && boundsContainBounds(bounds, targetSelectionBounds, containmentTolerance)) {
        targets.push(target);
      }
    }

    for (let imageIndex = 0; imageIndex < images.length; imageIndex += 1) {
      const target: SelectionTarget = {
        kind: "image",
        index: imageIndex,
      };
      const targetSelectionBounds = targetBounds(target);

      if (targetSelectionBounds && boundsContainBounds(bounds, targetSelectionBounds, containmentTolerance)) {
        targets.push(target);
      }
    }

    for (const target of orderedVectorSelectionTargets()) {
      const targetSelectionBounds = targetBounds(target);

      if (targetSelectionBounds && boundsContainBounds(bounds, targetSelectionBounds, containmentTolerance)) {
        targets.push(target);
      }
    }

    return targets;
  }

  function createSvgShapeMarkup(shape: Shape, minX: number, minY: number) {
    if (shape.kind === "line") {
      return `<line x1="${shape.start.x - minX}" y1="${shape.start.y - minY}" x2="${shape.end.x - minX}" y2="${shape.end.y - minY}" stroke="${escapeXml(shape.color)}" stroke-width="${shape.baseWidth}" stroke-linecap="round" />`;
    }

    if (shape.kind === "arrow") {
      const [leftHeadPoint, rightHeadPoint] = arrowHeadPoints(shape);

      return `<path d="M ${shape.start.x - minX} ${shape.start.y - minY} L ${shape.end.x - minX} ${shape.end.y - minY} M ${shape.end.x - minX} ${shape.end.y - minY} L ${leftHeadPoint.x - minX} ${leftHeadPoint.y - minY} M ${shape.end.x - minX} ${shape.end.y - minY} L ${rightHeadPoint.x - minX} ${rightHeadPoint.y - minY}" fill="none" stroke="${escapeXml(shape.color)}" stroke-width="${shape.baseWidth}" stroke-linecap="round" stroke-linejoin="round" />`;
    }

    if (shape.kind === "rectangle") {
      const bounds = shapeBounds(shape);

      return `<rect x="${bounds.minX - minX}" y="${bounds.minY - minY}" width="${Math.max(1, bounds.maxX - bounds.minX)}" height="${Math.max(1, bounds.maxY - bounds.minY)}" fill="none" stroke="${escapeXml(shape.color)}" stroke-width="${shape.baseWidth}" rx="${Math.min(12, shape.baseWidth * 1.5)}" ry="${Math.min(12, shape.baseWidth * 1.5)}" />`;
    }

    const centerX = (shape.start.x + shape.end.x) / 2;
    const centerY = (shape.start.y + shape.end.y) / 2;
    const radiusX = Math.max(0.5, Math.abs(shape.end.x - shape.start.x) / 2);
    const radiusY = Math.max(0.5, Math.abs(shape.end.y - shape.start.y) / 2);

    return `<ellipse cx="${centerX - minX}" cy="${centerY - minY}" rx="${radiusX}" ry="${radiusY}" fill="none" stroke="${escapeXml(shape.color)}" stroke-width="${shape.baseWidth}" />`;
  }

  function createSvgMarkup(exportVectors: VectorItem[]) {
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;

    for (const vectorItem of exportVectors) {
      const bounds = vectorItemBounds(vectorItem);

      if (!bounds) {
        continue;
      }

      minX = Math.min(minX, bounds.minX);
      minY = Math.min(minY, bounds.minY);
      maxX = Math.max(maxX, bounds.maxX);
      maxY = Math.max(maxY, bounds.maxY);
    }

    if (!Number.isFinite(minX) || !Number.isFinite(minY) || !Number.isFinite(maxX) || !Number.isFinite(maxY)) {
      throw new Error("No exportable vector items were available.");
    }

    const width = Math.max(1, maxX - minX);
    const height = Math.max(1, maxY - minY);

    const items = exportVectors.map((vectorItem) => {
      if ("points" in vectorItem) {
        if (vectorItem.points.length === 0) {
          return "";
        }

        const commands = vectorItem.points
          .map((point, index) => `${index === 0 ? "M" : "L"} ${point.x - minX} ${point.y - minY}`)
          .join(" ");

        return `<path d="${commands}" fill="none" stroke="${escapeXml(vectorItem.color)}" stroke-width="${vectorItem.baseWidth}" stroke-linecap="round" stroke-linejoin="round" />`;
      }

      return createSvgShapeMarkup(vectorItem, minX, minY);
    }).join("");

    return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">
${items}
</svg>`;
  }

  async function exportSvg() {
    const exportState = currentSvgExportState();

    if (!exportState.eligible) {
      updateSvgExportState();
      return;
    }

    const svgMarkup = createSvgMarkup(exportState.vectors);
    const suggestedFileName = selectedItems.length > 0 ? "canvas-selection.svg" : "canvas-document.svg";

    try {
      const savedPath = await invoke<string | null>("save_svg_export", {
        request: {
          suggestedFileName,
          svgMarkup,
        },
      });

      if (savedPath) {
        updateSvgExportStateMessage(`SVG exported to ${savedPath}`);
        return;
      }

      updateSvgExportStateMessage("SVG export was cancelled.");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      updateSvgExportStateMessage(`SVG export failed: ${message}`);
    }
  }

  function orderedVectorSelectionTargets() {
    return [...strokes, ...shapes]
      .sort((firstItem, secondItem) => secondItem.order - firstItem.order)
      .map((vectorItem) => ("points" in vectorItem
        ? {
          kind: "stroke" as const,
          index: strokes.findIndex((stroke) => stroke.id === vectorItem.id),
        }
        : {
          kind: "shape" as const,
          index: shapes.findIndex((shape) => shape.id === vectorItem.id),
        }))
      .filter((target) => target.index >= 0);
  }

  function selectedTargetId(target: SelectionTarget): string | null {
    if (target.kind === "image") {
      return images[target.index]?.id ?? null;
    }

    if (target.kind === "pdf_page") {
      return pdfPages[target.index]?.id ?? null;
    }

    if (target.kind === "shape") {
      return shapes[target.index]?.id ?? null;
    }

    return strokes[target.index]?.id ?? null;
  }

  function selectionTargetKey(target: SelectionTarget) {
    const id = selectedTargetId(target);
    return id ? `${target.kind}:${id}` : null;
  }

  function selectedKeySet() {
    const keys = new Set<string>();

    for (const target of selectedItems) {
      const key = selectionTargetKey(target);

      if (key) {
        keys.add(key);
      }
    }

    return keys;
  }

  function selectionContains(target: SelectionTarget) {
    const key = selectionTargetKey(target);
    return key ? selectedKeySet().has(key) : false;
  }

  function setSelection(targets: SelectionTarget[]) {
    const uniqueTargets: SelectionTarget[] = [];
    const seen = new Set<string>();

    for (const target of targets) {
      const key = selectionTargetKey(target);

      if (!key || seen.has(key)) {
        continue;
      }

      seen.add(key);
      uniqueTargets.push(target);
    }

    selectedItems = uniqueTargets;
    syncStyleControls();
    updateCursor();
  }

  function toggleSelectionTarget(target: SelectionTarget) {
    const targetKey = selectionTargetKey(target);

    if (!targetKey) {
      return;
    }

    if (selectedKeySet().has(targetKey)) {
      setSelection(selectedItems.filter((entry) => selectionTargetKey(entry) !== targetKey));
      return;
    }

    setSelection([...selectedItems, target]);
  }

  function currentSelectionSnapshot(): CanvasSelectionDocument | null {
    if (selectedItems.length === 0) {
      return null;
    }

    const targets = selectedItems
      .map((target) => {
        const id = selectedTargetId(target);

        if (!id) {
          return null;
        }

        return {
          kind: target.kind,
          id,
        } satisfies CanvasSelectionTargetDocument;
      })
      .filter((target): target is CanvasSelectionTargetDocument => Boolean(target));

    if (targets.length === 0) {
      return null;
    }

    return {
      targets,
    };
  }

  function resolveSelectionTarget(selection: CanvasSelectionTargetDocument): SelectionTarget | null {
    if (selection.kind === "image") {
      const index = images.findIndex((image) => image.id === selection.id);
      return index >= 0 ? { kind: "image", index } : null;
    }

    if (selection.kind === "pdf_page") {
      const index = pdfPages.findIndex((pdfPage) => pdfPage.id === selection.id);
      return index >= 0 ? { kind: "pdf_page", index } : null;
    }

    if (selection.kind === "shape") {
      const index = shapes.findIndex((shape) => shape.id === selection.id);
      return index >= 0 ? { kind: "shape", index } : null;
    }

    const index = strokes.findIndex((stroke) => stroke.id === selection.id);
    return index >= 0 ? { kind: "stroke", index } : null;
  }

  function resolveSelection(selection: CanvasSelectionDocument | null | undefined): SelectionTarget[] {
    if (!selection) {
      return [];
    }

    if ("targets" in selection) {
      return selection.targets
        .map((target) => resolveSelectionTarget(target))
        .filter((target): target is SelectionTarget => Boolean(target));
    }

    const target = resolveSelectionTarget(selection);
    return target ? [target] : [];
  }

  function normalizeSelection() {
    setSelection(resolveSelection(currentSelectionSnapshot()));
  }

  function hitTestSelection(point: Point): SelectionTarget | null {
    for (let pdfPageIndex = pdfPages.length - 1; pdfPageIndex >= 0; pdfPageIndex -= 1) {
      const pdfPage = pdfPages[pdfPageIndex];

      if (
        point.x >= pdfPage.x
        && point.x <= pdfPage.x + pdfPage.width
        && point.y >= pdfPage.y
        && point.y <= pdfPage.y + pdfPage.height
      ) {
        return {
          kind: "pdf_page",
          index: pdfPageIndex,
        };
      }
    }

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

    for (const target of orderedVectorSelectionTargets()) {
      const vectorItem = target.kind === "shape"
        ? shapes[target.index]
        : strokes[target.index];

      if (!vectorItem) {
        continue;
      }

      const bounds = vectorItemBounds(vectorItem);

      if (!bounds) {
        continue;
      }

      const withinBounds = point.x >= bounds.minX
        && point.x <= bounds.maxX
        && point.y >= bounds.minY
        && point.y <= bounds.maxY;

      if (!withinBounds) {
        continue;
      }

      const tolerance = selectionToleranceForWidth(vectorItem.baseWidth);

      if (target.kind === "shape") {
        const shape = vectorItem as Shape;

        if (shapeContainsPoint(shape, point, tolerance)) {
          return target;
        }
        continue;
      }

      const stroke = vectorItem as Stroke;

      if (strokeContainsPoint(stroke, point, tolerance)) {
        return target;
      }
    }

    return null;
  }

  function moveSelectedItems(deltaX: number, deltaY: number) {
    if (selectedItems.length === 0) {
      return;
    }

    for (const target of selectedItems) {
      if (target.kind === "image") {
        const image = images[target.index];

        if (!image) {
          continue;
        }

        image.x += deltaX;
        image.y += deltaY;
        continue;
      }

      if (target.kind === "pdf_page") {
        const pdfPage = pdfPages[target.index];

        if (!pdfPage) {
          continue;
        }

        pdfPage.x += deltaX;
        pdfPage.y += deltaY;
        continue;
      }

      if (target.kind === "shape") {
        const shape = shapes[target.index];

        if (!shape) {
          continue;
        }

        shape.start.x += deltaX;
        shape.start.y += deltaY;
        shape.end.x += deltaX;
        shape.end.y += deltaY;
        continue;
      }

      const stroke = strokes[target.index];

      if (!stroke) {
        continue;
      }

      for (const point of stroke.points) {
        point.x += deltaX;
        point.y += deltaY;
      }
    }
  }

  function remapPointBetweenBounds(point: Point, originalBounds: Bounds, nextBounds: Bounds): Point {
    const originalWidth = originalBounds.maxX - originalBounds.minX;
    const originalHeight = originalBounds.maxY - originalBounds.minY;
    const nextWidth = nextBounds.maxX - nextBounds.minX;
    const nextHeight = nextBounds.maxY - nextBounds.minY;
    const xRatio = originalWidth === 0 ? 0.5 : (point.x - originalBounds.minX) / originalWidth;
    const yRatio = originalHeight === 0 ? 0.5 : (point.y - originalBounds.minY) / originalHeight;

    return {
      x: nextBounds.minX + nextWidth * xRatio,
      y: nextBounds.minY + nextHeight * yRatio,
    };
  }

  function safeCoordinate(value: number, fallback: number) {
    return Number.isFinite(value) ? value : fallback;
  }

  function safeSpan(value: number, minimumSize: number) {
    if (!Number.isFinite(value)) {
      return minimumSize;
    }

    return Math.max(minimumSize, Math.abs(value));
  }

  function createResizeSession(handle: ResizeHandle, pointerPoint: Point): ResizeSession | null {
    const resizeTargets = currentResizableSelectionTargets();
    const originalBounds = currentResizeBounds();

    if (resizeTargets.length === 0 || !originalBounds) {
      return null;
    }

    const handlePosition = handlePoint(originalBounds, handle);

    return {
      handle,
      originalBounds,
      pointerOffset: {
        x: handlePosition.x - pointerPoint.x,
        y: handlePosition.y - pointerPoint.y,
      },
      targets: resizeTargets.map((target) => {
        if (target.kind === "stroke") {
          const stroke = strokes[target.index];

          return {
            target,
            originalStroke: stroke ? {
              baseWidth: stroke.baseWidth,
              points: stroke.points.map((point) => ({
                x: point.x,
                y: point.y,
                pressure: point.pressure,
              })),
            } : undefined,
          };
        }

        if (target.kind === "shape") {
          const shape = shapes[target.index];

          return {
            target,
            originalShapePoints: shape ? {
              start: { ...shape.start },
              end: { ...shape.end },
            } : undefined,
          };
        }

        if (target.kind === "image") {
          const image = images[target.index];

          return {
            target,
            originalBox: image ? {
              x: image.x,
              y: image.y,
              width: image.width,
              height: image.height,
            } : undefined,
          };
        }

        const pdfPage = pdfPages[target.index];

        return {
          target,
          originalBox: pdfPage ? {
            x: pdfPage.x,
            y: pdfPage.y,
            width: pdfPage.width,
            height: pdfPage.height,
          } : undefined,
        };
      }),
    };
  }

  function scaleFactor(originalSpan: number, nextSpan: number) {
    if (!Number.isFinite(originalSpan) || Math.abs(originalSpan) < 0.0001) {
      return 1;
    }

    if (!Number.isFinite(nextSpan)) {
      return 1;
    }

    return Math.abs(nextSpan / originalSpan);
  }

  function applyResize(session: ResizeSession, point: Point) {
    const { originalBounds, handle } = session;
    const minimumSize = Math.max(12 / camera.scale, 6);
    const fixedCorner = handle === "nw"
      ? { x: originalBounds.maxX, y: originalBounds.maxY }
      : handle === "ne"
        ? { x: originalBounds.minX, y: originalBounds.maxY }
        : handle === "se"
          ? { x: originalBounds.minX, y: originalBounds.minY }
          : { x: originalBounds.maxX, y: originalBounds.minY };
    const pointerX = safeCoordinate(point.x, fixedCorner.x);
    const pointerY = safeCoordinate(point.y, fixedCorner.y);
    const rawBounds = handle === "nw"
      ? {
        minX: pointerX,
        minY: pointerY,
        maxX: fixedCorner.x,
        maxY: fixedCorner.y,
      }
      : handle === "ne"
        ? {
          minX: fixedCorner.x,
          minY: pointerY,
          maxX: pointerX,
          maxY: fixedCorner.y,
        }
        : handle === "se"
          ? {
            minX: fixedCorner.x,
            minY: fixedCorner.y,
            maxX: pointerX,
            maxY: pointerY,
          }
          : {
            minX: pointerX,
            minY: fixedCorner.y,
            maxX: fixedCorner.x,
            maxY: pointerY,
          };
    const normalizedBounds = normalizeBounds(rawBounds);
    const nextBounds = {
      minX: normalizedBounds.minX,
      minY: normalizedBounds.minY,
      maxX: normalizedBounds.minX + safeSpan(normalizedBounds.maxX - normalizedBounds.minX, minimumSize),
      maxY: normalizedBounds.minY + safeSpan(normalizedBounds.maxY - normalizedBounds.minY, minimumSize),
    } satisfies Bounds;
    const widthScale = scaleFactor(originalBounds.maxX - originalBounds.minX, nextBounds.maxX - nextBounds.minX);
    const heightScale = scaleFactor(originalBounds.maxY - originalBounds.minY, nextBounds.maxY - nextBounds.minY);
    const strokeWidthScale = Math.max(0.25, (widthScale + heightScale) / 2);

    for (const targetSession of session.targets) {
      const { target } = targetSession;

      if (target.kind === "stroke") {
        const stroke = strokes[target.index];
        const originalStroke = targetSession.originalStroke;

        if (!stroke || !originalStroke) {
          continue;
        }

        stroke.points = originalStroke.points.map((originalPoint) => ({
          pressure: originalPoint.pressure,
          ...remapPointBetweenBounds(originalPoint, originalBounds, nextBounds),
        }));
        stroke.baseWidth = Math.max(1, originalStroke.baseWidth * strokeWidthScale);
        continue;
      }

      if (target.kind === "image") {
        const image = images[target.index];
        const originalBox = targetSession.originalBox;

        if (!image || !originalBox) {
          continue;
        }

        const nextTopLeft = remapPointBetweenBounds(
          { x: originalBox.x, y: originalBox.y },
          originalBounds,
          nextBounds,
        );
        const nextBottomRight = remapPointBetweenBounds(
          { x: originalBox.x + originalBox.width, y: originalBox.y + originalBox.height },
          originalBounds,
          nextBounds,
        );

        image.x = Math.min(nextTopLeft.x, nextBottomRight.x);
        image.y = Math.min(nextTopLeft.y, nextBottomRight.y);
        image.width = Math.max(minimumSize, Math.abs(nextBottomRight.x - nextTopLeft.x));
        image.height = Math.max(minimumSize, Math.abs(nextBottomRight.y - nextTopLeft.y));
        continue;
      }

      if (target.kind === "pdf_page") {
        const pdfPage = pdfPages[target.index];
        const originalBox = targetSession.originalBox;

        if (!pdfPage || !originalBox) {
          continue;
        }

        const nextTopLeft = remapPointBetweenBounds(
          { x: originalBox.x, y: originalBox.y },
          originalBounds,
          nextBounds,
        );
        const nextBottomRight = remapPointBetweenBounds(
          { x: originalBox.x + originalBox.width, y: originalBox.y + originalBox.height },
          originalBounds,
          nextBounds,
        );

        pdfPage.x = Math.min(nextTopLeft.x, nextBottomRight.x);
        pdfPage.y = Math.min(nextTopLeft.y, nextBottomRight.y);
        pdfPage.width = Math.max(minimumSize, Math.abs(nextBottomRight.x - nextTopLeft.x));
        pdfPage.height = Math.max(minimumSize, Math.abs(nextBottomRight.y - nextTopLeft.y));
        continue;
      }

      if (target.kind !== "shape") {
        continue;
      }

      const shape = shapes[target.index];
      const originalShapePoints = targetSession.originalShapePoints;

      if (!shape || !originalShapePoints) {
        continue;
      }

      shape.start = remapPointBetweenBounds(originalShapePoints.start, originalBounds, nextBounds);
      shape.end = remapPointBetweenBounds(originalShapePoints.end, originalBounds, nextBounds);
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

  function addLoadedPdfPage(
    image: HTMLImageElement,
    payload: PendingPdfPageImport,
  ) {
    const { width, height } = getCanvasSize();
    const centerWorld = screenPointToWorld(width / 2, height / 2);
    const maxImageScreenWidth = width * 0.65;
    const scale = Math.min(1, maxImageScreenWidth / image.naturalWidth);
    const pageWidth = (image.naturalWidth * scale) / camera.scale;
    const pageHeight = (image.naturalHeight * scale) / camera.scale;

    pdfPages.push({
      id: crypto.randomUUID(),
      sourcePdfPath: payload.sourcePdfPath,
      pageIndex: payload.pageIndex,
      assetPath: payload.assetPath,
      image,
      x: centerWorld.x - pageWidth / 2,
      y: centerWorld.y - pageHeight / 2,
      width: pageWidth,
      height: pageHeight,
      recolor: payload.recolor,
    });

    setSelection([{
      kind: "pdf_page",
      index: pdfPages.length - 1,
    }]);
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

  async function importImageFromTauriClipboard() {
    try {
      console.log("[canvas paste] Attempting native Tauri clipboard image read.");
      const clipboardImage = await readImage();
      const [{ width, height }, rgba] = await Promise.all([
        clipboardImage.size(),
        clipboardImage.rgba(),
      ]);

      console.log("[canvas paste] Native clipboard image size:", {
        width,
        height,
        bytes: rgba.length,
      });

      const imageCanvas = document.createElement("canvas");
      imageCanvas.width = width;
      imageCanvas.height = height;
      const imageContext = imageCanvas.getContext("2d");

      if (!imageContext) {
        throw new Error("Could not get 2D context for native clipboard image.");
      }

      imageContext.putImageData(
        new ImageData(new Uint8ClampedArray(rgba), width, height),
        0,
        0,
      );

      const source = imageCanvas.toDataURL("image/png");
      const image = await loadImageFromSource(source);
      addLoadedImage(image, crypto.randomUUID(), source);
      console.log("[canvas paste] Imported image from native Tauri clipboard.");
      return true;
    } catch (error) {
      console.error("[canvas paste] Native Tauri clipboard image read failed:", error);
      return false;
    }
  }

  async function importImageFromClipboardApi() {
    const importedFromTauriClipboard = await importImageFromTauriClipboard();

    if (importedFromTauriClipboard) {
      return true;
    }

    if (!navigator.clipboard?.read) {
      console.log("[canvas paste] Clipboard API read is unavailable in this runtime.");
      return false;
    }

    try {
      console.log("[canvas paste] Attempting navigator.clipboard.read() fallback.");
      const clipboardItems = await navigator.clipboard.read();
      console.log("[canvas paste] navigator.clipboard.read() returned items:", clipboardItems.length);

      for (const [index, clipboardItem] of clipboardItems.entries()) {
        console.log("[canvas paste] Clipboard item", index, "types:", clipboardItem.types);
        const imageMimeType = clipboardItem.types.find((type) => type.startsWith("image/"));

        if (!imageMimeType) {
          continue;
        }

        console.log("[canvas paste] Found image MIME type from Clipboard API:", imageMimeType);
        const blob = await clipboardItem.getType(imageMimeType);
        console.log("[canvas paste] Clipboard API blob size:", blob.size, "type:", blob.type);
        const file = new File([blob], `clipboard-image.${imageMimeType.split("/")[1] || "png"}`, {
          type: imageMimeType,
        });

        await addImageFromFile(file);
        console.log("[canvas paste] Imported image from Clipboard API fallback.");
        return true;
      }
    } catch (error) {
      console.error("Clipboard API image read failed:", error);
    }

    return false;
  }

  function tryImportFromClipboardData(clipboardData: DataTransfer | null) {
    if (!clipboardData) {
      console.log("[canvas paste] No clipboardData on paste event.");
      return false;
    }

    console.log("[canvas paste] clipboardData item count:", clipboardData.items.length);
    console.log("[canvas paste] clipboardData types:", Array.from(clipboardData.types));

    for (const item of clipboardData.items) {
      console.log("[canvas paste] clipboardData item type:", item.type, "kind:", item.kind);
      if (!item.type.startsWith("image/")) {
        continue;
      }

      const file = item.getAsFile();

      if (!file) {
        console.log("[canvas paste] Image item existed but getAsFile() returned null.");
        continue;
      }

      console.log("[canvas paste] Importing image file from paste event:", {
        name: file.name,
        type: file.type,
        size: file.size,
      });
      void addImageFromFile(file);
      return true;
    }

    const html = clipboardData.getData("text/html");
    console.log("[canvas paste] HTML payload length:", html.length);

    if (html) {
      const imageSource = getImageSourceFromHtml(html);
      console.log("[canvas paste] Parsed image source from HTML:", imageSource);

      if (imageSource) {
        addImageFromSource(imageSource);
        return true;
      }
    }

    const text = clipboardData.getData("text/plain");
    console.log("[canvas paste] Plain text payload preview:", text.slice(0, 200));

    if (text && looksLikeImageSource(text.trim())) {
      console.log("[canvas paste] Plain text looked like an image source.");
      addImageFromSource(text.trim());
      return true;
    }

    console.log("[canvas paste] No importable image found in paste event payload.");
    return false;
  }

  function createStroke(point: Point, pressure: number) {
    const stroke: Stroke = {
      id: crypto.randomUUID(),
      color: strokeColor,
      baseWidth: baseStrokeWidth,
      order: nextVectorOrder,
      points: [{
        ...point,
        pressure,
      }],
    };

    nextVectorOrder += 1;
    strokes.push(stroke);
    return stroke;
  }

  function createShape(point: Point) {
    const shape: Shape = {
      id: crypto.randomUUID(),
      kind: selectedShapeKind,
      color: strokeColor,
      baseWidth: baseStrokeWidth,
      order: nextVectorOrder,
      start: { ...point },
      end: { ...point },
    };

    nextVectorOrder += 1;
    shapes.push(shape);
    return shape;
  }

  function removeLastVectorItem() {
    const lastStroke = strokes[strokes.length - 1];
    const lastShape = shapes[shapes.length - 1];

    if (lastStroke && (!lastShape || lastStroke.order > lastShape.order)) {
      strokes.pop();
      normalizeSelection();
      return;
    }

    if (lastShape) {
      shapes.pop();
      normalizeSelection();
    }
  }

  const onPaste = (event: ClipboardEvent) => {
    console.log("[canvas paste] paste event received:", {
      type: event.type,
      targetTag: (event.target as HTMLElement | null)?.tagName ?? null,
      hasClipboardData: Boolean(event.clipboardData),
    });

    if (tryImportFromClipboardData(event.clipboardData)) {
      console.log("[canvas paste] Preventing default because import path matched paste payload.");
      event.preventDefault();
    }
  };

  const onPdfPageImport = (event: CustomEvent<PendingPdfPageImport>) => {
    loadImageFromSource(event.detail.assetPath)
      .then((image) => {
        addLoadedPdfPage(image, event.detail);
      })
      .catch((error) => {
        console.error("Failed to import PDF page into canvas:", error);
      });
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
      setSelection([]);
      redraw();
      return;
    }

    if (selectedTool === "select") {
      const additiveSelection = event.shiftKey || event.ctrlKey || event.metaKey;
      const resizeHandle = hitTestResizeHandle(point);
      const pointerSelection = hitTestSelection(point);
      const selectedBounds = currentSelectionBounds();
      const clickedSelectionBounds = !pointerSelection
        && !additiveSelection
        && Boolean(selectedBounds)
        && selectedBounds !== null
        && pointInsideBounds(point, selectedBounds, Math.max(8 / camera.scale, 4));

      if (!additiveSelection && resizeHandle) {
        resizeSession = createResizeSession(resizeHandle, point);

        if (resizeSession) {
          activeResizeHandle = resizeHandle;
          moveAnchorPoint = null;
          canvas.setPointerCapture(event.pointerId);
          updateCursor();
          redraw();
          return;
        }
      }

      if (clickedSelectionBounds) {
        activeResizeHandle = resizeHandle;
        moveAnchorPoint = point;
        canvas.setPointerCapture(event.pointerId);
        updateCursor();
        redraw();
        return;
      }

      if (pointerSelection) {
        if (additiveSelection) {
          toggleSelectionTarget(pointerSelection);
        } else if (!selectionContains(pointerSelection)) {
          setSelection([pointerSelection]);
        }
      } else if (!additiveSelection) {
        marqueeSession = {
          origin: point,
          current: point,
          additive: false,
        };
      } else {
        marqueeSession = {
          origin: point,
          current: point,
          additive: true,
        };
      }

      activeResizeHandle = hitTestResizeHandle(point);
      moveAnchorPoint = pointerSelection && selectionContains(pointerSelection) ? point : null;
      canvas.setPointerCapture(event.pointerId);
      updateCursor();
      redraw();
      return;
    }

    if (isShapeTool(selectedTool)) {
      selectedShapeKind = selectedTool;
      currentShape = createShape(point);
      canvas.setPointerCapture(event.pointerId);
      redraw();
      return;
    }

    if (selectedTool !== "pen") {
      return;
    }

    currentStroke = createStroke(point, pressure);
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

    if (selectedTool === "eraser") {
      if (event.buttons === 1) {
        eraseAtPoint(point);
        setSelection([]);
        redraw();
      }

      return;
    }

    if (selectedTool === "select") {
      activeResizeHandle = hitTestResizeHandle(point);

      if (resizeSession && (event.buttons & 1) !== 0) {
        applyResize(
          resizeSession,
          {
            x: point.x + resizeSession.pointerOffset.x,
            y: point.y + resizeSession.pointerOffset.y,
          },
        );
        redraw();
        return;
      }

      if (marqueeSession && (event.buttons & 1) !== 0) {
        marqueeSession.current = point;
        redraw();
        return;
      }

      if (selectedItems.length === 0 || !moveAnchorPoint || (event.buttons & 1) === 0) {
        updateCursor();
        return;
      }

      const deltaX = point.x - moveAnchorPoint.x;
      const deltaY = point.y - moveAnchorPoint.y;
      moveSelectedItems(deltaX, deltaY);
      moveAnchorPoint = point;
      redraw();
      return;
    }

    if (isShapeTool(selectedTool) && currentShape) {
      currentShape.end = { ...point };
      redraw();
      return;
    }

    if (selectedTool !== "pen" || !currentStroke) {
      return;
    }

    for (const sample of pointerSamples(event)) {
      appendPointToCurrentStroke(
        screenToWorld(sample.clientX, sample.clientY),
        getPointerPressure(sample),
      );
    }

    redraw();
  };

  const onPointerUp = (event: PointerEvent) => {
    if (marqueeSession) {
      const marqueeBounds = normalizeBounds({
        minX: marqueeSession.origin.x,
        minY: marqueeSession.origin.y,
        maxX: marqueeSession.current.x,
        maxY: marqueeSession.current.y,
      });
      const marqueeWidth = marqueeBounds.maxX - marqueeBounds.minX;
      const marqueeHeight = marqueeBounds.maxY - marqueeBounds.minY;

      if (marqueeWidth >= 4 / camera.scale || marqueeHeight >= 4 / camera.scale) {
        const matchedTargets = collectTargetsInBounds(marqueeBounds);
        setSelection(marqueeSession.additive ? [...selectedItems, ...matchedTargets] : matchedTargets);
      } else if (!marqueeSession.additive) {
        setSelection([]);
      }
    }

    currentStroke = null;
    currentShape = null;
    moveAnchorPoint = null;
    resizeSession = null;
    marqueeSession = null;
    isPanning = false;
    activeResizeHandle = null;
    redraw();
    updateCursor();

    if (canvas.hasPointerCapture(event.pointerId)) {
      canvas.releasePointerCapture(event.pointerId);
    }
  };

  const onPointerCancel = () => {
    currentStroke = null;
    currentShape = null;
    moveAnchorPoint = null;
    resizeSession = null;
    marqueeSession = null;
    isPanning = false;
    activeResizeHandle = null;
    redraw();
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
    const toolButtons = container.querySelectorAll<HTMLButtonElement>(".tool-picker");

    if (event.code === "Space") {
      isSpaceDown = true;
      updateCursor();
      event.preventDefault();
    }

    if (event.ctrlKey && event.key.toLowerCase() === "z") {
      event.preventDefault();
      removeLastVectorItem();

      if (
        selectedItems.some((target) => (
          (target.kind === "stroke" && target.index >= strokes.length)
          || (target.kind === "shape" && target.index >= shapes.length)
          || (target.kind === "image" && target.index >= images.length)
          || (target.kind === "pdf_page" && target.index >= pdfPages.length)
        ))
      ) {
        setSelection([]);
      }

      redraw();
    }

    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v") {
      console.log("[canvas paste] Keyboard paste shortcut detected. Trying native clipboard import.");
      void importImageFromClipboardApi().then((imported) => {
        console.log("[canvas paste] Clipboard import result:", imported);
        if (imported) {
          event.preventDefault();
        }
      });
    }

    if (isEditableTarget(event.target)) {
      return;
    }

    if (event.ctrlKey || event.metaKey || event.altKey) {
      return;
    }

    const shortcutKey = event.key.toLowerCase();
    const shortcutTool = toolShortcutByKey[shortcutKey];

    if (shortcutTool) {
      event.preventDefault();
      setActiveTool(toolButtons, shortcutTool);
      return;
    }

    const colorButtonId = colorButtonIdByShortcutDigit[shortcutKey];

    if (colorButtonId) {
      event.preventDefault();
      activateColor(colorButtonId, toolButtons);
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
  window.addEventListener("rpdf:preferences-changed", onPreferencesChanged as EventListener);
  window.addEventListener("rpdf:canvas-import-pdf-page", onPdfPageImport as EventListener);
  window.addEventListener("rpdf:request-canvas-svg-export", exportSvg as EventListener);
  canvas.addEventListener("pointerdown", onPointerDown);
  canvas.addEventListener("pointermove", onPointerMove);
  canvas.addEventListener("pointerup", onPointerUp);
  canvas.addEventListener("pointercancel", onPointerCancel);
  canvas.addEventListener("wheel", onWheel);
  canvas.addEventListener("contextmenu", onContextMenu);
  updateCursor();

  function isEditableTarget(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) {
      return false;
    }

    if (target.isContentEditable) {
      return true;
    }

    const editableAncestor = target.closest("input, textarea, select, [contenteditable='true']");
    return Boolean(editableAncestor);
  }

  function onPreferencesChanged(event: CustomEvent<{
    defaultStrokeWidth?: number;
    defaultInputQuality?: number;
    defaultShapeKind?: ShapeKind;
    defaultCanvasColor?: string;
  }>) {
    if (typeof event.detail.defaultStrokeWidth === "number") {
      baseStrokeWidth = event.detail.defaultStrokeWidth;
    }

    if (typeof event.detail.defaultInputQuality === "number") {
      inputQuality = clampInputQuality(event.detail.defaultInputQuality);
    }

    if (event.detail.defaultShapeKind === "rectangle" || event.detail.defaultShapeKind === "ellipse" || event.detail.defaultShapeKind === "line" || event.detail.defaultShapeKind === "arrow") {
      selectedShapeKind = event.detail.defaultShapeKind;
      if (selectedTool !== "pen" && selectedTool !== "select" && selectedTool !== "pan" && selectedTool !== "eraser") {
        selectedTool = selectedShapeKind;
        const toolButtons = container.querySelectorAll<HTMLButtonElement>(".tool-picker");
        setActiveToolButton(toolButtons, selectedTool);
        updateCursor();
      }
    }

    if (typeof event.detail.defaultCanvasColor === "string") {
      strokeColor = event.detail.defaultCanvasColor;
    }

    syncStyleControls();
    syncInputQualityControl();
  }

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

  function toCanvasShapeDocument(shape: Shape): CanvasShapeDocument {
    return {
      id: shape.id,
      kind: shape.kind,
      color: shape.color,
      width: shape.baseWidth,
      order: shape.order,
      start: {
        x: shape.start.x,
        y: shape.start.y,
      },
      end: {
        x: shape.end.x,
        y: shape.end.y,
      },
    };
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

  async function importPdfPages(pdfPagePlacements: CanvasPdfPagePlacementDocument[]) {
    const loadedPdfPages = await Promise.all(
      pdfPagePlacements.map(async (placement) => {
        const image = await loadImageFromSource(placement.assetPath);

        return {
          id: placement.id,
          sourcePdfPath: placement.sourcePdfPath,
          pageIndex: placement.pageIndex,
          assetPath: placement.assetPath,
          image,
          x: placement.x,
          y: placement.y,
          width: placement.width,
          height: placement.height,
          recolor: placement.recolor,
        } satisfies CanvasPdfPage;
      }),
    );

    pdfPages.length = 0;
    pdfPages.push(...loadedPdfPages);
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
          id: stroke.id,
          color: stroke.color,
          width: stroke.baseWidth,
          order: stroke.order,
          points: stroke.points.map((point) => ({
            x: point.x,
            y: point.y,
            pressure: point.pressure,
          })),
        })),
        shapes: shapes.map(toCanvasShapeDocument),
        images: images.map((image) => ({
          id: image.id,
          assetPath: image.assetPath,
          x: image.x,
          y: image.y,
          width: image.width,
          height: image.height,
        })),
        pdfPages: pdfPages.map((pdfPage) => ({
          id: pdfPage.id,
          sourcePdfPath: pdfPage.sourcePdfPath,
          pageIndex: pdfPage.pageIndex,
          assetPath: pdfPage.assetPath,
          x: pdfPage.x,
          y: pdfPage.y,
          width: pdfPage.width,
          height: pdfPage.height,
          recolor: pdfPage.recolor,
        })),
      };

      return {
        kind: "canvas",
        document,
        selection: currentSelectionSnapshot(),
      };
    },
    async importDocument(snapshot) {
      if (snapshot.kind !== "canvas") {
        throw new Error("Canvas workspace cannot load a non-canvas document.");
      }

      documentId = snapshot.document.id;
      setSelection([]);
      moveAnchorPoint = null;
      strokes.length = 0;
      shapes.length = 0;
      pdfPages.length = 0;

      strokes.push(
        ...snapshot.document.strokes.map((stroke, index) => ({
          id: stroke.id ?? crypto.randomUUID(),
          color: stroke.color,
          baseWidth: stroke.width,
          order: stroke.order ?? index + 1,
          points: stroke.points.map((point) => ({
            x: point.x,
            y: point.y,
            pressure: point.pressure,
          })),
        })),
      );

      shapes.push(
        ...snapshot.document.shapes.map((shape, index) => ({
          id: shape.id,
          kind: shape.kind,
          color: shape.color,
          baseWidth: shape.width,
          order: shape.order ?? snapshot.document.strokes.length + index + 1,
          start: {
            x: shape.start.x,
            y: shape.start.y,
          },
          end: {
            x: shape.end.x,
            y: shape.end.y,
          },
        })),
      );

      nextVectorOrder = Math.max(
        1,
        ...strokes.map((stroke) => stroke.order + 1),
        ...shapes.map((shape) => shape.order + 1),
      );

      await importImages(snapshot.document.images);
      await importPdfPages(snapshot.document.pdfPages ?? []);
      setSelection(resolveSelection(snapshot.selection));
      syncStyleControls();
      redraw();
    },
    destroy() {
      window.removeEventListener("paste", onPaste);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      window.removeEventListener("resize", resizeCanvas);
      window.removeEventListener("rpdf:preferences-changed", onPreferencesChanged as EventListener);
      window.removeEventListener("rpdf:canvas-import-pdf-page", onPdfPageImport as EventListener);
      window.removeEventListener("rpdf:request-canvas-svg-export", exportSvg as EventListener);
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

function escapeXml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function requireElement<T extends HTMLElement>(root: ParentNode, selector: string) {
  const element = root.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Required element "${selector}" was not found.`);
  }

  return element;
}
