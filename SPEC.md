# rpdf Specification

## Purpose

`rpdf` is an offline-first desktop study application for one primary user: a technically-minded student who reads dense PDFs, annotates them directly, and takes visual notes with a drawing tablet.

The system must support two equal workflows:

1. visual note-taking on an infinite canvas
2. PDF reading and PDF annotation

The system is not correct if either workflow is treated as a minor extra.

## Required Implementation Platform

The system must be implemented as a Tauri desktop application.

The system must use Rust for the native desktop side of the application.

This includes the parts of the system that are responsible for:

- desktop application packaging and runtime
- local filesystem interaction
- native document handling support
- export and save behavior
- offline-first system integration

The system must not be specified as a browser-only web app, an Electron app, a cloud-first app, or a mobile-first app.

Assumption:

This requirement exists because the project must remain a lightweight desktop application with strong local control, offline behavior, and direct access to native capabilities.

## Product Identity

The identity of `rpdf` comes from combining:

- a tablet-first desktop interaction model
- a minimal and understandable UI
- offline-first operation
- direct PDF annotation
- strong support for personal visual study notes
- better-than-usual reading support for technical PDFs

This is not a generic productivity workspace and not a collaboration platform.

## Intended User

The primary user is a solo electrical engineering student who:

- studies from technical PDFs and textbooks
- annotates documents directly
- takes visual notes
- uses a drawing tablet as a main input device
- wants low resource use
- wants to work offline
- wants text-to-speech support for technical and math-heavy material

This specification does not require support for teams, cloud accounts, or shared editing.

## System Scope

The system must provide:

- an infinite canvas mode
- a PDF mode
- pressure-sensitive pen input
- typed text input
- image import into the canvas
- PDF page import into the canvas
- direct PDF annotation
- PDF text-to-speech
- visual follow-along feedback during text-to-speech
- PDF recoloring for reading comfort
- portable export where the selected content allows it

The system does not need to provide:

- collaboration features
- cloud sync
- internet-dependent core behavior
- office-suite-style PDF editing
- browser deployment as the primary product form
- mobile-first behavior

## Operating Context

The system must operate as a desktop application.

The system must support normal use without internet access.

The system may include optional advanced features that use user-controlled local or self-hosted resources, but the core workflows defined in this specification must not depend on network access.

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
- move and arrange multiple elements on one canvas
- work on a canvas larger than a single fixed page
- choose a background reference pattern

The system must support at least these background pattern categories:

- dots
- lines
- squares

The background pattern must scale with zoom so it remains a usable spatial reference while drawing.

Imported PDF pages in the canvas must behave as placed visual content. Canvas behavior must remain usable whether the source PDF pages are well-structured, messy, or scanned.

## PDF Mode

PDF Mode is a document-reading and document-annotation mode.

The system must allow the user to:

- open a PDF as a document
- navigate through the PDF
- annotate the PDF directly
- listen to PDF content through text-to-speech
- see visual follow-along feedback while text-to-speech is active
- recolor the PDF view for reading comfort

PDF Mode must remain document-focused. It must not behave like an infinite canvas with a PDF placed inside it.

## Inputs

The system must accept these input categories:

- pen input from desktop drawing tablets, including pressure-sensitive drawing
- mouse input
- keyboard input for text entry and commands
- PDF files opened in PDF Mode
- PDF pages imported into Infinite Canvas Mode
- image files imported into Infinite Canvas Mode

Assumption:

This specification assumes support for desktop drawing tablets comparable to Wacom-style devices. It does not require mobile operating system support.

## Outputs

The system must produce these user-visible outputs:

- editable canvas documents
- editable PDF annotation state
- exported annotated PDFs
- exported recolored PDFs when the user chooses that option
- exported SVG files when the selected canvas content is compatible
- visible warnings when text-based reading support is unreliable

The system may use an internal format for autosave and recovery.

The system should prefer portable user-facing outputs over proprietary user-facing outputs whenever practical.

## Core Behaviors

## Pen And Annotation Behavior

The system must allow direct pen-based drawing and markup.

The system must support:

- freehand drawing
- pressure-sensitive strokes
- highlighting
- simple text notes

The annotation workflow must feel direct and low-friction. It must not require a complex editing model for normal study use.

## Typed Text Behavior

The system must allow typed text placement in the infinite canvas.

Typed text is valid canvas content and must be treated as SVG-compatible content when the export target contains only compatible content.

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

The system must prioritize spoken output that is useful for study. It must not treat equations as irrelevant, but it also must not pretend that perfect math interpretation is always available.

## Follow-Along Behavior

When text-to-speech is active, the system must provide visual follow-along feedback.

The system must support configurable highlighting behavior. When source quality allows it, the system must be able to present reading focus in different granularities, such as:

- word-level
- line-level
- sentence-level

If the PDF text layer is unreliable, inconsistent, or missing, the system must not fail silently.

Instead, it must use this fallback order:

1. use reliable native PDF text when available
2. attempt OCR when native text is not reliable enough
3. if neither result is reliable enough, warn the user clearly

If precise text-linked highlighting is not reliable, the system must still provide a visible fallback reading aid rather than pretending exact synchronization exists.

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

If the export target contains incompatible content such as raster images or imported PDF page content, the system must not pretend that full SVG export is supported for that target.

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
- reading PDFs locally
- creating and editing canvas notes
- saving work
- exporting work

## Text Reliability Behavior

The system must distinguish between:

- reliable native PDF text
- weaker OCR-derived text
- unreliable or unavailable text support

The system must communicate these states clearly enough that the user is not misled about reading accuracy.

## Save And Recovery Behavior

The system must protect user work against accidental loss during normal use.

The system must support:

- explicit saving
- autosave or equivalent recovery support
- recovery after interruption where practical

The exact internal save format is not specified here, but user-facing work must remain manageable as normal desktop files.

## Correctness Criteria

The system counts as correct only if all of the following are true:

1. it is a Tauri desktop application that uses Rust for the native desktop side
2. it supports both Infinite Canvas Mode and PDF Mode as first-class workflows
3. it works offline for core study behavior
4. it supports pressure-sensitive pen interaction for drawing and annotation
5. it supports direct PDF annotation
6. it supports text-to-speech in PDF Mode with visible follow-along behavior when reliable enough
7. it uses the required fallback order of native text, then OCR, then clear warning
8. it supports PDF recoloring as a reading aid
9. it supports selection-aware export, including SVG only where the content is compatible
10. it keeps the interface understandable and focused on the intended personal study workflow

## What Can Go Wrong

The specification must explicitly account for these failure cases:

- the PDF has poor or missing text structure
- the PDF is scanned and requires OCR
- OCR output is too weak to trust
- precise follow-along highlighting is not possible
- exported SVG is not valid for mixed raster/PDF content
- annotation colors that work in normal view do not work in recolored view
- optional non-local features are unavailable
- the user is offline

In these cases, the system is correct only if it degrades honestly and visibly instead of pretending the problem does not exist.
