# Consumer Kit

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

The Consumer Kit is the Query-owned way for downstream crates to prove they
consume `worth-query` correctly. Use it when a crate needs evidence reports,
hard-prohibition audits, support snapshots, support pins, or a real in-memory
test workspace without rebuilding Query's proof machinery locally.

This is not a bag of helper utilities. It is the ordinary downstream path for
consumer proof.

## Why You Use It

- you need a digest-bearing evidence report without hand-written digest or
  getter plumbing
- you need to enforce Query hard prohibitions without local source greps
- you need to pin support posture and fail a consumer build when a required
  support row regresses
- you need a test workspace that uses ordinary `WorthQueryWorkspace` behavior
  instead of fabricated receipts or adapter piles
- you need adoption or residue evidence showing a downstream crate is not
  carrying Query folklore

## Stable Entry Points

Import consumer-kit surfaces through:

```rust
use worth_query::facade::consumer_kit::*;
```

The main stable entry points are:

- `EvidenceReportDeclaration`
- `EvidenceReportScope`
- `hard_prohibition_registry()`
- `hard_prohibition_documentation_rows()`
- `hard_prohibition_boundary_audit()`
- `query_boundary_source_inventory()`
- `WorthQueryBoundaryAuditSourceSet`
- `project_support_snapshot(...)`
- `project_workspace_support_snapshot(...)`
- `load_support_snapshot_document(...)`
- `support_pinning_contract(...)`
- `load_support_pin_contract_document(...)`
- `in_memory_test_runtime()`
- `WorthQueryTestBackendSchema`
- `evidence_report_adoption_audit()`
- `query_consumer_residue_audit()`
- `worth_query_consumer_residue_certification_evidence()`
- `WorthQueryConsumerResidueClass`
- `WorthQueryConsumerResidueReport`
- `WorthQueryConsumerResidueSourceInventory`
- `WorthQueryConsumerResidueCertificationCaseEvidence`
- `query_test_backend_residue_audit()`

Durable persisted kit archives are not the stable contract. Persisted archives
and store-backed kit replay remain deferred.

## Core Mental Model

Downstream crates often need to prove things about their Query usage: which
facts they certified, which Query seams they avoided, which support rows they
depend on, and which test runtime behavior they exercised. Before the Consumer
Kit, that proof tended to become local folklore: hand-rolled report structs,
custom digest strings, source-text greps, local support row lists, and fake test
receipts.

The Consumer Kit moves that proof back to Query. A consumer declares what it
needs, and Query derives the canonical evidence, audit report, support
snapshot, pinning result, or test workspace posture.

The important boundary is:

- the consumer owns its domain facts and source files
- Query owns the proof shape for Query consumption
- evidence identity lowers through `WorthQueryEvidenceIdentity`
- support posture comes from the runtime support matrix
- hard-prohibition meaning comes from the Query prohibition registry
- test workspaces use the ordinary `WorthQueryWorkspace` facade

If the proof is about whether a crate consumed Query correctly, start here.

## How It Executes

The kit is split by proof job:

1. Evidence reports seal declared fields into canonical report identity.
2. The hard-prohibition registry publishes the seam set Query forbids.
3. The boundary audit evaluates consumer sources against that registry and
   emits typed findings.
4. Support snapshots project the live support matrix into a serialized,
   schema-versioned, digest-bound document.
5. Support pins evaluate a consumer's required rows against a snapshot and fail
   with localized findings when required posture regresses.
6. The in-memory test backend builds a real `WorthQueryWorkspace` over a
   declared test schema and fails closed for unsupported collections or lanes.
7. The generic consumer-residue audit detects local Query proof folklore:
   local Query reports, local Query proofs, raw support-row spelunking,
   support-matrix row searches, debug-derived proof strings, and
   delimiter-derived proof strings. Its report includes audited source paths,
   skipped non-Rust source count, finding identities, report identity, and a
   source-inventory digest so adoption tests can prove which downstream files
   were actually audited.
8. Adoption audits publish proof that covered reference-consumer surfaces no
   longer carry Query-owned folklore.

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

Pin support posture, audit sources, and build a test workspace:

```rust
use worth_query::facade::consumer_kit::{
    hard_prohibition_boundary_audit, project_workspace_support_snapshot,
    support_pinning_contract, WorthQueryBoundaryAuditSourceSet,
    WorthQueryPinnedSupportStatus, WorthQueryPinnedTeachingPosture,
    WorthQueryRuntimeFacadeFamily, WorthQueryTestBackendSchema,
    in_memory_test_runtime,
};

let schema = WorthQueryTestBackendSchema::single_collection("Task")
    .aspect("identity.id", "identity.id")?
    .aspect("title.value", "title.value")?;

let mut workspace = in_memory_test_runtime()
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

pins.evaluate_snapshot(&snapshot)?.assert_satisfied()?;

let sources = WorthQueryBoundaryAuditSourceSet::new("workflow-editor")
    .source_file(
        "read-tests",
        "tests/read_path.rs",
        r#"
        fn read_path(workspace: &mut WorthQueryWorkspace) {
            let _ = workspace.submissions();
        }
        "#,
    );

hard_prohibition_boundary_audit()
    .covering_sources(sources)
    .try_assert_clean()?;

let tasks = workspace.live_view::<worth_query::facade::runtime::WorthQueryUnrefinedLiveShape>("tasks", |view| {
    view.from("Task").select(["identity.id", "title.value"])
})?;

let receipt = workspace.insert("Task", |task| {
    task.aspect("identity.id", "task-1")
        .aspect("title.value", "Use Query proof kit")
})?;

assert_eq!(workspace.read(&tasks).len(), 1);
assert!(workspace.inspections()?.inspect(&receipt).is_ok());
```

The support snapshot is derived from the live matrix. The pins bind to live row
digests. The audit checks executable source against the Query-owned registry.
The workspace is the ordinary Query runtime facade, not a mock facade.

Read-only proof and diagnostics follow the same ownership rule. Inspect the
Query-owned public artifact and its typed getters instead of rebuilding the
boundary from support wrappers, raw rows, or local explainer helpers.

## How It Relates To Other Features

Use the Consumer Kit with [Support Matrix And Admission](support-matrix-and-admission.md)
when a consumer needs to freeze the support rows it depends on.

Use it with [Hard Prohibitions](hard-prohibitions.md) when a consumer needs a
machine-checkable guarantee that it is not using sealed or prohibited Query
seams.

Use it with [Downstream Runtime Integration](downstream-runtime-integration.md)
when onboarding a crate that should build on Query instead of lower-runtime
plumbing.

Use [Workspace Overview](workspace-overview.md) for ordinary runtime behavior.
The Consumer Kit proves the consumer is using that behavior correctly.

## Inspection And Debugging

Useful things to inspect:

- `EvidenceReport::report_digest()`
- `EvidenceReport::report_identity()`
- `EvidenceReport::field_inventory_identity()`
- `EvidenceReport::digest_participation_identity()`
- `WorthQueryBoundaryAuditReport::findings()`
- `WorthQueryBoundaryAuditReport::report_identity()`
- `WorthQuerySupportSnapshot::snapshot_digest()`
- `WorthQuerySupportPinReport::findings()`
- `WorthQuerySupportPinReport::report_digest()`
- `WorthQueryConsumerResidueReport::findings()`
- `WorthQueryConsumerResidueFinding::residue_class()`
- `WorthQueryTestBackendResidueReport`
- `support_report().consumer_kit_closure()`

Consumer Kit closeout is Query-owned certification evidence. Downstream code
inspects the public reports, source-backed adoption manifests, and typed
residue findings; it does not construct an application facade to manufacture
closure. A closed report requires documentation agreement and zero Query-owned
residue in the audited reference-consumer sources.

## Anti-Patterns

- building report identity with `Debug`, `Display`, delimiter-joined strings,
  or consumer-owned digest helpers
- grepping consumer sources for `.write(` or other forbidden text patterns
- treating support pins as advisory warnings
- checking in free-form strings as support row identity
- fabricating mutation receipts in tests
- building local Query proof with report structs, proof structs, raw support
  rows, support-matrix row searches, debug strings, or delimiter-joined strings
- implementing Query runtime adapter traits in a consumer test just to get a
  workspace
- reading support posture from autocomplete instead of the support matrix or a
  support snapshot
- teaching the in-memory test backend as proof that unsupported production
  lanes are supported

## Current Limits

- The boundary audit is syntax-based. Associated-path detection checks registry
  public-symbol suffixes such as `WorthQueryWorkspace::write`. Method-call
  detection is honest as method-name resolution, not compiler-backed type
  resolution.
- Comments, doc text, and string literals are intentionally ignored by the
  audit. Macro expansion and trait-dispatch type resolution are not closed by
  this first shipped audit.
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
