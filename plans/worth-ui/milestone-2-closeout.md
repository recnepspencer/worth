# Milestone 2 Closeout: Canonical UI Source, Lowering, And Runtime Artifact

## Status

Milestone 2 is complete as of 2026-06-12.

This closeout records completion of:

- `plans/worth-ui/milestone-2.md`
- `workspaces/worth-ui/crates/worth-ui`

The milestone closes Worth UI's source-to-artifact foundation: repo-authored
source packages and Rust-authored composition now lower through one
snapshot-bound, proof-bearing pipeline into canonical runtime artifact meaning.
Later hot reload, execution plans, shell semantics, Query-bound surfaces, and
tooling can consume that artifact instead of rediscovering UI meaning from
strings, mutable registries, or host-local glue.

## What Closed

Milestone 2 now ships the canonical lowering chain required before runtime work
can honestly begin:

- canonical source package and module identity
- import resolution and source-package digest posture
- syntax-only parsed source with span-aware diagnostics
- shared authoring-neutral artifact-input IR
- file-authored source and Rust-authored composition lanes converging on that
  IR
- snapshot-bound capability resolution over frozen `CapabilitySnapshot`
- structural legality lowering before artifact assembly
- semantic binding lowering for Query-bound and runtime-facing references
- stable identity seeding for canonical artifact nodes
- canonical runtime artifact assembly with typed handles and normalized
  structure
- derived artifact inspection and provenance surfaces
- canonical artifact digest and equivalence contracts
- incremental dependency metadata for source modules, artifact subtrees, and
  runtime hook boundaries
- realistic sample-app certification covering file and Rust authoring lanes

## Architectural Outcome

Milestone 2 makes the runtime artifact the first real UI authority after
registration.

The closed architecture preserves these boundaries:

- source package identity is pre-parse and owns multi-file source truth
- parsed source is syntax authority, not semantic authority
- artifact input is shared authoring IR, not snapshot-bound runtime meaning
- capability resolution consumes frozen snapshots, not mutable builder state
- structural legality is decided before canonical artifact assembly
- binding semantics preserve upstream Query/runtime posture rather than local
  pseudo-runtime wrappers
- identity seeding happens before artifact assembly so reload and persistence
  lanes can consume stable node identity
- artifact inspection and provenance are derived observation surfaces, not
  alternate sources of truth
- artifact digests cover canonical semantic meaning, not formatting,
  diagnostic richness, or incidental authoring order
- dependency metadata is explicit enough for later hot reload narrowing without
  source watcher folklore

The important milestone decision is now encoded mechanically: later Worth UI
runtime work must consume canonical artifact truth, not parse trees, source
strings, local Rust control flow, broad snapshot scans, or mutable registries.

## Proof Surfaces

Milestone 2 has proof coverage in these main lanes:

- source-package canonicalization, duplicate module rejection, import
  resolution, cycle rejection, and package digest stability
- parse determinism, source-span diagnostics, diagnostic accumulation, and
  recovery ordering
- shared artifact-input lowering for file and Rust authoring lanes
- snapshot-bound resolution for admitted, deferred, unsupported, missing, and
  platform-internal capability posture
- structural legality rejection for illegal mosaic and root structure
- semantic binding rejection for nested capability mismatches, Query posture,
  command references, theme tokens, and binding family mismatches
- identity seeding stability, carry-forward classification, replacement
  classification, and duplicate authored identity rejection
- canonical artifact assembly from proven inputs only
- artifact inspection and provenance over source and capability origin
- semantic artifact digest and equivalence behavior
- incremental dependency metadata, runtime hooks, subtree digests, and
  narrowed impact lookup
- Rust composition parity and containment as an authoring lane only
- sample app certification over a realistic multi-file app with source package,
  snapshot, artifact, digest, inspection, provenance, dependency, replay, and
  hostile diagnostic evidence

## QA Hardening

The final QA loops materially strengthened the milestone before closeout:

- dependency metadata proof now asserts narrow impact lookup and canonical
  handle indexing instead of only proving that metadata exists
- Rust composition parity is contained to the authoring lane and cannot bypass
  snapshot resolution, structural legality, binding semantics, or identity
  seeding
- sample certification proves source/Rust parity at canonical artifact meaning,
  not visual plausibility or helper-level agreement
- hostile sample cases now cover malformed parse input, missing capability
  references, unsupported capability posture, illegal structure, binding
  posture mismatch, and duplicate identity rejection
- sample certification carries authoring evidence, snapshot metrics, semantic
  digest, inspection evidence, dependency basis, and dependency metrics
- hostile boundary setup was split out of the normal certification pipeline so
  test support does not collapse into a private runtime

Those corrections matter because Milestone 2 is the handoff point into hot
reload and execution. A green suite that only proved a tiny happy path would
have left later milestones guessing where canonical UI meaning actually lives.

## What This Does Not Claim

Milestone 2 deliberately does not ship:

- hot reload transport
- plan swap or activation staging
- renderer patching
- command execution
- shell lifecycle execution
- frame-budget execution lanes
- Query execution
- plugin loading or sandboxing
- native adapter implementation
- visual component breadth
- design-system rendering

Those are downstream milestones. Milestone 2 closes the artifact truth and
proof surfaces they need to consume.

## Allowed Debt After Closeout

No Milestone 2 completion debt is being carried for source package identity,
parsing, artifact input, snapshot-bound resolution, structural legality,
binding semantics, identity seeding, canonical artifact assembly, inspection,
digest/equivalence, dependency metadata, Rust composition parity, or sample-app
certification.

Remaining work is future scope, not hidden incompleteness:

- hot reload candidate planning, validation, swap, and rollback
- execution-plan lowering and frame-budget lanes
- shell/runtime integration
- renderer and native platform adapters
- richer Query execution integration
- component implementation breadth
- plugin loading and extension runtime behavior
- large-application performance certification beyond the milestone's lowering
  boundaries

## Verification Snapshot

The closeout was verified against the active implementation with:

- `cargo fmt -p worth-ui --check`
- `cargo test -p worth-ui phase13_sample_app_certification_tests`
- `cargo check -p worth-ui --all-targets`
- `cargo test -p worth-ui`
- `git diff --check`
- phase-local line-cap and directory-size sweep
- debug-residue sweep for the phase 13 certification harness

## Next Active Milestone

With Milestone 2 closed, Milestone 3 can consume canonical artifact identity,
legality, provenance, dependency metadata, and digest/equivalence truth as its
input. Hot reload and execution-plan work should not reopen source lowering or
artifact authority unless it is explicitly extending the artifact contract and
proving the same phase-chain boundaries again.

