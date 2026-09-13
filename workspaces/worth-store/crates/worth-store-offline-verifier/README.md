# worth-store-offline-verifier

Owns the independent read-only verifier that walks persisted bytes through root
manifests, segment manifests, page headers, frame headers, slot directories,
and free-space maps without constructing the live store runtime.

This crate is a trust boundary. It should be able to disagree with the live
backend and report that disagreement as evidence.

For C.8 artifact inventory, use:

```text
physical_store_offline_observer c8-recovery-observe <store-root> <report-output> \
  <maximum-directory-entries> <maximum-directories> \
  <maximum-artifacts> <maximum-bytes>
```

The command emits a `store.physical.recovery-observer-report` version 1
envelope. It performs a bounded, deterministic read-only walk. The report is
observer evidence, not a recovery decision, provides no recovery decision, and
never Store authority.

The command uses the fixed C.8 four-axis budget supplied by its caller:
directory entries, directories, artifacts, and bytes. It rejects a report path
inside the observed root, symbolic links, changing files, malformed physical
records, and over-budget work. Transient `.lock` files are operational lease
state rather than Store artifacts and are not included in the evidence set.

The observer understands the shipped physical families, including the
production `WCP7REC\0` checkpoint stream, WAL-prefix frames, durable selector
and manifest frames, pageLSN headers, and residue. It emits
`store.physical.recovery-observer-report` version 1 with an inclusive one-
version window (`1..=1`). This is a descriptive Foundational report: it may
disagree with the recovery runtime, and it cannot select sources, authorize
recovery, publish a root, or become an authority input.

The observer is intentionally not a recovery engine. It performs no redo,
reconstruction, cleanup, rollback, PITR, or semantic repair. Write reports to
a directory outside the Store root (outside the observed root) and compare them with a fresh runtime
reopen when diagnosing `Recovered`, `Blocked`, or `Indeterminate` outcomes.
