# S.5.1: Cryptographic Boundary Seeds And Tenant Scope Metadata

## Goal

Introduce non-secret cryptographic-boundary, authenticity, and tenant-scope
metadata that would have been cheapest to reserve during `S.1`, `S.3`, and
`S.4`, without rewriting those closed milestone scopes or pretending they
already shipped it.

## Why This Milestone Exists

`S.5` establishes physically stable reads. Before Roadmap 2 proceeds into I/O
QoS, blobs, backup, repair, and full security, Store needs typed metadata,
admission, and proof surfaces for key scope, tenant scope, authenticity class,
and encrypted/authenticated frame capability posture. These are not full
encryption, identity, or compliance features. They are the structural seeds
that make later security work impossible to forget or bolt on dishonestly.

The hard part is not naming security fields. The hard part is placing them
where later physical work cannot bypass them: in Store-owned authority
vocabulary, in physical witnesses, in proof-carrying handoff types, and in
direct tests of the production boundaries that later milestones consume.

## Governing Summaries

- `MENTALITY.md` protects hard-problem-first design: security metadata that
  affects page identity, frame admission, backup, blobs, and repair must be
  made structural before later features multiply.
- `arch_laws.md` protects proof-bearing construction: key scope, tenant scope,
  authenticity class, and encrypted-frame readiness must become typed proofs,
  not comments or raw labels. Law 41 is especially load-bearing: each
  readiness type must encode what has been proven, and its construction must be
  sealed to the proving path.
- `composition_laws.md` protects responsibility boundaries: cryptographic
  metadata, tenant scope, authenticity admission, and terminal projections must
  live in named modules instead of becoming helper residue inside page, WAL, or
  blob code.
- `domain_structure_laws.md` protects authority topology: semantic truth,
  physical byte survival, authenticity evidence, and tenant blast radius must
  remain separate structural spaces.
- `perf_laws.md` protects visible cost: security metadata must carry exact
  counter surfaces so encryption/authenticity readiness does not hide broad
  scans, unbounded rewrap work, or per-page allocation surprises.
- `physical-database-roadmap.md` protects the physical database foundation gate:
  S.5.1 belongs between physically stable reads and I/O/blob/backup/security
  work because key scope, tenant scope, authenticity class, and custody posture
  must become physical metadata before later systems depend on those paths.

## Adversarial Constraint

Later Store work must not be able to introduce encrypted pages, authenticated
frames, tenant-scoped blobs, backup capsules, PITR bundles, or repair actions
through raw strings, ambient deployment assumptions, or terminal JSON
projections. A page, frame, WAL record, blob chunk, backup bundle, export
capsule, or repair plan that lacks typed key scope, tenant scope, authenticity
class, and key-version posture must fail admission before later milestones can
claim security, tenant isolation, or auditability.

Copied ids, copied digest rows, semantic commit ids, terminal projection text,
and `StoreCurrentAuthorityWitness` alone must be insufficient to mint any
S.5.1 security-scope readiness artifact. Readiness must originate from typed
Store physical evidence plus admitted key/tenant/authenticity/custody scope.

## Product Decision Lock

Store does not become an identity provider. Store consumes typed admission
evidence from external identity systems where needed, but Store owns durable
cryptographic authority boundaries: key scope, tenant scope, authenticity
evidence, key-version posture, backup/export custody, and tenant-scoped repair
blast radius.

## Term Grammar And Authority Boundaries

- `Requirement` means what a physical lane demands before it may proceed.
- `Posture` means the known support, freshness, availability, custody, or
  legacy state of a Store-owned security boundary.
- `Result` means the observed outcome of an admitted check. Results cannot be
  declared by physical metadata alone.
- `Witness` means a sealed Store authority artifact constructed by the admitted
  Store proving path.
- `Evidence` means publishable or certifiable output. Evidence can describe a
  Store decision, but it is not the authority that made the decision.
- `Readiness` means a typed permission for a later subsystem to proceed after
  Store-owned admission has succeeded.
- `Key scope` is Store's durable classification of which cryptographic and
  custody domain a physical artifact belongs to. It is not a KMS key id, key
  material, key handle, or sufficient authority to perform cryptographic
  operations.
- `Key-version posture` is the admitted freshness state of a key scope for a
  physical artifact: current, stale, rebind-required, unsupported, unavailable,
  or denied as appropriate to the lane.
- `Tenant scope` is Store's physical observation, replay, repair, dedupe,
  export, import, backup, PITR, and recovery-admission blast-radius boundary.
  It is not an application account id, semantic owner id, or
  identity-provider claim.
- `Authenticity requirement` and `Authenticity class` are the declared
  authenticity policy demanded by a physical lane. `Authenticity result` is the
  observed/admitted outcome for bytes under that policy. Metadata may declare
  only requirement/class. Only admission or checking may produce a result.
  None of these is a checksum or content digest.
- `Custody posture` is the admitted handling state for backup/export/repair and
  key lifecycle responsibilities. It is not proof that an external identity
  provider authenticated a person. At minimum the vocabulary must distinguish
  internal Store custody, export-prepared, exported out of custody, imported
  but unreadmitted, readmitted, custody unavailable, custody denied, and custody
  unsupported.
- `Legacy posture` names how pre-S.5.1 or migrated artifacts are treated. At
  minimum the vocabulary must distinguish legacy unscoped, readmission
  required, security metadata unavailable, and unsupported legacy artifact.
- `Unsupported` means the backend or deployment cannot supply the capability.
  `Unavailable` means the capability exists but the required evidence is not
  currently present. `Stale` means the evidence depends on an expired or
  superseded basis. These must remain separate outcomes.
- Store witnesses must not contain identity-provider claims as authority.
  Identity-derived declarations may be input to admission, but the admitted
  Store witness must be Store-owned. Specifically, a JWT subject is not tenant
  scope, an application org id is not tenant scope, a KMS key id is not key
  scope, an IAM role is not custody posture, and an operator identity is not
  repair authority.
- Serde representations are never admission authority. Deserialization may
  create raw declarations only, and every deserialized security value must be
  readmitted before it can become a witness or readiness input.

## Lane Policy Matrix

- Platform-grade lanes deny missing metadata, deny unsupported authenticity,
  treat stale key posture as stale or rebind-required, and deny wrong tenant
  scope.
- Legacy migration lanes may treat missing metadata as read-only and
  readmission-required, may admit unsupported authenticity only as explicitly
  degraded migration posture, and must still deny wrong tenant scope.
- Forensic lanes may observe missing metadata or stale key posture, but that
  observation is not admissible platform authority and may not publish ordinary
  readiness.

## Shared Crate Usage Contract

S.5.1 must use `worth-foundational` and `worth-proof` deliberately, not as
late implementation discovery.

**Use Store-owned types for authority**

- key scope, key version posture, tenant scope, authenticity requirement,
  authenticity result, custody posture, and repair blast radius remain
  Store-owned because they govern physical byte survival and security
  admission.
- Store-owned current-scope witnesses must not be replaced with Foundational
  identity wrappers or Proof witnesses while the meaning is still Store-owned.

**Use `worth-foundational` for shared boundary meaning**

- use `worth_foundational::aspects()` for native aspect authoring, validation,
  authoritative aspect state admission, masks, and patches where security-scope
  facts cross an aspect-native Store boundary.
- use `worth_foundational::compatibility()` only for explicit JSON/readmission
  tests or terminal projection bridge debt; native S.5.1 authoring must not go
  through JSON.
- use `worth_foundational::canonicalization_api::{common_path, lower_lane,
  stronger_lane}` and facade helpers such as `prepare_canonical_basis_sequence`,
  `prepare_canonical_basis_bundle`, `prepare_canonical_comparison`,
  `derive_canonical_digest`, and identity canonical-basis helpers when S.5.1
  evidence needs reproducible basis, mismatch classification, digest readiness,
  or trust-boundary readmission.
- use Foundational boundary-artifact facade exports for summary, report,
  artifact, receipt, authoritative-current, support-only, and materialization
  taxonomy when S.5.1 publishes boundary artifacts.
- use `worth_foundational::boundary_evidence_api::{common_path, lower_lane,
  stronger_lane}` and facade helpers under `boundary_evidence()` for
  provenance, executed receipts, completed receipts, support truth, degraded
  recovery posture, attachment materialization, and readmission.
- use `worth_foundational::profiles_api::{common_path, lower_lane,
  stronger_lane}` for requested/admitted/materialized diagnostic,
  compatibility, support, retention, and certification posture attached to
  S.5.1 boundary artifacts.
- use `worth_foundational::performance_api::{common_path, lower_lane,
  stronger_lane}` for performance claim authoring, policy-admission receipts,
  counter-backed receipts, report planning, certified/readmitted performance
  bundles, and readiness; elapsed time or logs are not substitutes.

**Use `worth-proof` for progression law**

- import through `worth_proof::prelude::*` by default and use raw surfaces only
  when the progression nouns must remain visually explicit.
- use `Recipe`/`recipe(...)` staged progression, `AuthorityWitness`,
  `CapabilityWitness`, `AssumptionBasis`, freshness/readmission states, and
  `ExecutionReadyRecipe`/executed progression where S.5.1 turns raw scope into
  admitted, lowered, ready, or executed handoff forms.
- use checked progression and `ProofOutcome`/`TransitionOutcome` so `Denied`,
  `Deferred`, `Stale`, `RebindRequired`, and `Failed` remain distinct.
- use `pair`, `non_empty`, `CanonicalVec`, `UniqueVec`, `join_ready`, and
  fixed-shape helpers when S.5.1 binds multiple evidence rows and the
  cardinality is part of the proof.

The rule is: strongest Store type first, Foundational at shared boundary
meaning, Proof for proof-bearing progression. Do not flatten Store authority
into generic Foundational vocabulary, and do not rebuild Proof progression with
local booleans or comments.

Store authority cannot be minted by Foundational evidence or Proof progression
alone. Foundational and Proof make Store authority inspectable, composable, and
lawful, but a Store-owned witness constructor remains the source of Store
security authority.

## Planned Directory Skeleton

Implementation may adjust exact filenames to match existing crate topology,
but the responsibility boundaries must remain visible:

- `worth-store-security/src/scope_vocabulary.rs` owns key scope, key version,
  tenant scope, authenticity class, and custody posture terms.
- `worth-store-security/src/scope_admission.rs` owns sealed admission into
  current security-scope witnesses.
- `worth-store-security/src/authenticity_posture.rs` owns authenticity result
  categories separate from physical integrity.
- `worth-store-security/src/security_scope_counters.rs` owns exact admission,
  denial, drift, stale-key, unsupported, and authenticity counters.
- `worth-store-physical-format/src/security_metadata.rs` or a narrower
  existing physical-format module owns page/frame/manifest metadata carriage.
- `worth-store-wal/src/security_metadata.rs` owns WAL/checkpoint metadata
  carriage if WAL topology is separate enough to require its own module.
- `worth-store-blob-chunks/src/security_scope_readiness.rs` owns S.7 blob
  readiness seeds.
- `worth-store-operations` or `worth-store-offline-verifier` owns backup,
  export, and repair readiness seeds where those artifacts already live.
- `worth-store-readiness/src/s6_security_scope_readiness.rs` owns the S.6
  handoff that I/O QoS must consume.
- `worth-store-certification` owns direct integration tests that exercise the
  production security-scope boundaries; it does not own a parallel scenario,
  oracle, transcript, or evidence authority.

## Phase Plan

### Phase 1: Security Scope Vocabulary And Authority Separation

Freeze the vocabulary that distinguishes semantic authority, physical byte
authority, authenticity authority, key custody, and tenant blast-radius scope.

**Relevant subsystems**

- `worth-store-security`
- `worth-store-authority`
- `worth-store-readiness`
- `worth-store-physical-format`
- `worth-store-proof`
- `worth-store-certification`
- `worth-foundational`
- `worth-proof`

**Relevant APIs**

- Store physical witnesses from `S.1` through `S.5`
- `worth_foundational::aspects()` for native aspect contracts, values,
  validation, authoritative state admission, projection/mutation/diagnostic
  masks, and patch vocabulary.
- `worth_foundational::canonicalization_api::lower_lane::basis` and
  `worth_foundational::canonicalization_api::stronger_lane` for canonical basis
  readiness and readmission vocabulary.
- `worth_proof::prelude::*` for witness-authorized progression surfaces.
- existing Store authority witnesses and current-authority admission surfaces

**Warnings**

- Do not model key ids, tenant ids, or authenticity classes as raw strings.
- Do not let semantic artifact identity double as cryptographic authority.
- Do not introduce actual encryption algorithms here; this phase freezes
  typed scope and authority categories.
- Do not make certification the owner of this vocabulary; certification may
  prove the vocabulary, but lower Store crates must define it.

**Test requirements**

- Adversarial equivalence: two physically identical page witnesses with
  different tenant scopes or key scopes are not equivalent for security
  admission even if their semantic commit basis matches.
- Adversarial rejection: raw string tenant labels, terminal JSON key labels, and
  unclassified authenticity labels cannot satisfy any cryptographic scope API.
- Authority-boundary compile-fail: semantic commit ids cannot be passed where
  current key-scope or tenant-scope witnesses are required.
- Cross-authority compile-fail: a `StoreCurrentAuthorityWitness` without
  security-scope admission cannot stand in for key, tenant, authenticity, or
  custody authority.

**Engineering decisions**

- Introduce distinct Store-owned types for key scope, key version, tenant
  scope, authenticity class, and cryptographic custody posture.
- Carry Foundational canonical-basis vocabulary at evidence boundaries, while
  keeping physical cryptographic authority Store-owned.
- Represent unavailable cryptographic capability as typed unsupported posture,
  not as omitted fields.
- Use `worth-proof` for legal progression and checked outcome topology; use
  Store-owned types for what physical security authority means; use
  Foundational for published evidence and boundary packaging.
- Use Foundational aspect contracts only for boundary-shaped security facts;
  keep hot physical metadata in Store-native representations until it crosses
  an evidence, canonicalization, diagnostic, or certification boundary.

**Open questions**

- Exact external identity admission formats are deferred to `S.11`; this
  milestone only creates Store-owned durable scope vocabulary.

### Phase 2: Sealed Security-Scope Admission And Current Witnesses

Turn the vocabulary from Phase 1 into proof-bearing, sealed admission artifacts
that later physical paths can consume without rediscovering or trusting labels.

**Relevant subsystems**

- `worth-store-security`
- `worth-store-authority`
- `worth-store-readiness`
- `worth-store-contracts`
- `worth-proof`

**Relevant APIs**

- current Store authority witnesses
- security-scope admission requests
- key-version posture and custody posture declarations
- Store admission receipt constructors and Foundational evidence-row
  constructors
- readiness handoff contracts
- `worth_proof::prelude::{recipe, AuthorityWitness, CapabilityWitness,
  ProofOutcome, ProofOutcomeKind}`
- `worth_proof::raw::{AssumptionBasis, TransitionOutcome}` when explicit
  freshness or transition nouns are clearer than the pleasant lane.
- `worth_foundational::boundary_evidence_api::lower_lane` for executed,
  completed, provenance, and support-truth receipt vocabulary after Store
  admission succeeds.
- `worth_foundational::canonicalization_api::lower_lane::basis` for reproducible
  admission basis rows; digest evidence must remain derived from ready basis.

**Warnings**

- Do not expose struct-literal construction for current key-scope, tenant-scope,
  authenticity-scope, or custody-scope witnesses.
- Do not accept lower-authority projections, copied receipt ids, copied proof
  ids, or copied counter rows as admission evidence.
- Do not collapse unsupported, unavailable, stale, failed, and wrong-scope into
  one denial state.
- Do not let identity-provider claims pass through as Store authority. JWT
  subjects, application org ids, KMS key ids, IAM roles, and operator
  identities may be raw declarations for admission only.
- Do not let deserialized security values become witnesses without explicit
  readmission.

**Test requirements**

- Adversarial progression: raw security-scope vocabulary can become a current
  security-scope witness only through the admitted Store proving function, and
  downstream readiness APIs accept only that proof-carrying form.
- Adversarial rejection: external crates cannot construct current key, tenant,
  authenticity, or custody witnesses by struct literal, copied fields, JSON
  readmission text, semantic commit ids, or `StoreCurrentAuthorityWitness`
  alone.
- Denial localization: stale key version, wrong tenant scope, unsupported
  secure posture, missing custody posture, and replayed admission evidence
  produce distinct typed denials.
- Admission identity binding: admitted scope witnesses bind Store authority,
  physical evidence identity, security-scope identity, and proof progression
  identity as one source. Admission emits counter-backed receipts; witnesses
  carry counter lineage only when downstream certification or performance proof
  requires it.
- Identity-provider denial: JWT subject, application org id, KMS key id, IAM
  role, and operator identity values cannot be passed directly where Store
  key, tenant, custody, authenticity, or repair authority witnesses are
  required.

**Engineering decisions**

- Define sealed current-scope witness types in lower Store crates, with private
  fields and read-only accessors.
- Make admission consume typed Store physical evidence plus typed
  key/tenant/authenticity/custody scope inputs, not raw ids or certification
  rows.
- Emit Foundational boundary evidence as courtroom output from admission; do
  not let that evidence become the source authority for Store security scope.
- Use Proof-backed progression to express how admission moves from raw scope to
  current witness to readiness; do not describe that progression as evidence.
- Carry exact counters on admission receipts so later phases can prove they did
  not silently scan or reclassify security metadata. Attach counter lineage to
  witnesses only when a downstream proof contract needs that lineage.
- Model raw -> admitted -> ready security-scope progression with Proof
  transition shapes or a locally named Store wrapper that consumes Proof
  progression categories; do not encode the progression as booleans on one
  mutable struct.
- Attach Foundational boundary evidence only after Store admission has produced
  the current-scope witness.

**Open questions**

- None. S.11 may choose external identity and key-management integrations, but
  S.5.1 must define Store's current-scope admission shape now.

### Phase 3: Physical Security Metadata Carriers

Add the Store-native carrier types that let pages, frames, WAL/checkpoint
records, and manifests carry admitted security scope without turning that scope
into terminal projection text or generic metadata.

**Relevant subsystems**

- `worth-store-security`
- physical page headers
- frame headers
- WAL/checkpoint records
- segment and root manifests
- physical root admission
- physical security metadata constructors

**Relevant APIs**

- page/frame header constructors
- WAL record framing
- manifest/root publication witnesses
- current security-scope witness accessors from Phase 2
- `worth_foundational::aspects().authoritative_state()` for boundary state
  snapshots when security metadata is exposed as aspect-native evidence.
- `worth_foundational::canonicalization_api::common_path` and
  `lower_lane::comparison` for comparing independently produced physical
  security metadata basis without digest-only equivalence.
- `worth_foundational::profiles_api::lower_lane::attachment` when attaching
  diagnostic, compatibility, certification, or support posture to physical
  metadata boundary artifacts.

**Warnings**

- Do not rewrite S.1, S.3, or S.4 history. This phase explicitly backfills the
  metadata they now need to carry forward.
- Do not make metadata optional in ordinary platform-grade paths; unsupported
  capability must be explicit.
- Do not use metadata as authenticity proof. Physical metadata may carry
  authenticity requirement/class, but only admitted checking may produce
  authenticity result.
- Do not let metadata lower to terminal JSON before it reaches physical
  admission.
- Do not let serde-loaded metadata bypass readmission; deserialization creates
  raw declarations only.

**Test requirements**

- Adversarial parity: page, frame, WAL, and manifest witnesses preserve their
  existing physical identity while additionally carrying typed key scope,
  tenant scope, authenticity requirement/class, custody posture, legacy
  posture, and key-version posture.
- Adversarial rejection: a stale page/frame witness or WAL record missing
  security metadata cannot be admitted into a platform-grade physical metadata
  carrier.
- Compile-fail bypass: external callers cannot attach security metadata to a
  page/frame/WAL/manifest witness by copying fields from an admitted witness.
- Canonical parity: independently produced physical metadata carriers compare
  through Foundational canonical basis and mismatch classification, not string
  labels or digest-only equality.
- Result separation: physical metadata constructors cannot set
  `AuthenticityResult`; only admission/checking APIs can produce that value.

**Engineering decisions**

- Metadata belongs in Store physical witnesses and canonical basis rows, not in
  serde/JSON projections.
- Frame and WAL compatibility must reserve enough structure for later
  encryption/authentication without choosing algorithms now.
- Physical metadata constructors consume Phase 2 current-scope witnesses and
  produce physical security metadata witnesses with private fields.
- Missing metadata in migrated or unsupported paths must become typed
  unsupported, unavailable, or legacy/readmission-required posture, not `None`.
- Canonical comparison of metadata must use Foundational canonical basis and
  mismatch classification; it may not compare terminal strings, debug labels,
  or digest bytes as authority.

**Open questions**

- Exact binary layout expansion strategy may be selected by implementation,
  but the public witness vocabulary may not remain absent.

### Phase 4: Stable Read And Recovery Scope Propagation

Make S.5 stable-read protection, root observation, recovery admission, and
logical-decode entry preserve the physical security metadata from Phase 3
without rediscovering, weakening, or projecting it.

**Relevant subsystems**

- `worth-store-security`
- S.5 stable read protection
- physical root admission
- recovery admission
- logical decode entry
- certification evidence rows

**Relevant APIs**

- S.5 stable read-plan admission
- protected read observation handles
- root publication and recovery witnesses
- physical metadata carriers from Phase 3
- `worth_foundational::profiles_api::lower_lane::attachment` for diagnostic,
  compatibility, certification, or support posture on propagated metadata.
- `worth_foundational::boundary_evidence_api::lower_lane` for drift and
  closeout evidence after Store propagation has produced typed outcomes.

**Warnings**

- Do not let stable-read protection strip, clone loosely, or summarize security
  scope.
- Do not let recovery admission reconstruct security scope from page ids,
  digests, logs, or terminal projections.
- Do not let logical decode observe bytes before scope drift is localized.

**Test requirements**

- Stable-read preservation: S.5 stable read plans preserve key scope, tenant
  scope, authenticity requirement, custody posture, and key-version posture
  across protection, observation, and release.
- Drift localization: mismatched tenant scope between page header and manifest
  is reported as scope drift before logical decode.
- Recovery replay: recovery over WAL/checkpoint/root evidence preserves
  security scope exactly and rejects stale or missing scope before replay can
  publish a read-admissible root.
- Compile-fail bypass: logical decode cannot consume raw physical bytes or a
  root witness unless the propagated security scope witness is present.

**Engineering decisions**

- Stable-read, recovery, and logical-decode entry APIs consume Phase 3 metadata
  carriers and produce propagation outcomes with exact counters.
- Scope drift is a physical security denial, not a semantic decode error.
- Propagation outcomes may attach Foundational evidence after Store typed
  propagation succeeds or denies; Foundational evidence cannot replace the
  propagation witness.
- Security-scope propagation counters must distinguish preserved, missing,
  stale, drifted, and unsupported posture cases.

**Open questions**

- None. Stable read and recovery paths must preserve security scope before
  later authenticity, blob, backup, or repair phases consume those paths.

### Phase 5: Authenticity Distinct From Integrity

Make authenticity admission structurally separate from checksums and physical
corruption detection.

**Relevant subsystems**

- `worth-store-security`
- S.3 physical integrity reports
- scrub and quarantine evidence
- frame/page admission
- certification evidence rows
- stable-read and recovery admission

**Relevant APIs**

- checksum validation reports
- physical quarantine reports
- proof evidence rows for corruption and drift
- authenticity requirement and authenticity result vocabulary
- Phase 2 current security-scope witnesses
- `worth_proof::prelude::ProofOutcomeKind` or a locally named Store outcome
  enum that preserves the same success, denied, deferred, stale,
  rebind-required, and failed categories for authenticity-required admission.
- `worth_foundational::boundary_evidence_api::lower_lane` for executed receipt,
  support-truth, degraded/unavailable posture, and closeout evidence.
- `worth_foundational::profiles_api::lower_lane::progression` for requested,
  admitted, and materialized authenticity diagnostic/certification posture.

**Warnings**

- A checksum match is not authenticity success.
- A content digest is not proof that the bytes came from the admitted key
  scope, tenant scope, or custody posture.
- Do not let later operator tooling infer authenticity from "no corruption."
- Do not let unsupported authenticity posture silently pass in lanes that
  require authenticity.
- Do not let physical metadata, serde input, or terminal projections declare
  `AuthenticityResult`; they may declare only requirement/class raw inputs that
  must pass Store admission/checking.

**Test requirements**

- Adversarial equivalence: a page can be checksum-valid while authenticity is
  unavailable, unsupported, or failed, and the result must remain
  machine-distinguishable.
- Adversarial rejection: authenticity-required lanes reject checksum-valid
  bytes when the authenticity witness is absent, stale, wrong-scope, or
  unsupported.
- Certification localization: reports distinguish `corrupt`, `authenticity
  failed`, `authenticity unavailable`, and `authenticity unsupported`.
- Proof separation: a content digest or checksum receipt cannot be passed to an
  API requiring authenticity proof.
- Policy switch: changing authenticity requirement policy changes admission
  outcome and counters without changing physical decode results.
- Declaration/result split: `AuthenticityRequirement` and
  `AuthenticityClass` can be present on physical metadata while
  `AuthenticityResult` remains absent until the admission/checking path runs.

**Engineering decisions**

- Add typed authenticity requirement/class declarations and typed authenticity
  result categories independent of integrity categories.
- Carry exact counters for checksum-valid/authenticity-failed and
  checksum-valid/authenticity-unavailable cases.
- Keep authenticity evidence policy-switchable without changing physical decode
  results.
- Model authenticity policy posture as a typed input with at least required,
  unavailable-admitted, unavailable-denied, unsupported-denied, and failed
  categories rather than booleans.
- Preserve Proof non-success categories for authenticity admission: wrong
  scope is denial, stale key posture is stale or rebind-required, missing
  evidence can be deferred or denied by policy, and unsupported capability is
  its own typed denial.

**Open questions**

- Algorithm choice and MAC/signature mechanics remain `S.11` work.

### Phase 6: Blob Chunk Scope And Dedupe Readiness

Make S.7 blob chunk metadata and dedupe admission consume security scope before
large-object storage can treat digest equality as a physical sharing claim.

**Relevant subsystems**

- `worth-store-security`
- `worth-store-readiness`
- `worth-store-blob-chunks`
- S.7 blob chunk metadata
- Roadmap 1 Milestones 14, 20, and 22

**Relevant APIs**

- chunk-tree manifest plans
- dedupe admission and equivalence-basis declarations
- Phase 2/3 security-scope witnesses
- `worth_foundational::canonicalization_api::lower_lane::comparison` for
  cross-scope dedupe equivalence/mismatch classification.
- `worth_foundational::boundary_evidence_api::common_path` and `lower_lane`
  for provenance, executed receipt, support-truth, degraded recovery, and
  readmission attachments on chunk readiness evidence.
- `worth_foundational::profiles_api::lower_lane::attachment` for retention,
  compatibility, certification, and support posture on blob/chunk artifacts.
- `worth_proof::prelude::{CanonicalVec, UniqueVec, non_empty, pair,
  join_ready}` where chunk/security evidence sets require fixed cardinality,
  uniqueness, or ready-join proof.

**Warnings**

- Do not dedupe blob chunks across tenant/key scopes unless the later security
  policy explicitly admits that equivalence.
- Do not use digest equality as authorization to share physical storage across
  tenants, key scopes, custody postures, or authenticity requirements.
- Do not let blob readiness become a proxy for backup/export or repair
  readiness; those are separate phases with different custody and blast-radius
  laws.

**Test requirements**

- Adversarial equivalence: identical blob content under different tenant or key
  scopes does not collapse into a shared physical claim unless an admitted
  dedupe policy proves it safe.
- Adversarial rejection: chunk manifests without typed key scope, tenant scope,
  authenticity requirement, and custody posture cannot enter platform-grade
  blob lanes.
- Streaming preservation: chunk ingest, verification, export-read preparation,
  tier movement, and reclaim preserve security scope without whole-object
  memory residency.
- Collision denial: digest collision or digest-only equality cannot admit
  cross-scope dedupe without the explicit equivalence basis.

**Engineering decisions**

- Add readiness witnesses for blob chunks and chunk manifests that carry key
  scope, tenant scope, authenticity class, and custody posture.
- Use Proof-backed readiness progression for readiness receipts; keep
  Store-owned physical witness types as the authority and Foundational boundary
  evidence as the published proof package.
- Make dedupe policy require explicit equivalence basis rather than digest-only
  equality when tenant or key scope differs.
- Surface unsupported posture as a typed denial when a later lane claims
  platform-grade security, not as a runtime log or warning.
- Cross-scope blob dedupe must compare Foundational canonical equivalence basis
  plus Store security-scope witnesses; digest equality alone never admits
  dedupe.

**Open questions**

- Whether cross-tenant encrypted dedupe is ever allowed remains a later product
  and security decision; this milestone must make the unsafe default
  unrepresentable.

### Phase 7: Backup, PITR, Export, And Import Custody Readiness

Make S.10 backup, PITR, export, and import declarations bind security scope and
custody posture before any byte stream, capsule, or terminal projection can be
emitted.

**Relevant subsystems**

- `worth-store-security`
- `worth-store-readiness`
- `worth-store-offline-verifier`
- `worth-store-operations`
- S.10 backup, PITR, disaster recovery, and forensics
- S.11 security and key lifecycle

**Relevant APIs**

- backup/PITR bundle declarations
- export/import capsule declarations
- backup/export custody posture witnesses
- Phase 2/3 security-scope witnesses
- Foundational boundary-artifact facade exports for backup, PITR, export, and
  import artifact/receipt/report taxonomy.
- `worth_foundational::boundary_evidence_api::common_path` and `lower_lane`
  for provenance, executed receipt, completed receipt, support-truth, degraded
  recovery, and readmission attachments.
- `worth_foundational::profiles_api::lower_lane::attachment` for custody,
  retention, compatibility, certification, and support posture on backup and
  export artifacts.
- `worth_proof::prelude::{CanonicalVec, UniqueVec, non_empty, pair,
  join_ready}` where backup/export evidence bundles require fixed cardinality,
  uniqueness, or ready-join proof.

**Warnings**

- Do not let backup/export capsules omit key-scope, tenant-scope, authenticity,
  key-version, or custody posture.
- Do not let terminal projection or byte-stream emission occur before custody
  readiness is admitted.
- Do not treat blob chunk readiness as backup/export custody readiness.
- Do not treat trust-boundary crossing as a vague deployment concern. A
  different deployment, Store instance, key-scope generation, tenant-scope
  authority, custody domain, offline export/import path, or backup restoration
  after key rotation must trigger explicit readmission.

**Test requirements**

- Backup/export custody proof: backup, PITR, and export declarations bind
  custody posture and key-version posture to the artifact bundle before any
  byte stream or terminal projection can be emitted.
- Adversarial rejection: backup/export/import declarations without typed key
  custody, tenant scope, authenticity posture, and key-version posture cannot
  enter platform-grade lanes.
- Readmission replay: an imported or restored bundle crossing a trust boundary
  must regain current security scope through explicit readmission before use.
  Tests must cover at least a different deployment, different Store instance,
  different key-scope generation, different tenant-scope authority, different
  custody domain, offline export/import, and backup restoration after key
  rotation.
- Terminal projection denial: projected capsule metadata cannot reconstruct
  custody readiness or key-version posture without Store readmission.
- Serde denial: deserialized backup/import metadata produces raw declarations
  only and cannot satisfy custody, key-version, or tenant-scope readiness until
  readmitted.

**Engineering decisions**

- Add backup/export/import custody readiness witnesses separate from blob and
  repair readiness.
- Backup/export/import artifacts publish Foundational boundary artifacts and
  boundary evidence after Store readiness, not before it.
- Custody readiness uses Proof-backed progression so planned, admitted, ready,
  exported, imported, and readmitted bundle states cannot collapse into one
  artifact.
- Custody posture must distinguish internal Store custody, export-prepared,
  exported out of custody, imported unreadmitted, readmitted, custody
  unavailable, custody denied, and custody unsupported.
- Exact counters must distinguish custody admitted, custody denied,
  key-version stale, unsupported secure posture, unavailable custody evidence,
  and readmission required.

**Open questions**

- Exact capsule format remains later S.10/S.11 work; S.5.1 freezes the
  custody-readiness evidence it must consume.

### Phase 8: Repair And Quarantine Blast-Radius Readiness

Make repair, quarantine, and offline-verifier planning consume tenant and key
scope before any repair read can observe bytes outside its admitted physical
blast radius.

**Relevant subsystems**

- `worth-store-security`
- `worth-store-readiness`
- `worth-store-offline-verifier`
- `worth-store-operations`
- repair/quarantine plans
- S.10 disaster recovery and forensics

**Relevant APIs**

- repair/quarantine plan declarations
- tenant blast-radius witnesses
- physical region and quarantine witnesses
- Phase 2/3 security-scope witnesses
- Foundational boundary-artifact facade exports for repair and quarantine
  artifact/receipt/report taxonomy.
- `worth_foundational::boundary_evidence_api::lower_lane` for support-truth,
  degraded recovery, quarantine, and repair-read closeout evidence.
- `worth_foundational::profiles_api::lower_lane::attachment` for repair
  support, certification, and custody posture.

**Warnings**

- Do not let repair plans cross tenant blast-radius boundaries by default.
- Do not let correct physical region ids override wrong tenant scope, stale key
  version, unavailable custody posture, or missing authenticity requirement.
- Do not let offline verifier reports become repair authority.
- Do not let operator identity become repair authority. Repair blast-radius
  readiness proves where repair may physically observe or read; it does not
  prove who may initiate the repair.

**Test requirements**

- Blast-radius proof: a tenant-scoped repair plan cannot reference physical
  regions outside its admitted tenant scope.
- Repair denial: a repair plan with correct physical region ids but wrong
  tenant scope, stale key version, unavailable custody posture, or missing
  authenticity requirement fails before repair reads can observe bytes.
- Quarantine replay: quarantined physical regions preserve security scope and
  cannot be readmitted by copying quarantine report fields.
- Offline-verifier boundary: offline verification may produce Foundational
  evidence and support truth, but cannot mint repair readiness without Store
  blast-radius admission.
- Operator-auth separation: a valid operator identity, IAM role, or audit
  record cannot satisfy repair blast-radius readiness, and valid repair
  readiness cannot satisfy operator authorization.

**Engineering decisions**

- Add repair blast-radius readiness witnesses separate from blob and
  backup/export readiness.
- Repair and quarantine artifacts publish Foundational boundary evidence only
  after Store blast-radius admission or denial.
- Exact counters must distinguish repair admitted, repair denied,
  cross-scope-region rejection, stale key-version rejection, custody
  unavailable, and quarantine-preserved scope.
- Repair readiness must feed S.10 and S.11 without becoming full operator
  authorization or identity-provider semantics. It is physical observation/read
  permission only.

**Open questions**

- Exact operator authorization and proof-of-possession behavior remains S.11
  work; S.5.1 freezes physical repair blast-radius readiness.

### Phase 9: Later-Milestone Readiness Handoff Closure

Publish separate readiness handoffs for S.6, S.7, S.10, and S.11 so later
milestones consume S.5.1 security scope instead of inventing local metadata.

**Relevant subsystems**

- `worth-store-security`
- `worth-store-readiness`
- `worth-store-io-scheduler`
- `worth-store-blob-chunks`
- `worth-store-operations`
- S.6 I/O QoS
- S.7 blob chunks
- S.10 backup/export custody
- S.10 repair blast-radius
- S.11 security and key lifecycle

**Relevant APIs**

- S.6 I/O QoS readiness handoff
- S.7 blob readiness handoff
- S.10 backup/export custody readiness handoff
- S.10 repair blast-radius readiness handoff
- S.11 security-scope foundation handoff
- Store readiness constructors from earlier S.5.1 phases
- `worth_proof::prelude::{proof_flow, recipe, gate_ready, ready_now,
  join_ready, compose_ready}` for staged progression and ready-join handoff
  construction.
- `worth_foundational::boundary_evidence_api::lower_lane` for published
  handoff evidence after Store readiness has been admitted.

**Warnings**

- Do not collapse S.6, S.7, S.10, and S.11 handoffs into one generic
  readiness object.
- Do not let certification-only rows mint any handoff.
- Do not make S.11 security-foundation readiness claim actual encryption,
  rotation, audit, or operator admission behavior.
- Do not make handoffs ceremonial wrappers. Each handoff must prove a distinct
  downstream permission that no other handoff type can satisfy.

**Test requirements**

- Handoff compile-fail: S.6 I/O scheduler readiness, S.7 blob readiness, S.10
  backup/export custody readiness, S.10 repair blast-radius readiness, and
  S.11 security-scope foundation readiness cannot be minted from raw fields,
  copied proof ids, copied counter receipts, terminal projections, or
  certification-only rows.
- Handoff separation: a valid S.7 blob readiness witness cannot satisfy S.6
  I/O QoS readiness, S.10 backup/export readiness, S.10 repair readiness, or
  S.11 security-foundation readiness.
- Handoff meaning: S.6 proves physical I/O may schedule/read without stripping
  scope; S.7 proves blob chunk/dedupe scope was admitted; S.10 backup/export
  proves custody posture and key-version posture; S.10 repair proves tenant/key
  blast radius; S.11 proves security lifecycle foundation only, not encryption,
  key rotation, audit, or operator authorization.
- Readiness replay: replaying the same physical evidence with changed tenant
  scope, key-version posture, or authenticity requirement fails before handoff
  publication.
- Downstream admission: S.6, S.7, S.10, and S.11 entry APIs accept only the
  matching S.5.1 handoff type.

**Engineering decisions**

- Add separate handoff types for S.6 I/O QoS security readiness, S.7 blob
  security readiness, S.10 backup/export custody readiness, S.10 repair
  blast-radius readiness, and S.11 security-foundation readiness.
- Define the unique authority of each type in its constructor and accessors:
  S.6 scheduler admission, S.7 blob/dedupe admission, S.10 backup/export
  custody admission, S.10 repair observation/read blast radius, and S.11
  security lifecycle foundation readiness.
- Handoff progression uses Proof readiness/execution distinctions so planned,
  admitted, ready, executed, and published handoff states cannot collapse into
  one artifact.
- Published handoff evidence uses Foundational boundary evidence after Store
  readiness, not before it.
- Exact counters must state which handoff was admitted, denied, stale,
  unsupported, or unavailable.

**Open questions**

- Final names may follow implementation topology, but the five handoff
  responsibilities above must remain separate.

### Phase 10: Direct Security-Boundary Verification

Exercise security-scope propagation and denial through production-owned
admission, stable-read, publication, checkpoint, and repair-read boundaries
before full S.11 encryption exists.

**Relevant subsystems**

- S.5 physical isolation owners
- Store certification integration tests
- `worth-store-security`
- `worth-store-readiness`

**Relevant APIs**

- production security-scope admission APIs
- stable-read and publication APIs
- checkpoint and repair-read admission APIs
- S.5.1 readiness handoffs from Phase 9

**Warnings**

- This is not S.11 certification. It is security-scope readiness that prevents
  later milestones from ignoring the metadata.
- Do not add log-based proof, JSON-shaped verdicts, generated transcripts, or
  synthetic oracle layers.
- Tests do not decide identity-provider semantics.

**Test requirements**

- Adversarial execution: production APIs preserve key scope and tenant scope
  across stable read plans, root swaps, checkpoint publication, and repair-read
  admission.
- Adversarial rejection: stale key version, wrong tenant scope,
  missing authenticity requirement, and replayed custody posture produce typed
  denials before logical decode.
- Re-admission tests change tenant scope, key-version posture, or authenticity
  requirement at the real boundary and observe drift rather than reused
  readiness.
- JSON appears only in terminal projection or hostile/readmission tests.

**Engineering decisions**

- Tests call lower Store owners through their public production facades and
  assert returned capabilities, denials, persisted state, and counters.
- Shared test setup is limited to fixture construction; it cannot mint verdicts
  or restate production authority.

**Open questions**

- None. Direct boundary tests must expose S.5.1 scope drift before later
  S.6/S.7/S.10 work depends on those contracts.

### Phase 11: Integration Verification And Cleanup

Run the affected owner and integration suites, remove obsolete test support,
and document the public security-scope contracts that later milestones consume.

**Relevant subsystems**

- Store security and readiness owners
- Store certification integration tests
- downstream S.6/S.7/S.10/S.11 consumers
- `worth-store-readiness`

**Relevant APIs**

- focused owner and integration test targets
- direct counters returned by production execution
- S.5.1 readiness handoffs from Phase 9

**Warnings**

- Integration tests consume Store security/readiness contracts; they do not
  define them.
- Do not create closeout rows, evidence bundles, generated reports, or
  self-comparison machinery.

**Test requirements**

- Direct counter assertions cover security-scope admissions, denials,
  stale-key rejections, tenant-scope drifts, authenticity-unavailable results,
  and unsupported-capability denials.
- Compile-fail tests prove certification and test-support code cannot construct
  Store readiness witnesses.
- Integration tests execute the named production APIs and assert outcomes at
  the consuming boundary.

**Engineering decisions**

- Keep constructors and readiness admission in lower Store crates.
- Delete support code that only translates production outcomes into another
  test-owned verdict or report.
- A focused failing test is the evidence; no separate closeout artifact is
  maintained.

**Open questions**

- None. Later phases consume the production contracts, not a certification
  artifact.

## Must Ship

- Store-owned key scope, key version, tenant scope, authenticity class, and
  custody posture vocabulary, with requirement/posture/result/witness/evidence
  and readiness nouns kept distinct.
- Sealed current security-scope admission witnesses that bind Store authority,
  physical evidence identity, security-scope identity, and proof progression
  identity. Counter-backed receipts are emitted by admission and carried by
  witnesses only where downstream proof requires counter lineage.
- Physical security metadata carriers for pages, frames, WAL/checkpoint
  records, manifests, and root/recovery admission. They may carry authenticity
  requirement/class, but not authenticity result.
- Stable-read, recovery, and logical-decode propagation surfaces that preserve
  security scope and localize drift before logical decode.
- Distinct integrity versus authenticity result categories and counters.
- Blob chunk and cross-scope dedupe readiness witnesses for S.7.
- Backup, PITR, export, and import custody readiness witnesses for S.10/S.11.
- Repair and quarantine blast-radius readiness witnesses for S.10/S.11.
- Separate handoff artifacts for S.6 I/O QoS, S.7 blobs, S.10 backup/export,
  S.10 repair, and S.11 security-foundation work.
- Explicit Foundational adoption surfaces for aspect-native security facts,
  canonical basis/mismatch/digest readiness, boundary artifacts, boundary
  evidence, profiles, and counter-backed performance receipts.
- Explicit Proof adoption surfaces for security-scope progression, freshness,
  readmission, readiness, execution, non-success outcome topology, and
  fixed-shape evidence binding.
- Direct integration tests proving security-scope metadata survives physical
  execution and rejects stale, missing, or wrong-scope inputs.
- Identity-provider claims and serde projections remain raw declarations until
  Store admission/readmission produces Store-owned witnesses.

## Must Preserve

- Store owns physical byte survival and cryptographic boundary evidence.
- `worth-relational` owns semantic truth, MVCC, identity semantics, and
  transaction meaning.
- External identity systems may provide admission evidence, but Store does not
  become an identity provider. Store witnesses must not contain JWT subjects,
  application org ids, KMS key ids, IAM roles, or operator identities as their
  authority.
- `worth-proof` provides progression law and checked outcomes; Store lower
  crates own the cryptographic boundary contract and readiness constructors.
- Foundational describes and packages shared boundary meaning; it does not
  replace stronger Store physical/security authority types.
- Proof encodes legal progression and witness-authorized transitions; it does
  not become a runtime scheduler, diagnostics crate, or storage authority.
- S.1, S.3, and S.4 are not rewritten retroactively; their missing security
  metadata is backfilled here as explicit follow-on foundation work.
- JSON remains confined to terminal projection or hostile/readmission lanes.
  Serde creates raw declarations only and cannot create admission authority.

## Acceptance Evidence

- Compile-fail tests proving raw strings, semantic ids, JSON projections, and
  lower-authority digests cannot satisfy key-scope, tenant-scope,
  authenticity, or custody APIs.
- Compile-fail tests proving JWT subjects, application org ids, KMS key ids,
  IAM roles, and operator identities cannot satisfy tenant-scope, key-scope,
  custody-posture, or repair-authority APIs.
- Compile-fail tests proving deserialized security values and terminal
  projections cannot construct witnesses or readiness without Store
  readmission.
- Compile-fail tests proving `StoreCurrentAuthorityWitness`, copied proof ids,
  copied counter receipts, copied witness fields, and certification-only rows
  cannot mint S.5.1 readiness.
- Runtime and integration tests proving metadata carriers, stable read plans,
  manifests, WAL/checkpoint records, blob readiness, backup/export readiness,
  repair readiness, and S.6/S.7/S.10/S.11 handoffs carry security scope through
  admitted physical paths.
- API-adoption tests or compile-time assertions proving ordinary S.5.1 evidence
  uses Foundational native aspect, canonicalization, boundary artifact,
  boundary evidence, profile, and performance lanes where the spec names them.
- Progression tests proving S.5.1 uses Proof outcome/readiness/freshness
  topology without flattening denial, deferred, stale, rebind-required, and
  failed states into one error.
- Adversarial tests proving wrong tenant, stale key version, missing
  authenticity requirement, unsupported secure posture, and cross-scope dedupe
  are rejected with typed diagnostics.
- Policy-matrix tests proving platform-grade, legacy migration, and forensic
  lanes differ exactly as specified for missing metadata, unsupported
  authenticity, stale keys, and wrong tenant scope.
- Trust-boundary tests proving different deployment, Store instance, key-scope
  generation, tenant-scope authority, custody domain, offline export/import,
  and backup restoration after key rotation require readmission.
- Authenticity split tests proving metadata can declare requirement/class while
  only admission/checking can produce `AuthenticityResult`.
- Repair-auth separation tests proving repair readiness authorizes only
  physical observation/read blast radius and cannot initiate repair as an
  operator authorization proof.
- Exact counters for security-scope admissions, key-version observations,
  tenant-scope drifts, authenticity failures/unavailable results,
  unsupported-capability denials, custody-posture denials, cross-scope dedupe
  denials, and cross-scope repair rejections.
- Separate S.6, S.7, S.10 backup/export, S.10 repair, and S.11 handoff
  artifacts proving later milestones consume the security-scope foundation
  rather than inventing parallel metadata.

## Sequencing Notes

`S.5.1` belongs immediately after `S.5` because physical read stability must
carry security scope before I/O QoS, blob chunks, backup/PITR, repair, and full
security work depend on those physical paths. It is intentionally before `S.6`
and `S.7`; otherwise encrypted/authenticated I/O and blob metadata become
retrofit work. It is also intentionally before S.10 and S.11 so backup,
export, repair, key lifecycle, cryptographic erasure, and operator/service
admission consume an already-typed Store security boundary instead of defining
their own local vocabulary.
