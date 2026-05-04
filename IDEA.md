# rpdf

## One-line idea

`rpdf` is a minimal desktop app for personal visual note-taking and PDF annotation, built for drawing-tablet use and strong PDF text-to-speech support for math-heavy study material.

## Who this is for

This project is primarily for one type of user: you.

More specifically, it is for an electrical engineering student who:

- studies from dense technical PDFs and textbooks
- wants to annotate PDFs directly
- wants a better place for visual note-taking
- uses a drawing tablet as a primary input device
- cares about low resource use and a simple interface
- needs better text-to-speech support than current PDF tools usually provide
- wants the tool to work offline

The project does not need to begin as a general-purpose product for everyone. It should first solve this workflow well for a technically-minded solo user with clear needs.

## Why this should exist

Current note-taking and PDF tools do not match the workflow you want.

The gap is not just "PDF annotation" or just "infinite canvas note-taking." The gap is the combination of:

- a visual workspace for freeform notes
- direct PDF annotation
- good desktop support for drawing tablets
- useful text-to-speech while reading technical material
- offline-first use
- low complexity and low resource use

Most existing tools are too heavy, too general, too weak at tablet-first desktop interaction, or not good enough at reading support for technical PDFs with math.

## Core product identity

`rpdf` is not mainly a keyboard-centric productivity tool.

It is a desktop application with a minimal UI, understandable features, and tablet-first interaction. The user should be able to spend most of their time drawing, marking, and reading, without fighting a cluttered interface.

Its identity comes from combining two separate but related modes:

1. an infinite canvas for visual notes
2. a PDF mode for reading and annotation

These are both core parts of the product. They should be treated as separate modes, not as one mode awkwardly pretending to be the other.

## Mode 1: Infinite canvas

The infinite canvas is a freeform visual note-taking space.

In this mode, the user should be able to:

- draw using a drawing tablet
- use pressure sensitivity
- write typed text with the keyboard
- import images
- import one or more PDF pages
- arrange visual material spatially
- mix handwriting, drawing, typed notes, and imported content in one workspace
- use configurable background reference patterns such as dots, lines, or squares

This mode is for thinking, studying, collecting fragments, and building personal visual notes.

It should feel open and flexible, but still simple.

The background pattern is not just decorative. It should scale with zoom so it remains a useful spatial reference while drawing and measuring by eye.

## Mode 2: PDF mode

PDF mode is a separate reading and annotation mode.

In this mode, the user opens a PDF and interacts with it as a document, not as an infinite canvas.

The purpose of this mode is:

- reading PDFs
- annotating PDFs directly
- listening to PDFs with text-to-speech
- following the current spoken content visually
- visually recoloring the PDF for more comfortable reading

This mode should stay focused. It is not supposed to turn every PDF into a canvas workspace. That separation is important for conceptual clarity.

## Annotation expectations

Annotation is a core requirement in both the broader project and the PDF workflow.

The product should support:

- handwritten annotation
- pressure-sensitive drawing
- highlighting
- simple text notes
- fast, direct markup without a complicated editing model

The main goal is to make annotation feel natural for drawing-tablet use, especially on desktop systems with devices such as Wacom-style tablets.

Users should also be able to customize annotation colors for both:

- normal viewing
- recolored viewing

This matters because annotation colors that work well on a dark recolored page may not work well on a normal light page, and vice versa.

## Text-to-speech as a core feature

Text-to-speech is not an optional extra. It is part of the product identity.

You want a PDF reader that actually helps with reading, not just a viewer with a basic voice feature attached to it. This matters especially for textbooks and technical documents that contain a lot of math.

In product terms, reading and annotation are equally important in PDF mode. The app should support careful study reading and direct pen-based thinking as two equally central parts of the same workflow.

The intended experience is:

- the PDF can be read aloud
- the user can keep following along visually
- the currently spoken content is highlighted
- the highlight style can be user-controlled

The highlighting should ideally support multiple modes, such as:

- word-by-word
- line-by-line
- sentence-by-sentence

This flexibility matters because different documents and different reading situations may call for different guidance styles.

If a PDF has unreliable or missing text structure, text highlighting should not simply fail silently. The product should be able to fall back toward more manual behavior, including a highlighting/drawing style that still lets the user track content visually even when reliable text-linked highlighting is not available.

If the text layer is poor, the app should try OCR. If OCR is still not good enough, the app should warn the user clearly instead of pretending the result is reliable.

## Recoloring as a core reading aid

PDF recoloring is an important part of the reading experience, especially for studying at night and reducing visual strain.

It should work first as a temporary viewing aid, but it should not be limited to display only. If the user wants, the product should also support saving or exporting a recolored result rather than treating recoloring as purely temporary.

This means the idea includes both:

- recoloring as a viewing mode
- recoloring as an optional output choice

Recoloring should also apply to PDF pages imported into the infinite canvas, where it should be controllable per page or across multiple selected pages.

## Math-heavy reading support

Support for mathematical content is a very important part of the idea.

The project is motivated partly by the fact that many technical readers do not have good options when they want text-to-speech for documents containing substantial mathematical notation.

This means `rpdf` should aim to be useful for:

- engineering textbooks
- lecture materials
- academic PDFs
- math-heavy technical documents

The idea should not treat math as a rare edge case. For this product, it is part of the normal expected workload.

The goal is to read math as well as possible. Conceptually, this means the product should try to turn mathematical content into spoken output that feels natural enough for actual studying, rather than only spelling out symbols mechanically. The exact mechanism does not belong in the idea stage, but the user-facing promise does: math should be handled as intelligently as possible, not ignored or treated as an afterthought.

## Input model

The app is primarily for desktop use with a drawing tablet.

Important implications:

- support for devices like Wacom-style tablets is essential
- pressure sensitivity is essential
- mouse-and-keyboard use may still exist, but it is not the primary design center
- heavy dependence on keyboard-only workflows is not desirable

The user may use the keyboard for typed text entry, but the interaction model should not assume that the keyboard is the dominant input device.

## Offline-first use

`rpdf` is intended to be an offline-first personal tool.

Normal reading, annotation, canvas work, export, and text-to-speech should not depend on internet connectivity.

If advanced AI-assisted features ever exist, they should still fit the same personal/offline spirit as much as possible, for example by working through local or self-controlled infrastructure rather than assuming a cloud product.

## Export philosophy

The project should avoid unnecessary proprietary formats for normal user work.

An internal format may still exist for autosave and recovery, but the user-facing workflow should prefer standard and portable outputs whenever possible.

## SVG export from vector-based canvas content

If the canvas content is made only from vector-style drawing, typed text, and other SVG-compatible native elements, the user should be able to export it as SVG.

If the whole canvas contains raster images or imported PDF content, full-canvas SVG export may no longer make sense. However, export should be selection-aware:

- by default, export assumes the whole canvas
- if the user selects specific items, export should apply to that selection
- if the selected items are fully vector and text based, SVG export should be available

This allows mixed canvases to still produce clean SVG output for the parts that support it.

## Minimalism in this project

For this project, "minimal" means:

- low resource use
- simple understandable features
- minimal UI
- low clutter
- direct interaction

It does not mean removing important capabilities.

The right interpretation is:

"Include the features that matter deeply to this workflow, but present them in a simple and lightweight way."

## Product boundaries

`rpdf` should be:

- a personal study and note-taking tool
- desktop-first
- drawing-tablet-first
- visual
- lightweight
- focused on solo use
- offline-first
- based on portable user-facing outputs where practical

`rpdf` should not try to become:

- a full office-style PDF suite
- a large collaboration platform
- a general-purpose cloud notebook ecosystem
- a bloated all-in-one productivity app
- a keyboard-obsessed tool that ignores pen-based workflows

## Key assumptions

The current idea assumes:

- the main user flow is solo study and note-taking
- desktop drawing tablets matter while touch-first mobile tablets should be ignored
- the value comes from combining visual notes and PDF reading in one coherent tool
- strong TTS support is important enough to shape the product identity
- math-heavy documents are common, not exceptional
- simplicity and low resource use matter more than feature breadth
- visual comfort features such as recoloring are part of the core workflow
- annotation should remain dependable even when PDF text extraction is poor
- OCR fallback is acceptable when native PDF text is weak

## Open tensions to keep in mind

These are not implementation decisions yet, but they are idea-level tensions that will matter later:

### 1. One product, two modes

The project clearly has two core modes. That is good, but it raises a design challenge: they should feel related without being confused with each other.

### 2. Minimalism versus capability

The app should stay simple, but it also needs serious features:

- pressure-sensitive drawing
- PDF annotation
- TTS
- math-aware reading support
- recoloring
- selection-aware export

The project must avoid both extremes:

- becoming too bare to be useful
- becoming too complex to stay minimal

### 3. Technical PDF quality

The product should assume that technical PDFs may be difficult, especially when they contain complex layouts or mathematical notation. That does not weaken the idea, but it does mean the product promise should stay grounded in "helpful reading support for technical PDFs" rather than unrealistic perfection across all documents.

### 4. Reliable study workflow versus unreliable PDF text

The product should not let weak PDF text extraction destroy the study workflow. Annotation and visual study behavior should remain dependable even for scanned or messy PDFs, while TTS and text-linked highlighting should use the best available path:

- native text when reliable
- OCR when needed
- clear warning when neither is good enough

## Working idea statement

`rpdf` is a minimal desktop application for personal visual note-taking and PDF study. It has two separate core modes: an infinite canvas for freeform visual notes and a PDF mode for reading and direct annotation. It is built around drawing-tablet input with pressure sensitivity, low resource use, a minimal UI, and offline-first use. A major part of its identity is strong text-to-speech support for PDFs, including math-heavy technical material, with configurable visual highlighting that helps the user follow along while listening.

It also emphasizes visual comfort through PDF recoloring, configurable annotation appearance across normal and recolored viewing, scalable reference-pattern backgrounds in the canvas, and export behavior that favors portable formats such as SVG whenever the selected canvas content supports it.
