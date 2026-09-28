# Operational Identity Authority

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

Query separates an identity you can print, store, or use to find candidates
from an identity that may authorize current runtime work. Use the operational
identity APIs when a workflow must prove that a snapshot, entity, receipt,
lease, or installed operation still belongs to the exact runtime state that
created it.

## Why You Use It

- reject a copied snapshot or entity label before mutation or continuation
- retain current authority while moving through an installed operation
- log and compare diagnostic projections without turning them into runtime keys

## Stable Entry Points

- `WorthQueryCommitIdentity::is_same_current_identity_as(...)`
- `WorthQuerySnapshotIdentity::is_same_current_identity_as(...)`
- `WorthQueryEntityIdentity::is_same_current_identity_as(...)`
- `from_bridge_commit_projection(...)`,
  `from_bridge_snapshot_projection(...)`, and
  `from_bridge_record_projection(...)` for typed adapter output that remains
  non-authoritative until runtime admission
- `evidence_identity()` and `terminal_projection_for_reporting()` for
  observation only
- `facade::identity_authority::{QueryExternalIdentityToken, ...IdentityKind}`
  for typed but untrusted input
- typed installed-operation, compatibility, lease, replay, and lineage
  outcomes in `facade::domain`

Query does not export its owner authority markers or witness factories. The
runtime APIs that perform an operation mint and retain the appropriate owner
artifact.

## Core Mental Model

An operational identity is a current right held by an owning runtime. A
projection is a description of that identity. Two descriptions may contain the
same bytes without carrying the same right.

`worth-proof` carries generic typed progression, freshness, and weakening.
`worth-foundational` supplies shared identity categories such as authority,
boundary-bridged, projection, digest evidence, and external token. Neither
crate decides that Query, Relational, Runtime Bridge, or Signal work is
authorized. The owning runtime mints the stronger artifact after validating
its own state.

Crossing a runtime boundary weakens identity. The receiving owner must validate
the retained source artifact and mint its own authority. A projection, digest,
label, debug string, or external token cannot perform that transition. A
recovery wire identity follows the same rule: it can locate a candidate, but
only the owning Query runtime can readmit it into a live recovery handle. A
registered backend can return typed Bridge projections, but Query makes a
mutation receipt current only when the Bridge causality bundle retained by that
execution matches the exact commit, snapshot, collection, Relational record,
mutation kind, and canonical Foundational aspect-touch set.

## How It Executes

1. The source runtime performs or observes authoritative work.
2. It mints an owner artifact and, when needed, a weakened boundary identity.
3. Runtime Bridge retains the admitted source correspondence rather than only
   its printable coordinates. For mutation writeback it binds the target record
   and Foundational patch/touch meaning into causality before execution. Bridge
   carries the comparison through lowering and denies authority execution when
   the bound subject patch and effect intent drift.
4. Query validates that retained handoff against the returned receipt and mints
   a Query-owned current identity or progression artifact. One mutation
   causality bundle admits exactly one matching mutation delta.
5. Operational APIs validate the current artifact and lifecycle generation.
6. Reporting APIs derive projections or digest evidence with no reverse path.

Indexes may use projections to find candidates. A key hit is not authorization;
the selected candidate remains paired with retained owner state and is
validated by the operation that consumes it.

## Small Example

```rust
use worth_query::facade::foundation::WorthQuerySnapshotIdentity;

fn same_current_snapshot(
    retained: &WorthQuerySnapshotIdentity,
    candidate: &WorthQuerySnapshotIdentity,
) -> bool {
    retained.is_same_current_identity_as(candidate)
}

fn snapshot_label(snapshot: &WorthQuerySnapshotIdentity) -> String {
    snapshot.terminal_projection_for_reporting()
}
```

The first function asks Query a named operational question. The second derives
text suitable for logs and reports. Comparing the returned strings does not
answer the first question.

## Real Example

```rust
use std::sync::Arc;
use worth_query::facade::{
    foundation::WorthQuerySnapshotIdentity,
    identity_authority::{QueryExternalIdentityToken, QuerySnapshotIdentityKind},
};

fn external_snapshot_is_not_current(
    current: &WorthQuerySnapshotIdentity,
    copied_label: &str,
) -> bool {
    let token = QueryExternalIdentityToken::<Arc<str>, QuerySnapshotIdentityKind>::new(
        Arc::from(copied_label),
    );
    let projected = WorthQuerySnapshotIdentity::admit_external_token(token);
    !current.is_same_current_identity_as(&projected)
}
```

The token preserves the caller's declared identity kind, which is useful for
admission and diagnostics. It does not claim that the caller observed the
current Query runtime. Query therefore keeps the resulting identity in a
projection posture until an owning runtime supplies real admission evidence.

Public runtime adapters that already hold typed Relational/Bridge parts should
return the corresponding `from_bridge_*_projection(...)` value. This preserves
record and snapshot structure for Bridge checks without granting authority to
the adapter or to arbitrary callers. Query admits the value only when it
returns through the registered snapshot boundary or through an exactly matching
Bridge mutation-authority bundle. Pairing a valid bundle with a different
collection, record, mutation kind, touch set, commit, or snapshot remains a
projection-only receipt.

## How It Relates To Other Features

- Installed operations carry the same owner law through execution,
  publication, replay, lineage, sharing, and invalidation.
- Projection consumption moves sealed fact authority; evidence getters remain
  observation only.
- Relational owns committed truth, Runtime Bridge owns admitted crossing and
  correspondence, Signal owns conditional decisions, and Query owns its
  installed-operation progression.
- Historical reads use explicit retained historical admission. A matching
  current label does not silently become historical authority.

## Inspection And Debugging

Use evidence identities, terminal reporting projections, typed denial kinds,
and semantic comparison reports to explain a decision. Operational identity
debug output intentionally reports posture rather than raw key material.

When a copied representation is rejected, inspect the missing owner handoff,
runtime generation, installed binding, lease, or historical admission. Do not
reconstruct a stronger identity from the diagnostic output.

## Anti-Patterns

- comparing `Debug`, `Display`, serialized output, or digest bytes as authority
- rebuilding entity or snapshot coordinates and passing them to an operational
  API
- treating a generic Proof artifact as Query, Relational, Bridge, or Signal
  authority
- treating a Foundational projection or boundary identity as current owner
  authority
- returning authority directly from an index solely because a derived key hit

## Current Limits

- Query identities retain equality, ordering, or hashing only where legitimate
  collections require representation-level candidate handling. Operational
  decisions use named owner methods or proof-bearing artifacts.
- Collection cursor and patch authority remain separate capabilities; an
  invalidation or identity projection does not manufacture them.
- Certification replay may compare exact retained meaning but cannot become an
  ordinary completed trace or publication authority.

## Related Docs

- [Runtime-Installed Domains And Operations](../domain-capabilities/runtime-installed-domains.md)
- [Installed Operation Re-Execution And Replay](../domain-capabilities/installed-operation-reexecution-and-replay.md)
- [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](../domain-capabilities/bound-projection-sharing-and-invalidation.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
- [worth-proof Authority And Workflow Contracts](../../../../../../crates/worth-proof/docs/features/authority-and-workflow-contracts.md)
