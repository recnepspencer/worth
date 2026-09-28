# Consumer Kit

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

The Consumer Kit is the Query-owned way for downstream crates to prove they
consume `worth-query` correctly. Use it when a crate needs evidence reports,
support snapshots, support pins, or a real in-memory test workspace without
rebuilding Query's proof machinery locally.

This is not a bag of helper utilities. It is the ordinary downstream path for
consumer proof.

## Why You Use It

- you need a digest-bearing evidence report without hand-written digest or
  getter plumbing
- you need to pin support posture and fail a consumer build when a required
  support row regresses
- you need a test workspace that uses ordinary `WorthQueryWorkspace` behavior
  instead of fabricated receipts or adapter piles

## Stable Entry Points

Import consumer-kit surfaces through:

```rust
use worth_query::facade::consumer_kit::*;
```

The entry points, by proof job:

- **Evidence reports:** `EvidenceReportDeclaration`, `EvidenceReportScope`,
  `EvidenceReport`
- **Support snapshots:** `project_support_snapshot(...)`,
  `project_workspace_support_snapshot(...)`,
  `load_support_snapshot_terminal_json_document(...)`,
  `WorthQuerySupportSnapshot`
- **Support pins:** `support_pinning_contract(...)`,
  `load_support_pin_contract_terminal_json_document(...)`,
  `WorthQuerySupportPinContract`, `WorthQuerySupportPinReport`,
  `WorthQueryRuntimeFacadeFamily`
- **In-memory test backend:** `in_memory_test_runtime()`,
  `WorthQueryTestBackendSchema`, `WorthQueryControlledTestWorkspace`,
  `compare_test_backend_write_receipts(...)`,
  `WorthQueryTestBackendEquivalenceReport`

Durable persisted kit archives are not the stable contract. Persisted archives
and store-backed kit replay remain deferred.

## Core Mental Model

Downstream crates often need to prove things about their Query usage: which
facts they certified, which support rows they depend on, and which test runtime
behavior they exercised. Before the Consumer Kit, that proof tended to become
local folklore: hand-rolled report structs, custom digest strings, local support
row lists, and fake test receipts.

The Consumer Kit moves that proof back to Query. A consumer declares what it
needs, and Query derives the canonical evidence, support snapshot, pinning
result, or test workspace posture.

The important boundary is:

- the consumer owns its domain facts and source files
- Query owns the proof shape for Query consumption
- evidence identity lowers through `WorthQueryEvidenceIdentity`
- support posture comes from the runtime support matrix
- test workspaces use the ordinary `WorthQueryWorkspace` facade

Which Query seams a consumer may not use is not a runtime question. The
compiler answers it: the seams listed in [Hard Prohibitions](hard-prohibitions.md)
are private or absent, so code that reaches for them does not build.

If the proof is about whether a crate consumed Query correctly, start here.

## How It Executes

The kit is split by proof job:

1. Evidence reports seal declared fields into canonical report identity.
2. Support snapshots project the live support matrix into a serialized,
   schema-versioned, digest-bound document.
3. Support pins evaluate a consumer's required rows against a snapshot and fail
   with localized findings when required posture regresses.
4. The in-memory test backend builds a real `WorthQueryWorkspace` over a
   declared test schema and fails closed for unsupported collections or lanes.

## Small Example

Build one sealed evidence report:

```rust
use worth_query::facade::consumer_kit::{
    EvidenceReportDeclaration, EvidenceReportScope,
};

let report = EvidenceReportDeclaration::new(
    EvidenceReportScope::new("workflow-editor.query-proof")?,
    "read-path-proof",
)?
.shape_participating("consumer", "workflow-editor")?
.value_participating("surface", "workspace.read")?
.bool_participating("runtime_backed", true)?
.diagnostic_value_nonparticipating("note", "debug wording may change")?
.seal()?;

assert!(!report.report_identity().as_str().is_empty());
```

This is the smallest honest example because it shows the core rule: declare
fields and participation once, then let Query produce the sealed identity.

## Real Example

Build a test workspace, then pin the support posture it depends on:

```rust
use worth_query::facade::consumer_kit::{
    in_memory_test_runtime, project_workspace_support_snapshot,
    support_pinning_contract, WorthQueryPinnedSupportStatus,
    WorthQueryPinnedTeachingPosture, WorthQueryRuntimeFacadeFamily,
    WorthQueryTestBackendSchema,
};

// `task_aspect_contracts()` returns the worth-foundational `AspectContract`s
// for the `identity` and `title` aspects. The test backend refuses a mapping
// whose aspect has no installed contract.
let schema = WorthQueryTestBackendSchema::single_collection("Task")
    .aspect_contracts(task_aspect_contracts())?
    .aspect("identity.id", "identity.id")?
    .aspect("title.value", "title.value")?;

let workspace = in_memory_test_runtime()
    .with_schema(schema)
    .workspace("workflow-editor.tests")?;

let snapshot = project_workspace_support_snapshot(&workspace);

let pins = support_pinning_contract("workflow-editor")
    .against_snapshot(&snapshot)?
    .require_family(WorthQueryRuntimeFacadeFamily::Write, |row| {
        row.status(WorthQueryPinnedSupportStatus::Supported)
            .teaching_posture(WorthQueryPinnedTeachingPosture::OrdinaryRuntimeDx)
            .bind_live_row_digest()
    })?
    .require_family(WorthQueryRuntimeFacadeFamily::Inspect, |row| {
        row.status(WorthQueryPinnedSupportStatus::Supported)
            .teaching_posture(WorthQueryPinnedTeachingPosture::OrdinaryRuntimeDx)
            .bind_live_row_digest()
    })?
    .seal()?;

let report = pins.evaluate_snapshot(&snapshot)?;
report.assert_satisfied()?;
assert_eq!(report.blocking_finding_count(), 0);
```

The support snapshot is derived from the live matrix. The pins bind to live row
digests. The workspace is the ordinary Query runtime facade, not a mock facade:
insert, live views, reads and inspections behave as they do in production. The
test backend's own tests (`src/consumer_kit/test_backend/workspace_behavior_tests.rs`)
show those journeys end to end.

Read-only proof and diagnostics follow the same ownership rule. Inspect the
Query-owned public artifact and its typed getters instead of rebuilding the
boundary from support wrappers, raw rows, or local explainer helpers.

## How It Relates To Other Features

Use the Consumer Kit with [Support Matrix And Admission](support-matrix-and-admission.md)
when a consumer needs to freeze the support rows it depends on.

Read [Hard Prohibitions](hard-prohibitions.md) for the Query seams that are
sealed. The compiler enforces them; the Consumer Kit has no audit for them.

Use it with [Downstream Runtime Integration](downstream-runtime-integration.md)
when onboarding a crate that should build on Query instead of lower-runtime
plumbing.

Use [Workspace Overview](workspace-overview.md) for ordinary runtime behavior.
The Consumer Kit proves the consumer is using that behavior correctly.

## Inspection And Debugging

Useful things to inspect:

- `EvidenceReport::report_identity()`
- `EvidenceReport::field_inventory_identity()`
- `EvidenceReport::digest_participation_identity()`
- `EvidenceReport::fields()`
- `WorthQuerySupportSnapshot::snapshot_digest()`
- `WorthQuerySupportSnapshot::rows()`
- `WorthQuerySupportPinReport::findings()`
- `WorthQuerySupportPinReport::report_digest()`
- `WorthQuerySupportPinContract::contract_digest()`
- `WorthQueryTestBackendEquivalenceReport::report_identity()`

## Anti-Patterns

- building report identity with `Debug`, `Display`, delimiter-joined strings,
  or consumer-owned digest helpers
- grepping consumer sources for `.write(` or other forbidden text patterns;
  sealed seams do not compile, so a grep proves nothing the compiler has not
- treating support pins as advisory warnings
- checking in free-form strings as support row identity
- fabricating mutation receipts in tests
- implementing Query runtime adapter traits in a consumer test just to get a
  workspace
- reading support posture from autocomplete instead of the support matrix or a
  support snapshot
- teaching the in-memory test backend as proof that unsupported production
  lanes are supported

## Current Limits

- Support snapshots are projections of the live support matrix. They are not a
  second support authority.
- Support pins fail for required row regressions. Unpinned row drift can be
  reported as evidence without blocking a consumer.
- The in-memory test backend is honestly postured and fail-closed. It is not a
  production backend and does not imply support for families it denies.
- Durable persisted kit archives remain deferred.

## Related Docs

- [Downstream Runtime Integration](downstream-runtime-integration.md)
- [Support Matrix And Admission](support-matrix-and-admission.md)
- [Hard Prohibitions](hard-prohibitions.md)
- [Workspace Overview](workspace-overview.md)
- [AI Agent Orientation](../AI_README.md)
