# Worth Store Recovery Runtime

This crate owns fresh-process physical recovery orchestration. It accepts only
an existing Store root, qualified recovery media, finite limits, current static
configuration, and a concrete platform authority minted in this process.

It never imports a live ordinary Store runtime, a C.7 recovery handoff, decoded
observer artifacts, or caller-supplied Signal and scheduler instances. Entry
constructs a new recovery session, Signal graph, and bounded C.5.1 scheduling
envelope. Persisted stable Store identity joins the authority world only after
the backend has admitted the existing namespace.

The production entry runs the complete consuming recovery progression:

```text
physical_store_recover <store-root> --bounded-profile=c8-phase2-admission-v1 \
  [--report=<path>]
```

The optional report is a descriptive `store.physical.recovery-report` version
1 envelope. Decoding it never grants Store authority and does not replace the
returned process status or a fresh reopen.

## Phase 8 operator contract

The shipped command admits two fixed bounded profiles:

```text
physical_store_recover <store-root> --bounded-profile=c8-phase2-admission-v1 \
  [--report=<path outside store-root>]
```

The `c8-phase8-fate-coverage-v1` profile is a bounded 4 MiB evidence profile
used for the explicit all-fates recovery fixture; ordinary Phase 8 admission
continues to use the 512 KiB `c8-phase2-admission-v1` profile.

The report is descriptive evidence only. The fresh process mints platform
authority, samples the existing namespace, selects physical sources, and
returns one terminal outcome: `Refused`, `Blocked`, `Recovered`, or
`PublicationIndeterminate`. A report, observer payload, caller-supplied generation, or
decoded artifact cannot mint authority, publish a root, apply redo, or reopen
the Store. The report path must be outside the observed Store root so report
emission cannot mutate the evidence being inspected.

The runtime consumes the C.7 namespace-durable checkpoint, contiguous WAL
prefix, pageLSN and manifest facts. It does not provide backup/PITR,
rollback, semantic repair, or an alternate ordinary Store lane. C.8 recovery
is a fresh reopen and remains separate from C.9 integrity classification.

An exact-successor root candidate left by a dead C.7 writer is bounded,
non-current material. Recovery may reuse its exact immutable bytes only when
redo needs that successor and decoded candidate topology proves the same final
placements, segment membership, free-space inventory, frontiers, capacity,
tail, counts, and checksums as an independently derived incremental plan.
Recovery reproduces C.7 allocation and packing order for both same-capacity and
capacity-transition publication, then compares one canonical artifact at a
time. It does not retain a second complete expected candidate. Absence follows
ordinary construction; malformed, stale, foreign, or conflicting topology
blocks with typed evidence before effects. The runtime never skips the
successor generation, overwrites mismatched immutable bytes, or deletes the
candidate to force progress.

The report counters are stage-honest descriptive evidence: `recovery_effects`
comes from the admitted physical media effect count, while
`cleanup_performed` and `cleanup_deferred` come only from the cleanup posture.
`peak_recovery_bytes` is the independently carried peak retained selected-page
and recovery-plan/materialization accounting and is checked against the admitted
`recovery_memory_bytes` limit before effects begin. Candidate topology families
share one cumulative manifest-entry budget. Candidate lifecycle memory is the
larger of observation plus one-artifact comparison scratch and final
publication materialization; failed observation retains and reports only the
prefix reached before the exact denial.
Staging, publication, and fresh-reopen counters remain owned by their sealed
handoff evidence and are never relabeled as cleanup. A blocked process emits a
typed `C8_RECOVERY_BLOCKED` cause marker and exits unsuccessfully; a recovered
process emits `C8_RECOVERY_RUNTIME` and exits successfully.

## Successor boundary

`RecoveredPhysicalRuntimeHandoff` is the sole physical handoff from C.8. It
carries the freshly reopened, quiescent runtime and exact recovery facts; a
report file is never a substitute. C.9 extends this boundary with integrity,
corruption-localization, quarantine, and offline-truth behavior. C.10 extends
it with stable reads, epochs, reclaim, scheduled I/O, and maintenance
interference. Neither successor owns a second recovery entry path.

The direct process suite builds the writer, recoverer, and offline observer as
separate executables, kills the writer or recoverer at production yieldpoints,
and compares the resulting bytes from the independent observer. It runs
serially because it is a process scenario lane; it does not maintain a
cross-test lock, evidence ledger, source fingerprint inventory, or recursive
unit-test tree for its parent-oracle helpers.
