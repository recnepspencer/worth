# Milestone 10 Boundary Baseline

This record freezes the pre-Milestone-10 boundary used by the Phase 1 red
controls. It is intentionally a boundary record, not a claim that the old
diagnostics tiers were orthogonal performance objectives.

## Current vocabulary

- `FoundationalProfileSet` has six profile families.
- `DiagnosticRichnessProfile` describes eligible descriptive detail.
- `RetentionDeliveryProfile` describes lifetime and delivery.
- Signal's `DiagnosticsTier` also selects execution, maintenance, and parallel
  policy through the diagnostics policy module.
- optional counters and descriptive lineage enter ordinary Signal paths before
  an observation session exists.

## Recorded baseline

The existing fintech mixed-fanout benchmark recorded the following warm
operation medians before the M10 cutover:

| Signal policy | Median microseconds |
| --- | ---: |
| Operational | 11,142 |
| Development | 17,213 |
| Forensic | 17,111 |

The values are comparative metadata for the named world only. They are not a
universal throughput promise and do not transfer to Store, geometry, or other
domains.

## Reproduction manifest

The pre-cutover source boundary is the clean revision `6c9a4f277` (the parent
of the uncommitted M10 candidate). The supporting performance workbook is
[`signal_performance_baseline.md`](../WORTH_signal/signal_performance_baseline.md),
section **March 24, 2026 Most Recent Validated Reference → Fintech Fanout
Matrix**. Its required command is the following three-sample, serial run:

```text
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal --lib tests::performance_profiles::fintech_fanout::perf_fintech_mixed_fanout_profile_matrix -- --ignored --nocapture --test-threads=1
```

The recorded workload is the fintech mixed-fanout production fixture, with the
same workload and feature configuration for Operational, Development, and
Forensic. The medians above are comparative capture values, not a promise of
wall-clock determinism. Reproduction should record `rustc --version`,
`cargo --version`, the active feature set, and the emitted JSON lines alongside
the command.

The Phase 1 structural control is independently executable with:

```text
cargo test -p worth-signal --lib tests::domains::fintech::invalidation::locality_red_controls -- --nocapture --test-threads=1
cargo test -p worth-signal --lib tests::domains::fintech::invalidation::operational_digest_parity::operational_authority_digest_is_independent_of_diagnostic_tier -- --exact --nocapture --test-threads=1
```

That control uses `FinancialWorldDefinition::convergent_factor_batch(41, 0)`
and the sparse/partitioned variants from the same locality courtroom. Its
structural snapshot is intentionally assertion-shaped: no observation token
is admitted; the Operational path records nonzero `WorkItemsAdmitted` and
lineage records; the manifest's necessary output set is exactly the evaluated
set; and no transitive subscriber walk is credited as semantic work. The
operational digest artifact covers committed values, runtime artifact identity
and hot state, node state, dependency revisions and snapshots, canonical
pending causes, readiness epoch, and published output-commit order. It excludes
tier, sidecars, counters, timings, and descriptive history. Phase 1 does not
claim branch/restore or the later performed-work/session oracle; those remain
later M10 evidence.

## Red controls

The Phase 1 control is the old Operational path with no explicit observation
request. Before M10 gates are installed it remains red by construction: the
performed-counter state is still updated and the lineage recorder still admits
descriptive records on ordinary invalidation/evaluation entrances. The control
must fail if those entrances are accidentally reported as on-demand before the
Signal observation-session phase is implemented.

Phase 1 also records canonical digest parity for the existing Operational,
Development, and Forensic policy presets. The parity record protects the
pre-cutover meaning while the compiler changes strategy ownership; later M10
phases may change cost, but may not silently change authoritative graph truth,
stable artifact identity, commit ordering, or replay linkage.
