# Bridge Truth Identity Exposure Report

> First Phase 2 compiler-discovery run for
> `milestone-9.6-bridge-truth-identity-lowering.md`.

## Run

Command:

```powershell
cargo check --workspace
```

Raw transcript:

```text
plans/worth-query/bridge_truth_identity_phase2_first_red_cargo_check.txt
```

Result:

```text
workspace RED
worth-runtime-bridge failed to compile
277 bridge-local authority-category errors
```

## Frontier

The first dependency frontier is `worth-runtime-bridge`.

The root break was installed in `crates/worth-runtime-bridge/src/identity.rs`:

- `BridgeIdentity::<Tag>::new(...)` now requires `BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>`.
- `BridgeIdentity::<Tag>::from_reference_evidence(...)` now requires `BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>`.
- `BridgeIdentityEvidence::from_external_authority(...)` now requires `BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>`.
- `BridgeIdentityEvidence::from_query_evidence_identity(...)` no longer accepts raw scope/token strings.
- `BridgeIdentityEvidence` no longer implements `AsRef<str>` or `Display`.

This is an expected Phase 2 red state. Do not restore raw text admission to make
downstream crates compile farther.

## Failure Ledger

| Failure id | Compiler class | Broken API | Attempted category | Required category | First observed path | Owning phase | Closure route |
|------------|----------------|------------|--------------------|-------------------|---------------------|--------------|---------------|
| BTI-P2-001 | `E0308` | `BridgeIdentity::<Tag>::new` | raw `String` from `format!` | `BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/continuity/lowering.rs:46` | Phase 4 | Replace synthetic formatted bridge identity construction with owner-admitted bridge authority or terminal projection. |
| BTI-P2-002 | `E0308` | `BridgeIdentity::<Tag>::new` | raw `String` from digest formatting | `BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/continuity/lowering.rs:65` | Phase 4 | Carry digest as evidence/projection; do not mint authority from digest text. |
| BTI-P2-003 | `E0599` | removed `BridgeIdentityEvidence::AsRef<str>` | generic string projection via `as_ref()` | explicit terminal projection accessor | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/assembly/request.rs:66` | Phase 4 / Phase 7 | Rename/reporting-quarantine diagnostic output; do not pass evidence to generic string consumers. |
| BTI-P2-004 | `E0599` | removed `BridgeIdentityEvidence::AsRef<str>` | generic string projection via `as_ref()` | explicit terminal projection accessor | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/assembly/request.rs:67` | Phase 4 / Phase 7 | Same as BTI-P2-003 for causal observation anchor projection. |
| BTI-P2-005 | `E0599` | removed `BridgeIdentityEvidence::AsRef<str>` | generic string projection via `as_ref()` | explicit terminal projection accessor | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/assembly/request.rs:90` | Phase 4 / Phase 7 | Same as BTI-P2-003 for causal envelope reporting accessors. |
| BTI-P2-006 | `E0308` | `BridgeIdentity::<Tag>::from_reference_evidence` | `&BridgeIdentityEvidence` | `BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/retained_mapping/retained_artifact_digest/planning_checkpoint.rs:31` | Phase 4 / Phase 7 | Preserve retained evidence as typed bridged authority or keep it terminal evidence; do not reconstruct from text. |
| BTI-P2-007 | `E0308` | `BridgeIdentity::<Tag>::from_reference_evidence` | `&BridgeIdentityEvidence` | `BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/retained_mapping/retained_artifact_digest/planning_checkpoint.rs:42` | Phase 4 / Phase 7 | Same retained-evidence reconstruction closure as BTI-P2-006. |
| BTI-P2-008 | `E0308` | `BridgeIdentity::<Tag>::from_reference_evidence` | `&BridgeIdentityEvidence` used for stream checkpoint token reconstruction | `BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/retained_mapping/retained_artifact_digest/planning_checkpoint.rs:53` | Phase 4 / Phase 7 | Split retained checkpoint evidence from current bridge authority. |
| BTI-P2-009 | `E0308` | `BridgeIdentity::<Tag>::new` | raw `&str` from retained checkpoint token | `BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/retained_mapping/retained_artifact_digest/planning_checkpoint.rs:143` | Phase 4 / Phase 7 | Do not mint stream checkpoint authority from retained text. |
| BTI-P2-010 | `E0308` | `BridgeIdentity::<Tag>::from_reference_evidence` | `&BridgeIdentityEvidence` | `BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/retained_mapping/retained_artifact_digest/route_history_preview.rs:21` | Phase 4 / Phase 7 | Same retained-route reconstruction closure as BTI-P2-006. |
| BTI-P2-011 | `E0308` | `BridgeIdentity::<Tag>::new` | raw `&str` from bridge evidence reporting accessor | `BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/diagnostics/causal_envelope/retained_mapping/retained_artifact_digest/route_history_preview.rs:41` | Phase 4 / Phase 7 | Replace historical record lookup identity reconstruction with typed retained evidence or terminal projection. |
| BTI-P2-012 | `E0308` | `BridgeIdentityEvidence::from_external_authority` | `impl AsRef<str>` external evidence | `BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/writeback/declaration.rs:25` | Phase 4 / Phase 7 | Admit external bridge evidence through explicit external-token category before owner admission. |
| BTI-P2-013 | `E0308` | `BridgeIdentity::<Tag>::new` | raw digest `String` | `BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/writeback/candidate.rs:137` | Phase 4 / Phase 7 | Treat writeback candidate digest as digest evidence/projection until owner admission. |
| BTI-P2-014 | `E0308` | `BridgeIdentity::<Tag>::new` | raw digest `String` | `BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/writeback/contracts.rs:106` | Phase 4 / Phase 7 | Treat writeback contract digest as digest evidence/projection until owner admission. |
| BTI-P2-015 | `E0308` | `BridgeIdentityEvidence::from_external_authority` | `impl AsRef<str>` external evidence | `BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-runtime-bridge/src/writeback/effect/causality.rs:22` | Phase 4 / Phase 7 | Same external-token closure as BTI-P2-012. |

## Aggregate Failure Classes

| Aggregate | Compiler class | Count signal | Meaning | Closure |
|-----------|----------------|--------------|---------|---------|
| Raw bridge identity minting | `E0308` | many | Callers pass `String`, `&str`, or formatted digest labels to `BridgeIdentity::<Tag>::new`. | Replace with owner-admitted authority or downgrade to terminal projection/digest evidence. |
| Generic projection consumption | `E0599` | early causal-envelope failures | `BridgeIdentityEvidence` was consumed through `AsRef<str>`. | Add explicitly named reporting/projection accessors only at terminal output boundaries. |
| Retained evidence reconstruction | `E0308` | many retained-mapping failures | `&BridgeIdentityEvidence` was used to rebuild bridge identities. | Carry typed boundary-bridged identity or keep the value as evidence; require owner revalidation before authority use. |
| External evidence re-entry | `E0308` | writeback failures | `impl AsRef<str>` external evidence entered bridge authority APIs. | Introduce explicit `BridgeTruthExternalIdentityToken` at the boundary, then owner admission/revalidation. |

## Required Next Step

Start Phase 4 repair at the bridge frontier before trying to expose Query or
downstream failures. The first repair slice should not weaken these root breaks.
It should introduce the owner-admission/revalidation path for bridge truth
authority, then rerun `cargo check --workspace` to expose the next frontier.

## Phase 2B-1 Query Evidence/Feeder Root Break

Command:

```powershell
cargo check -p worth-query --lib
```

Raw transcript:

```text
plans/worth-query/frontier2_query_root_break_cargo_check.txt
```

Workspace command:

```powershell
cargo check --workspace
```

Raw transcript:

```text
plans/worth-query/frontier2_workspace_root_break_cargo_check.txt
```

Result:

```text
worth-query lib RED
workspace RED at worth-query
534 Query evidence/feeder root-break errors after QA-expanded root cuts
312 unique reported paths in the compiler output
```

This is an expected Phase 2B red state. Do not restore
`field_identity(impl AsRef<str>)`, `field_identity_sequence(...)`,
`field_bridge_identity(...)`, `optional_identity(...)`,
`WORTHQueryEvidenceIdentity: Display`,
`WORTHQueryEvidenceIdentity: AsRef<str>`, public
`WORTHQueryEvidenceIdentity::as_str()`, public bridge evidence export,
public free composition, public `BridgeIdentity::<Tag>::evidence_identity()`,
public `BridgeIdentityEvidence::as_str()`, public digest `as_str()` accessors,
or `crate::identity::hash_parts` to make downstream code compile farther.

### Phase 2B-1 Failure Ledger

| Failure id | Compiler class | Broken API | Attempted category | Required category | First observed path | Owning phase | Closure route |
|------------|----------------|------------|--------------------|-------------------|---------------------|--------------|---------------|
| 2B-B-001 | `E0599` | `WORTHQueryEvidenceIdentityEncoder::field_identity` | raw/projection/digest string in an identity slot | typed evidence identity, admitted Query authority, bridge evidence, or terminal value field | `crates/worth-query/src/application/declaration_bridge_routing/digest.rs` | Phase 3-7 repair | Replace each identity-slot use with typed evidence/authority carry; use value/reporting fields only for terminal projection. |
| 2B-B-002 | `E0599` | `WORTHQueryEvidenceIdentityEncoder::field_identity_sequence` | sequence of raw/projection strings | typed evidence identity sequence or terminal value sequence | `crates/worth-query/src/application/support/closure.rs` | Phase 3-7 repair | Carry `WORTHQueryEvidenceIdentity`/authority handles through the collection before composing sequence evidence. |
| 2B-B-003 | `E0432` | `crate::identity::hash_parts` | digest-string folklore composition | typed evidence composition, owner digest derivation, or terminal certification digest owner | `crates/worth-query/src/application/capability/facade.rs` | Phase 5-9 repair | Replace free-form string hashing with category-owned evidence/digest constructors; do not re-export the primitive. |
| 2B-B-004 | `E0277` / `E0599` | removed `WORTHQueryEvidenceIdentity: Display` / `ToString` | formatting evidence identity into text | terminal projection quarantine or typed evidence carry | `crates/worth-query/src/application/domain_entry/support_snapshot.rs` | Phase 3-7 repair | Keep evidence identities typed through composition; add explicitly named reporting projection only at output edges. |
| 2B-B-005 | `E0599` | removed `WORTHQueryEvidenceIdentity: AsRef<str>` | passing evidence identity through generic string API | typed evidence API or terminal value API | `crates/worth-query/src/application/declaration_entry_seam/contribution/scope.rs` | Phase 3-7 repair | Replace generic `AsRef<str>` paths with concrete evidence/authority parameters or value-only projection. |
| 2B-B-006 | `E0277` | missing `WORTHQueryEvidenceIdentity: AsRef<str>` bound | generic lower-authority string constraint on evidence identity | typed evidence/authority category bound | `crates/worth-query/src/effect_lifecycle/execution_bridge.rs` | Phase 7-9 repair | Split authority/evidence inputs from string labels; do not satisfy generic string constraints with evidence identities. |
| 2B-B-007 | `E0599` | removed `WORTHQueryEvidenceIdentityEncoder::field_bridge_identity` | bridge evidence flattened through terminal projection at compose time | typed bridge evidence carry, boundary-bridged evidence, or owner readmission before authority use | `crates/worth-query/src/domain_capabilities/canonical_runtime/workflow/preview_identity.rs` | Phase 5-7 repair | Replace bridge projection flattening with a typed bridge-evidence lane or quarantine it as terminal reporting; do not rebuild bridge/query evidence from `terminal_projection_for_reporting()`. |
| 2B-B-008 | visibility restriction | `WORTHQueryEvidenceIdentity::compose`, `bridge_evidence_identity`, and `bridge_external_identity_evidence` are crate-private | external consumers could previously compose/export evidence without owner control | owner-controlled Query facade/admission path | public API surface, exposed when downstream crates compile past current Query frontier | Phase 8-9 repair | Keep construction and bridge export behind Query-owned admission/crossing APIs; add downstream compile-fail coverage when the crate compiles far enough to exercise external consumers. |
| 2B-D-001 | `E0624` | `BridgeIdentity::<Tag>::evidence_identity` made crate-private | current bridge truth identity downgraded directly into bridge evidence | typed bridge evidence carry, owner readmission, or explicit terminal projection | `crates/worth-query/src/domain_capabilities/canonical_runtime/workflow/preview_identity.rs` | Phase 5-7 repair | Do not treat current bridge truth as external-authority evidence; preserve a bridge evidence carrier or revalidate at the owning boundary. |
| 2B-D-002 | `E0624` | `BridgeIdentityEvidence::as_str` made crate-private | bridge evidence projected through raw text inside Query/certification composition | terminal projection accessor only at reporting edges | `crates/worth-query/src/effect_lifecycle/certification/seeded/support.rs` | Phase 7-9 repair | Replace evidence text consumption with typed evidence carry or `terminal_projection_for_reporting()` only in terminal report output. |
| 2B-D-003 | `E0599` | bridge-domain wrapper `evidence_identity()` factories removed | bridge-domain wrapper text converted into `BridgeIdentityEvidence` after construction | owner-carried evidence, typed bridge authority, or terminal projection | `crates/worth-query/src/source/grouped_truth_view/support.rs` | Phase 5-7 repair | Do not synthesize bridge evidence from wrapper display text; preserve the original evidence source or perform owner-controlled admission. |
| 2B-D-004 | visibility restriction + compile-fail guard | `BridgeIdentityEvidence::is_empty` made crate-private | public bridge evidence treated as string-like value with reusable predicates | terminal reporting projection or owner-internal evidence validation | public bridge evidence surface, guarded by `evidence_identity_empty_predicate_private.rs` | Phase 5-9 repair | Keep bridge evidence opaque outside the owning crate; callers that need validation must use an owner-owned typed path, not public representation predicates. |
| 2B-E-001 | `E0432` / `E0425` | `crate::identity::hash_parts` export removed | free-form digest construction from arbitrary strings | digest-family owned construction or typed evidence composition | `crates/worth-query/src/application/capability/facade.rs` | Phase 5-9 repair | Replace free hashing with owner digest/evidence constructors; if the value is only for a report, quarantine it as terminal projection. |
| 2B-E-002 | visibility restriction | `Canonical*Digest::as_str` made crate-private | digest wrapper projected as reusable text outside Query | terminal reporting accessor or digest evidence only | public digest facade, exposed when downstream compiles past current Query frontier | Phase 8-9 repair | Do not expose digest text as latent authority; add explicit reporting/proof surfaces where consumers genuinely need terminal output. |
| 2B-E-003 | `E0599` | `Canonical*Digest::evidence_identity` removed | digest wrapper rebuilt a `WORTHQueryEvidenceIdentity` from its own text | owner digest/evidence derivation or typed evidence carried from source | `crates/worth-query/src/workflow/foundation.rs` | Phase 5-9 repair | Replace digest-to-evidence helpers with owner-specific digest evidence derivation; do not let digest text re-enter evidence composition through a crate-wide method. |

### Phase 2B-1 Aggregate Failure Classes

| Aggregate | Compiler class | Count | Meaning | Closure |
|-----------|----------------|-------|---------|---------|
| Raw identity sink removal | `E0599` | 104 | Callers still compose identity slots through `field_identity(...)`. | Replace with typed evidence/authority composition or terminal value fields. |
| Digest folklore import/removal | `E0432` / `E0425` | 206 total failures (`193` imports + `13` direct module calls) | Callers depend on free `hash_parts` to produce authority-adjacent digest material. | Move to owner digest/evidence derivation; keep low-level hashing private. |
| Raw identity sequence removal | `E0599` | 19 | Callers compose identity sequences from raw/projected strings. | Carry typed evidence identity sequences. |
| Bridge projection flattening removal | `E0599` | 36 | Callers compose bridge evidence by projecting it to text inside Query evidence identity. | Carry bridge evidence as a typed category or quarantine it as terminal reporting. |
| Bridge truth-to-evidence downgrade removal | `E0624` | 65 | Callers derive bridge evidence directly from current bridge truth identity. | Preserve typed bridge evidence or perform owner readmission at the crossing. |
| Bridge evidence raw accessor removal | `E0624` | 12 | Callers consume bridge evidence through `as_str()`. | Use terminal reporting projection only at output edges. |
| Bridge wrapper evidence factory removal | `E0599` | 8 | Callers synthesize `BridgeIdentityEvidence` from bridge-domain wrapper text. | Carry the original evidence or use owner-controlled bridge admission. |
| Digest wrapper evidence converter removal | `E0599` | 26 | Callers rebuild `WORTHQueryEvidenceIdentity` from digest wrapper text through `Canonical*Digest::evidence_identity()`. | Replace with owner-specific digest evidence derivation or carry the original typed evidence. |
| Evidence formatting removal | `E0277` / `E0599` | 30 | Callers rely on `Display`, `ToString`, or formatting to project evidence identity. | Terminal projection quarantine only. |
| Evidence string coercion removal | `E0599` / `E0277` | 27 | Callers pass evidence identity through `AsRef<str>` APIs. | Replace string-like API with typed evidence/category parameter. |

## Phase 2B-1 Next Step

Start the repair pass at the first upstream Query compiler frontier, not at the
most convenient leaf. Each repair row must choose one closure route:

- typed evidence carry,
- Query authority admission,
- bridged readmission,
- digest evidence derivation,
- terminal projection quarantine.

Do not add compatibility aliases or restore the removed roots.

## Phase 2B-5 Workflow / Domain Preview Repair

Command:

```powershell
cargo check -p worth-query --lib
```

Result:

```text
worth-query lib still RED
504 Query evidence/feeder root-break errors
```

2B-5 closed the first workflow/domain-preview bridge/digest cluster without
restoring any removed roots:

- `BridgeIdentity::<Tag>::bridge_trust_boundary()` now exports bridge truth
  identity only as `BridgeTruthBoundaryBridgedIdentity`; it does not expose raw
  text or rebuild `BridgeIdentityEvidence`.
- `WORTHQueryEvidenceIdentityEncoder::field_bridge_authority_identity(...)`
  accepts the typed boundary-bridged bridge identity category. This is the
  typed replacement used by workflow/domain preview; the removed
  `field_bridge_identity(...)` string-flattening root remains absent.
- `workflow/foundation.rs`, `workflow/lowering/writeback.rs`, and
  `domain_capabilities/canonical_runtime/workflow/preview_identity.rs` no
  longer use `field_bridge_identity(...)`,
  `BridgeIdentity::<Tag>::evidence_identity()`, or
  `Canonical*Digest::evidence_identity()` in the repaired cluster.
- Workflow/domain preview digest use now goes through local owner-specific
  digest evidence helpers that use terminal value fields inside a named
  workflow evidence identity, rather than a universal digest-to-evidence
  converter.

Closure routes:

| Closed ledger id | Cluster | Closure route |
|------------------|---------|---------------|
| `2B-B-007` | workflow/domain preview bridge composition | typed boundary-bridged bridge authority field |
| `2B-D-001` | workflow/domain preview bridge truth-to-evidence downgrade | `bridge_trust_boundary()` instead of `evidence_identity()` |
| `2B-E-003` | workflow/domain preview digest wrapper evidence converters | owner-specific workflow/domain digest evidence helpers |

2B-5 QA closure:

- `BridgeIdentity::<Tag>::bridge_trust_boundary()` now preserves tag-specific
  marker kinds for the repaired workflow/domain-preview and writeback families:
  `BridgePreviewSessionIdentityKind`,
  `BridgePreviewSessionDeclarationIdentityKind`,
  `BridgePreviewExecutionRecordIdentityKind`, and
  `BridgeWritebackDeclarationIdentityKind`. It no longer erases these repaired
  families to `BridgeEvidenceReferenceIdentityKind`.
- Future bridge-family repair slices must add their own tag-specific marker kind
  before using `bridge_trust_boundary()`; falling back to
  `BridgeEvidenceReferenceIdentityKind` is allowed only for retained evidence
  revalidation owned by the bridge internals.
- `WORTHQueryEvidenceIdentityEncoder::field_bridge_authority_identity(...)`
  is an internal typed replacement while `worth-query` is red. Compile-fail
  coverage for rejecting `BridgeIdentityEvidence`, projections, digests,
  external tokens, and raw text at this exact encoder method is blocked until
  the Query trybuild harness can compile far enough to execute internal boundary
  fixtures.

## Phase 2B Repair Closure (2026-06-15)

Command:

```powershell
cargo check -p worth-query --lib
cargo check -p worth-query --lib --tests
cargo check -p worth-runtime-bridge --lib
cargo test -p worth-query --test phase_boundaries_query_identity_authority_compile_fail
cargo test -p worth-runtime-bridge --test phase_boundaries_bridge_truth_identity_compile_fail
```

Result:

```text
worth-query lib GREEN
worth-query lib tests GREEN
worth-runtime-bridge lib GREEN
query + bridge identity compile-fail suites PASS
```

Phase 2B repair closed without restoring removed roots:

- Frontier B encoder/string ingress roots remain cut; call sites use typed
  evidence and bridge authority fields.
- Frontier C truth ID admission: removed public string mint
  (`from_external_authority_label`, `authored_command`); external labels enter
  via `QueryExternalIdentityToken` + `query_truth_identity_admission_authority`.
- Removed `impl Display` on evidence and memory-workspace truth IDs; terminal
  reporting uses explicit `terminal_projection_for_reporting()`.
- Downstream workspace crates explicitly deferred to Phase 8+ (master baseline).

Post-2B Phase 7 QA: **CLEARED** for `worth-query` scope.

## Phase 4 Bridge Frontier Repair

Command:

```powershell
cargo check -p worth-runtime-bridge --lib
```

Raw transcript:

```text
plans/worth-query/bridge_truth_identity_phase4_bridge_lib_check.txt
```

Result:

```text
worth-runtime-bridge lib PASS
```

Bridge-local closure:

- Added explicit bridge owner admission helpers in `crates/worth-runtime-bridge/src/identity_authority/admission.rs`.
- Added `BridgeIdentity::<Tag>::admit_bridge_owned(...)` for bridge-owned authority minting without restoring raw `new(...)`.
- Added `BridgeIdentity::<Tag>::from_retained_evidence_reference(...)` for explicit retained-evidence revalidation.
- Added `BridgeIdentityEvidence::terminal_projection_for_reporting(...)` so reporting no longer depends on generic `AsRef<str>`.
- Added `BridgeIdentityEvidence::from_bridge_owner_external_authority(...)` so legacy bridge wrappers create a typed external token before evidence admission.
- Removed the bridge-local compile failures without restoring `BridgeIdentityEvidence: AsRef<str>`, `Display`, or raw `BridgeIdentity::<Tag>::new(String/&str)`.

Workspace command:

```powershell
cargo check --workspace
```

Raw transcript:

```text
plans/worth-query/bridge_truth_identity_phase4_workspace_check.txt
```

Result:

```text
workspace RED
worth-runtime-bridge passed
worth-query failed to compile
7 query-local authority-category errors
```

Next compiler frontier:

| Failure id | Compiler class | Broken API | Attempted category | Required category | First observed path | Owning phase | Closure route |
|------------|----------------|------------|--------------------|-------------------|---------------------|--------------|---------------|
| BTI-P4-QRY-001 | `E0308` | `BridgeIdentityEvidence::from_external_authority` | `&WORTHQueryEvidenceIdentity` | `BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-query/src/application/declaration_bridge_routing/lower.rs:341` | Phase 5 | Query must classify bridge-lowering evidence as external token, projection, digest evidence, or owner-admitted authority before entering bridge evidence. |
| BTI-P4-QRY-002 | `E0308` | `BridgeIdentityEvidence::from_external_authority` | `WORTHQueryEvidenceIdentity` | `BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-query/src/application/declaration_bridge_routing/lower.rs:416` | Phase 5 | Same query evidence-token boundary as BTI-P4-QRY-001. |
| BTI-P4-QRY-003 | `E0308` | `BridgeIdentityEvidence::from_external_authority` | `&str` observed digest | `BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-query/src/effect_lifecycle/oracle/bridge_oracle.rs:202` | Phase 5 / Phase 7 | Observed digest must remain digest evidence/projection unless explicitly tokenized for the bridge boundary. |
| BTI-P4-QRY-004 | `E0308` | `BridgeIdentityEvidence::from_query_evidence_identity` | raw scope `&str` plus raw identity `&str` | `BridgeTruthProjectionIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>` plus `BridgeTruthDigestIdentityEvidence<BridgeCanonicalDigestIdentityBasis, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-query/src/evidence_identity/artifact.rs:60` | Phase 5 / Phase 7 | Query evidence identity export must provide typed projection and digest evidence, not raw string accessors. |
| BTI-P4-QRY-005 | `E0599` | removed `BridgeIdentityEvidence::AsRef<str>` | generic evidence string consumption | explicit terminal projection accessor | `crates/worth-query/src/evidence_identity/encoder.rs:77` | Phase 7 | Encoder must distinguish terminal reporting text from authority or evidence composition. |
| BTI-P4-QRY-006 | `E0277` | `WORTHQueryEvidenceIdentityEncoder::field_identity` | `&BridgeIdentityEvidence` through `impl AsRef<str>` | explicit terminal projection accessor or typed evidence field | `crates/worth-query/src/runtime/support/authority_artifacts/bridge_imports.rs:74` | Phase 7 | Evidence encoder API needs a bridge-evidence-specific path or terminal projection quarantine. |
| BTI-P4-QRY-007 | `E0308` | `BridgeIdentityEvidence::from_external_authority` | `WORTHQueryEvidenceIdentity` | `BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>` | `crates/worth-query/src/workflow/lowering/writeback.rs:228` | Phase 5 | Writeback lowering must tokenize query evidence before bridge evidence admission. |

Phase 4 closeout status:

- Bridge frontier is closed for the library target.
- The next slice should start at the Query evidence-token boundary, not by weakening bridge authority APIs.
- The current bridge repair intentionally centralizes retained-evidence revalidation; Phase 7 should decide which of those retained-mapping paths are allowed owner revalidation versus terminal-only projection.

## Query Evidence Bridge Tokenization

Status:

```text
closed for worth-query --lib
```

Command:

```powershell
cargo check -p worth-query --lib
cargo check -p worth-runtime-bridge --lib
cargo test -p worth-query --test phase_boundaries_bridge_truth_identity_compile_fail
```

Raw transcripts:

```text
plans/worth-query/bridge_truth_identity_query_evidence_tokenization_check.txt
plans/worth-query/bridge_truth_identity_query_evidence_tokenization_bridge_check.txt
plans/worth-query/bridge_truth_identity_query_evidence_tokenization_trybuild.txt
```

Result:

```text
worth-query lib PASS
worth-runtime-bridge lib PASS
bridge truth identity trybuild PASS
```

Phase 2B supersession note:

This earlier closure was valid before the evidence/feeder root break reopened
the Query frontier. Phase 2B-B-007 removes
`WORTHQueryEvidenceIdentityEncoder::field_bridge_identity(...)`; do not restore
it to recreate this green state. The replacement repair route is typed bridge
evidence carry, owner readmission, or terminal projection quarantine.

Implemented closure:

- `WORTHQueryEvidenceIdentity::bridge_external_identity_evidence()` now performs
  the explicit Query -> bridge external-token crossing.
- `WORTHQueryEvidenceIdentity::bridge_evidence_identity()` now exports Query
  evidence through bridge projection and digest evidence categories, not raw
  scope/token strings.
- Historical: `WORTHQueryEvidenceIdentityEncoder::field_bridge_identity(...)`
  previously used `BridgeIdentityEvidence::terminal_projection_for_reporting()`
  instead of generic `AsRef<str>`. Phase 2B-B-007 now treats that method itself
  as a projection-flattening root and removes it.
- Declaration bridge lowering, workflow writeback lowering, bridge oracle
  observation identity, and bridge import identity composition now use the
  centralized Query evidence bridge helpers.

Next compiler frontier after this closure:

- `cargo check --workspace` reaches downstream crates.
- `worth-server` compatibility HTTP surfaces still call removed digest accessors
  such as `basis_digest()` and `basis_binding_digest()`.
- `worth-topo` production mutation-authority paths still pass formatted
  `String` values where `WORTHQueryMutationAuthorityIdentity` is required.

## Phase 2A Upstream Boundary Gate

Status:

```text
closed for current upstream compiler gates; Query repair may resume
```

Reason:

The first post-bridge workspace frontier is `worth-query`, but Query should not
be repaired until upstream relational and signal boundaries are explicitly
classified. Otherwise Query may accidentally encode bridge-local assumptions
around values that originate in relational source truth or signal domain/proof
lanes.

Relational audit summary:

- `worth-relational/src/identity_authority/` exists and classifies relational
  source-truth families.
- `presentation/bridge/identities.rs::parse_bridge_commit_identity` currently
  recovers native commit authority through `TruthCommitIdentity::relational_commit_id()`.
- `presentation/bridge/identities.rs::parse_bridge_snapshot_identity` currently
  recovers native snapshot/version authority through
  `TruthSnapshotIdentity::relational_snapshot_parts()`.
- `record_ref_identity` and `record_ref_from_identity_parts` carry entity and
  relation authority through `RelationalBridgeRecordIdentityParts`.
- Compile-fail guards added under
  `crates/worth-relational/tests/ui/phase_2a/` prove display labels such as
  `commit-*`, `entity:*`, `relation:*`, grouped row labels, and bridge snapshot
  text cannot re-enter bridge truth/record authority.

Signal audit summary:

- `worth-signal` does not currently use a dedicated `FoundationalAuthorityIdentity`
  category scaffold like relational, bridge, or Query.
- Signal branch authority is carried through `SignalBranchBasisIdentity`,
  `SignalBranchBasisArtifact`, and boundary-bridged proof artifacts.
- Signal merge compatibility is proof-scoped through
  `SignalMergeCompatibilityBasis`, `SignalMergeCompatibilityWitness`, and
  `ScopedMergeProofPacket`.
- Host/domain tokens such as `OutputIdentity`, `ArtifactContinuityToken`, and
  `PartitionToken` are signal correspondence/scoping tokens only.
- Compile-fail guards added under `crates/worth-signal/tests/ui/phase_2a/`
  prove `OutputIdentity`, `PartitionToken`, branch-basis digest text, and
  scoped merge proof digest text cannot satisfy the public signal basis/proof
  authority APIs.

Verification:

- `cargo test -p worth-relational --test phase_boundaries_compile_fail phase_2a_upstream_identity_boundaries_reject_projection_reentry` passed.
- `cargo test -p worth-signal --test phase_2a_signal_boundaries` passed.
- `cargo check -p worth-relational --lib` passed.
- `cargo check -p worth-signal --lib` passed.
- Captured logs:
  `plans/worth-query/bridge_truth_identity_phase2a_relational_trybuild_verify.txt`,
  `plans/worth-query/bridge_truth_identity_phase2a_signal_trybuild_verify.txt`,
  `plans/worth-query/bridge_truth_identity_phase2a_relational_lib_check.txt`,
  `plans/worth-query/bridge_truth_identity_phase2a_signal_lib_check.txt`.

Next action:

Resume at the current `worth-query` compiler frontier. Keep Phase 2A rows open
only as watchpoints if a later public signal or relational API exposes a new
authority-category crossing.

## Phase 7 Signal Feeder Lifecycle Slice

Status:

```text
active-lane signal identity and continuation lifecycle admission hardened;
feeder bundle remains open for support/counter/budget/reporting follow-up
```

Implemented closure:

- `ActiveSubscriptionLaneAdmission` and `ActiveSubscriptionLane` now store the
  signal strategy as `WORTHQueryEvidenceIdentity` instead of
  `signal_strategy_digest: String`.
- Active-lane certification now compares the typed signal strategy identity
  rather than comparing reporting digest text.
- `admit_subscription_continuation_evidence(...)` and
  `admit_subscription_continuation_evidence_with_active_identity(...)` require
  typed `WORTHQueryEvidenceIdentity` values for source, target, basis,
  checkpoint, and authority evidence.
- Ordinary continuation checkpoint evidence is minted as an explicit evidence
  identity from the active lane identity instead of the old
  `"active-checkpoint-ordinary"` literal.
- Continuation endpoint identity composition now uses
  `field_evidence_identity(...)`; projection text remains available only through
  reporting accessors.
- Query certification harnesses and subscription tests now build typed
  continuation fixture evidence instead of passing raw labels.
- Added compile-fail coverage for raw continuation identity substitution, and
  kept raw remap width as a separate guard.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- `cargo test -p worth-query subscription::tests::active:: --lib` still fails
  before executing filtered tests because older causal-inspection/workflow test
  fixtures still call bridge identity APIs with raw or display-shaped evidence.
  Those failures are outside this slice and remain part of the causal
  inspection/workflow follow-up rows.
- Captured logs:
  `plans/worth-query/bridge_truth_identity_signal_feeder_lifecycle_check.txt`,
  `plans/worth-query/bridge_truth_identity_signal_feeder_lifecycle_trybuild.txt`,
  `plans/worth-query/bridge_truth_identity_signal_feeder_lifecycle_tests.txt`.

Next action:

Continue the feeder bundle with the remaining support/counter/budget/reporting
surfaces. Do not close the row until those terminal projections are either
quarantined as reports or replaced with typed evidence in lifecycle decisions.

## Upstream Foundation Gate

Status:

```text
closed for current relational, signal, and runtime-bridge public authority gates
```

Result:

- `worth-relational` remains sealed for the current source-truth bridge
  presentation roots: raw commit text, raw snapshot text, raw record labels, and
  grouped row labels cannot satisfy bridge truth/record authority APIs.
- `worth-signal` remains quarantined as a signal proof/domain-token lane:
  `OutputIdentity`, `PartitionToken`, branch-basis digest text, and merge-proof
  digest text cannot satisfy signal basis/proof authority APIs.
- `worth-runtime-bridge` now has all declared bridge truth identity compile-fail
  targets materialized. Projection identity, digest evidence, external token,
  boundary-retained revalidation, wrong marker kind, raw text, retained text
  rebuild, and truth identity string-facade access all fail at the public bridge
  membrane.
- The runtime-bridge digest ledger now hashes the full bridge truth identity UI
  fixture set instead of only the original string-facade guard.

Verification:

- `cargo test -p worth-runtime-bridge --test phase_boundaries_bridge_truth_identity_compile_fail` passed.
- `cargo test -p worth-runtime-bridge --test phase_boundaries_bridge_truth_identity_digest` passed.
- `cargo test -p worth-relational --test phase_boundaries_compile_fail phase_2a_upstream_identity_boundaries_reject_projection_reentry` passed.
- `cargo test -p worth-signal --test phase_2a_signal_boundaries` passed.
- `cargo check -p worth-runtime-bridge --lib` passed.
- `cargo check -p worth-relational --lib` passed.
- `cargo check -p worth-signal --lib` passed.
- Captured logs:
  `plans/worth-query/upstream_foundation_gate_runtime_bridge_trybuild_verify.txt`,
  `plans/worth-query/upstream_foundation_gate_runtime_bridge_digest.txt`,
  `plans/worth-query/upstream_foundation_gate_runtime_bridge_lib.txt`,
  `plans/worth-query/upstream_foundation_gate_relational_trybuild.txt`,
  `plans/worth-query/upstream_foundation_gate_relational_lib.txt`,
  `plans/worth-query/upstream_foundation_gate_signal_trybuild.txt`,
  `plans/worth-query/upstream_foundation_gate_signal_lib.txt`.

Risk posture:

Future upstream issues should be treated as new explicit gates, not as implicit
permission to reopen Query work around strings. The highest remaining upstream
watchpoint is still runtime-bridge diagnostics/causal-envelope retained mapping,
but public bridge truth identity authority now has compiler guards for the
declared Law 42 substitutions.

## Subscription Bridge-Parity Typed Witness

Status:

```text
bridge-parity witness/explanation source validation now typed; feeder row remains open for support/counter/budget/certification/reporting follow-up
```

Implemented closure:

- `QuerySubscriptionManualBridgeWitness` stores typed
  `WORTHQueryEvidenceIdentity` values for query declaration, bridge
  declaration, basis binding, signal strategy, and activation. Existing digest
  accessors are now reporting projections over those typed identities.
- `explain_query_subscription_bridge_parity(...)` validates source coherence
  with typed identity drift checks instead of comparing digest/reporting text.
- Bridge-parity declaration tests assert the witness typed identities match the
  canonical declaration, lowering, basis, signal, and activation artifacts.
- The subscription row remains open because support/counter/budget,
  certification, runtime receipt, and reporting surfaces still need the same
  projection quarantine pass.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- `cargo test -p worth-query subscription::tests::declaration::bridge_parity --lib`
  still fails before reaching the filtered test because the broader lib-test
  build hits the known causal-inspection/workflow fixture wall where old tests
  still call bridge identity APIs with raw or display-shaped evidence.
- Captured logs:
  `plans/worth-query/goal_mode_subscription_bridge_parity_typed_check.txt`,
  `plans/worth-query/goal_mode_subscription_bridge_parity_typed_trybuild.txt`,
  `plans/worth-query/goal_mode_subscription_bridge_parity_typed_tests.txt`.

Next action:

Continue the subscription/live/session feeder bundle through
support/counter/budget/certification/reporting, or switch to the causal
inspection fixture wall if compiler discovery shows that it blocks focused
subscription tests.

## Causal Inspection Fixture Bridge-Evidence Admission Slice

Status:

```text
causal-inspection fixture bridge-evidence admission wall cleared; remaining causal fixture folklore stays open
```

Implemented closure:

- `runtime/tests/causal_inspection` now has one shared bridge-evidence helper
  that makes the Law 42 categories explicit: external token, projection label,
  and digest evidence are created before calling `BridgeIdentityEvidence`
  constructors.
- Materialization, contract, artifact, temporal async, and lower-runtime slot
  fixture helpers delegate to that shared helper instead of calling
  `BridgeIdentityEvidence::from_external_authority(...)` with raw text or
  double-wrapping already admitted bridge evidence.
- Causal materialization support no longer formats `BridgeIdentityEvidence` into
  patch/head/stream commit labels. Patch identities derive from typed relational
  commit positions when available, and branch-head fixtures derive from typed
  branch payloads.
- Workflow certification control evidence now uses
  `WORTHQueryEvidenceIdentity::bridge_external_identity_evidence()` rather than
  treating Query evidence as an external bridge token.
- Targeted scan confirms the only remaining direct bridge evidence constructors
  in the causal-inspection fixture tree are inside the shared category-aware
  helper.

Verification:

- `cargo test -p worth-query subscription::tests::declaration::bridge_parity --lib`
  passed; the previously blocking causal/workflow fixture wall no longer stops
  the focused subscription bridge-parity tests from compiling and running.
- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Targeted scan:
  `rg -n "BridgeIdentityEvidence::from_external_authority\\(|BridgeIdentityEvidence::from_query_evidence_identity\\(|evidence_identity\\(\\)\\s*\\)" crates/worth-query/src/runtime/tests/causal_inspection crates/worth-query/src/harness/workflow_certification/tests.rs`
  returns only the shared causal-inspection helper constructor calls.
- Captured logs:
  `plans/worth-query/goal_mode_causal_fixture_bridge_evidence_frontier.txt`,
  `plans/worth-query/goal_mode_causal_fixture_bridge_evidence_query_check.txt`,
  `plans/worth-query/goal_mode_causal_fixture_bridge_evidence_trybuild.txt`.

Next action:

Continue the Phase 9 causal inspection fixture tree through remaining
truth-commit/patch/head/snapshot folklore and writeback support labels, or
return to the Phase 7 subscription feeder row now that its focused bridge-parity
tests are unblocked.

## Subscription Runtime Support Evidence Adapter Boundary

Status:

```text
runtime subscription activation support evidence now enters receipts as typed evidence, not raw support text
```

Implemented closure:

- `WORTHQueryRuntimeSubscriptionActivationAdapter` now requires
  `support_evidence_identity() -> WORTHQueryEvidenceIdentity`.
- `support_evidence_for_reporting()` is an explicit terminal projection helper
  over the typed support evidence identity for support-profile labels.
- `SubscriptionActivationReceipt::from_activation(...)` requires typed support
  evidence and composes receipt support identity from that source witness instead
  of re-admitting `impl Into<String>`.
- Runtime test adapters, certification fixtures, transcript runtime fixtures,
  lower-runtime phase-six fixtures, and public bridge runtime support now create
  typed support evidence through
  `runtime_subscription_support_evidence_identity(...)`.
- The facade exports the helper so external/public adapters have a standard
  typed path rather than writing bespoke support-evidence identities.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::declaration::bridge_parity --lib` passed.
- `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib` passed.
- `cargo test -p worth-query --test causal_inspection_public_dx` passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_subscription_support_typed_check.txt`,
  `plans/worth-query/goal_mode_subscription_support_typed_bridge_parity_tests.txt`,
  `plans/worth-query/goal_mode_subscription_support_typed_active_lifecycle_tests.txt`,
  `plans/worth-query/goal_mode_subscription_support_typed_public_dx.txt`,
  `plans/worth-query/goal_mode_subscription_support_typed_trybuild.txt`.

Next action:

Continue the subscription feeder row through counter/budget/certification
projection quarantine, then rerun the Phase 7 QA gate before treating the row as
closable.

## Subscription Lifecycle Context And Preview Evidence Boundary

Status:

```text
lifecycle context and preview closeout/certification evidence now compose from typed evidence identities; subscription feeder row remains open
```

Implemented closure:

- `SubscriptionLifecycleCertificationContext` stores policy, tenant-basis,
  relationship-proof, view-shape, and basis-posture as
  `WORTHQueryEvidenceIdentity` values. Digest-named accessors are reporting
  projections over those identities.
- `lifecycle_certification_bundle_identity(...)` now receives those lifecycle
  context identities and seals them via `field_evidence_identity`, not raw
  digest text.
- Preview discard and promotion closeout artifacts store the residue report
  identity. Their `residue_report_digest()` accessors remain terminal reporting
  projections only.
- Preview lifecycle certification evidence now carries typed absent/isolation,
  residue, and performance identities internally. The old `"none"` performance
  sentinel path was replaced with typed absent identities.
- Preview residue support evidence and promotion-residue composition now bind
  the residue report, handoff, and authoritative lane through typed evidence
  identities rather than string projections.
- Compile-fail fixtures now catch the removed raw
  `residue_report_digest`/lifecycle digest fields and the typed
  `SubscriptionLifecycleCertificationContext::admitted(...)` boundary.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::certification:: --lib`
  passed.
- `cargo test -p worth-query subscription::tests::active::active_preview_isolation --lib`
  passed.
- `cargo test -p worth-query subscription::tests::diagnostic_bundles --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_subscription_lifecycle_preview_typed_check.txt`,
  `plans/worth-query/goal_mode_subscription_lifecycle_preview_typed_certification_tests.txt`,
  `plans/worth-query/goal_mode_subscription_lifecycle_preview_typed_preview_tests.txt`,
  `plans/worth-query/goal_mode_subscription_lifecycle_preview_typed_diagnostic_tests.txt`,
  `plans/worth-query/goal_mode_subscription_lifecycle_preview_typed_trybuild.txt`.

Next action:

Continue row 1051 through remaining runtime live subscription,
counter/budget/reporting projection quarantine before attempting to close the
subscription feeder row.

## Active Subscription Lane Source Identity Boundary

Status:

```text
active lane source identities are typed through admission/opening/certification; row 1051 remains open for counter/budget/reporting surfaces
```

Implemented closure:

- `ActiveSubscriptionLaneAdmission` and `ActiveSubscriptionLane` now retain
  typed activation, admission, query-declaration, and bridge-declaration
  identities instead of storing those source values as `String` digests.
- The old `activation_digest()`, `admission_digest()`,
  `query_declaration_digest()`, and `bridge_declaration_digest()` accessors are
  reporting projections over typed identities.
- `open_active_subscription_lane(...)` carries the typed source identities into
  the opened lane record without re-stringifying.
- `certify_subscription_lifecycle(...)` validates active-lane source coherence
  with typed identity drift checks for activation/admission/query/bridge/basis
  and signal evidence instead of digest equality.
- The active-lane compile-fail fixture now captures that raw digest-named fields
  cannot be used to fabricate an active lane.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib`
  passed.
- `cargo test -p worth-query subscription::tests::certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_active_lane_source_typed_check.txt`,
  `plans/worth-query/goal_mode_active_lane_source_typed_active_lifecycle_tests.txt`,
  `plans/worth-query/goal_mode_active_lane_source_typed_certification_tests.txt`,
  `plans/worth-query/goal_mode_active_lane_source_typed_trybuild.txt`.

Next action:

Continue row 1051 through subscription scale/counter snapshots and runtime
certification support-report surfaces that still store digest strings for
reporting or comparison.

## Subscription Scale Slope Identity Boundary

Status:

```text
scale counter snapshots and slope reports now compose from typed evidence identities; row 1051 remains open for runtime certification support-report and coverage surfaces
```

Implemented closure:

- `QuerySubscriptionScaleCounterSnapshot` stores typed activation, admission,
  counter, and snapshot evidence identities. Digest-named accessors remain
  terminal reporting projections over those identities.
- `QuerySubscriptionScaleSlopeReport` stores typed report, source, snapshot,
  and structural-counter identities instead of reconstructing report identity
  from digest strings.
- `certify_query_subscription_scale_slope(...)` validates activation/admission
  source coherence through typed identity drift checks and composes the slope
  report with `scale_slope_report_identity(...)`.
- `certify_query_subscription_activation(...)` validates scale source coherence
  through typed drift checks and feeds the certification bundle with the borrowed
  scale report identity, not a temporary digest reconstruction.
- Compile-fail fixtures now capture that the removed raw digest-named fields
  cannot fabricate scale snapshots or slope reports.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::certification::scale_slope --lib`
  passed.
- `cargo test -p worth-query subscription::tests::certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_scale_slope_typed_check.txt`,
  `plans/worth-query/goal_mode_scale_slope_typed_scale_tests.txt`,
  `plans/worth-query/goal_mode_scale_slope_typed_certification_tests.txt`,
  `plans/worth-query/goal_mode_scale_slope_typed_trybuild.txt`,
  `plans/worth-query/goal_mode_scale_slope_typed_runtime_bridge_digest.txt`.

Next action:

Continue row 1051 through runtime certification support-report and coverage
surfaces, especially any remaining budget/reporting projections that still
participate in lifecycle decisions instead of staying terminal reports.

## Runtime Certification Bundle Identity Boundary

Status:

```text
runtime certification scope, bundle, coverage rows, matrices, variation sets, and handles now carry typed source evidence where upstream artifacts expose it; row 1051 remains open for bridge-parity explanation identity and remaining basis/view-shape projection quarantine
```

Implemented closure:

- `build_query_subscription_runtime_certification_scope(...)` validates support,
  bridge-parity witness, and lifecycle declaration/bridge sources with typed
  identity drift checks instead of comparing reporting strings.
- `QuerySubscriptionRuntimeCertificationBundle` stores typed subscription
  declaration, bridge declaration, signal strategy, support report, diagnostic
  bundle, lifecycle certification, hostile coverage, runtime certification
  bundle, and counter identities. Digest-named accessors now project those
  identities for reporting.
- `SubscriptionCertificationCoverageWidth` and `CertificationCoverageReceipt`
  now carry typed evidence identities; their `digest()` accessors are reporting
  projections.
- Runtime certification identity composition moved into
  `runtime_certification/identities.rs`, keeping `bundle.rs` under the
  workspace line-cap rule after the hard break.
- The former over-cap `runtime_certification/coverage.rs` monolith is split
  into a small facade plus `coverage/row.rs`, `coverage/validation.rs`,
  `coverage/variations.rs`, and `coverage/matrix.rs`; all new files are under
  the workspace line cap.
- `QuerySubscriptionFamilyCoverageRow` now stores typed subscription
  declaration, bridge declaration, signal strategy, support report, lifecycle
  certification, diagnostic bundle, policy, tenant-basis, relationship-proof,
  and row identities. Digest-named accessors are projections.
- `QuerySubscriptionFamilyCoverageMatrix`, variation sets, hostile coverage,
  and `CertifiedFamilyCoverageHandle` now compose typed row/variation identities
  instead of rebuilding authority evidence from digest strings.
- Compile-fail coverage now captures that the old raw digest-named runtime
  certification bundle fields cannot fabricate a bundle.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_runtime_certification_bundle_typed_check.txt`,
  `plans/worth-query/goal_mode_runtime_certification_bundle_typed_tests.txt`,
  `plans/worth-query/goal_mode_runtime_certification_bundle_typed_trybuild.txt`,
  `plans/worth-query/goal_mode_runtime_certification_bundle_typed_runtime_bridge_digest.txt`,
  `plans/worth-query/goal_mode_runtime_coverage_split_check.txt`,
  `plans/worth-query/goal_mode_runtime_coverage_typed_check.txt`,
  `plans/worth-query/goal_mode_runtime_coverage_typed_tests.txt`,
  `plans/worth-query/goal_mode_runtime_coverage_typed_trybuild.txt`,
  `plans/worth-query/goal_mode_runtime_coverage_typed_runtime_bridge_digest.txt`.

Next action:

Continue row 1051 through bridge-parity explanation identity and remaining
basis/view-shape projection quarantine. Coverage is no longer the blocking
runtime-certification surface.

## Bridge-Parity Explanation Identity Boundary

Status:

```text
bridge-parity explanation, comparison, receipt, width, counter, and failure artifacts now carry typed evidence identities; row 1051 remains open for budget/reporting projections and basis/view-shape projection quarantine
```

Implemented closure:

- `bridge_parity/identities.rs` defines local composition helpers over
  `WORTHQueryEvidenceIdentity` for bridge-parity counters, width, receipt,
  comparison, explanation, and failures. These helpers use the canonical
  evidence encoder and do not introduce a crate-local authority category.
- `SubscriptionBridgeParityWidth`, `BridgeParityReceipt`,
  `QuerySubscriptionBridgeParityFailure`,
  `QuerySubscriptionBridgeParityComparison`, and
  `QuerySubscriptionBridgeParityExplanation` now store typed evidence
  identities. Existing digest-named accessors are reporting projections over
  those identities.
- Runtime certification coverage rows and certification bundles now retain the
  bridge-parity explanation identity as typed evidence. Row lookup compares
  typed parity identity directly instead of comparing projected text.
- Runtime certification identity composition now uses `field_evidence_identity`
  for bridge-parity evidence, closing the last bridge-parity string re-entry in
  the runtime certification bundle/coverage path.
- Compile-fail fixtures were updated so the old raw digest-named bundle fields
  and the new private identity-backed fields cannot fabricate bridge parity or
  runtime certification artifacts.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::declaration::bridge_parity --lib`
  passed.
- `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_bridge_parity_identity_check.txt`,
  `plans/worth-query/goal_mode_bridge_parity_identity_tests.txt`,
  `plans/worth-query/goal_mode_bridge_parity_identity_runtime_certification_tests.txt`,
  `plans/worth-query/goal_mode_bridge_parity_identity_trybuild.txt`.

Next action:

Continue row 1051 through budget/reporting projections and basis/view-shape
projection quarantine. Bridge parity is no longer the blocking runtime
certification surface.

## Runtime Coverage Basis/View-Shape Identity Boundary

Status:

```text
runtime certification coverage no longer rebuilds basis or view-shape variation evidence from digest strings; row 1051 remains open for budget/reporting projections and other lifecycle decisions that still rely on digest text
```

Implemented closure:

- `SubscriptionLifecycleCertificationBundle` now preserves typed
  `view_shape_identity` and `basis_posture_identity` values from the lifecycle
  context instead of storing only reporting strings.
- `QuerySubscriptionFamilyCoverageRow` stores typed basis and view-shape
  evidence identities. `basis_digest()` and `view_shape_digest()` are reporting
  projections only.
- `coverage_row_identity(...)` now requires typed basis and view-shape evidence
  identities and composes them with `field_evidence_identity`.
- Runtime certification variation sets now compose typed evidence identity
  sequences via `coverage_evidence_variation_set_identity(...)`; the digest list
  remains available only for reporting/emptiness checks.
- `build_certified_family_coverage_handle(...)` collects policy, tenant-basis,
  relationship-proof, basis, and view-shape variation evidence from row identity
  accessors rather than digest strings.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_runtime_coverage_basis_view_typed_check.txt`,
  `plans/worth-query/goal_mode_runtime_coverage_basis_view_typed_tests.txt`,
  `plans/worth-query/goal_mode_runtime_coverage_basis_view_typed_trybuild.txt`.

Next action:

Continue row 1051 through the remaining budget/reporting projection paths and
other lifecycle decisions that still rely on digest text.

## Runtime Budget Identity Verification Boundary

Status:

```text
runtime live subscription budget verification now compares typed evidence identity values; row 1051 remains open for other budget/reporting projections and lifecycle decisions that still rely on digest text
```

Implemented closure:

- The legacy-named `runtime_subscription_budget_digest()` test helper now
  returns `WORTHQueryEvidenceIdentity` instead of reconstructing and returning a
  raw budget string.
- Runtime assembly tests compare
  `WORTHQueryRuntimeLiveSubscriptionInstallation::runtime_budget_identity()`
  with the typed helper identity directly.
- The runtime budget reporting accessor remains a terminal projection over the
  stored `WORTHQueryEvidenceIdentity`; it no longer participates in the tested
  verification path.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query runtime::tests::assembly::builder:: --lib` passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_runtime_budget_identity_check.txt`,
  `plans/worth-query/goal_mode_runtime_budget_identity_assembly_tests.txt`,
  `plans/worth-query/goal_mode_runtime_budget_identity_trybuild.txt`.

Next action:

Continue row 1051 through the remaining lifecycle/reporting digest decisions,
especially places where `*_for_reporting()` values still participate in source
coherence checks rather than diagnostics.

## Preview Lifecycle Source Identity Boundary

Status:

```text
preview discard/promotion lifecycle certification compares closeout, isolation, discard, and handoff source evidence through typed identities; row 1051 remains open for activation/admission, runtime-certification diagnostic, and other lifecycle/reporting digest decisions
```

Implemented closure:

- `SubscriptionLifecycleCloseout` now retains the typed source identity used to
  compose the closeout evidence instead of exposing only `source_digest()`.
- Preview discard certification compares lifecycle closeout basis, checkpoint,
  source, discard basis, discard checkpoint, and discard preview epoch through
  `typed_identity_drift(...)`.
- Preview promotion certification compares lifecycle closeout basis,
  checkpoint, source, handoff preview basis, handoff preview checkpoint, and
  handoff preview epoch through `typed_identity_drift(...)`.
- The `subscription_lifecycle_closeout_constructor_private` compile-fail fixture
  now explicitly guards the new private `source_identity` field.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query subscription::tests::active::active_preview_isolation --lib`
  passed.
- `cargo test -p worth-query subscription::tests::certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_preview_lifecycle_source_typed_check.txt`,
  `plans/worth-query/goal_mode_preview_lifecycle_source_typed_preview_tests.txt`,
  `plans/worth-query/goal_mode_preview_lifecycle_source_typed_certification_tests.txt`,
  `plans/worth-query/goal_mode_preview_lifecycle_source_typed_trybuild.txt`.

Next action:

Continue row 1051 with runtime-certification diagnostic lifecycle/reporting
comparisons and other subscription/runtime-session decisions that still rely on
digest text.

## Activation Admission And Runtime Coverage Source Alignment

Status:

```text
activation/admission query declaration coherence and runtime family coverage source alignment now compare typed evidence identities; row 1051 remains open for runtime-certification diagnostic lifecycle/reporting comparisons and other subscription/runtime-session digest decisions
```

Implemented closure:

- `certify_query_subscription_activation(...)` now compares activation and
  admission query declaration identity through `typed_identity_drift(...)`
  instead of comparing `query_declaration_for_reporting()` projection strings.
- Runtime certification coverage validation now compares support-subject,
  bridge-parity comparison, and lifecycle declaration/bridge identities through
  typed evidence identity drift checks.
- The old digest/reporting accessors remain only in denial detail payloads,
  where they are terminal diagnostics rather than authority/coherence inputs.

Verification:

- `cargo check -p worth-query --lib` completed successfully in the saved log.
- `cargo test -p worth-query subscription::tests::certification:: --lib`
  passed.
- `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_activation_coverage_typed_check.txt`,
  `plans/worth-query/goal_mode_activation_coverage_typed_certification_tests.txt`,
  `plans/worth-query/goal_mode_activation_coverage_typed_runtime_certification_tests.txt`,
  `plans/worth-query/goal_mode_activation_coverage_typed_trybuild.txt`.

Next action:

Continue row 1051 through any remaining subscription/runtime-session digest
decisions after diagnostic trace source checks.

## Diagnostic Trace Source Identity Boundary

Status:

```text
diagnostic stage traces now retain typed source identities and admitted/denied/runtime-certification trace validators compare typed evidence identity; row 1051 remains open for any remaining subscription/runtime-session digest decisions
```

Implemented closure:

- `QuerySubscriptionDiagnosticEvidence` now stores the typed source identity
  passed by the stage producer instead of retaining only its reporting string.
- `QuerySubscriptionDiagnosticStageTrace` now carries that typed source identity
  directly and composes the stage trace from it. The old synthetic
  `query_subscription_diagnostic_stage_source_projection_v1` source identity is
  removed.
- Admitted diagnostic bundle trace validation now compares family selection,
  declaration, bridge lowering, admission, support, certification, continuation,
  preview isolation, and closeout stage sources through `typed_identity_drift`.
- Denied diagnostic bundle validation now uses typed failure, selection,
  declaration, bridge lowering, admission, and support source identities for
  stage alignment.
- Runtime certification hostile diagnostic alignment now compares denied trace
  stage sources against lifecycle declaration, bridge, and admission identities
  with typed identity drift checks.

Verification:

- `cargo check -p worth-query --lib` completed successfully in the saved log.
- `cargo test -p worth-query subscription::tests::diagnostic_bundles --lib`
  passed.
- `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_diagnostic_trace_source_typed_check.txt`,
  `plans/worth-query/goal_mode_diagnostic_trace_source_typed_diagnostic_tests.txt`,
  `plans/worth-query/goal_mode_diagnostic_trace_source_typed_runtime_certification_tests.txt`,
  `plans/worth-query/goal_mode_diagnostic_trace_source_typed_trybuild.txt`.

Next action:

Continue row 1051 by searching for any remaining subscription/runtime-session
coherence checks that still compare `*_digest()` or `*_for_reporting()` output
instead of typed identity values.

## Diagnostic Bundle Support And Lifecycle Identity Boundary

Status:

```text
diagnostic bundles now carry typed support-report and lifecycle-certification identities into runtime certification scope/coverage checks; row 1051 remains open pending a final search for remaining subscription/runtime-session digest decisions
```

Implemented closure:

- `QuerySubscriptionDeniedDiagnosticBundle` now carries optional typed
  `support_report_identity` beside its reporting digest.
- `QuerySubscriptionAdmittedDiagnosticBundle` now carries typed
  `support_report_identity` and `lifecycle_certification_identity` beside the
  existing reporting digest accessors.
- Runtime certification scope source validation now compares admitted diagnostic
  support/lifecycle references through `typed_identity_drift(...)`.
- Runtime certification coverage row assembly now validates admitted diagnostic
  support/lifecycle references through typed identity drift before indexing the
  row.
- Hostile coverage validation now compares denied diagnostic support-report
  identity against the supplied support report identity, not report text.
- The `subscription_diagnostic_bundle_constructor_private` compile-fail fixture
  was updated to guard the new private identity fields.

Verification:

- `cargo check -p worth-query --lib` completed successfully in the saved log.
- `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`
  passed.
- `cargo test -p worth-query subscription::tests::diagnostic_bundles --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed after
  refreshing the one intentional constructor-private stderr.
- Captured logs:
  `plans/worth-query/goal_mode_diagnostic_bundle_support_identity_check.txt`,
  `plans/worth-query/goal_mode_diagnostic_bundle_support_identity_runtime_certification_tests.txt`,
  `plans/worth-query/goal_mode_diagnostic_bundle_support_identity_diagnostic_tests.txt`,
  `plans/worth-query/goal_mode_diagnostic_bundle_support_identity_trybuild_verify.txt`.

Next action:

Continue row 1051 by running a final targeted subscription/runtime-session scan
for remaining authority/coherence checks that still operate on projection text.

## Active Lifecycle Source Identity Boundary

Status:

```text
active lifecycle certification, delivery-window/work-packet construction, and continuation checkpoint reporting now use typed source identities for lane, attachment, delta, window, and checkpoint coherence; row 1051 remains open pending the remaining active registry/runtime-session scan
```

Implemented closure:

- `certify_subscription_lifecycle(...)` now requires a typed
  `WORTHQueryEvidenceIdentity` delivery-window witness instead of accepting an
  `impl Into<String>` delivery-window digest.
- Lifecycle certification source checks now compare active lane, attachment,
  maintenance delta, lowering report, work packet, delivery batch, receipt,
  acknowledgement, continuation, and closeout references through
  `typed_identity_drift(...)` over evidence identities.
- `QueryMaintenanceDeltaLoweringReport` now retains the typed maintenance delta
  identity it was lowered from, so work packets and lifecycle certification no
  longer validate the lowering report against delta text.
- `ActiveDeliveryWorkPacket::new(...)` and `QueryDeliveryBatch::new(...)` now
  validate lane/attachment/delta source alignment through typed evidence
  identity drift checks.
- `SubscriptionContinuationReport` now retains typed checkpoint identity, and
  continuation tests assert typed checkpoint preservation rather than comparing
  `checkpoint_identity_digest()` projection text.
- Test and harness lifecycle certification callers now pass
  `delivery_batch.delivery_window_identity()` as the required typed witness.
- The `subscription_continuation_report_constructor_private` compile-fail
  fixture was updated to guard the new private `checkpoint_identity` field.

Verification:

- `cargo check -p worth-query --lib` completed successfully in the saved log.
- `cargo test -p worth-query subscription::tests::certification:: --lib`
  passed.
- `cargo test -p worth-query subscription::tests::active::active_delivery --lib`
  passed.
- `cargo test -p worth-query subscription::tests::active::active_continuation --lib`
  passed.
- `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed after
  refreshing the one intentional continuation-report stderr.
- Captured logs:
  `plans/worth-query/goal_mode_lifecycle_typed_sources_check.txt`,
  `plans/worth-query/goal_mode_lifecycle_typed_sources_certification_tests_verify.txt`,
  `plans/worth-query/goal_mode_lifecycle_typed_sources_active_delivery_tests.txt`,
  `plans/worth-query/goal_mode_lifecycle_typed_sources_active_continuation_tests_fixed.txt`,
  `plans/worth-query/goal_mode_lifecycle_typed_sources_runtime_certification_tests.txt`,
  `plans/worth-query/goal_mode_lifecycle_typed_sources_trybuild_verify.txt`.

Next action:

Continue row 1051 with active registry/runtime-session source checks, especially
places that still index or compare active lane and attachment identities through
string keys.

## Active Registry Typed Lookup Boundary

Status:

```text
active lane and attachment registry lookup no longer admits raw string keys; row 1051 active-registry source checks are closed, with remaining row 1051 work limited to non-registry subscription/runtime-session scans
```

Implemented closure:

- `ActiveSubscriptionLaneRegistry` now indexes active lanes by
  `ActiveSubscriptionLaneDigest` instead of `String`.
- Consumer attachment registration and closeout now accept
  `SubscriptionConsumerAttachmentDigest` at the registry boundary instead of
  `&str`, so callers cannot drive registry membership with projection text.
- `ActiveSubscriptionLaneDigest` and
  `SubscriptionConsumerAttachmentDigest` implement deterministic ordering at the
  narrowed digest-wrapper layer using typed evidence scope, scheme, and token.
  The broader `WORTHQueryEvidenceIdentity` type remains non-ordered/non-hashable
  so generic evidence does not become a reusable string-key substitute.
- `ActiveSubscriptionRuntime` passes typed attachment digests into registry
  registration/closeout. Remaining `as_str()` uses in the slice are terminal
  denial/reporting projections only.

Verification:

- `cargo check -p worth-query --lib` completed successfully in the saved log.
- `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib`
  passed.
- `cargo test -p worth-query subscription::tests::active::active_sharing --lib`
  passed.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed.
- Captured logs:
  `plans/worth-query/goal_mode_active_registry_typed_keys_check.txt`,
  `plans/worth-query/goal_mode_active_registry_typed_keys_active_lifecycle_tests.txt`,
  `plans/worth-query/goal_mode_active_registry_typed_keys_active_sharing_tests.txt`,
  `plans/worth-query/goal_mode_active_registry_typed_keys_trybuild.txt`.

Next action:

Continue row 1051 with a non-registry subscription/runtime-session scan for
remaining authority/coherence checks that still compare lane, attachment,
declaration, signal, support, or lifecycle identities through projection text.

## Live Read Result Shape Authority Boundary

Status:

```text
live-read receipts now retain canonical result-shape digest/evidence from declarative canonicalization instead of reusing the live-view/view-family projection as result-shape authority; row 1051 live-read result-shape projection re-entry is closed
```

Implemented closure:

- `LoweredRuntimeLiveSubscriptionRequest` carries the canonical result-shape
  digest from `DeclarativeLiveQuerySession::canonical().result_shape()`.
- `WORTHQueryRuntimeLiveSubscriptionInstallation` stores that
  `CanonicalResultShapeDigest` and its evidence identity beside the existing
  live-view family identity. The live-view identity remains a live subscription
  input, not the result-shape witness.
- `WORTHQueryLiveReadReceipt` stores `CanonicalResultShapeDigest` plus typed
  evidence identity. The legacy `view_shape_digest()` accessor now projects the
  canonical result-shape digest for compatibility, instead of projecting the
  live-view/view-family identity.
- Retained/live projection test fixtures were hard-broken so test-only live
  receipts must pass a `CanonicalResultShapeDigest`, not a raw shape string.
- `runtime/live_subscription.rs` was split so the touched runtime files remain
  under the workspace Rust line cap; installation accessors now live in
  `runtime/live_subscription_accessors.rs`.

Verification:

- `cargo check -p worth-query --lib` completed successfully before and after
  the accessor split.
- `cargo test -p worth-query projection_consumption::tests::retained_live --lib`
  passed before and after the accessor split.
- `cargo test -p worth-query runtime::tests::live --lib` passed before and
  after the accessor split.
- `cargo test -p worth-query runtime::tests::live_receipts --lib` passed before
  and after the accessor split.
- `cargo test -p worth-query --test phase_boundaries_compile_fail` passed before
  and after the accessor split.
- Captured logs:
  `plans/worth-query/goal_mode_live_read_result_shape_typed_check.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_check_after_split.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_retained_live_tests.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_retained_live_tests_after_split.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_runtime_live_tests.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_runtime_live_tests_after_split.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_live_receipts_tests.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_live_receipts_tests_after_split.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_trybuild.txt`,
  `plans/worth-query/goal_mode_live_read_result_shape_typed_trybuild_after_split.txt`.

Next action:

Continue row 1051 with remaining non-registry subscription/runtime-session
projection checks, especially activation support, live declaration admission
receipts, and any source metadata still allowing projection text to stand in for
canonical evidence.

## Live Declaration Admission Typed Shape Boundary

Status:

```text
live declaration admission receipts now retain declarative view shape as the typed `DeclarativeLiveViewShape`; shape drift no longer compares receipt shape authority through projected shape text
```

Implemented closure:

- `LiveViewDeclarationAdmissionReceipt` stores the request
  `DeclarativeLiveViewShape` directly instead of copying
  `request.view_shape().as_str()` into a raw string field.
- Drift detection compares the stored typed shape to
  `request.view_shape()` instead of comparing two reporting labels.
- The old shape projection is quarantined behind
  `view_shape_for_reporting()`, while `view_shape()` returns the typed
  declarative shape.
- The target collection remains a declaration/reporting label for this seam;
  it is exposed as `target_collection_for_reporting()` and is not promoted into
  an authority identity.

Verification:

- `cargo check -p worth-query --lib` completed successfully.
- `cargo test -p worth-query live_view_declaration_receipt_captures_request_shape --lib`
  passed.
- Captured logs:
  `plans/worth-query/goal_mode_live_declaration_admission_typed_shape_check.txt`,
  `plans/worth-query/goal_mode_live_declaration_admission_typed_shape_test.txt`.

Next action:

Continue row 1051 with remaining non-registry subscription/runtime-session
projection checks, especially activation support and any source metadata still
allowing projection text to stand in for canonical evidence.

## Active Lifecycle Denial Kind Boundary

Status:

```text
active lifecycle closeout no longer infers attachment denial class by comparing projected digest strings
```

Implemented closure:

- `ActiveSubscriptionLifecycleDenialKind` now has explicit
  `AttachmentNotActive` and `AttachmentLaneMismatch` variants for registry
  closeout failures.
- `ActiveSubscriptionLaneRegistry::close_attachment(...)` emits those variants
  directly instead of reporting every attachment closeout failure as generic
  registry equivalence mismatch.
- `close_subscription_lifecycle(...)` maps lifecycle denial kind to closeout
  denial kind by enum classification instead of checking whether
  `error.source_digest() == request.attachment_digest().as_str()`.
- The remaining attachment digest text in this slice is diagnostic/reporting
  projection, not control-flow authority.

Verification:

- `cargo check -p worth-query --lib` completed successfully.
- `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib`
  passed.
- `cargo test -p worth-query subscription::tests::active::active_closeout --lib`
  passed.
- Targeted scan found no remaining `source_digest()` string equality in
  `crates/worth-query/src/subscription` or `crates/worth-query/src/runtime`.
- Captured logs:
  `plans/worth-query/goal_mode_active_lifecycle_denial_kind_check.txt`,
  `plans/worth-query/goal_mode_active_lifecycle_denial_kind_active_lifecycle_tests.txt`,
  `plans/worth-query/goal_mode_active_lifecycle_denial_kind_active_closeout_tests.txt`.

Next action:

Continue row 1051 with remaining subscription/runtime-session projection checks,
especially activation support metadata and broader lifecycle/certification
attachment projection rows.

## Subscription Support Profile Typed Source Boundary

Status:

```text
subscription support profiles compose source identity from typed evidence, not source digest text
```

Implemented closure:

- `QuerySubscriptionSupportProfile::{admitted, denied,
  active_runtime_admitted}` now require `&WORTHQueryEvidenceIdentity` as the
  source input.
- `QuerySubscriptionSupportProfile` stores the source evidence identity beside
  its reporting projection.
- Support profile identity composition now uses
  `field_evidence_identity("source", source_identity)` instead of
  `field_shape("source", source_digest)`.
- Subscription admission passes the bridge declaration identity directly into
  the admitted support profile.
- Admission denial passes the diagnostic source evidence identity directly into
  the denied support profile.
- Lifecycle closeout passes the request source evidence identity directly into
  the active-runtime support profile.

Verification:

- `cargo check -p worth-query --lib` completed successfully.
- `cargo test -p worth-query subscription::tests::admission --lib` passed.
- `cargo test -p worth-query subscription::tests::diagnostics --lib` passed.
- `cargo test -p worth-query subscription::tests::active::active_closeout --lib`
  passed.
- Targeted scan confirms `query_subscription_support_profile_v1` now composes
  source via `field_evidence_identity`; remaining support subject/report source
  strings are the next projection-review target.
- Captured logs:
  `plans/worth-query/goal_mode_support_profile_typed_source_check.txt`,
  `plans/worth-query/goal_mode_support_profile_typed_source_admission_tests.txt`,
  `plans/worth-query/goal_mode_support_profile_typed_source_diagnostics_tests.txt`,
  `plans/worth-query/goal_mode_support_profile_typed_source_active_closeout_tests.txt`.

Next action:

Continue row 1051 with support subject/report source projection review and
broader lifecycle/certification attachment projection rows.

## Subscription Support Report Typed Source Boundary

Status:

```text
subscription support reports retain typed source identity; source digest remains a terminal reporting projection
```

Implemented closure:

- `QuerySubscriptionSupportSubject` already retained typed source identity for
  declaration, activation, active lifecycle, continuation, and preview-closeout
  support subjects.
- `QuerySubscriptionSupportReport` now stores `source_identity:
  WORTHQueryEvidenceIdentity` beside the existing reporting projection.
- `QuerySubscriptionSupportReport::source_identity()` exposes the typed source
  identity for downstream certification/report consumers that need provenance
  without re-entering through `source_digest()`.
- `source_digest()` remains a terminal reporting accessor.

Verification:

- `cargo check -p worth-query --lib` completed successfully.
- `cargo test -p worth-query subscription::tests::support --lib` passed.
- `cargo test -p worth-query subscription::tests::runtime_certification --lib`
  passed.
- Captured logs:
  `plans/worth-query/goal_mode_support_report_typed_source_check.txt`,
  `plans/worth-query/goal_mode_support_report_typed_source_support_tests.txt`,
  `plans/worth-query/goal_mode_support_report_typed_source_runtime_certification_tests.txt`.

Next action:

Continue row 1051 with broader lifecycle/certification attachment projection
rows and any remaining runtime-session source metadata paths.

## Runtime Certification Diagnostic Identity Match Boundary

Status:

```text
runtime certification scope and coverage variation sets no longer use projection text for diagnostic identity matching or evidence identity deduplication
```

Implemented closure:

- `QuerySubscriptionRuntimeCertificationScope` now matches admitted coverage
  rows to the admitted diagnostic bundle via typed diagnostic bundle identity
  drift checks instead of `row.diagnostic_bundle_digest() ==
  admitted_diagnostic_bundle.bundle_digest()`.
- Runtime certification coverage variation sets now sort and deduplicate
  `WORTHQueryEvidenceIdentity` values by typed evidence scope, scheme, and
  token, instead of deduplicating by `as_str()` alone.
- Digest/reporting accessors remain available for terminal failure evidence and
  report output, but not for coverage membership or variation identity
  equivalence.

Verification:

- `cargo check -p worth-query --lib` completed successfully.
- `cargo test -p worth-query subscription::tests::runtime_certification --lib`
  passed.
- `cargo test -p worth-query subscription::tests::runtime_certification_closure_support --lib`
  passed.
- Targeted scan found no remaining diagnostic-bundle digest equality or
  `as_str()`-only runtime-certification variation dedup.
- Captured logs:
  `plans/worth-query/goal_mode_runtime_certification_diagnostic_identity_check.txt`,
  `plans/worth-query/goal_mode_runtime_certification_diagnostic_identity_tests.txt`,
  `plans/worth-query/goal_mode_runtime_certification_diagnostic_identity_closure_support_tests.txt`.

Next action:

Continue row 1051 with remaining runtime-session source metadata and lifecycle
certification projection checks.

## Runtime Attachment Cursor Seed Identity Boundary

Status:

```text
runtime live subscription attachment requests no longer build delivery cursor identity from activation reporting text
```

Implemented closure:

- `SubscriptionConsumerAttachmentRequest` now carries a typed
  `delivery_cursor_seed_identity: WORTHQueryEvidenceIdentity` beside its
  terminal reporting projection.
- `SubscriptionConsumerAttachmentRequest::from_consumer_identity(...)` now
  requires a typed cursor seed identity instead of `impl Into<String>`.
- `SubscriptionConsumerAttachment` composes delivery cursor identity with
  `field_evidence_identity("seed", request.delivery_cursor_seed_identity())`
  instead of `field_value("seed", request.delivery_cursor_seed())`.
- `WORTHQueryRuntime::install_live_subscription_for_request(...)` passes
  `activation_receipt.activation_identity().clone()` as the cursor seed identity
  instead of `activation_receipt.activation_for_reporting()`.
- The raw-label `SubscriptionConsumerAttachmentRequest::admitted(...)` helper
  now classifies its fixture/user seed as a local cursor-seed evidence identity
  before attachment identity composition.

Verification:

- `cargo check -p worth-query --lib` completed successfully.
- `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib`
  passed.
- `cargo test -p worth-query subscription::tests::active::active_sharing --lib`
  passed.
- `cargo test -p worth-query runtime::tests::live --lib` passed.
- Targeted scan found no remaining runtime-session
  `activation_for_reporting()` input to `from_consumer_identity(...)`.
- Captured logs:
  `plans/worth-query/goal_mode_attachment_cursor_seed_identity_check.txt`,
  `plans/worth-query/goal_mode_attachment_cursor_seed_identity_active_lifecycle_tests.txt`,
  `plans/worth-query/goal_mode_attachment_cursor_seed_identity_active_sharing_tests.txt`,
  `plans/worth-query/goal_mode_attachment_cursor_seed_identity_runtime_live_tests.txt`.

Next action:

Run a final row 1051 targeted scan for non-terminal subscription/runtime-session
projection re-entry before deciding whether the row can close or must remain
open with a specific residual owner.

## Row 1051 Runtime Session Closure Scan

Status:

```text
subscription/runtime-session row 1051 has no remaining targeted non-terminal projection or digest re-entry in the scanned authority paths
```

Implemented closure:

- Runtime live subscription lowering helpers moved from
  `runtime/runtime_sessions.rs` into `runtime/runtime_session_lowering.rs`,
  keeping the session entrypoint file under the workspace Rust line cap after
  row 1051 edits.
- The split preserves the authority boundary shape: runtime session entrypoints
  call typed lowering/activation helpers, and live attachment still receives the
  activation receipt identity as typed cursor-seed evidence.
- Final targeted scans found no direct non-terminal equality against
  `*_for_reporting()`, `*_digest()`, or `source_digest()` in the row 1051
  subscription/runtime-session paths.
- Final targeted scans found no authority identity composition through
  `field_identity(...)` or `hash_parts(...)` fed by reporting, digest, or
  `as_str()` projections in the row 1051 subscription/runtime-session paths.
- Remaining search hits are terminal/reporting accessor plumbing in runtime
  receipts and the typed `SubscriptionConsumerAttachmentRequest::from_consumer_identity(...)`
  call site.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query runtime::tests::live --lib` passed.
- `cargo test -p worth-query runtime::tests::session_label_outputs --lib`
  passed after the runtime-session file split.
- `runtime/runtime_sessions.rs` is now 284 lines and
  `runtime/runtime_session_lowering.rs` is 153 lines.
- Captured logs:
  `plans/worth-query/goal_mode_row_1051_runtime_session_split_check.txt`,
  `plans/worth-query/goal_mode_row_1051_runtime_session_split_live_tests.txt`,
  `plans/worth-query/goal_mode_row_1051_runtime_session_split_session_label_tests.txt`.

Next action:

Close the row 1051 ledger entry, then continue the next compiler-first milestone
slice from the remaining open 9.6 rows.

## Workflow And Domain Capability Typed Coherence Boundary

Status:

```text
row 1052 workflow/domain-capability production coherence checks no longer compare reporting digest text
```

Implemented closure:

- `inspect_post_merge_outcome(...)` now verifies declaration/outcome coherence
  through typed workflow query and basis identities:
  `declaration.binding().query_identity()` against
  `outcome.source_query_identity()`, and
  `declaration.binding().basis_identity()` against
  `outcome.source_basis_identity()`.
- Workflow inspection identity composition helpers moved into
  `workflow/inspection/identities.rs`, keeping
  `workflow/inspection/operations.rs` under the workspace Rust line cap after
  the typed-coherence change.
- Domain capability materialization admission now compares typed target binding
  identities rather than `binding_digest()` text when deciding whether an
  admitted contribution is still bound to the current target.
- Digest/reporting accessors remain as terminal report-row projections for
  replay bundles, inspection rows, and denial details; they no longer drive the
  production coherence gates covered by row 1052.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query workflow::tests::inspection --lib` passed.
- `cargo test -p worth-query domain_capabilities::canonical_runtime_workflow_inspection_tests --lib`
  passed.
- `cargo test -p worth-query domain_capabilities::canonical_runtime_support_workflow_tests --lib`
  passed.
- `cargo test -p worth-query domain_capabilities::materialization_tests --lib`
  passed.
- Targeted scans over `workflow/` and `domain_capabilities/` found no remaining
  production authority equality or identity composition using
  `*_for_reporting()`, `*_digest()`, `binding_digest()`, `query_for_reporting()`,
  or `basis_for_reporting()` in the row 1052 production slice.
- `workflow/inspection/operations.rs` is now 306 lines and
  `workflow/inspection/identities.rs` is 183 lines.
- Captured logs:
  `plans/worth-query/goal_mode_workflow_domain_typed_identity_check.txt`,
  `plans/worth-query/goal_mode_workflow_domain_typed_identity_workflow_tests.txt`,
  `plans/worth-query/goal_mode_workflow_domain_typed_identity_domain_workflow_tests.txt`,
  `plans/worth-query/goal_mode_workflow_domain_typed_identity_support_workflow_tests.txt`,
  `plans/worth-query/goal_mode_workflow_domain_typed_identity_materialization_tests.txt`.

Next action:

Continue Phase 7 with row 1054 effect lifecycle batch admission, where the
ledger still names scoped-basis and lower-runtime binding comparisons through
digest accessors.

## Effect Lifecycle Batch Admission Typed Coherence Boundary

Status:

```text
row 1054 batch admission coherence no longer compares scoped-basis or lower-runtime binding digest text
```

Implemented closure:

- `validate_batch_component_lane_coherence(...)` now checks mixed basis identity
  through `NormalizedEffectIntent::scoped_basis_identity()` and
  `NormalizedEffectIntent::expected_lower_runtime_binding_identity()`.
- The old comparison through `scoped_basis_digest()` and
  `expected_lower_runtime_binding_digest()` is removed from the production
  batch-admission gate.
- Digest/reporting accessors remain available for terminal effect reports,
  closeout certification rows, and diagnostic output, but they no longer decide
  whether batch components share an authority-compatible basis.
- A remaining bridge-oracle record matcher in `effect_lifecycle/oracle/bridge.rs`
  still compares bridge runtime diagnostic record digests to execution receipt
  digests; that belongs to the bridge/causal retained diagnostics slice rather
  than the row 1054 batch-admission blocker.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query effect_lifecycle::tests::batch::admission --lib`
  passed.
- `cargo test -p worth-query effect_lifecycle::tests::batch --lib` passed.
- Targeted scan over `effect_lifecycle/` found no remaining
  `scoped_basis_digest()` or `expected_lower_runtime_binding_digest()` equality
  in production batch admission.
- `effect_lifecycle/batch_admission.rs` is 319 lines.
- Captured logs:
  `plans/worth-query/goal_mode_effect_batch_typed_coherence_check.txt`,
  `plans/worth-query/goal_mode_effect_batch_typed_coherence_batch_admission_tests.txt`,
  `plans/worth-query/goal_mode_effect_batch_typed_coherence_batch_tests.txt`.

Next action:

Continue Phase 7 with row 1055 Query causal inspection or row 1056 bridge
causal retained mapping; the bridge-oracle diagnostic record matcher should be
classified there.

## Query Causal Inspection Projection Quarantine

Status:

```text
row 1055 Query causal inspection projection re-entry is quarantined to terminal certification/reporting evidence
```

Implemented closure:

- Causal inspection redaction certification now validates redaction stability
  through typed `QueryCausalInspectionArtifact::causal_identity()` and
  `artifact_identity()` comparisons rather than
  `causal_identity_for_reporting()` / `artifact_for_reporting()` string
  comparisons.
- Representative matrix validation now checks advisory posture through the
  typed `CausalInspectionArtifactKind::Advisory` enum instead of comparing
  `artifact.kind().as_str()` to a string literal.
- Causal materialization tests no longer assert that Query materialized
  observation identity equals the bridge-exported evidence label. They now
  compare Query observation reporting projection to the Query subject's own
  observation projection, preserving the category split.
- The over-cap materialization artifact test file was split into
  `artifact.rs` and `artifact_denied.rs` after the test rewrite.
- Remaining causal equality scan hits are proof-shape certification digest
  bindings against certification artifacts/matrices/boundary audits; these are
  terminal certification evidence and do not feed authority construction or
  request/materialization admission.

Verification:

- `cargo check -p worth-query --lib` passed.
- `cargo test -p worth-query runtime::tests::causal_inspection::certification --lib`
  passed.
- `cargo test -p worth-query runtime::tests::causal_inspection::materialization --lib`
  passed.
- Targeted causal inspection scan found no projection-fed
  `field_identity(...)` or `hash_parts(...)` composition.
- Targeted causal inspection equality scan found only proof-shape certification
  digest binding, classified as terminal certification evidence.
- Touched line counts: `admission_decision.rs` 397 lines,
  `certification/validation.rs` 166 lines,
  `certification/matrix_validation.rs` 192 lines,
  `materialization/artifact.rs` 325 lines,
  `materialization/artifact_denied.rs` 171 lines.
- Captured logs:
  `plans/worth-query/goal_mode_causal_projection_quarantine_check.txt`,
  `plans/worth-query/goal_mode_causal_projection_quarantine_certification_tests.txt`,
  `plans/worth-query/goal_mode_causal_projection_quarantine_materialization_tests.txt`.

Next action:

Continue Phase 7 with row 1056 runtime-bridge causal retained mapping, including
the effect-lifecycle bridge-oracle runtime diagnostic record matcher classified
from the previous slice.

## Runtime Bridge Causal Retained Mapping Typed Lookup Boundary

Status:

```text
row 1056 retained mapping lookup adapters no longer cross string-keyed bridge lookup APIs
```

Implemented closure:

- Runtime-bridge causal retained mapping now revalidates retained bridge
  evidence into typed bridge identities before lookup. The route/history,
  planning-checkpoint, source/materialization, structural, stream checkpoint,
  stream replay, continuity, and merge retained-artifact adapters no longer
  reconstruct lookup authority through `from_reference_evidence(...).as_str()`.
- `BridgeDiagnosticsFacade` and `BridgeDiagnosticsState` lookup APIs for source
  materialization records, source failure records, structural remap records,
  structural branch comparison records, stream checkpoints, stream replay
  records, and replay-by-checkpoint records now require typed identity handles
  instead of raw `&str` lookup keys.
- `ConsumerCheckpointToken` now exposes `CheckpointTokenIdentity` as the
  authority accessor and keeps string output behind
  `checkpoint_token_identity_for_reporting()`. `CanonicalStreamReplayRecord`
  now retains the typed checkpoint identity instead of an `Arc<str>`.
- `RuntimeBridge::resume_stream_window_from_checkpoint(...)` now requires a
  typed `CheckpointTokenIdentity`, closing the public resume escape hatch that
  previously allowed arbitrary retained checkpoint strings to satisfy the
  lookup boundary.
- Stream harness JSON and certification bundle projections were updated to use
  explicit reporting accessors for checkpoint labels, preserving terminal
  export behavior without teaching authority identities to stringify
  implicitly.
- The bridge-oracle digest matcher carried forward from row 1054 remains a
  Query-side diagnostic-record matcher, not a runtime-bridge retained lookup
  adapter. It should be handled with the remaining Query effect/oracle or
  hostile-certification rows rather than reopening row 1056.

Verification:

- `cargo check -p worth-runtime-bridge --lib` passed.
- `cargo check -p worth-query --lib` passed.
- Targeted retained-mapping scan found no retained-artifact lookup call feeding
  `from_reference_evidence(...).as_str()` or raw retained-reference text into
  the row-1056 bridge lookup APIs.
- Targeted lookup API scan found the production source/structural/stream
  diagnostic lookup methods now take typed identities; remaining raw-string
  uses are terminal reporting/projection paths or unrelated test-fixture
  compiler fallout.
- `cargo test -p worth-runtime-bridge retained_mapping --lib` passed after the
  compiler-discovered fixture fallout was repaired with typed fixture
  constructors and bridge-owner external evidence helpers rather than
  compatibility shims.
- Touched production file line counts remain under the 400-line cap:
  `diagnostics/state/query.rs` 354 lines, `facade/runtime/stream.rs` 266 lines,
  `stream/checkpoints.rs` 176 lines, `stream/replay.rs` 139 lines,
  retained-artifact modules 125-257 lines.

Next action:

Continue with the next compiler-discovered open Phase 9 fixture row, starting
from the public bridge runtime or transcript runtime support where mutation
receipts and hostile certification still preserve commit/snapshot authority as
formatted strings.
