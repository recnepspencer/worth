# Deferred Work

Work the platform has chosen to postpone, across every workspace. Each entry
names what is deferred, where its specification lives, what it would take,
and what should bring it back. An entry leaves this document when its work is
scheduled into an active milestone or completed, not when it becomes
inconvenient. Deferring is a scope decision, never a relaxation: gates,
ceilings, and certification stay as they are, and a deferred claim is not made.

## Worth UI

### 3.16.2 per-frame structural work and the 60 Hz drag qualification

Spec: [milestone-3.16.2.md](worth-ui/milestone-3.16.2.md), items 2a and 3.

A live border drag of Pulse presents continuously and correctly, but slowly.
When deferred, each frame cost about 24 ms of main-thread work, and the
accepted visible-frame gap was about 35 ms at p95 and 50 ms at p99. The
milestone gate is p95 ≤25 ms and p99 ≤50 ms, and the product goal is p99
within one 60 Hz refresh (16.7 ms). The work counters, the headless offscreen
drag (`platform-pulse --worth-ui-offscreen-drag`), and its budget tests are in
place. The budget tests are the working metric; wall-clock timings on a busy
machine are not.

Remaining, highest leverage first:

1. Block-level retained text. A move-only resize frame should touch zero
   glyphs in every text stage; today it re-derives all ~3,000 visible glyphs.
   Needs a move-only paint-command change, demand identity v3 with placement
   separated from content, and block-local GPU glyph buffers drawn at an
   offset.
2. Interned glyph raster keys, replacing hashing of large key structs.
3. Reuse of the async Query admission declaration across frames while its
   basis is unchanged, with a written currentness argument under AP07.
4. Incremental runtime passes.
5. A render thread, after writing the AP07 handoff argument.
6. Vsync-aligned frame start.
7. DXGI Desktop Duplication capture. The GDI capture samples only every
   ~21 ms, so it cannot prove a 16.7 ms gap.

Then run the timed qualification once, on a quiet machine. Bring this back
when drag smoothness becomes a release priority. Steps 1-3 alone are projected
to leave p99 near two refreshes; the render thread is what reaches one.

### 3.16.1 precision-device scrolling qualification

Spec: [milestone-3.16.1.md](worth-ui/milestone-3.16.1.md). Closed under the
OS-neutral functional gate. Qualification on real precision scrolling hardware
remains unverified until that hardware is available; no display-cadence or
latency claim is made.

### 3.17 dimensional operands and bulk expression inputs

Spec: [milestone-3.17.md](worth-ui/milestone-3.17.md), out of scope.

3.17 expressions read text, boolean, integer, and decimal facts, one value per
operand. Two things wait:

- Quantity operands with dimensions, such as `quantity(900.0, mm)`. These need
  Query projections that carry units.
- Column or bulk inputs for evaluating one program over many rows. These need
  the kernel's Phase 5 input API, deferred with 9.17.6.1 below.

Keyed repetition and row-scoped operands are scheduled in 3.18, not deferred.
Bring this back when an authored UI needs a dimensional comparison or a
per-row evaluation cost shows up in a budget test.

## WORTH Query

### 9.17.6.1 phases 4 and 5: the shared expression language's closure

Spec: [milestone-9.17.6.1.md](WORTH-query/milestone-9.17.6.1.md), phases 4-5,
the adversarial courtroom, and completion.

Phases 1-3 are done: the language, the pure evaluator, and the Query
workflow-condition cutover. Worth UI 3.17 adopts that shared language for its
own DSL expressions and ships what the product needs. It does not take on the
language milestone's closure. Deferred:

- Phase 4's full courtroom beyond what 3.17 needs for working DSL expressions:
  the complete stale-generation, world-switch, revocation, and budget-exhaustion
  matrix, and reuse of this milestone's evidence as 3.17's.
- Phase 5 in full:
  - managed element-granular map/filter reuse and bulk adapters;
  - the CAD and digital courts, including the proprietary House unit-bearing
    clearance and member-selection courts and the digital snapshot court moved
    there from phase 3;
  - profile saturation and aggregate memory;
  - registry replacement and cold rebuild;
  - every independent scale axis;
  - the foundational expression docs and the compatibility/deletion inventory.
- The phase 2 gap: AArch64 parity is type-checked only, because this host has
  no AArch64 linker or runner.

Bring this back when an engineering consumer (CAD rules, digital logic) needs
expressions at scale, or before the language is published as a stable
public contract.
