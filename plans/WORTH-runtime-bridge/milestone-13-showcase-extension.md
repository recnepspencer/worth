# Milestone 13 Showcase Extension: Executive-Grade Crisis Simulation And Trust Proof

> **Status:** In Progress
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Milestone parent:** [milestone-13.md](./milestone-13.md)
>
> **Closeout parent:** [milestone-13-closeout.md](./milestone-13-closeout.md)
>
> **Certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** take the now-closed Milestone 13 pricing certification surface and turn it into an executive-grade proof artifact that is still mechanically honest, replay-safe, provenance-complete, and impossible to dismiss as demo theater

## Goal

Turn the pricing-shock certification workload into a showcase-grade simulation
surface that can:

- stage a realistic multi-factor crisis instead of a toy isolated shock
- explain retained branch history, merge history, and shock lineage at any
  retained commit
- surface portfolio-level blast radius and strategy consequences
- fail closed under adversarial trust attacks
- emit one filmable, machine-checkable report artifact that demonstrates all of
  the above without reopening runtime internals

## Why This Exists

Milestone 13 closed the bridge as a certifiable protocol boundary.

That is necessary, but not yet maximally persuasive.

The next step is not a new bridge authority and not a UI milestone.
It is a proof-quality extension that makes the existing certification surface
feel undeniable to skeptical technical leadership:

- this is not a fake spreadsheet demo
- this is not a hand-held branch comparison
- this is not a test harness that secretly knows the answer
- this is a dual-runtime system that can stage crisis, preserve lineage,
  explain causality, compare strategies, survive hostile variation, and prove
  the result offline

## Adversarial Constraint

The showcase extension must survive this hostile condition:

> A seeded multi-factor crisis over at least 100 products, with correlated
> material and logistics streams, interleaved main-branch churn, speculative
> branch-local interventions, tolerance-gated repricing, merge-bearing history,
> replay, restart, and trust-attack injection, must produce one offline
> inspectable showcase artifact that preserves identical causal meaning,
> provenance meaning, portfolio-risk meaning, and typed trust-failure meaning
> across equivalent runs while remaining able to explain any retained commit
> without hidden scenario-local memory.

If the showcase:

- depends on host logs or test comments to explain a crisis
- loses factor provenance at a retained commit
- cannot distinguish portfolio blast radius from raw price movement
- stages a crisis that is economically unstructured or independent by accident
- can be fooled by stale, corrupted, or shadow-protocol-like artifacts
- requires runtime re-execution to explain branch comparison or strategy choice

then the extension has failed.

## Hard Part

The hard part is not making prettier output.

The hard part is keeping the proof honest while making it impressive.

The extension must not introduce:

- bridge-owned economic authority
- bundle fields that cannot be reconstructed from canonical artifacts
- presenter-only summaries that omit the underlying trust evidence
- scenario-local explanation that bypasses retained truth lineage

It must instead elevate the already-honest pricing workload into a stronger
artifact model.

## Phases

### Phase 1: Showcase Artifact And Time-Travel Explorer

Ship one canonical showcase artifact surface on top of the pricing workload.

This phase must add:

- one report-grade bundle export that combines:
  - timeline
  - branch comparison
  - historical provenance
  - merge state
  - discard residue proof
  - writeback authority proof
- one retained-commit explorer model that can answer:
  - what changed here
  - what factors contributed
  - what downstream prices moved
  - what portfolio metrics changed
- one canonical markdown or JSON export suitable for filming, screenshots, or
  offline inspection

Phase 1 is complete only when one retained commit can be inspected from the
showcase artifact alone without touching hidden scenario state.

### Phase 2: Multi-Factor Crisis And Portfolio Decision Surface

Replace the single-shock feel with a true crisis model.

This phase must add:

- one multi-factor crisis lane combining at least:
  - raw material pressure
  - fuel/logistics pressure
  - tariff or policy pressure
  - supplier-style discrete shock
- correlated regime-aware stream behavior across those inputs
- portfolio-level blast radius metrics such as:
  - SKUs breaching repricing thresholds
  - SKUs breaching margin floors
  - category-level margin erosion
  - top causal materials by exposure
  - shipping-sensitive versus material-sensitive product families
- strategy comparison outputs for:
  - hold
  - reprice
  - speculative merge or promotion choice

Phase 2 is complete only when the showcase artifact can explain not just "what
happened" but "what matters" for the portfolio.

### Phase 3: Trust Attacks And Executive Demo Flow

Make the showcase impossible to dismiss as a happy-path performance.

This phase must add:

- explicit trust-attack lanes such as:
  - stale historical comparison basis
  - corrupted or drifted replay basis
  - wrong-policy re-evaluation
  - fake-equivalent merge
  - shadow-protocol-like mapper or provenance mismatch
- one scriptable executive demo flow that walks:
  - stable live world
  - crisis fork
  - split reality
  - retained lineage inspection
  - strategy comparison
  - commit or discard outcome
  - replay and trust proof
- one final showcase artifact family that can be emitted for:
  - control
  - crisis
  - hostile trust attack
  - replay

Phase 3 is complete only when the executive demo can be driven from canonical
artifact generation and the hostile lanes fail typed and explainably.

## Must Ship

- one showcase-grade pricing report artifact
- one retained historical commit explorer surface for the pricing workload
- one multi-factor crisis scenario over the generated domain world
- one portfolio blast-radius model
- one strategy comparison surface
- one trust-attack matrix over the showcase artifacts
- one scriptable executive demo flow driven from canonical bundle generation

## Must Preserve

- truth remains authoritative in `worth-relational`
- computation remains derived in `worth-signal`
- the bridge remains a protocol boundary rather than an economics engine
- every showcase summary must remain mechanically attributable to canonical
  truth and bridge artifacts
- diagnostics richness may change retained detail but not strategy meaning,
  crisis meaning, or provenance meaning

## Acceptance Evidence

The showcase extension is complete only when:

- one canonical showcase artifact can explain a retained crisis commit
- one multi-factor crisis lane produces correlated, portfolio-relevant effects
- branch comparison, merge history, and writeback authority all appear in the
  same filmable artifact family
- hostile trust-attack lanes fail typed and offline-diagnosably
- replay preserves showcase artifact meaning across equivalent runs
- the executive demo flow can be regenerated from canonical artifacts alone

## Architectural Notes

This extension should prefer dedicated harness-side subdomains such as:

- `harness/tests/pricing_showcase/`
- `harness/tests/pricing_showcase/artifact.rs`
- `harness/tests/pricing_showcase/timeline.rs`
- `harness/tests/pricing_showcase/portfolio.rs`
- `harness/tests/pricing_showcase/demo_flow.rs`
- `harness/tests/pricing_showcase/trust_attacks.rs`

Do not collapse these into one giant `pricing_shock.rs` file.

The showcase artifact is allowed to be richer than the milestone-close bundle,
but it must remain derived from the same canonical evidence, not a parallel
explanation system.

## Sequencing Notes

This extension belongs after Milestone 13 close because it depends on the
bridge already being certifiable.

It belongs before any broader public demo or UI work because it turns the
closed pricing reference workload into the strongest possible proof artifact
without weakening architectural honesty.

## Self-Check

- This solves a real structural problem: persuasion and proof quality are still
  weaker than they could be even though Milestone 13 is closed.
- The adversarial constraint is precise and load-bearing: a crisis artifact
  that cannot survive provenance, replay, and trust attack pressure is not a
  serious showcase.
- Authority boundaries are preserved: economics remain in the harness domain;
  the bridge keeps protocol ownership only.
- The spec defines proof obligations, not aesthetics.
- A competent engineer can map this into honest modules, types, and tests.
