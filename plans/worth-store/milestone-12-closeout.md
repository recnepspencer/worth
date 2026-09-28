# Milestone 12 Closeout: Artifact Format Evolution And Rolling Compatibility

> **Note:** The legacy root crate `crates/worth-store` was removed on 2026-09-27. Paths into it below name its files as they were; they are no longer links. The store now lives in [`workspaces/worth-store`](../../workspaces/worth-store/README.md).

Status: Completed on 2026-04-21

Parent spec: [milestone-12.md](./milestone-12.md)

Roadmap: `worth_store_roadmap.md`

## Summary

Milestone 12 is closed.

`worth-store` now has a typed compatibility subsystem that survives restart,
gates semantic exposure before truth becomes visible, routes derived rebuild
through the already-closed Milestone 11 runtime, executes rolling and restore
compatibility through the real `WorthStore` facade, and admits bounded
deterministic adapters only when their declared id, digest, cost class, and
control-lane parity all hold.

The closure claim is:

- authoritative and derived compatibility are explicit store-owned contracts,
  not decoder tolerance
- restart reconstructs compatibility posture from persisted manifest records
  instead of artifact-row scans
- facade reads, writes, restores, rolling publication, derived rebuild, and
  bounded adapter execution all flow through compatibility-gated production
  paths
- incompatible or under-proven version crossings fail typed before partial
  truth acceptance
- the Milestone 12 certification runner emits machine-checkable accepted and
  rejected lanes with no remaining in-scope runtime-gap labels

## What Shipped

- compatibility admission, manifest, authoritative, rolling, restore, derived,
  certification, and production surfaces in
  `crates/worth-store/src/compatibility`
- production execution modules for derived rebuild, rolling publication, and
  authoritative adapter execution in
  `crates/worth-store/src/backend/engine/compatibility_production`
- runtime compatibility gating and recovered-manifest index reconstruction in
  `crates/worth-store/src/backend/engine/compatibility_runtime.rs`
- durable compatibility manifest records, state defaults, SQLite schema/load/
  persist support, and boot-time integrity verification in
  `crates/worth-store/src/backend/records/compatibility.rs`,
  `crates/worth-store/src/backend/sqlite/schema/compatibility.rs`,
  `crates/worth-store/src/backend/sqlite/load/compatibility.rs`,
  `crates/worth-store/src/backend/sqlite/persist/compatibility.rs`,
  and
  `crates/worth-store/src/backend/integrity/compatibility_records.rs`
- public store compatibility surfaces in
  `crates/worth-store/src/facade/authority.rs`,
  `crates/worth-store/src/facade/maintenance.rs`,
  and
  `crates/worth-store/src/facade/support.rs`
- milestone-specific evidence and certification reporting in
  `crates/worth-store/src/evidence/milestone_12.rs`
- named and focused hostile coverage in
  `crates/worth-store/src/tests/compatibility_facade_integration.rs`,
  `crates/worth-store/src/tests/compatibility_persistence.rs`,
  `crates/worth-store/src/tests/compatibility_rebuild_execution.rs`,
  `crates/worth-store/src/tests/compatibility_restore_execution.rs`,
  `crates/worth-store/src/tests/compatibility_rolling_execution.rs`,
  and
  `crates/worth-store/src/tests/compatibility_adapter_execution.rs`

## Acceptance Mapping

Milestone 12 is considered closed against
[milestone-12.md](./milestone-12.md)
and
[test-requirements.md](./test-requirements.md)
because the named milestone suite and the restart/hostile production-path tests
now map directly to shipped code and machine-checkable evidence.

### `Artifact Format Evolution And Rolling Compatibility Test`

Covered by:

- `crates/worth-store/src/compatibility/certification_runner.rs`
- `crates/worth-store/src/tests/compatibility_facade_integration.rs`
- `crates/worth-store/src/tests/compatibility_persistence.rs`
- `crates/worth-store/src/tests/compatibility_rebuild_execution.rs`
- `crates/worth-store/src/tests/compatibility_restore_execution.rs`
- `crates/worth-store/src/tests/compatibility_rolling_execution.rs`
- `crates/worth-store/src/tests/compatibility_adapter_execution.rs`

What is proven:

- authoritative reads and writes no longer rely on decode success alone; they
  require manifest-backed compatibility admission first
- restart-visible manifest summaries persist through local-file and SQLite
  reopen, and boot integrity rejects missing rows, family drift, sequence gaps,
  or digest drift as typed compatibility failures
- derived families rebuild through the real Milestone 11 maintenance runtime
  instead of through a side execution path
- rolling publication admits only declared windows and rejects missing-edge or
  multi-writer windows through typed compatibility failures
- authoritative export restore executes through a real facade path and rejects
  publication conflicts before truth becomes visible
- bounded first-ship adapter execution is admitted only when the declared
  adapter id and digest match and a control-lane parity witness is produced;
  digest drift increments the adapter parity-failure lane instead of silently
  adapting
- the certification runner emits accepted and rejected lanes for authoritative,
  derived, rolling, restore, disaster-recovery, and adapter parity behavior
  with no remaining in-scope runtime-gap labels

### Compile-time and construction boundary enforcement

Covered by:

- `crates/worth-store/tests/phase_boundaries_compile_fail.rs`
- `crates/worth-store/tests/ui`

What is proven:

- quarantined artifacts, checked artifacts, read/write receipts, restore
  witnesses, rolling witnesses, derived rebuild witnesses, and adapter parity
  witnesses cannot be fabricated through public constructors
- semantic compatibility proof remains crate-owned rather than becoming a loose
  host-constructed data contract

## Acceptance Evidence

The Milestone 12 certification output emits:

- `artifact_digest`
- `failure_digest`
- `compatibility_matrix_digest`
- `version_skew_digest`
- `diagnostics_digest`
- `counter_snapshot_digest`

The certification evidence bundle also carries:

- `Milestone12CompatibilityMatrix`
- `Milestone12VersionSkewReport`
- `Milestone12ComplexitySurface`
- `Milestone12AdmissionReport`

## Additional Hardening Added Before Close

The closeout pass intentionally hardened the evidence surface instead of only
writing status prose:

- public rolling, rebuild, restore, and adapter execution paths now expose
  admission reports; rolling, rebuild, restore, and adapter all have explicit
  certification-alignment coverage
- rolling certification evidence is tied back to the shipped public rolling path
  instead of only the earlier planning surface
- adapter parity failures are counted explicitly in Milestone 12 evidence rather
  than disappearing into generic rejection totals
- persisted manifest reopen coverage now proves both local-file and SQLite
  restart reconstruction and verifies that the runner no longer underclaims
  durability

## Explicit Deferrals

No in-scope Milestone 12 closeout debt remains for the shipped compatibility
contract.

Future work still exists, but it belongs to later roadmap layers rather than
hidden Milestone 12 incompleteness:

- optional convenience migration tooling may remain future work as long as
  compatibility admission continues to require declared edges and proof-bearing
  adapter parity
- later milestones may add more artifact families, wider version windows, or
  richer replication/repair workflows by extending this compatibility subsystem
  rather than bypassing it

## Verification

The focused closeout verification run used:

- `cargo test -p worth-store compatibility_persistence --lib`
- `cargo test -p worth-store compatibility_adapter_execution --lib`
- `cargo test -p worth-store compatibility_restore_execution --lib`
- `cargo test -p worth-store compatibility_rolling_execution --lib`
- `cargo test -p worth-store compatibility_rebuild_execution --lib`
- `cargo test -p worth-store artifact_format_evolution --lib`
- `cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1`

All passed after the final production-path and durability evidence corrections.

## Operational Conclusion

Milestone 12 is now closed at the store level.

`worth-store` no longer treats artifact compatibility as informal version
tolerance. Compatibility is explicit, persisted, restart-honest, typed at the
facade boundary, machine-checkable in certification, and bounded when adapters
are admitted. Rolling upgrades, restore, derived rebuild, and adapter execution
now preserve semantic truth only where the declared compatibility model says
they may, which is the milestone's core architectural law.
