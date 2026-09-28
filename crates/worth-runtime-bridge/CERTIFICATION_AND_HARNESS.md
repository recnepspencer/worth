# Certification And Harness

This guide explains how the bridge proves itself.

`worth-runtime-bridge` is not just supposed to run.
It is supposed to be certifiable.

That means the bridge should be able to demonstrate:

- deterministic routing
- explicit truth-view basis
- speculative isolation
- discard zero-residue
- promotion authority boundaries
- replay-safe retained evidence

## Why The Harness Exists

The harness is where the bridge's architecture gets pressure-tested under real,
adversarial workloads rather than only narrow feature tests.

For Milestone 13, the reference workload is the pricing-shock matrix:

- shared component costs
- 100-product fanout
- live shared-input cost churn
- speculative `rubber +300%` branch shock
- interleaved main-branch updates
- discard or promotion
- canonical replay and restart recovery
- hostile failure injection

The important implementation detail now is that this workload is being
assembled as one top-level workload certification bundle rather than a pile of
scenario-local assertions. The bundle is expected to collect:

- ordinary-path reference and replay artifacts
- aspect-aware routing artifacts
- hostile missing-basis failure artifacts
- discard lifecycle artifacts
- promotion lifecycle artifacts
- 100-product fanout artifacts
- restart replay artifacts
- restart drift-rejection artifacts
- writeback authority artifacts covering commit, noop, and typed rejection
- merge/history artifacts covering aspect-aware reconciliation and revisitable
  pre-merge, speculative, and merged truth states
- historical provenance artifacts carrying retained shock criteria for
  bridge-readable commit lineage

The current harness direction also expects that bundle to expose:

- a structured summary export suitable for offline inspection
- a stable bundle digest suitable for parity and replay comparison
- explicit suite-shaped certification artifacts for Milestone 13 suites 25
  through 27
- one representative exact counter snapshot for the milestone-level bundle
  obligations
- one historical provenance surface that can explain a retained pricing commit
  from canonical truth artifacts alone

## What Certification Means

Certification is not just "the final prices matched."

Certification means the bridge preserved:

- causality meaning
- failure meaning
- branch identity meaning
- residue meaning
- replay meaning

For the pricing-shock workload, that should be diagnosable from the emitted
bundle without reopening the runtime.

In practice, that now means the workload should be able to hand an auditor:

- one bundle summary
- one bundle digest
- one suite 25 causality artifact
- one suite 26 failure-localization artifact
- one suite 27 certification-sufficiency artifact
- one exact representative counter snapshot
- one historical provenance artifact

and let them answer the major Phase 3 questions without replaying the whole
scenario by hand.

If those drift while top-line values happen to match, the bridge has not really
proved itself.

## Public Surfaces Involved

This area leans heavily on the retained-record and replay portions of
`worth_runtime_bridge::facade`, including canonical records and replay-safe
artifacts.

Examples include:

- `BridgeCanonicalRouteRecord`
- `BridgeCanonicalHistoricalEvaluationRecord`
- `BridgePreviewReplayBundle`
- `BridgeWritebackReplayBundle`
- `BridgeRouteContractProof`
- `CanonicalBridgeWorkloadRequest`

The pricing-shock workload now adds one more important expectation on top of
those specialist pieces: the harness should be able to compare one nested
workload bundle across diagnostics tiers, replay paths, hostile faults, and
restart variation without relying on ad hoc test-local interpretation.

The current pricing certification surface also expects the comparison itself to
be machine-checkable, so parity decisions are made from bundle reports rather
than from hand-authored prose about what "should have happened." That now
includes bridge-visible historical provenance reads for retained pricing
commits, not just lifecycle and replay state.

## Relationship To Everyday Docs

Everyday docs teach how to use the bridge.

Certification docs teach how to trust the bridge under adversarial conditions.

Both matter, but they serve different reader journeys.

Milestone 13 is where those journeys reconnect: the everyday bridge surface is
now being judged by whether the pricing-shock workload can produce a coherent
offline-certifiable bundle through the intended public paths.
