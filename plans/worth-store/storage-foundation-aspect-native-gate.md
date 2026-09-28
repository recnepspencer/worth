# Worth Store Aspect-Native Workspace Gate

## Goal

Make the dedicated Worth Store workspace aspect-native before the physical
database foundation is built on top of it.

This gate makes Foundational aspect values, aspect contracts, authoritative
aspect state, patches, locators, canonical basis, boundary evidence, receipts,
and performance evidence the normal Store workspace boundary language. JSON may
exist only as a terminal projection at explicitly named external presentation
edges.

## Why This Gate Exists

Worth Query and Worth Relational both exposed the cost of bolting aspect-native
discipline onto a working subsystem after JSON-shaped carriers had already
become convenient. Store is earlier in its physical workspace arc. This is the
cheap moment to close that class of mistake.

This gate belongs before `S.0` because `S.0` is supposed to establish Store
source boundaries and claim vocabulary. If the workspace still accepts JSON,
serde payloads, raw strings, or projection text as evidence authority, then
`S.0` cannot honestly hand typed source facts to `S.1`.

## Governing Summaries

- `MENTALITY.md` protects hard-problem-first architecture. The strongest
  shaping constraint is that aspect-native authority must be built before page,
  WAL, recovery, and certification work depend on weaker carriers.
- `arch_laws.md` protects proof-bearing boundaries and identity authority. The
  strongest shaping constraint is that Store APIs must consume typed values
  carrying what has been proven, not raw JSON, raw strings, or regenerated
  projection text.
- `composition_laws.md` protects responsibility-shaped files. The strongest
  shaping constraint is that JSON projection, native admission, canonical
  basis, and certification scans must live in separate named files rather than
  one broad codec/helper surface.
- `domain_structure_laws.md` protects authority and projection topology. The
  strongest shaping constraint is that source truth, derived facts, diagnostic
  artifacts, compatibility residue, and terminal projections must occupy
  distinct structural homes.
- `perf_laws.md` protects visible cost and proof carry. The strongest shaping
  constraint is that canonical digest, comparison, and performance evidence
  must derive from native canonical basis with counters, not from JSON
  serialization order or ad hoc stringification.
- `physical-database-roadmap.md` protects physical database credibility. The
  strongest shaping constraint is that Roadmap 2 cannot depend on full-store
  serde materialization, backend-private residue, or weak semantic carriers.

## Adversarial Constraint

No Worth Store workspace authority, evidence, digest basis, handoff, recovery
input, certification row, or test fixture may require or accept JSON-shaped
state, `serde_json::Value`, arbitrary serde serialization, raw string identity,
or terminal projection text as semantic authority.

JSON is permitted only at explicitly named terminal projection boundaries, and
any JSON ingress must immediately lower through Foundational compatibility or
Store-owned admission into native aspect values before it can be consumed by
Store authority.

## Product Decision Lock

- Store physical bytes are framed binary records, pages, segments, extents,
  chunks, WAL records, and manifests. They are not JSON documents.
- Store semantic boundary facts are Foundational aspect-native values and
  Store-owned physical witnesses, not JSON payload trees.
- Terminal JSON projection is display, external document compatibility, or
  operator export. It is never authority, never canonical digest basis, never a
  recovery source, and never an S.* handoff artifact.
- Legacy `crates/worth-store` serde/JSON persistence is residue inventory for
  migration and compatibility quarantine. It is not precedent for the dedicated
  Roadmap 2 workspace.
- The new workspace may use `worth_foundational::compatibility().json()` only
  at named ingress boundaries whose output is native aspect-state material.

## Planned Directory Skeleton

The gate must create or reserve these homes in the dedicated Store workspace
before S.0 implementation proceeds:

```text
workspaces/worth-store/crates/worth-store-aspect-native/
  src/
    lib.rs
    value_admission.rs
    contract_admission.rs
    authoritative_state.rs
    authoritative_patch.rs
    boundary_locators.rs
    identity_authority.rs
    canonical_basis.rs
    canonical_basis_sources.rs
    digest_authority.rs
    equivalence_basis.rs
    terminal_projection_digest_separation.rs
    evidence_receipts.rs
    performance_receipts.rs
    terminal_projection.rs
    terminal_json_projection.rs
    json_ingress_readmission.rs
    residue_inventory.rs
    s0_handoff.rs

workspaces/worth-store/crates/worth-store-aspect-native-certification/
  src/
    lib.rs
    production_json_residue_scan.rs
    terminal_projection_allowlist.rs
    canonical_basis_source_tests.rs
    digest_authority_denial_scan.rs
    equivalence_basis_parity_tests.rs
    terminal_projection_digest_tests.rs
    serde_authority_denial_scan.rs
    raw_string_identity_denial_scan.rs
    aspect_native_boundary_compile_fail.rs
    terminal_json_readmission_compile_fail.rs
    canonical_basis_parity_tests.rs
    s0_readiness_tests.rs
```

If the implementation chooses to merge these responsibilities into an existing
workspace crate, it must keep the same responsibility names as modules and must
not hide them inside a generic `compatibility`, `codec`, `helpers`, or `json`
surface.

## Phase Plan

### Phase 1: Residue Inventory And No-New-JSON Freeze

Freeze the difference between existing compatibility residue and future Store
workspace authority.

**Relevant subsystems**
- `crates/worth-store`
- `workspaces/worth-store`
- Store certification and UI compile-fail suites

**Relevant APIs**
- `serde_json::Value`, `serde_json::to_vec`, `serde_json::from_slice`
- `serde::{Serialize, Deserialize}`
- Store backend persistence/load helpers
- Query precedent: production JSON residue allowlist and terminal codec scans

**Warnings**
- Inventory is discovery evidence only. It must not normalize JSON as an
  acceptable substrate for Roadmap 2.
- Existing compile-fail tests that prove some types are not deserializable are
  not enough while nearby production surfaces still derive `Serialize` or
  digest through `serde_json`.
- Tests may use JSON only to prove hostile rejection or terminal projection
  behavior, not as ordinary fixture authoring.

**Test requirements**
- `store_json_residue_inventory_classifies_every_occurrence`: scans
  `crates/worth-store` and `workspaces/worth-store`, classifies each
  `serde_json`, `json!`, `Serialize`, `Deserialize`, and raw JSON helper by
  zone, owner, authority risk, and removal or quarantine condition.
- `unclassified_store_json_residue_fails_the_gate`: injects or detects an
  unlisted JSON occurrence in a production Store workspace source path and
  fails with the exact file, symbol, and forbidden authority category.
- `ordinary_store_tests_do_not_import_json_preludes`: proves shared Store test
  preludes do not hand `serde_json::Value`, `json!`, or JSON document helpers
  to normal production-quality tests.

**Engineering decisions**
- Add a machine-readable residue inventory, but keep it as certification input,
  not as a semantic artifact consumed by S.* work.
- Classify the older root `crates/worth-store` persistence path as legacy
  compatibility residue unless a later migration phase explicitly readmits a
  surface through native Store contracts.
- Classify dedicated workspace crates as the enforcement target: new Roadmap 2
  code cannot add non-terminal JSON.

**Open questions**
- None.

### Phase 2: Foundational Aspect-Native Store Vocabulary

Define the Store boundary vocabulary over existing Foundational aspect and
proof-carrying surfaces before any S.* handoff consumes it.

**Relevant subsystems**
- `worth-store-aspect-native`
- `worth-store-authority`
- `worth-store-contracts`
- `worth-store-readiness`
- `worth-store-certification`

**Relevant APIs**
- `worth_foundational::AspectValue`
- `worth_foundational::StructAspectValue`
- `worth_foundational::AspectKey`
- `worth_foundational::CanonicalFieldPath`
- `worth_foundational::AspectMask`, `MutationMask`, `ProjectionMask`,
  `DiagnosticMask`
- `worth_foundational::ContractValidatedAspectValue`
- `worth_foundational::AuthoritativeRecordAspectState`
- `worth_foundational::AuthoritativeRecordAspectPatch`
- `worth_foundational::AspectLocator`, `AspectFieldLocator`,
  `AspectValueLocator`, `BoundaryArtifactLocator`
- Foundational boundary artifact, receipt, canonical basis, diagnostics, and
  performance APIs exposed through the facade

**Warnings**
- Store must import shared aspect vocabulary without moving physical byte
  survival authority into Foundational.
- `AspectValue::String` is still a native aspect value only after admission.
  A raw `String`, display label, digest text, or path projection is not an
  admitted aspect key, identity, locator, or value.
- Foundational compatibility JSON lowering is an ingress tool, not Store's
  normal authoring or persistence model.

**Test requirements**
- `store_boundary_values_are_foundational_aspect_values`: constructs each S0
  boundary fact family through `AspectValue`, `StructAspectValue`, validated
  aspect artifacts, authoritative state, or authoritative patches, and proves
  no semantic field is represented by `serde_json::Value`.
- `raw_value_cannot_satisfy_store_authority`: compile-fail proof that
  `AspectValue`, `String`, terminal projection text, and unvalidated
  `StructAspectValue` cannot enter Store authority APIs requiring validated
  state or a Store-owned physical witness.
- `store_aspect_keys_are_admitted_not_parsed_from_paths`: proves Store aspect
  identity flows through `AspectKey` and locators, while dotted path strings
  are terminal projections only.

**Engineering decisions**
- Introduce Store-local wrapper types only when the distinction is
  Store-owned, such as physical witness role, backend capability authority, or
  byte-survival source role.
- Use Foundational `AspectKey`, masks, locators, values, state, patch, receipt,
  canonical basis, and performance evidence where the concept is shared across
  Worth crates.
- Keep Store-owned physical ids, LSNs, page ids, frame headers, segment ids,
  and durability witnesses as Store types that can materialize Foundational
  boundary evidence when crossing crates.

**Open questions**
- None.

### Phase 3: Terminal Projection Quarantine

Make terminal projection a visible boundary instead of a convenience accessor.

**Relevant subsystems**
- `worth-store-aspect-native::terminal_projection`
- `worth-store-aspect-native::terminal_json_projection`
- `worth-store-aspect-native::json_ingress_readmission`
- external operator export/import surfaces

**Relevant APIs**
- Foundational compatibility JSON front doors
- Store terminal projection wrappers
- Store readmission/admission functions

**Warnings**
- A terminal projection accessor must not have a neutral name such as
  `as_str`, `value`, `payload`, `to_json`, `rows`, or `metadata`.
- Terminal projection text must not be usable as a key, identity, locator,
  digest basis, recovery source, or handoff input.
- Ingress from terminal JSON must produce a new native admission result with
  denial evidence; it cannot rehydrate authority by deserializing directly into
  authority types.

**Test requirements**
- `terminal_json_projection_is_one_way_until_readmitted`: exports native Store
  facts to terminal JSON, proves no authority API accepts the projection, then
  readmits only through an explicit ingress function that returns validated
  native aspect material.
- `neutral_string_accessors_do_not_exist_on_store_authority`: compile-fail
  proof that Store authority identities, evidence identities, aspect locators,
  and handoff facts do not implement `Display`, `ToString`, `AsRef<str>`, or
  neutral string accessors unless the method name says terminal projection.
- `terminal_projection_modules_are_the_only_json_homes`: scans production
  Store workspace files and fails if a file importing JSON is not under an
  approved terminal projection or external ingress module.

**Engineering decisions**
- Terminal projection modules may format JSON for external tools, but every
  public function must name `terminal` and `projection`.
- JSON ingress modules must name `ingress`, `readmission`, or `external_io`,
  and their successful output must be a native value/state/patch artifact.
- Terminal projection artifacts may be stored only as projections, not as
  source manifests, canonical basis, recovery plans, or certification evidence.

**Open questions**
- None.

### Phase 4: Canonical Basis Source Ownership

Name every Store fact family that requires canonical basis and assign the
authority that is allowed to prepare it.

**Relevant subsystems**
- `worth-store-aspect-native::canonical_basis`
- `worth-store-aspect-native::canonical_basis_sources`
- `worth-store-certification`
- `worth-store-readiness`
- `worth-store-physical-format`
- `worth-store-physical-integrity`
- `worth-store-recovery-physics`
- `worth-store-wal`

**Relevant APIs**
- Foundational canonical basis entries and rule versions
- Foundational boundary artifact locators
- Foundational diagnostic locators
- Store physical format, integrity, WAL, checkpoint, page, segment, extent,
  blob, recovery, and performance witnesses

**Warnings**
- Canonical basis is not one mechanism. Source manifests, page headers,
  physical witnesses, recovery receipts, performance reports, and S.* handoffs
  need separate source roles even when they use the same Foundational basis
  vocabulary.
- A fact family without an owner will eventually get a local digest helper.
- Terminal projection fields must be classified as projection fields at this
  phase so later phases do not accidentally digest them as authority.

**Test requirements**
- `store_canonical_basis_source_map_covers_every_evidence_family`: scans every
  Store workspace evidence, handoff, receipt, diagnostic, performance, and
  recovery family and proves it has an assigned native canonical-basis source
  owner.
- `unowned_canonical_basis_source_fails_the_gate`: injects or detects a Store
  evidence family without a source owner and fails with the owning subsystem
  that must classify it.
- `terminal_projection_fields_are_not_basis_sources`: proves fields marked as
  terminal projection, operator display, document checksum, or compatibility
  text cannot be selected as canonical-basis sources.

**Engineering decisions**
- Store canonical basis starts with a source-role map before any digest helper
  exists.
- Each source role names the owning crate/module, allowed input types, output
  basis artifact, and denied projection fields.
- Recovery, WAL, physical format, integrity, performance, and S0 handoff basis
  families must be distinct because they fail and evolve differently.

**Open questions**
- None.

### Phase 5: Native Canonical Entry Construction

Build Store canonical basis entries from native Foundational aspect material
and Store physical witnesses, not from serialization.

**Relevant subsystems**
- `worth-store-aspect-native::canonical_basis`
- `worth-store-aspect-native::canonical_basis_sources`
- `worth-store-certification`
- `worth-store-physical-format`
- `worth-store-physical-integrity`

**Relevant APIs**
- Foundational canonical basis entries and rule versions
- Foundational aspect values, locators, receipts, diagnostics, and performance
  basis preparation
- Store physical format and integrity witnesses

**Warnings**
- `serde_json::to_vec` over a struct is not a canonical basis.
- Pretty, stable, or normalized JSON is still JSON projection unless the
  canonicalization authority explicitly lowers native facts into Foundational
  basis entries.
- A native basis constructor that accepts `T: Serialize`, `String`, or
  terminal projection text has already lost the authority boundary.

**Test requirements**
- `store_canonical_basis_is_native_and_order_stable`: generates equivalent
  Store boundary facts in different construction orders and proves the native
  canonical basis matches without serializing through JSON.
- `raw_json_cannot_enter_canonical_basis_construction`: compile-fail proof that
  `serde_json::Value`, `String`, terminal projection text, and generic
  `Serialize` inputs cannot satisfy native basis constructors.
- `missing_physical_witness_denies_basis_construction`: proves physical
  evidence families cannot construct basis entries from semantic aspect values
  alone when a Store-owned byte-survival witness is required.

**Engineering decisions**
- Canonical basis is built from native Foundational entries and Store physical
  witness fields, not from generic serialization.
- Basis constructors are source-role-specific and accept only admitted native
  inputs for that role.
- Canonical entry construction does not compute final Store digest authority;
  it produces the typed material a later phase is allowed to digest.

**Open questions**
- None.

### Phase 6: Digest Authority And Equivalence Contracts

Make every Store digest and equivalence claim name the native canonical basis
that justifies it.

**Relevant subsystems**
- `worth-store-aspect-native::digest_authority`
- `worth-store-aspect-native::equivalence_basis`
- `worth-store-aspect-native::canonical_basis`
- `worth-store-certification`
- `worth-store-readiness`

**Relevant APIs**
- Foundational canonical digest ids
- Foundational equivalence basis ids
- Foundational canonical basis comparison surfaces
- Store digest evidence wrappers

**Warnings**
- Digest text is evidence/projection, not identity authority.
- A digest helper that accepts `T: Serialize` is an unbounded authority leak.
- Equivalence without an explicit basis is a cache heuristic, not a Store
  correctness claim.

**Test requirements**
- `serde_json_digest_basis_is_rejected`: compile-fail or scan proof that no
  production Store digest helper accepts `T: Serialize` or calls
  `serde_json::to_vec` as canonical authority.
- `digest_text_cannot_satisfy_store_authority`: compile-fail proof that digest
  strings cannot construct identity, recovery source, checkpoint, page, WAL, or
  certification authority.
- `store_equivalence_requires_named_native_basis`: proves reuse, parity,
  suppression, and digest comparison APIs require a typed equivalence basis and
  deny basis-free comparison.
- `same_native_basis_yields_same_digest`: constructs the same native basis
  through different valid input orderings and proves the typed Store digest is
  stable.

**Engineering decisions**
- Digest APIs must name the basis they digest and return typed evidence, not
  naked strings.
- Store digest evidence carries its source role and equivalence basis id.
- Digest construction is downstream of native basis construction and upstream
  of certification evidence.

**Open questions**
- None.

### Phase 7: Terminal Projection Digest And Checksum Separation

Prove terminal projection changes cannot perturb native Store authority, while
terminal document checksums remain explicitly lower authority.

**Relevant subsystems**
- `worth-store-aspect-native::terminal_projection_digest_separation`
- `worth-store-aspect-native::terminal_projection`
- `worth-store-aspect-native::terminal_json_projection`
- `worth-store-aspect-native::digest_authority`
- external operator export/import surfaces

**Relevant APIs**
- Store terminal projection wrappers
- Store digest authority wrappers
- Store terminal document checksum wrappers
- Foundational readmission and boundary evidence APIs

**Warnings**
- If an external JSON document needs a checksum, that checksum is labeled as a
  terminal document checksum and cannot satisfy Store authority digest slots.
- Terminal JSON parity tests must not become proof that JSON is a supported
  authority representation.
- Projection round trips are readmission tests, not digest authority tests.

**Test requirements**
- `terminal_projection_changes_do_not_change_native_digest`: changes terminal
  JSON field ordering, whitespace, and display labels while proving native
  canonical basis and Store digest evidence remain unchanged.
- `terminal_document_checksum_cannot_satisfy_authority_digest`: compile-fail
  proof that a terminal JSON document checksum cannot enter APIs requiring
  Store digest authority.
- `projection_roundtrip_recomputes_native_basis_after_readmission`: exports to
  terminal JSON, readmits through the explicit ingress lane, and proves the
  resulting native basis is rebuilt from admitted native facts rather than
  trusted from the projection.

**Engineering decisions**
- Terminal projection checksum APIs live outside digest authority modules.
- Projection parity exists to prove quarantine, not to prove JSON equivalence
  with native authority.
- Any test that compares terminal JSON must also name whether it is checking
  presentation stability, hostile readmission, or authority quarantine.

**Open questions**
- None.

### Phase 8: Authority, Projection, And Readmission Types

Close the authority/projection gap that lets lower-authority representation
masquerade as current Store truth.

**Relevant subsystems**
- `worth-store-aspect-native::identity_authority`
- `worth-store-aspect-native::boundary_locators`
- `worth-store-authority`
- `worth-store-claim-boundaries`

**Relevant APIs**
- Foundational authority identity categories
- Foundational external identity tokens
- Foundational derived identity and lifecycle markers
- Store physical witness identities and source-role classifications

**Warnings**
- A projection may be useful for logs or operators, but it cannot be promoted
  back to authority without owner readmission.
- Derived evidence, digest text, retained historical evidence, and external
  tokens are different authority categories even when their representation is
  identical.
- Store must not reconstruct identity from display, path, digest, or filename
  text.

**Test requirements**
- `store_projection_cannot_reconstruct_authority_identity`: compile-fail proof
  that terminal projection text, digest strings, filenames, and external tokens
  cannot construct current Store authority identities.
- `store_external_tokens_require_owner_readmission`: proves an imported
  external token can become usable only after the owning Store authority
  validates it and returns a typed current-authority witness.
- `retained_evidence_remains_retained_evidence`: proves historical evidence
  can be reported and compared but cannot be passed into APIs requiring current
  physical authority without a readmission witness.

**Engineering decisions**
- Use distinct types for current authority, external tokens, derived evidence,
  retained evidence, and terminal projection text.
- Restrict constructors so only owning admission functions can create current
  Store authority witnesses.
- Store readmission results must carry denial evidence for stale, mismatched,
  lower-authority, or unsupported sources.

**Open questions**
- None.

### Phase 9: Aspect-Native Test Harness Authoring

Make the Store harness author native aspects and physical witnesses directly
so tests do not keep JSON alive as the convenient mental model.

**Relevant subsystems**
- `worth-store-test-support`
- `worth-store-aspect-native-certification`
- Store S0-S12 certification suites

**Relevant APIs**
- Foundational aspect vocabulary front doors
- Store physical witness builders
- Store hostile fixture generation
- Store terminal projection readmission functions

**Warnings**
- A test that builds a JSON blob and then lowers it is testing the
  compatibility lane, not the ordinary Store API.
- Snapshot/golden JSON fixtures can certify terminal projections only. They
  cannot certify native authority, recovery, digest, or handoff correctness.
- Test helpers must not hide native admission behind generic fixture builders.

**Test requirements**
- `ordinary_store_harness_authors_native_aspects`: proves normal fixtures build
  Store evidence from `AspectKey`, `AspectValue`, `StructAspectValue`,
  validated values, authoritative states, patches, locators, and Store physical
  witnesses without JSON authoring.
- `json_fixture_can_only_target_terminal_projection`: attempts to use a JSON
  fixture in a non-terminal suite and fails with a denial that names the
  terminal-only boundary.
- `hostile_readmission_tests_use_json_only_as_attacker_input`: keeps JSON in
  adversarial ingress tests, but requires those tests to assert readmission,
  denial, and native output explicitly.

**Engineering decisions**
- Shared harness preludes expose native builders, not `json!`.
- Test support separates native fixture builders from terminal projection
  fixtures by module path and type name.
- Hostile tests may carry malformed JSON bytes only when the test name and
  module identify the boundary as external ingress or terminal projection.

**Open questions**
- None.

### Phase 10: Public Facade And Dependency Enforcement

Make the aspect-native boundary the only ordinary public Store workspace lane.

**Relevant subsystems**
- `workspaces/worth-store/crates/worth-store/src/lib.rs`
- workspace crate facades
- Cargo manifests and dependency graph checks

**Relevant APIs**
- Store facade exports
- Foundational facade exports
- workspace dependency declarations

**Warnings**
- Having a clean module tree is not sufficient if public exports still reveal
  internal JSON compatibility helpers.
- `serde_json` as a workspace dependency is dangerous unless every use is
  terminal-projection classified.
- Generic `T: Serialize` or `T: DeserializeOwned` public helpers create an
  unbounded JSON/serde authority lane even if no caller uses JSON today.

**Test requirements**
- `store_public_facade_exports_aspect_native_boundary_only`: proves ordinary
  downstream code can author Store boundary facts through native aspect and
  witness types and cannot import internal JSON/serde compatibility helpers.
- `workspace_serde_json_dependency_is_terminal_only`: scans each dedicated
  Store workspace crate and fails if `serde_json` appears in a non-terminal
  dependency, module, public export, or ordinary test prelude.
- `generic_serde_authority_helpers_are_rejected`: compile-fail or scan proof
  that public Store APIs do not expose `T: Serialize`, `T: DeserializeOwned`,
  `serde_json::Value`, or raw JSON document constructors for authority paths.

**Engineering decisions**
- Public Store workspace facades export native admission, terminal projection,
  readmission, and certification surfaces as separate named capabilities.
- Crates that do not own terminal projection must not depend on `serde_json`.
- Any required terminal projection dependency must be localized to the smallest
  crate and module that owns the external format.

**Open questions**
- None.

### Phase 11: S0 Handoff Contract

Produce the typed gate output that S0 must consume.

**Relevant subsystems**
- `worth-store-aspect-native::s0_handoff`
- `worth-store-readiness`
- `worth-store-s0-reclassification`
- `worth-store-certification`

**Relevant APIs**
- Foundational boundary artifacts
- Foundational receipts
- Foundational readiness reports
- Store S0 source-set and claim-boundary vocabulary

**Warnings**
- S0 handoff cannot be a JSON report, markdown checklist, log output, or
  terminal projection artifact.
- The handoff must include negative proof: where JSON can still exist, why it
  is terminal, and why S0 cannot consume it as authority.
- S0 cannot close if it consumes stale residue inventory instead of current
  scan evidence.

**Test requirements**
- `s0_handoff_is_native_boundary_artifact`: proves the S0 readiness artifact is
  a typed native boundary artifact with canonical basis, receipts, diagnostics,
  and performance evidence, not JSON.
- `s0_rejects_terminal_projection_as_handoff`: attempts to feed terminal JSON
  projection output into the S0 handoff consumer and receives a typed denial.
- `s0_handoff_replays_from_native_evidence`: reconstructs the handoff verdict
  from native canonical basis and evidence receipts without reading logs or
  terminal JSON.

**Engineering decisions**
- The S0 handoff artifact must include residue scan evidence, terminal
  projection allowlist, Foundational API adoption map, canonical basis proof,
  public facade proof, and harness proof.
- The handoff consumer accepts only the typed native readiness artifact.
- S0 inherits this gate's denial vocabulary for terminal projections,
  unclassified JSON residue, raw string identity, generic serde authority, and
  non-native digest basis.

**Open questions**
- None.

## Must Ship

- Dedicated Store aspect-native boundary module or crate with native admission,
  contract validation, authoritative state, authoritative patch, locators,
  identity authority, canonical basis source ownership, native canonical entry
  construction, digest authority, equivalence basis, evidence receipts,
  performance receipts,
  terminal projection, JSON readmission, residue inventory, and S0 handoff
  responsibilities.
- Certification crate or module with production JSON residue scans, terminal
  projection allowlist checks, canonical basis source coverage, native basis
  construction denials, digest authority denials, equivalence-basis parity,
  terminal projection digest separation, serde authority denials, raw string
  identity denials, compile-fail suites, and S0 readiness tests.
- A current inventory of all JSON/serde residue in old and new Store surfaces,
  classified by terminal, hostile, compatibility residue, or forbidden.
- Public Store workspace facades that expose aspect-native authoring and
  evidence as the ordinary path.
- Native test harness builders for all ordinary Store S.* boundary facts.

## C.7 Derived Durability Work-Basis Contracts

C.7 adds exactly six aspect-native contracts to the physical work profile:

| Contract key | Basis posture and Signal role | Admitted masks | Signal families | Partition |
| --- | --- | --- | --- | --- |
| `store.physical.durability.policy-binding-basis` | projection; `Dependency` | projection only | `WalAppend`, `DurabilityBarrier`, `CheckpointCapture`, `RootPublication`, `WalReclamation` | `physical-durability-policy/{policy-identity}` |
| `store.physical.durability.wal-append-basis` | mutation; `DependencyAndOutput` | whole projection and whole mutation | `WalAppend` | `physical-durability-store/{store-identity}` |
| `store.physical.durability.wal-barrier-basis` | mutation; `DependencyAndOutput` | whole projection and whole mutation | `DurabilityBarrier` | `physical-durability-store/{store-identity}` |
| `store.physical.durability.checkpoint-capture-basis` | mutation; `DependencyAndOutput` | whole projection and whole mutation | `CheckpointCapture` | `physical-durability-store/{store-identity}` |
| `store.physical.durability.root-publication-basis` | mutation; `DependencyAndOutput` | whole projection and whole mutation | `RootPublication` | `physical-durability-store/{store-identity}` |
| `store.physical.durability.wal-reclamation-basis` | mutation; `DependencyAndOutput` | whole projection and whole mutation | `WalReclamation` | `physical-durability-store/{store-identity}` |

These contracts route dependency readiness and invalidation only. They do not
own WAL durability, data settlement, checkpoint publication, current-root
advancement, reclamation eligibility, or acknowledgment. Exact WAL ranges,
barrier artifacts, checkpoint identities, root candidates, and reclaim targets
remain in typed Store work declarations and executor commands. Signal receives
the admitted family and partition; it never receives authority to construct or
widen physical truth. The installed Signal graph already has one runtime owner;
the partition therefore stays stable for the Store across reopen while exact
runtime-incarnation authority remains in Store commands and progression types.

`FoundationalPolicyAdmissionReceipt` separately proves finite scheduler budget
admission for the named work. Worth Proof separately carries transition
posture. Neither value can enter Store progression as physical evidence.

The gate
`c7_durability_aspects_are_six_exact_store_partitioned_contracts` and its
unpartitioned, ephemeral-runtime, missing-contract, and wrong-key mutants
enforce this table.

## Must Preserve

- Store owns physical byte survival. Foundational owns shared boundary
  vocabulary. Proof owns shared proof/reporting vocabulary. Relational owns
  semantic truth/MVCC meaning.
- Physical binary records, pages, frames, WAL records, and blob chunks remain
  binary Store structures, not JSON documents and not Foundational semantic
  authority.
- Terminal projection remains useful for external documents, operators, and
  compatibility export, but it remains lower authority than native evidence.
- Legacy root-crate JSON persistence can be inventoried and quarantined, but it
  cannot become the precedent for Roadmap 2 workspace implementation.

## Acceptance Evidence

- `store_json_residue_inventory_classifies_every_occurrence`
- `unclassified_store_json_residue_fails_the_gate`
- `store_boundary_values_are_foundational_aspect_values`
- `raw_value_cannot_satisfy_store_authority`
- `terminal_json_projection_is_one_way_until_readmitted`
- `neutral_string_accessors_do_not_exist_on_store_authority`
- `store_canonical_basis_source_map_covers_every_evidence_family`
- `unowned_canonical_basis_source_fails_the_gate`
- `store_canonical_basis_is_native_and_order_stable`
- `raw_json_cannot_enter_canonical_basis_construction`
- `serde_json_digest_basis_is_rejected`
- `digest_text_cannot_satisfy_store_authority`
- `store_equivalence_requires_named_native_basis`
- `terminal_document_checksum_cannot_satisfy_authority_digest`
- `store_projection_cannot_reconstruct_authority_identity`
- `ordinary_store_harness_authors_native_aspects`
- `store_public_facade_exports_aspect_native_boundary_only`
- `workspace_serde_json_dependency_is_terminal_only`
- `s0_handoff_is_native_boundary_artifact`
- `s0_rejects_terminal_projection_as_handoff`

## Sequencing Notes

This gate precedes `S.0`.

`S.0` may start only after the dedicated Store workspace has a typed native
handoff artifact proving that JSON is confined to terminal projection or
explicit hostile/readmission tests. S0 then defines Store source boundaries and
claim vocabulary on top of that native substrate.

`S.1` through `S.12` inherit this rule. Their evidence, handoffs,
certification rows, diagnostics, canonical basis, and performance receipts must
be native aspect/physical-witness artifacts unless the field is explicitly a
terminal projection.

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes. It prevents JSON-shaped authority from becoming Store's
  hidden database substrate before Roadmap 2 implementation starts.
- Is the adversarial constraint precise and load-bearing? Yes. It forbids JSON,
  arbitrary serde, raw string identity, and terminal text in authority,
  evidence, digest, handoff, recovery, and certification roles.
- Does the roadmap justify this milestone now? Yes. Roadmap 2 is a greenfield
  physical foundation and S0 already depends on typed aspect-native source
  facts.
- Does the spec preserve crate authority boundaries? Yes. Foundational supplies
  shared vocabulary; Store keeps byte survival authority; Proof keeps proof
  reporting vocabulary; Relational keeps semantic truth authority.
- Are the phases carrying most of the real design information? Yes. The gate is
  defined primarily through ordered enforcement phases.
- Is each phase centered on one conceptual detail or boundary? Yes.
- Does each phase contain at least 2 adversarial tests by default? Yes.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. The directory skeleton, APIs, phases, and named tests are
  implementation-ready.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  It belongs immediately before S0 because every later storage milestone
  depends on native source and evidence handoffs.
