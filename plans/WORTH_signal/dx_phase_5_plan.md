# WORTH Signal DX Phase 5 Plan

## Purpose

Phase 5 is the policy-surface cleanup pass.

The goal is not to delete power.

The goal is to make policy control feel intentional, layered, and easy to steer
without making users learn the runtime's internal decomposition just to get sane
behavior.

This phase should aggressively remove fragmentation:

- one decision should have one obvious owner
- the default path should be persuasive
- advanced overrides should exist, but not sprawl across the product
- policy plumbing should not dominate the day-to-day story

This is a production-grade cleanup phase, not a documentation pass.

---

## Inputs

This phase builds on:

- [`plans/worth_signal/dx_plan.md`](./dx_plan.md)
- [`plans/worth_signal/dx_export_decision_matrix.md`](./dx_export_decision_matrix.md)
- [`plans/worth_signal/dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
- [`plans/worth_signal/dx_condensation_map.md`](./dx_condensation_map.md)
- [`plans/worth_signal/dx_boundary_spec.md`](./dx_boundary_spec.md)
- [`plans/worth_signal/dx_phase_4_review.md`](./dx_phase_4_review.md)

---

## North Star

After Phase 5, policy should feel like this:

- `SignalRuntime::build_for::<Ctx>(graph)` gives a strong recommended default
- named presets express the big posture choices
- advanced tuning exists behind clear doors
- users do not have to set five knobs to express one intention
- there is one obvious place to look when behavior needs to change
- docs teach the preset-first story before exposing low-level control

The runtime should feel like:

- normal path: choose the runtime posture, then work
- advanced path: refine one bounded policy section when needed
- specialist path: reach for raw policy objects only when the higher-level
  policy surfaces are not enough

---

## Scope

Phase 5 covers every policy or policy-like control surface that changes runtime
behavior, retention, execution admission, restore behavior, or merge behavior.

### In Scope

- runtime policy presets and their internal bundles
- diagnostics retention and replay richness
- comparator and comparator-resolver policy
- condition policy and evaluation gating policy
- executor and parallel execution policy
- tier policy
- checkpoint policy and checkpoint barriers
- branch, restore, replay, and merge policy
- any builder, runtime, graph, transaction, history, or diagnostics methods that
  overlap in what they control

### Out Of Scope

- broad docs rewrite beyond the policy story
- unrelated naming cleanup unless a name directly blocks policy clarity
- deep bridge redesign unless a bridge-facing policy surface is clearly
  fragmenting the main runtime story
- new domain starter packs

---

## Policy Families To Audit

These are the families that must be covered explicitly in the audit.

### 1. Runtime Posture

Examples:

- `SignalRuntimePolicy`
- runtime preset constructors like `build_for`, `operational_for`,
  `forensic_for`, `web_development_for`, `fintech_for`
- builder-level runtime policy selection

Questions:

- what should the recommended default actually mean?
- which presets are true first-class product choices versus niche convenience?
- are there overlapping ways to select the same runtime posture?

### 2. Diagnostics Richness And Retention

Examples:

- artifact retention
- replay detail
- provenance retention
- explanation retention
- frontier tracing and related richness knobs

Questions:

- which richness controls should stay bundled under runtime posture?
- which should be advanced overrides?
- are users being exposed to internal retention decomposition too early?

### 3. Execution And Parallelism

Examples:

- `StageExecutor`
- `ParallelExecutionPolicy`
- parallel admission policy
- runtime-level execution defaults versus raw graph execution choices

Questions:

- which decisions belong to runtime posture?
- which belong to an execution section or execution policy object?
- are users being asked to think about executor topology too early?

### 4. Comparator And Semantic Equality

Examples:

- `VersionComparatorPolicy`
- comparator resolver plumbing
- default comparator resolution

Questions:

- what is the normal user meant to control directly?
- what is bridge/framework authoring material?
- are comparator decisions spread across too many entry points?

### 5. Conditions And Evaluation Gating

Examples:

- condition policies
- evaluation triggers
- dependency mode or dirty propagation settings where they affect admission

Questions:

- which of these are declaration-time concerns?
- which are runtime posture concerns?
- which are too low-level for the day-to-day path?

### 6. Tiering And Checkpoints

Examples:

- `TierPolicy`
- `CheckpointPolicy`
- `CheckpointBarrier`
- tier policy tables and supporting knobs

Questions:

- what belongs in the everyday runtime story?
- what belongs in advanced runtime control?
- are tier and checkpoint controls scattered across builder, runtime, and graph
  entry points?

### 7. History, Restore, And Merge Policy

Examples:

- snapshot restore modes
- artifact restore and retention modes
- replay and restore intent
- reconciliation and merge policy

Questions:

- what is the canonical specialist owner for these controls?
- how much should stay in runtime/history/merge entry surfaces versus raw policy
  objects?
- are restore and merge semantics split across too many types?

---

## Core Rules

These are the non-negotiable rules for the phase.

### One Decision, One Owner

If multiple public knobs steer the same conceptual decision, choose one owner.

Examples:

- runtime posture should have one canonical owner
- diagnostics richness should have one canonical owner
- parallel admission should have one canonical owner

Everything else must either forward to that owner or move down a layer.

### Preset First, Raw Second

The published story should prefer:

1. a named preset
2. a bounded advanced section
3. raw policy objects only when necessary

Do not force users into raw structs when they are really just choosing a known
posture.

### No Double-Official APIs

Do not leave two equally respectable ways to do the same thing.

If both must exist temporarily, one must be clearly transitional or specialist.

### Product Before Taxonomy

Policies should be grouped by what users are trying to control, not by how the
 implementation happens to be split internally.

### Defaults Must Sell The Product

The default should help users feel the value of the library quickly.

For this library, that means:

- rich enough diagnostics to feel premium
- safe enough defaults to avoid self-sabotage
- operational lean-down as an explicit choice, not the hidden default

### Bridge Power Must Not Pollute Day-To-Day Use

If a policy concept exists mainly because bridge authors or internal machinery
need it, it does not get equal status in the normal runtime story.

---

## Deliverables

Phase 5 is only complete when all of these exist and are current.

### 1. Policy Inventory

A complete list of current policy knobs with:

- type or method name
- owning module
- current public path
- what decision it actually controls
- current default
- likely audience:
  - daily user
  - advanced runtime user
  - integration author
  - internal/support

### 2. Policy Ownership Map

A clear mapping of each major runtime decision to its canonical owner.

Example decision buckets:

- runtime posture
- diagnostics richness
- execution admission
- parallelism
- semantic equality
- checkpoints
- restore behavior
- merge behavior

### 3. Consolidation Map

For every overlap or fragmentation point:

- current competing controls
- target owner
- whether the fix is:
  - removal
  - containment
  - renaming
  - forwarding alias
  - preset bundling
  - advanced sectioning

### 4. Canonical Policy Story

A short product-facing story for:

- which presets ordinary users should know
- which advanced sections exist
- which raw policy objects remain public and why

### 5. Compatibility Notes

For every meaningful policy cleanup:

- what changes
- what remains temporarily
- what docs/examples must update
- what test coverage must move

---

## Work Plan

Phase 5 should be executed in strict order.

### Step 1. Build The Policy Inventory

Audit every policy surface in:

- `facade`
- runtime constructors and builders
- diagnostics policy
- execution and parallel policy
- state/history/restore policy
- merge and reconciliation policy

Output:

- a source-of-truth inventory document

Exit condition:

- we can point to every public policy knob without guessing

### Step 2. Map Decision Ownership

For each policy knob in the inventory:

- write the actual decision it controls
- identify overlaps
- identify where one decision is split across several knobs
- decide the canonical owner

Output:

- an ownership map, not just a type list

Exit condition:

- every major decision has one named owner

### Step 3. Choose The Public Shape

For each decision family, decide whether the final public shape is:

- preset only
- preset plus bounded advanced section
- advanced section only
- raw specialist object only

This step must be decisive.

Do not leave placeholders like "may condense later."

Output:

- a concrete consolidation plan

Exit condition:

- the target policy shape is explicit enough to code against

### Step 4. Implement Runtime-First Consolidation

Start with the highest-traffic runtime path:

- runtime constructors
- runtime builder
- runtime policy defaults
- diagnostics richness defaults
- execution defaults

This is where daily UX gains are highest.

Output:

- code changes that make the normal runtime path feel simpler and less split

Exit condition:

- ordinary users can choose posture without touching raw policy machinery

### Step 5. Implement Advanced Sectioning

Move remaining legitimate power into bounded specialist sections.

Likely targets:

- comparator tuning
- checkpoint and tier controls
- advanced execution overrides
- history/restore tuning
- merge/reconciliation policy

Output:

- clearer module and method ownership for specialist controls

Exit condition:

- advanced policy exists, but it no longer competes with the normal path

### Step 6. Rewrite The Policy Story In Docs And Examples

Update:

- runtime constructors and rustdoc
- runtime policy docs
- parallel execution docs
- history and replay docs
- merge and specialist docs
- examples that currently over-teach raw policy

Output:

- a preset-first docs story

Exit condition:

- docs teach the same policy model the code actually exposes

### Step 7. Verify And Review

Run:

- `cargo check -p worth-signal`
- `cargo test -p worth-signal`

Then write a Phase 5 review that states:

- what was simplified
- what remains intentionally advanced
- what still needs follow-up in later phases

Exit condition:

- the policy story is clean enough to mark Phase 5 complete with a straight face

---

## High-Risk Smells

If any of these remain after Phase 5, the phase is not actually done.

- users must set several knobs to express one ordinary runtime posture
- runtime posture and diagnostics richness are controlled from unrelated places
- graph, runtime, and builder all feel equally official for the same policy
  decision
- executor and parallel controls show up too early in normal examples
- comparator plumbing is visible before normal users even understand the default
- restore or merge policy is spread across many small raw types with no clear
  guided owner
- docs explain policy taxonomy instead of telling users what to choose

---

## Default Position To Target

Unless the audit proves otherwise, Phase 5 should steer toward this default
shape:

- `SignalRuntime::build_for::<Ctx>(graph)` is the recommended default
- named runtime presets are the main posture choice
- runtime posture owns the default diagnostics richness story
- advanced runtime tuning is grouped into bounded sections, not scattered single
  knobs
- raw policy structs remain available only where they encode real specialist
  control

This is the working hypothesis.

If the audit proves it wrong, change it. Otherwise build toward it.

---

## Exit Criteria

Phase 5 is complete only when all of the following are true:

- every major runtime decision has one obvious public owner
- the default runtime path does not require raw policy machinery
- policy presets tell a coherent product story
- low-level policy details no longer dominate day-one docs or examples
- advanced policy controls still exist, but are clearly bounded and
  non-fragmenting
- bridge/specialist policy no longer pollutes the normal product identity
- the new policy story is verified by compile/test coverage and updated docs

If any of those are false, the phase is still open.
