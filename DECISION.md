# DECISION

## Context

This decision pass compared the candidate plans in `plans/` for `rpdf`, with `SPEC.md` as the governing product constraint.

The plans considered in this pass were:

- `plans/PLANSIMPLE.md`
- `plans/PLANFAST.md`
- `plans/PLANLEAN.md`
- `plans/PLANROBUST.md`
- `plans/PLANUX.md`

The main tradeoffs were:

- fastest delivery versus stronger long-term structure
- immediate PDF engine commitment versus a later architecture gate
- prototype-first implementation versus a more durable foundation

The user answered:

- `1. robust`
- `2. fast`
- `3. robust`

This means the chosen direction is:

- use the robust long-term architecture bias as the main planning stance
- accept a faster early PDF-engine commitment now
- prefer a durable implementation foundation over a throwaway-first prototype path

## Decision Records

### D-001

- Decision ID: D-001
- Topic: Overall planning bias
- Chosen option: Use the robust long-term architecture direction as the primary planning stance
- Source plan or plan sections: `plans/PLANROBUST.md`, contrasted with `plans/PLANFAST.md`, `plans/PLANSIMPLE.md`, and `plans/PLANUX.md`
- What the user said that led to the choice: The user answered `1. robust`.
- Why it was accepted: This directly answers the main tradeoff between fastest delivery, strongest long-term structure, and UX-first implementation bias. It fits the project's hard parts: PDF reliability, annotation/export correctness, OCR fallback honesty, and future math-aware reading support.
- Competing alternatives considered: Faster prototype bias from `plans/PLANFAST.md` and `plans/PLANSIMPLE.md`; UX-first bias from `plans/PLANUX.md`
- Why those alternatives were rejected, delayed, or only partially used: The fast plans sacrifice maintainability and clean internal models too aggressively for a project where reading reliability, recovery, and document behavior are central. The UX-first plan remains useful, but it was not chosen as the top-level planning bias because the user explicitly preferred robustness over UX-first prioritization.
- Consequences of the choice: The plan should use clearer subsystem boundaries, stronger internal models, more explicit reliability state handling, and a slower, more deliberate implementation sequence.
- Conditions that should trigger revisiting the decision: Revisit if the robust architecture meaningfully blocks progress on the first usable offline desktop build, or if the user later decides that early workflow validation matters more than durable internal structure.

### D-002

- Decision ID: D-002
- Topic: PDF engine decision timing
- Chosen option: Commit early to the faster PDF-engine path now instead of keeping the engine undecided as a later architecture gate
- Source plan or plan sections: `plans/PLANFAST.md`, `plans/PLANSIMPLE.md`, contrasted with `plans/PLANROBUST.md` and `plans/PLANLEAN.md`
- What the user said that led to the choice: The user answered `2. fast`.
- Why it was accepted: This directly answers the question of whether to commit now to the pragmatic engine route, stay conservative with system-tool-only integration, or defer the engine choice. The user's answer selects speed at this specific decision point.
- Competing alternatives considered: Keep the engine undecided for later review from `plans/PLANROBUST.md`; use Linux-first system tools and dependency minimization from `plans/PLANLEAN.md`
- Why those alternatives were rejected, delayed, or only partially used: Deferring the engine keeps architectural optionality, but slows concrete implementation. The lean/system-tools path reduces dependency and license risk, but it likely gives up capability and cross-workflow consistency too early. The user explicitly chose the faster path here.
- Consequences of the choice: The project should proceed assuming a Pdfium-style Rust-integrated PDF path, as proposed by `plans/PLANFAST.md` and `plans/PLANSIMPLE.md`. This increases bundling and integration responsibility, and it accepts earlier lock-in in exchange for faster implementation progress.
- Conditions that should trigger revisiting the decision: Revisit if Pdfium bundling becomes a serious blocker, if annotation/export capability is insufficient, if thread-safety or packaging issues become costly, or if later license/capability review shows a materially better option.

### D-003

- Decision ID: D-003
- Topic: First-version foundation versus throwaway prototype bias
- Chosen option: Build on a durable foundation rather than optimizing for a rewrite-likely prototype
- Source plan or plan sections: `plans/PLANROBUST.md`, contrasted with `plans/PLANFAST.md` and `plans/PLANSIMPLE.md`
- What the user said that led to the choice: The user answered `3. robust`.
- Why it was accepted: This directly resolves whether the first serious implementation should tolerate rough persistence and weak internal structure for speed, or whether it should establish durable models for documents, annotations, reliability states, and recovery from the beginning.
- Competing alternatives considered: Prototype-first approach from `plans/PLANFAST.md`; smallest-correct implementation bias from `plans/PLANSIMPLE.md`
- Why those alternatives were rejected, delayed, or only partially used: Those alternatives are attractive for speed, but they increase the chance of later rewrites in exactly the subsystems that are central to this product: persistence, annotation behavior, export correctness, and reading-support trust states. The user explicitly preferred robustness here.
- Consequences of the choice: The project should use stronger internal document/session models, explicit reliability states, and cleaner save/recovery boundaries earlier than a fast prototype plan would.
- Conditions that should trigger revisiting the decision: Revisit if the robust foundation work delays the first usable end-to-end build too much, or if the user later prefers early validation of the workflow over internal durability.

### D-004

- Decision ID: D-004
- Topic: Relationship between robustness and speed in the selected direction
- Chosen option: Use a hybrid direction: robust architecture overall, but do not keep the PDF-engine decision open
- Source plan or plan sections: `plans/PLANROBUST.md` plus the Pdfium-leaning engine choices in `plans/PLANFAST.md` and `plans/PLANSIMPLE.md`
- What the user said that led to the choice: The user chose `robust` for overall direction, `fast` for engine commitment, and `robust` again for first-version foundation.
- Why it was accepted: The three answers together do not support any single plan wholesale. They support an assembled direction: strong internal structure plus an early concrete engine choice.
- Competing alternatives considered: Pure robust path with undecided engine gate from `plans/PLANROBUST.md`; pure fast path from `plans/PLANFAST.md`; pure simple path from `plans/PLANSIMPLE.md`
- Why those alternatives were rejected, delayed, or only partially used: The pure robust path conflicts with the user's explicit preference for the faster engine decision now. The pure fast and pure simple paths conflict with the user's explicit preference for robustness at the architectural and foundation levels.
- Consequences of the choice: The future `PLAN.md` should be assembled mainly from `plans/PLANROBUST.md`, but should pull the early Pdfium commitment and pragmatic engine-forward execution stance from `plans/PLANFAST.md` and `plans/PLANSIMPLE.md`.
- Conditions that should trigger revisiting the decision: Revisit if the robust architecture and the early engine lock-in begin to work against each other in practice, for example if the selected engine forces brittle implementation shortcuts that undermine the robustness goal.

### D-005

- Decision ID: D-005
- Topic: Status of UX-first ideas
- Chosen option: Keep UX-first plan material as a secondary influence, not the main decision driver
- Source plan or plan sections: `plans/PLANUX.md`
- What the user said that led to the choice: The user did not choose the UX-first option when asked for the main planning bias.
- Why it was accepted: The project still depends heavily on mode clarity, pen-first ergonomics, honest reliability messaging, and study comfort, but the user did not prioritize UX-first planning over robustness.
- Competing alternatives considered: Make the user-experience-first plan the primary planning stance
- Why those alternatives were rejected, delayed, or only partially used: The user explicitly preferred robustness. UX concerns remain important, but they should be incorporated where they do not conflict with the stronger architecture and durability choices.
- Consequences of the choice: The eventual assembled plan may still borrow specific UX guidance from `plans/PLANUX.md`, especially around mode separation, pen-first behavior, and trust-state communication, but these should support the robust architecture rather than redefine it.
- Conditions that should trigger revisiting the decision: Revisit if the app becomes structurally correct but unpleasant or unclear to use in the intended tablet-first study workflow.

## Deferred And Open Items

- The exact Pdfium integration method is still not fully specified here. The decision is to commit early to that direction, not to lock every low-level implementation detail yet.
- `plans/PLANLEAN.md` is not selected as the main direction, but its warnings about dependency burden, packaging simplicity, and license risk remain relevant as review criteria.
- `plans/PLANUX.md` is not selected as the top-level plan, but its pen-first and trust-surface guidance remains eligible for selective reuse later.
