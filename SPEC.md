# rpdf Specification

## Purpose

`rpdf` is an offline-first desktop application for personal study work.

It must support two equal workflows:

1. visual note-taking on an infinite canvas
2. PDF reading and PDF annotation

The system must be optimized for a drawing tablet with pressure sensitivity and a minimal, understandable interface. It must remain useful even when PDF text extraction is weak or unavailable.

## Intended User

The primary user is a solo electrical engineering student who:

- studies from technical PDFs and textbooks
- annotates documents directly
- takes visual notes
- uses a drawing tablet as a main input device
- wants low resource use
- wants to work offline
- wants text-to-speech support for technical and math-heavy material

This specification does not require support for collaboration, cloud sync, or multi-user workflows.

## System Scope

The system must provide:

- an infinite canvas mode
- a PDF mode
- pressure-sensitive pen input
- typed text input
- PDF page import into the canvas
- image import into the canvas
- PDF annotation
- PDF text-to-speech
- visual follow-along highlighting during text-to-speech
- PDF recoloring for comfortable reading
- portable export where the selected content allows it

The system does not need to provide:

- collaboration features
- cloud accounts
- internet-dependent core functionality
- office-suite-style PDF editing
- broad document management features
- mobile or touch-tablet-first behavior

## Operating Context

The system must operate as a desktop application.

The system must support normal use without internet access.

The system may include optional advanced features that use user-controlled local or home-server resources, but the core workflows defined in this specification must not depend on network access.

## Modes

## Infinite Canvas Mode

Infinite Canvas Mode is a freeform workspace for visual study notes.

The system must allow the user to:

- create a new canvas
- draw with a pen device
- vary stroke output with pressure sensitivity
- place typed text
- import images
- import one or more PDF pages
- move, arrange, and keep multiple elements on one canvas
- work on a canvas larger than a single fixed page
- choose a background reference pattern

The system must support at least these background pattern categories:

- dots
- lines
- squares

The background pattern must scale with zoom so it remains a usable size reference while drawing.

Infinite Canvas Mode must remain usable whether the imported PDF pages are well-structured, messy, or scanned, because imported PDF pages in this mode are treated as placed visual elements.

## PDF Mode

PDF Mode is a document-reading and document-annotation mode.

The system must allow the user to:

- open a PDF as a document
- navigate through the PDF
- annotate the PDF directly
- listen to PDF content through text-to-speech
- see visual follow-along feedback while text-to-speech is active
- recolor the PDF view for reading comfort

PDF Mode must remain a document-focused mode. It must not behave like an infinite canvas.

Reading and annotation are equally important in this mode. The system is not correct if one of those workflows is treated as a minor extra.

## Inputs

The system must accept these input categories:

- pen input from desktop drawing tablets, including pressure-sensitive drawing
- mouse input
- keyboard input for text entry and commands
- PDF files opened in PDF Mode
- PDF pages imported into Infinite Canvas Mode
- image files imported into Infinite Canvas Mode

Assumption:

This specification assumes support for desktop drawing tablets comparable in behavior to Wacom-style devices. It does not require support for mobile operating systems or touchscreen tablets.

## Outputs

The system must produce these user-visible outputs:

- editable canvas documents
- editable PDF annotation state
- exported annotated PDFs
- exported recolored PDFs when the user chooses that option
- exported SVG files when the selected canvas content is compatible
- visual warnings when text-based reading support is unreliable

The system may use an internal format for autosave and recovery.

The system should prefer portable user-facing outputs over proprietary user-facing outputs whenever that is practical.

## Core Behaviors

## Pen And Annotation Behavior

The system must allow direct pen-based drawing and markup.

The system must support:

- freehand drawing
- pressure-sensitive strokes
- highlighting
- simple text notes

The annotation workflow must feel direct and low-friction. It must not require a complex editing process for normal study use.

## Typed Text Behavior

The system must allow typed text placement in the infinite canvas.

Typed text is valid canvas content and must be treated as exportable SVG-compatible content when the selected export target contains only compatible content.

## PDF Import Into Canvas

The system must allow importing one or more PDF pages into the infinite canvas.

Imported PDF pages in the canvas must behave as placed visual content.

The system must support recoloring imported PDF pages in the canvas.

The system must allow recoloring control:

- per imported page
- across multiple selected imported pages

## PDF Annotation Behavior

The system must allow annotation of opened PDFs.

The system must keep PDF annotation usable for:

- normal text PDFs
- scanned PDFs
- messy PDFs with weak structure

If text extraction is poor, the annotation workflow must still remain dependable.

## Text-To-Speech Behavior

The system must support text-to-speech in PDF Mode.

The system must aim to read math-heavy technical material as well as possible.

This means the user-visible behavior must prioritize spoken output that is useful for study, rather than treating equations as irrelevant or reducing all mathematics to an obviously broken reading experience.

This specification does not require perfect math interpretation for every PDF.

## Follow-Along Highlighting Behavior

When text-to-speech is active, the system must provide visual follow-along feedback.

The system must support configurable highlighting behavior. The supported behavior must include the ability to present the current reading focus in different granularities when the source text quality allows it.

Examples of allowed granularities include:

- word-level
- line-level
- sentence-level

If the PDF text layer is unreliable, inconsistent, or missing, the system must not silently fail.

Instead, it must use this fallback order:

1. use reliable native PDF text when available
2. attempt OCR when native text is not reliable enough
3. if neither result is reliable enough, warn the user clearly

If precise text-linked highlighting is not reliable, the system must still provide a usable visual fallback for tracking content, such as a more manual highlighting-style tool or equivalent visible reading aid.

## Recoloring Behavior

The system must support PDF recoloring as a reading aid.

Recoloring must work as a viewing feature during normal reading.

The system must also allow the user to choose whether a saved or exported annotated PDF includes recoloring.

The system must allow annotation appearance to remain visible and usable in both:

- normal viewing
- recolored viewing

The system must let the user customize annotation colors for those two viewing contexts separately.

## Canvas Export Behavior

The system must support export from Infinite Canvas Mode.

By default, export must target the whole canvas.

If the user selects specific items, export must target the selection rather than the whole canvas.

The system must allow SVG export when the export target contains only SVG-compatible native content, including at least:

- vector drawing
- typed text

If the export target contains incompatible content such as raster images or imported PDF page content, the system must not pretend that SVG export is fully supported for that target.

In that case, the system must either:

- disable SVG export for that target
- or clearly indicate that SVG export is unavailable for that target

The system must still allow SVG export for a vector-only selection taken from a mixed-content canvas.

## Reliability And Failure Behavior

## Offline Behavior

The system must remain usable offline for its core workflows.

If any optional feature is unavailable because it depends on a non-local service, that failure must not break:

- opening PDFs
- annotating PDFs
- working on the canvas
- exporting supported outputs
- standard text-to-speech if that feature is defined as core on the local machine

## Poor PDF Text Structure

If a PDF has poor text structure, the system must:

- keep annotation available
- attempt OCR for text-based reading support
- warn clearly when reliable reading support cannot be produced

The system must not mislead the user into believing that text-linked TTS or highlighting is reliable when it is not.

## Unsupported Export Target

If the user requests SVG export for a target that contains incompatible content, the system must refuse that export clearly rather than produce a misleading partial result without explanation.

## Visibility Failures

If recoloring would make current annotation colors hard to see, the system must provide a way for the user to adjust annotation appearance for recolored viewing.

## Assumptions

- The product is for a solo user.
- The product is offline-first.
- Canvas and PDF mode are both first-class.
- Desktop drawing tablet support is essential.
- Keyboard-first interaction is not the design center.
- PDF text quality varies widely.
- OCR fallback is acceptable when native PDF text is weak.
- Math-heavy reading support is important, but perfect math interpretation is not required for correctness.

## Non-Goals

- cloud synchronization
- collaboration
- broad document management
- internet-required operation
- full office-style PDF editing
- replacing every existing note-taking or PDF workflow

## Correctness Criteria

The system counts as correct only if all of the following are true:

1. The user can use Infinite Canvas Mode for pen-based visual note-taking with pressure-sensitive drawing, typed text, imported images, imported PDF pages, and scalable reference backgrounds.
2. The user can use PDF Mode for both annotation and reading support without one workflow depending on the other being absent.
3. The user can annotate scanned or messy PDFs even when text extraction is poor.
4. The system attempts native text first, then OCR, then a clear warning for unreliable text-based reading support.
5. PDF recoloring works as a reading aid and can be optionally included in exported output.
6. Annotation appearance can be configured separately for normal and recolored viewing.
7. SVG export works for compatible canvas targets and is clearly unavailable for incompatible targets.
8. Core workflows do not require internet access.
9. The interface remains consistent with a minimal, study-focused desktop tool rather than a broad office suite.

## Acceptance Checks

These checks define observable acceptance behavior for the system.

### Canvas checks

- A user can create a canvas, draw with pressure-sensitive pen input, place typed text, import an image, and import a PDF page.
- A user can zoom in and out and the chosen dots, lines, or squares background remains a meaningful size reference.
- A user can select only vector-and-text content from a mixed canvas and the system offers SVG export for that selection.
- A user can attempt SVG export on an incompatible target and receives a clear refusal or unavailable state.

### PDF checks

- A user can open a PDF, annotate it, and use TTS in the same mode.
- A user can switch on recoloring for comfort while reading.
- A user can choose whether exported output includes recoloring.
- A user can set annotation appearance for normal view and recolored view separately.

### Robustness checks

- A user can open a scanned or messy PDF and still annotate it without depending on text extraction.
- A user can trigger TTS on a PDF with weak native text and the system tries OCR before giving up.
- A user receives a clear warning when the system cannot provide reliable text-based reading support.
- A user can continue core work without internet access.
