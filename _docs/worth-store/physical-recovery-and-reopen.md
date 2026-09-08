# Physical Recovery And Reopen

## What This Feature Is

Physical recovery lets an operator reopen an existing Worth Store after its
writer process is gone. The recovery command reads persisted physical truth,
builds a non-current generation, publishes it durably, reopens it through a
fresh handle, and returns one terminal result.

## Why You Use It

- restart a Store after process death;
- inspect why a bounded recovery was refused or blocked;
- create a portable runtime report and an independent artifact observation.

It is not backup restore, point-in-time recovery, semantic transaction repair,
or permission to choose an arbitrary older root.

## Stable Entry Points

- `physical_store_recover`
- `WorthStoreRecovery::recover`
- `RecoveryReportEnvelope`
- `physical_store_offline_observer c8-recovery-observe`
- `RecoveryObserverReport`

Application code should use the runtime facade. Lower Store coordination,
backend media, and recovery-physics types are not alternate entry points.

## Core Mental Model

C.8 discovery consumes C.9-admitted physical artifacts before owner decoding.
Integrity rejection preserves the exact damaged, unsupported, unknown, or
indeterminate posture and scope; it cannot select a recovery source or grant
redo/publication authority. C.8 still owns source precedence, durable continuity,
and the recovered-runtime handoff. A valid older checkpoint is bound to its
exact retained root, not promoted to the current serving root by integrity.
See [Physical Integrity And Offline Verification](physical-integrity-and-offline-verification.md)
for the separate managed scrub and independent all-family observer workflows.

Persisted selectors, checkpoints, manifests, page or extent frames, and WAL are
the source of truth. Recovery derives a plan from them under finite limits.
The runtime report describes what that process concluded. The offline observer
walks the directory independently and describes what it read. Neither report
can authorize writes or be admitted as Store truth.

## How It Executes

The runtime consumes these phases in order: admitted, discovered, selected,
planned, staged, namespace-durable, freshly reopened, then handed off. Cleanup
runs only after the fresh reopen and may defer without invalidating recovered
success. A refusal performs no recovery work; a blocked or indeterminate result
retains exact completed work and effect evidence.

The terminal outcomes are `Refused` (admission or precondition denial),
`Blocked` (bounded work cannot safely continue), `Recovered` (the new generation
was durably published and freshly reopened), and `PublicationIndeterminate`
(the process
cannot prove the final effect). These outcomes are typed conclusions, not
operator-selected status labels.

`Refused` means recovery stopped before an effect-bearing phase. `Blocked`
retains the Store identity, source generation when known, exact typed cause,
and the physical-effect count already visible to the fresh process; its CLI
exits unsuccessfully and emits `C8_RECOVERY_BLOCKED`. A
`PublicationIndeterminate` result means an effect boundary was ambiguous, so
the next restart must rediscover and reopen from persisted bytes rather than
retrying an in-memory publication. `Recovered` reports cleanup separately:
cleanup may be complete or `cleanup-deferred` without changing recovery success.

Successful CLI recovery emits `C8_RECOVERY_RUNTIME <store-identity> <runtime-identity>
<root-generation>` as a descriptive process-boundary line; it is not an authority
token and must not be fed back into admission.

The recovery process also emits one `C8_RECOVERY_FATE` line per reconciled
operation. Each line carries the stable idempotency identity and typed fate;
the aggregate `C8_RECOVERY_FATES` line is only a summary and is not the
identity-level evidence.

Operation fates are physical evidence, not acknowledgments: a durable
completion may be retried only as the same stable operation identity; a
`ProvenNoEffect` operation may be submitted again; `Indeterminate` must not be
automatically retried or cleaned up because absence is not proof of no effect.
The runtime report is descriptive and carries the physical recovery effect
count plus cleanup performed/deferred counts. Staging performed effects,
publication candidate/root/namespace effects, fresh-reopen selector/root reads,
and cleanup actions remain exact owner-specific counters and are not substituted
for one another.

## Small Example

```text
physical_store_recover D:\stores\orders \
  --bounded-profile=c8-phase2-admission-v1 \
  --report=D:\reports\orders-recovery-v1.bin
```

This is the smallest honest operator call: it names an existing Store root, a
finite built-in profile, and an optional descriptive report destination.
Keep that report destination outside the Store root so observation output
cannot mutate the physical truth it describes.

The process-proof build additionally exercises
`--bounded-profile=c8-phase8-refused-v1` through the shipped recovery command
to prove a real refusal report, and its certification-authority build uses
`c8-phase8-publication-indeterminate-v1` to prove the indeterminate terminal
report. These are deterministic proof profiles, not operator shortcuts.

## Real Example

Run recovery in one process, then inspect the resulting directory from another:

```text
physical_store_recover D:\stores\orders \
  --bounded-profile=c8-phase2-admission-v1 \
  --report=D:\reports\runtime-v1.bin

physical_store_offline_observer c8-recovery-observe \
  D:\stores\orders D:\reports\observer-v1.bin \
  1000000 100000 1000000 4294967295
```

The first process owns recovery authority and effects. The second owns only a
bounded read-only observation. Comparing the two reports belongs to a
certification or operator tool; the observer must not decide recovery success.

## How It Relates To Other Features

C.4 remains the only physical effect executor. C.5.1 schedules recovery work.
C.7 remains the ordinary durability and checkpoint publisher. The offline
verifier is an independent inspection boundary, not a Store runtime.
`RecoveredPhysicalRuntimeHandoff` is the only successor boundary. C.9 supplies
integrity, corruption localization, descriptive quarantine, and independent
offline truth; C.10 adds
stable reads, epochs, reclaim, scheduled I/O, and maintenance interference.
Neither milestone may treat a recovery or observer report as authority.

## Inspection And Debugging

`RecoveryReportDecodeDenial` distinguishes malformed bytes, wrong protocol
family, unsupported version, and digest damage. The observer has the equivalent
typed denials plus directory-entry, directory, artifact, byte, media, path, and
file-type limits. Version 1 of each protocol accepts exactly version 1.

If a writer dies during candidate publication, leave the root in place and run
the same bounded recovery command again. Recovery reselects the persisted
namespace-durable basis, blocks or publishes according to the bytes it can
prove, and never trusts the dead writer's heap, runtime identity, scheduler
state, counters, or report. A synchronized exact-successor candidate remains
non-current material: when redo requires that successor, recovery adopts its
exact bytes only after bounded decoding proves the same final placements,
membership, free-space state, frontiers, tail, and checksums. It does not skip
the generation, overwrite a mismatch, or delete the candidate before
publication. A report path must remain outside the Store root.

## Anti-Patterns

- Do not feed either report back into Store admission.
- Do not call recovery while the ordinary writer is live.
- Do not bypass the facade through backend media or Store coordination types.
- Do not treat cleanup deferral as failed recovery.

## Current Limits

The shipped CLI's ordinary profile uses finite selector,
checkpoint, manifest, WAL, redo, staging, publication, cleanup, and
`recovery_memory_bytes` limits. The refusal and publication-indeterminate
profiles are deterministic process-proof entry points using the same finite
admission posture; the latter requires the certification-test-authority build.
`recovery_memory_bytes` admits the runtime's
peak retained selected-page and recovery-plan/materialization accounting, and
the recovered report exposes that peak as `peak_recovery_bytes`; it is distinct from the
ordinary writer's checkpoint-memory behavior. `observation_bytes` bounds
recovery discovery and planning evidence and is not a memory-budget alias.
The offline observer has separate directory/artifact/byte limits and inventories
bytes and paths; it does not localize or repair corruption. Cross-version report
migration is unsupported because each initial compatibility window is exactly
version 1.

## Related Docs

- [Physical Durability And Checkpoints](physical-durability-and-checkpoints.md)
- [C.8 Fresh-Process Recovery Specification](physical-reconstruction-c8-fresh-process-recovery-and-reopen.md)
- [Physical Foundation Reconstruction Roadmap](physical-foundation-reconstruction-roadmap.md)
