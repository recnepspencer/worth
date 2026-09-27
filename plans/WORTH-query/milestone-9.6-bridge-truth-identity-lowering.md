# Milestone 9.6 Bridge Truth Identity Lowering

> **Status:** Phase 2B **closed** (2026-06-15) â€” evidence/feeder root breaks and
> repair are complete for `worth-query` + `worth-runtime-bridge`. Downstream
> crates (worth-topo, worth-server, etc.) are explicitly out of Phase 2B scope
> and remain for Phase 8+. Phase 7 QA re-cleared post-2B under compiler-enforced
> category boundaries.
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Hard gate for:** [milestone-9.7.md](./milestone-9.7.md) Phases 14â€“16
>
> **Purpose:** truth-routing identity authority is mechanically preserved from
> relational source truth through the runtime bridge, Query intake, receipts,
> feeders, downstream adapters, and hostile certification. Lower-authority
> categories must fail at authority APIs; the compiler discovers the real graph.

## Goal

Force every lower-authority identity category to fail at authority APIs, let
compiler breakage discover the real dependency graph, then fix top-down from
source-truth roots to bridge carriers, Query intake/storage/feeders, and
downstream consumers.

The milestone is not primarily a string cleanup. Strings were the visible
collapse. The architectural defect is authority lifecycle collapse: projection
labels, digest evidence, external tokens, bridged identities, cached keys, and
raw representations were allowed to masquerade as current owner-admitted
authority.

## Why

`9.6` now closes Law 42 for this seam:

> Identity authority must be preserved, not reconstructed from representation.

If authority cannot be preserved across a boundary, the owning subsystem must
perform fresh admission from source truth with an explicit witness. No
lower-authority representation may promote itself.

This blocks `9.7`, `9.8`, and `9.9` because shared read pinning, downstream
support kit adoption, and graph-touch obligation authority all depend on one
stable fact: receipts and bridge evidence must say what authority they carry.
If a digest, projection, or external token can re-enter as authority, later
milestones will build journal, report, support, and graph APIs on sand.

## Mechanical Breakage Rules

This milestone is **break -> compiler-discover -> fix -> guard**. Not migrate.
Not strangle. Not validate along the way.

1. **Law 42 is the primary frame.** String collapse is a symptom. The root
   categories are current owner-admitted authority, boundary-bridged authority,
   external token, projection label, digest evidence, and raw representation.

2. **Root hard breaks come before matrix work.** Phase 1 defines the authority
   categories and root APIs to break. Phase 2 installs the breaks. The compiler,
   not a pre-curated table, becomes the authoritative discovery tool.

3. **Compiler failure is the discovery ledger.** Run `cargo check --workspace`
   immediately after the root hard break. Every discovered failure is classified
   by attempted category, required category, owning phase, and required guard.

4. **No projection re-entry.** Reporting/display/digest accessors may terminate
   at logs, UI, JSON, diagnostics, and human explanation. They cannot drive
   composition, lookup, admission, routing, comparison, or coherence checks.

5. **No authority reconstruction.** `Display`, `AsRef<str>`, `as_str()`, raw
   `String`, `Arc<str>`, `&str`, digest labels, bridge retained evidence text,
   and `from_external_authority` are not authority constructors.

6. **No legacy compatibility.** No deprecated aliases, escape hatches, dual APIs,
   feature flags, default stringify methods, or temporary downstream shims.
   Restoring a lower-authority route so a downstream crate compiles is a
   milestone failure, not a migration tactic.

7. **The old matrix is demoted.** Existing trace maps and matrix rows are
   retained as historical inventory, but row status cannot close unless it maps
   to one of:
   - compiler failure fixed by authority-category type
   - compile-fail guard added
   - terminal projection explicitly quarantined
   - deferred owner milestone named

8. **Tests prove the new authority model.** No old-vs-new parity, string
   round-trips, golden fixtures of deleted display conventions, or tests that
   keep old authority reconstruction alive for comparison.

### Required Compiler Gates

- Projection cannot satisfy authority.
- Digest evidence cannot satisfy authority.
- External token cannot satisfy authority.
- Bridged identity cannot satisfy current authority.
- Wrong `Kind` marker cannot satisfy another identity family.
- Raw `String`, `Arc<str>`, `&str`, `Display`, or `AsRef<str>` cannot enter
  authority APIs.
- Reporting accessors cannot feed authority constructors or equality/coherence
  checks.
- Bridge retained evidence cannot be rebuilt from text or
  `from_external_authority` outside owner-controlled admission.

### WORTH-Foundational Substrate

Use `worth-foundational` as the standard generic boundary vocabulary. It must
remain runtime-agnostic; bridge/query crates supply their own authority and kind
markers.

| Authority category | Standard substrate | Meaning in this milestone | May feed authority APIs? |
|--------------------|--------------------|---------------------------|--------------------------|
| Current owner-admitted authority | `FoundationalAuthorityIdentity<Value, Authority, Kind>` | The owning subsystem admitted the value as current authority with a witness. | Yes, only when `Authority` and `Kind` match. |
| Boundary-bridged authority | `FoundationalBoundaryBridgedIdentity<Value, Authority, Kind>` | A prior authority value crossed a boundary and now requires revalidation before current-authority use. | No; must be revalidated by owner. |
| External token | `FoundationalExternalIdentityToken<Value, Kind>` | Host/runtime/external text arrived from outside the owner. | No; must be admitted by owner. |
| Projection label | `FoundationalProjectionIdentity<Label, Kind>` | Human/reporting/display identity derived from authority. | No; terminal reporting only. |
| Digest evidence | `FoundationalDigestIdentityEvidence<Basis, Authority, Kind>` | Canonical or digest proof/equivalence evidence. | No; proof/equivalence only. |
| Raw representation | `String`, `Arc<str>`, `&str`, `Display`, `AsRef<str>` | Representation without authority category or witness. | Never. |

### Authority Category Decision Table

Implementers do not decide category policy per call site. Use this table.

| If the code is doing this | Required input category | Forbidden substitutes |
|---------------------------|-------------------------|-----------------------|
| Minting relational truth identity from relational source truth | Current owner-admitted authority with relational owner witness | External token, projection, digest, bridged value, raw text |
| Carrying identity across runtime bridge boundary | Boundary-bridged authority produced from current authority | Projection, digest, raw text, external token |
| Query intake, receipt construction, storage, or feeder composition | Current owner-admitted authority, or explicitly revalidated bridged authority where Query owns admission | Projection, digest, external token, raw text |
| Retained bridge evidence lookup | Typed retained evidence identity or bridge-owned bridged authority | `as_str()`, `Display`, `from_external_authority`, Query reporting labels |
| Equality, coherence, routing, or admission checks | Typed authority/evidence value at the same family and kind | Reporting accessor output, digest label text, raw strings |
| Human output, JSON, diagnostic logs, status tables | Projection label or digest evidence | Re-entering the projected text into authority APIs |

## Fix Order

Work top-down. Do not restore any forbidden API to unblock a downstream crate.

```
1  -> root-category hard break design
2  -> compiler-discovery break - workspace RED (Frontier A: truth routing)
2A -> upstream relational/signal boundary verification gate
2B -> compiler-discovery break - workspace RED (Frontier Bâ€“F: evidence/feeder roots)
3  -> worth-relational source-truth admission
4  -> worth-runtime-bridge authority carriers
5  -> worth-query intake, storage, adapters, and harnesses
6  -> worth-query receipts, write surfaces, and shared read pins
7  -> projection re-entry purge across evidence, intent, signal, and feeders
8  -> downstream production adapters, including worth-topo and worth-kernel
9  -> hostile compile-fail certification and downstream/harness closure
10 -> workspace GREEN + compiler ledger closeout
```

### Phase 1 â€” root-category hard break design (do not fix call sites)

**What this phase is:** define the exact authority categories, marker kinds, and
root API breaks that will make lower-authority identity fail mechanically. This
phase prepares the compiler-discovery cut; it does not try to finish discovery
by manual scan.

**What this phase is not:**
- Not a comprehensive agent scan.
- Not filling a matrix as the source of truth.
- Not migrating call sites or deleting every downstream use.
- Not a Rust `include_str!` inventory module.
- Not a proof that grep has found the whole graph.

**Design outputs**

| Output | Required content |
|--------|------------------|
| Root authority family list | Commit, snapshot, patch, entity, branch, signal route, evidence, session, basis, receipt, feeder, retained bridge mapping, and downstream adapter identity families. |
| Marker-kind plan | One owner authority marker and one identity kind marker per family; wrong `Kind` markers must not unify across identity families. |
| Family category map | For each family: owner authority, `Kind`, allowed authority categories, and first Phase 2 compiler frontier. |
| Root break list | Constructors, accessors, `Display`, `AsRef<str>`, raw-evidence constructors, trait methods, receipt fields, registry keys, and equality paths that currently admit lower-authority values. **Phase 2B extension:** concrete Tier 3â€“6 symbol paths in [Phase 2B](#phase-2b--evidencefeeder-root-break-compiler-discovery-frontier-2). |
| Compiler ledger schema | Failure id, compiler error, broken API, attempted category, required category, owning phase, row status, and compile-fail guard. |
| Trybuild plan | Boundary-specific compile-fail suites for bridge, Query, and downstream/harness consumers. |

**Root families to classify**

1. **Relational source truth** â€” `CommitId`, `EntityId`, `SnapshotId`,
   `VersionId`, branch/workspace truth, and bridge presentation export.
2. **Runtime bridge carriers** â€” `TruthCommitIdentity`,
   `TruthSnapshotIdentity`, `TruthPatchIdentity`, `TruthBranchIdentity`, bridge
   causal references, envelopes, receipts, and retained mapping evidence.
3. **Query intake and storage** â€” runtime backend/source adapter contracts,
   mutation receipts, deltas, live patches, write receipts, read receipts,
   snapshot/current-state adapters, and memory workspace records.
4. **Query feeder spines** â€” evidence, intent, signal invalidation, workflow,
   domain-capability, materialization, effect-lifecycle, causal inspection,
   subscription/session, and bridge retained-evidence feeders.
5. **Downstream and harness consumers** â€” worth-topo, worth-kernel, worth-server,
   hadwiger-research, hostile certification fixtures, support-kit tests, and
   old receipt literal construction.

**Phase 1 hard-break checklist**

- [x] Every root identity family has an owner authority marker.
- [x] Every root identity family has a `Kind` marker that cannot satisfy another
  family by structural coincidence.
- [x] Every lower-authority category has an intended `worth-foundational`
  substrate type.
- [x] Every root family appears in a Phase 1 family map with owner authority,
  `Kind`, allowed category forms, and first Phase 2 compiler frontier.
- [x] Every public constructor/accessor slated for removal or restriction is
  named before Phase 2 begins.
- [x] The compiler failure ledger schema is present before the first red run.
- [x] Every required compiler gate has a planned compile-fail target path.
- [x] No call-site fixes are attempted in Phase 1.

**Phase 1A closeout system**

Phase 1 is complete only when the authority design exists in four coordinated
manifests, not as scattered marker names:

| Manifest | Location | Purpose |
|----------|----------|---------|
| Family map | `identity_authority/phase_one_family_map.rs` | Names every family, owner authority, `Kind`, allowed category forms, and first Phase 2 compiler frontier. |
| Root break targets | `identity_authority/phase_one_root_break_targets.rs` | Names the constructors, accessors, trait methods, equality paths, receipt/storage fields, and reconstruction APIs Phase 2 must hard-break. |
| Compiler-fail targets | `identity_authority/phase_one_compile_fail_targets.rs` | Names the future trybuild fixtures for every forbidden substitution category. |
| Facade export | crate `facade.rs` through `identity_authority` | Makes the design visible to downstream repair and certification without deep imports. |

Phase 2 must start from those manifests in this order:

1. Pick the first upstream root-break target.
2. Install the authority-category restriction without fixing callers.
3. Run `cargo check --workspace`.
4. Record failures in the compiler ledger using the family map and compile-fail
   target paths.
5. Stop at the first dependency frontier instead of weakening the break.

If a Phase 2 failure cannot be classified by the family map, the correct action
is to extend the Phase 1 family map and ledger classification, not to guess at
the call site.

**Common mistakes**

- Treating string search as complete discovery before the compiler is allowed
  to break.
- Adding typed accessors beside lower-authority string accessors.
- Designing marker kinds so broadly that one identity family can satisfy another.
- Letting reporting accessors keep authoritative names like `identity()`,
  `reference_identity()`, or `digest()` when they are terminal projections.

---

### Phase 2 â€” compiler-discovery break (workspace RED)

**What this phase is:** install the root authority-category break designed in
Phase 1, run `cargo check --workspace`, and record the first compiler failure
inventory as the authoritative discovery ledger.

**What this phase is not:**
- Not fixing downstream call sites.
- Not trying to make the workspace green.
- Not adding typed replacements beside lower-authority entrypoints.
- Not limiting the break to two facade surfaces if Phase 1 identified deeper
  root authority APIs.
- Not using error count as progress; the first failure inventory is a map, not
  a scoreboard.

**Root hard breaks**

| Break area | Required action |
|------------|-----------------|
| Constructors | Delete, privatize, or witness-gate constructors that admit raw `String`, `Arc<str>`, `&str`, external authority text, projections, or digests into authority types. |
| Accessors | Remove or rename authority-looking `as_str()`, `reference_identity()`, `digest()`, and `*_identity()` accessors when they expose projection/reporting text. |
| Traits and receipts | Replace raw identity fields and adapter trait returns with category-specific authority/evidence types. |
| Equality and coherence | Prevent comparison through reporting strings, digest labels, or formatted identities. |
| Bridge retained evidence | Block reconstruction from text or `from_external_authority` outside owner-controlled admission. |
| Display and `AsRef<str>` | Remove implementations from authority types when they allow lower-authority APIs to recover authority by formatting. |

**Compiler Failure Ledger (required artifact)**

After root breaks land, run `cargo check --workspace 2>&1` and write the first
inventory to `plans/worth-query/bridge_truth_identity_exposure_report.md`. Each
failure row must record:

| Field | Meaning |
|-------|---------|
| Failure id | Stable id used by the milestone doc and follow-up PRs. |
| Compiler error | Error code or diagnostic class, plus the shortest useful path/function. |
| Broken API | The root API whose authority contract rejected the call. |
| Attempted category | Projection, digest evidence, external token, bridged authority, raw representation, wrong kind, or unknown. |
| Required category | Current owner-admitted authority or explicit owner revalidation. |
| Owning phase | Phase 3 through Phase 9 slice that must fix it. |
| Closure route | Authority-category fix, compile-fail guard, terminal projection quarantine, or named deferred owner milestone. |

**Dependency-frontier rule**

If an upstream break prevents later crates from compiling far enough to reveal
their own failures, keep the workspace red and record the frontier explicitly.
Then fix top-down until the next frontier is exposed. Do not weaken the root
break so more downstream code can compile.

**Trybuild suites to add or update**

- `worth-runtime-bridge`: bridge identity constructor/accessor and
  evidence-category failures.
- `worth-query`: receipt/intake/storage/feeders reject projection, digest,
  external-token, bridged, raw, and wrong-kind substitutes.
- Downstream/harness: old receipt literals, raw identity strings, and reporting
  accessor inputs fail.

**Done when (Frontier A only â€” Phase 2A slice)**

- [x] Truth-routing root APIs reject lower-authority categories at compile time.
- [x] `cargo check --workspace` was red for truth-routing authority reasons after
  the first root break.
- [x] The first compiler failure ledger is written and classified.
- [x] Every ledger row has an owning phase and closure route.
- [x] Bridge/relational/signal compile-fail fixtures exist for the Phase 2A
  dependency frontier.
- [x] No call-site fix was used to make the Phase 2A break look smaller.

**Phase 2B red frontier opened (2026-06-15):** Query evidence composition roots
are now hard-broken instead of deferred. The first red pass is captured in
`plans/worth-query/frontier2_query_root_break_cargo_check.txt`.

Phase 2B root cuts installed:

- `WORTHQueryEvidenceIdentityEncoder::field_identity(impl AsRef<str>)` removed.
- `WORTHQueryEvidenceIdentityEncoder::field_identity_sequence(...)` removed.
- `WORTHQueryEvidenceIdentityEncoder::field_bridge_identity(...)` removed.
- `WORTHQueryEvidenceIdentityEncoder::optional_identity(...)` removed.
- `WORTHQueryEvidenceIdentity::as_str()` made crate-private.
- `WORTHQueryEvidenceIdentity::compose(...)` made crate-private.
- `WORTHQueryEvidenceIdentity::bridge_evidence_identity()` made crate-private.
- `WORTHQueryEvidenceIdentity::bridge_external_identity_evidence()` made
  crate-private.
- `BridgeIdentityEvidence::as_str()` made crate-private.
- `BridgeIdentity::<Tag>::evidence_identity()` made crate-private.
- Bridge-domain wrapper `evidence_identity()` factories that synthesized
  `BridgeIdentityEvidence` from stored wrapper text were removed.
- `crate::identity::hash_parts` re-export removed.
- `Canonical*Digest::as_str()` accessors made crate-private.
- `WORTHQueryEvidenceIdentity` no longer implements `Display`.
- `WORTHQueryEvidenceIdentity` no longer implements `AsRef<str>`.
- `crate::identity::hash_parts` privatized behind the digest module's own
  constructors.
- Query authority compile-fail fixture paths in
  `tests/ui/query_identity_authority/*` now exist and are wired by
  `phase_boundaries_query_identity_authority_compile_fail.rs`.

First Phase 2B compiler inventory:

| Error class | Count | Broken root | Meaning |
|-------------|-------|-------------|---------|
| `E0599` missing `field_identity` | 104 | Encoder raw identity sink | Call site attempted to compose identity material from string-like input. |
| `E0432` / `E0425` unresolved `hash_parts` | 206 | Low-level digest primitive | Module attempted to compose digest/evidence material through string hash folklore. |
| `E0624` private `BridgeIdentity::<Tag>::evidence_identity` | 65 | Bridge truth-to-evidence downgrade | Call site attempted to turn current bridge truth identity into evidence without an owning crossing. |
| `E0599` missing `field_bridge_identity` | 36 | Encoder bridge projection sink | Call site attempted to flatten bridge evidence through terminal projection at compose time. |
| `E0599` missing `field_identity_sequence` | 19 | Encoder raw identity sequence sink | Call site attempted identity-sequence composition without typed evidence handles. |
| `E0277` missing `Display` | 22 | Evidence identity projection | Call site attempted formatting/to-string projection of evidence identity. |
| `E0599` missing `as_ref` | 24 | Evidence identity string coercion | Call site attempted to pass evidence identity through an `AsRef<str>` lane. |
| `E0624` private `BridgeIdentityEvidence::as_str` | 12 | Bridge evidence raw accessor | Call site attempted raw bridge evidence projection outside a terminal reporting edge. |
| `E0599` missing bridge wrapper `evidence_identity` | 8 | Bridge wrapper text-to-evidence factory | Call site attempted to synthesize bridge evidence from wrapper text. |
| `E0277` missing `AsRef<str>` bound | 3 | Evidence identity string coercion | Generic string-like API attempted to accept `WORTHQueryEvidenceIdentity`. |

The refreshed frontier contains 508 Query evidence/feeder root-break errors
across 312 unique reported source paths. Phase 2B is intentionally red. The
next implementation slice must fix failures top-down by category and must not
restore these removed roots.

**Phase 2 -> Phase 3 handoff**

Phase 3 begins at the first upstream authority frontier. If relational lawful
mint is red, fix relational first. If bridge carrier admission is red, fix the
bridge before Query. If Query cannot compile far enough for its trybuild suites,
that is a frontier note, not permission to restore lower-authority APIs.

**Phase 2A upstream boundary verification gate**

Phase 2A is mandatory after the first bridge-local red cut and before any Query
repair slice. It prevents the milestone from treating the bridge as the first
source of truth when relational and signal boundaries can still leak lower
authority into bridge/query authority APIs.

Relational gate:

- `worth-relational/src/identity_authority/` must classify every source-truth
  family used by bridge presentation: commit, snapshot, version, entity,
  relation, branch, workspace, and bridge-presentation export.
- `presentation/bridge/identities.rs` parse helpers are allowed only if they
  recover native IDs from typed bridge payload extractors such as
  `relational_commit_id()` and `relational_snapshot_parts()`. They must not
  parse display text.
- `record_ref_identity` and `record_ref_from_identity_parts` must remain the
  authority path for entity/relation records. `entity:*` and `relation:*`
  labels are diagnostics only.
- Grouped truth row and projection digests are artifact evidence. Request keys,
  row labels, and grouped member strings must never re-enter relational,
  bridge, or Query authority.

Signal gate:

- `worth-signal` does not currently use the same
  `FoundationalAuthorityIdentity` category scaffold as relational/bridge/query.
  Its protection comes from `worth_proof::Artifact` authority bases, typed
  branch IDs, scoped proof packets, and domain tokens.
- `SignalBranchBasisIdentity`, `SignalBranchBasisArtifact`, and
  `BoundaryBridgedSignalBranchBasisArtifact` are a signal authority lane only.
  Their basis/component digests are proof/reporting evidence, not bridge/query
  identity authority.
- `SignalMergeCompatibilityBasis`, `SignalMergeCompatibilityWitness`, and
  `ScopedMergeProofPacket` carry proof digests. Those digests may certify signal
  compatibility, but must not become Query, bridge, relational, or downstream
  identity authority.
- `OutputIdentity`, `ArtifactContinuityToken`, `PartitionToken`, and
  identity-matcher names are host/domain tokens for signal correspondence and
  partitioning. They are not relational entity/relation identity and are not
  bridge/query truth identity.

Runtime bridge membrane gate:

- `BridgeIdentityEvidence::from_external_authority(...)` must require
  `BridgeTruthExternalIdentityToken`; raw text, projection identity, digest
  evidence, or retained lookup label text cannot satisfy it.
- `BridgeIdentityEvidence::from_query_evidence_identity(...)` must require the
  explicit projection + digest-evidence pair; external tokens and raw strings
  cannot rebuild bridge evidence.
- Boundary-bridged retained evidence requires bridge-owner revalidation. Public
  consumers cannot call retained-reference revalidation or treat a bridged value
  as current bridge truth authority.
- Wrong bridge identity families remain separated by marker kind. Commit,
  branch, patch, snapshot, and evidence-reference families cannot substitute for
  each other to satisfy authority APIs.
- `TruthCommitIdentity`, `TruthPatchIdentity`, `TruthBranchIdentity`, and
  `TruthSnapshotIdentity` must not expose `Display`, `AsRef<str>`, public
  `as_str()`, or raw string equality.

Phase 2A closes only when the compiler ledger names the current upstream status:
relational typed and sealed, relational red and owned by Phase 3, signal
proof-lane quarantined, signal category scaffold required, runtime-bridge
membrane compiler-gated, or a named deferred owner milestone. Do not proceed to
Query while this is ambiguous.

Phase 2A current result:

- `worth-relational` is typed and sealed for the upstream bridge-presentation
  string re-entry cases exercised so far. Compile-fail guards prove raw
  `commit-*` text, raw bridge snapshot text, raw `entity:*`/`relation:*`
  labels, and grouped row labels cannot satisfy bridge truth/record authority.
- `worth-signal` is quarantined as a signal proof/domain-token lane for the
  upstream cases exercised so far. Compile-fail guards prove
  `OutputIdentity`, `PartitionToken`, branch-basis digest text, and scoped
  merge proof digest text cannot satisfy the public signal basis/proof
  authority APIs.
- `worth-runtime-bridge` membrane identity categories are compiler-gated for
  the current public bridge truth surface. Compile-fail guards prove projection
  identity, digest evidence, external token, boundary-retained revalidation,
  wrong marker kind, raw text, retained text rebuild, and string facade access
  cannot satisfy bridge truth/evidence authority APIs.
- Verification artifacts:
  `plans/worth-query/bridge_truth_identity_phase2a_relational_trybuild_verify.txt`,
  `plans/worth-query/bridge_truth_identity_phase2a_signal_trybuild_verify.txt`,
  `plans/worth-query/bridge_truth_identity_phase2a_relational_lib_check.txt`,
  `plans/worth-query/bridge_truth_identity_phase2a_signal_lib_check.txt`,
  `plans/worth-query/upstream_foundation_gate_runtime_bridge_trybuild_verify.txt`,
  `plans/worth-query/upstream_foundation_gate_runtime_bridge_digest.txt`,
  `plans/worth-query/upstream_foundation_gate_runtime_bridge_lib.txt`,
  `plans/worth-query/upstream_foundation_gate_relational_trybuild.txt`,
  `plans/worth-query/upstream_foundation_gate_relational_lib.txt`,
  `plans/worth-query/upstream_foundation_gate_signal_trybuild.txt`, and
  `plans/worth-query/upstream_foundation_gate_signal_lib.txt`.
- Phase 2A does not add a `worth-signal` `FoundationalAuthorityIdentity`
  scaffold yet. The current scaffold decision is: no new generic signal
  identity scaffold until a public signal authority API needs admission across
  WORTH identity-family boundaries. Today the public boundary is already
  enforced by proof artifacts and typed domain tokens.

**Common mistakes**

- Treating `cargo check` as an end-of-phase test instead of the discovery tool.
- Restoring `Display`, `AsRef<str>`, or `as_str()` so downstream code compiles.
- Marking old matrix rows fixed without a compiler failure, compile-fail guard,
  projection quarantine, or named deferral.
- Letting `WORTHQueryEvidenceIdentity` become authority merely because it
  contains stable evidence text.

### Phase 2B â€” evidence/feeder root break (compiler-discovery frontier 2)

**What this phase is:** install the **true parent** authority-category breaks
that Phase 2 skipped. Frontier A closed truth routing (receipts, bridge
`Truth*`, relational export). Frontier Bâ€“F must close the **Query evidence
composition substrate** and mandatory `worth-foundational` category admission so
the compiler discovers the full feeder graph â€” not grep, not matrix rows, not
bundle-local fixes.

**What this phase is not:**

- Not fixing individual feeder files (`subscription/support/report.rs`,
  `causal/.../row_digest/artifact.rs`, etc.) before the parent APIs break.
- Not adding typed fields beside `*_for_reporting: String` dual lanes.
- Not marking Phase 7 or the milestone closed when only Frontier A is green.
- Not another incremental `field_evidence_identity` migration while
  `field_identity(AsRef<str>)` remains public.

**Why Phase 2B exists**

Phase 2 broke **Tier 1â€“2 + part of Tier 5** (truth routing). Feeders still
compile because **Tier 3** (evidence encoder + evidence string surface) was never
cut. `QueryAuthorityIdentity` wrappers exist but are opt-in (~5 call sites).
`tests/ui/query_identity_authority/*` is named in
`identity_authority/phase_one_compile_fail_targets.rs` but **does not exist on
disk**. Hostile QA therefore keeps finding projection re-entry while
`cargo check --workspace` stays green.

#### Authority dependency tiers (parent vs child)

Break **parents** first. Breaking a **child** fixes three files and leaves the
graph alive elsewhere.

| Tier | Layer | Frontier | Break status |
|------|-------|----------|--------------|
| 0 | `worth-foundational` admission / projection / digest derive | substrate | Types exist; not **required** at Query roots |
| 1 | Relational source-truth mint â†’ bridge export | A | Closed |
| 2 | Bridge `BridgeIdentity` / `BridgeIdentityEvidence` | A + D | Partial â€” see open roots below |
| 3 | **Query evidence composition** (`WORTHQueryEvidenceIdentity` encoder) | **B** | **Not broken â€” primary parent** |
| 4 | Query truth IDs (`WORTHQueryCommitIdentity`, snapshot, entity) | C | Partial â€” external-label + round-trip open |
| 5 | Runtime intake traits + typed receipts | A | Largely closed |
| 6 | Feeder structs, certification `hash_parts`, `*_for_reporting: String` | children | Do **not** break first; must catch fire from Tier 3 |

**Parent test:** Does this API accept both authority and projection through the
same type (usually `impl AsRef<str>` or `&str`)? If yes, it is a root or
near-root. If breaking it produces fewer than ~20 unique error **files** in
`worth-query`, you broke a child â€” move up a tier.

**Anti-patterns (child vs parent)**

| Break target | Verdict |
|--------------|---------|
| One feeder's struct fields or one certification helper | Child â€” insufficient |
| `WORTHQueryEvidenceIdentityEncoder::field_identity(AsRef<str>)` globally | **Parent** â€” expect 50â€“150+ error files |
| `WORTHQueryEvidenceIdentity::as_str` / `AsRef<str>` globally | **Parent** |
| Pattern `*_for_reporting() -> &str` replaced with `QueryProjectionIdentity` | **Parent pattern** |
| Single bundle's cached `String` field removed | Child unless parent already broke |

#### Frontier sequence (do not skip)

```
Frontier A  relational + bridge Truth* + query receipt string fields     [CLOSED]
Frontier B  Query evidence encoder + EvidenceIdentity string surface     [NEXT]
Frontier C  Query truth ID witness admission; remove external-label mint
Frontier D  BridgeIdentityEvidence public surface cleanup
Frontier E  hash_parts / Canonical*Digest privatization (certification wave)
Frontier F  Land tests/ui/query_identity_authority/* trybuild + wire CI
Frontier G  Downstream/harness (server, worth-topo fixtures)             [last]
```

**Dependency rule:** If breaking Frontier B does not turn
`cargo check -p worth-query` red across subscription, causal, intent,
lower_runtime, domain_capabilities, and effect_lifecycle **in the same check**,
Frontier B is not broken hard enough. Do not weaken the break.

#### Phase 1 manifest extension (required before first 2B cut)

Extend `crates/worth-query/src/identity_authority/phase_one_root_break_targets.rs`
with **concrete symbol paths**. The existing vague row
`receipt/intake/storage/feeders identity fields` is insufficient. Add at minimum:

**Tier 3 â€” Query evidence composition (Frontier B)**

| API | Required restriction |
|-----|----------------------|
| `WORTHQueryEvidenceIdentityEncoder::field_identity` | Remove public `impl AsRef<str>` ingress; identity slots accept category types only |
| `WORTHQueryEvidenceIdentityEncoder::field_identity_sequence` | Same |
| `WORTHQueryEvidenceIdentityEncoder::optional_identity` / `optional_evidence_identity` on `&str` | Same |
| `WORTHQueryEvidenceIdentityEncoder::field_evidence_identity` | Must not delegate to string ingress; require `&WORTHQueryEvidenceIdentity` or admitted authority handle without `AsRef<str>` collapse |
| `WORTHQueryEvidenceIdentityEncoder::field_bridge_identity` | Must not flatten to `terminal_projection_for_reporting()` at compose time |
| `WORTHQueryEvidenceIdentity::compose` | Witness-gate or restrict to owner admission modules |
| `WORTHQueryEvidenceIdentity::as_str` | Terminal projection module only |
| `impl AsRef<str> for WORTHQueryEvidenceIdentity` | Remove |
| `WORTHQueryEvidenceIdentity::bridge_evidence_identity` | Remove or owner-gate â€” no rebuild from digest token text |
| `WORTHQueryEvidenceIdentity::bridge_external_identity_evidence` | External-token path only with explicit admission |

**Tier 4 â€” Query truth IDs (Frontier C)**

| API | Required restriction |
|-----|----------------------|
| `WORTHQueryCommitIdentity::from_external_authority_label` | Remove; external labels â†’ `QueryExternalIdentityToken` â†’ witness admission |
| `WORTHQuerySnapshotIdentity::from_external_authority_label` | Same |
| `WORTHQueryEntityIdentity::authored_command` | Same |
| `WORTHQuery*Identity::preview(WORTHQueryEvidenceIdentity)` | Require admitted evidence / owner witness |
| `WORTHQuery*Identity::evidence_identity` | Must not feed compose/routing/admission; terminal export or digest evidence only |
| `impl Display for WORTHQuery*Identity` | Remove or opaque debug only |
| `WORTHQueryEntityIdentity: Ord/Hash` via `as_str()` | Compare typed authority, not projection text |

**Tier 2 â€” Bridge evidence (Frontier D)** â€” extend bridge manifest if not closed

| API | Required restriction |
|-----|----------------------|
| `BridgeIdentityEvidence::from_external_authority` | Public only for `BridgeTruthExternalIdentityToken`; raw/projection/digest text cannot enter |
| `BridgeIdentityEvidence::as_str` | Terminal reporting module only |
| `BridgeIdentityEvidence::is_empty` | Crate-private; evidence must not expose public string-like predicates |
| `BridgeIdentityEvidence::from_query_evidence_identity` | Keep category pair; block raw/substitute inputs |
| `BridgeIdentity::<Tag>::evidence_identity` | Must not downgrade truth to external-authority evidence |

**Tier 3 mid â€” Digest folklore (Frontier E)**

| API | Required restriction |
|-----|----------------------|
| `identity::digest::hash_parts` | `pub(crate)` to terminal digest / certification modules only |
| `Canonical*Digest::from_parts` | Same |
| `Canonical*Digest::as_str` public | Terminal projection only |
| `Canonical*Digest::evidence_identity` rebuilding from `self.as_str()` | Remove self-referential compose |

**Tier 6 pattern â€” reporting storage (follows B)**

| Pattern | Required restriction |
|---------|----------------------|
| `fn *_for_reporting(&self) -> &str` on authority-bearing artifacts | Return `QueryProjectionIdentity<â€¦>` or crate-private terminal only |
| struct fields `*_for_reporting: String` | Remove; no cached projection beside typed authority |

**Mandatory admission (Frontier B/C)**

Feeder bundle boundaries listed in `phase_one_family_map.rs` must **store**
`Query*AuthorityIdentity<â€¦>` (or bridged/external token types), not bare
`WORTHQueryEvidenceIdentity`, at:

- subscription lifecycle certification bundle authority
- causal inspection certification bundle authority
- lower-runtime basis binding authority (partially landed â€” extend pattern)
- every family with owner `QueryFeederAuthority`, `QuerySubscriptionAuthority`,
  `QueryCausalInspectionAuthority`, `QueryWorkflowAuthority`,
  `QueryDomainCapabilityAuthority`, `QueryEffectLifecycleAuthority`

#### Trybuild suite (Frontier F â€” land with or immediately after Frontier B)

Create every path named in
`identity_authority/phase_one_compile_fail_targets.rs`:

| Fixture path | Forbidden substitution |
|--------------|------------------------|
| `tests/ui/query_identity_authority/projection_cannot_satisfy_query_authority.rs` | projection â†’ authority |
| `tests/ui/query_identity_authority/digest_cannot_satisfy_query_authority.rs` | digest evidence â†’ authority |
| `tests/ui/query_identity_authority/external_token_cannot_satisfy_query_authority.rs` | external token â†’ authority |
| `tests/ui/query_identity_authority/bridged_cannot_satisfy_current_query_authority.rs` | bridged â†’ current without readmission |
| `tests/ui/query_identity_authority/wrong_kind_cannot_satisfy_query_family.rs` | wrong `Kind` marker |
| `tests/ui/query_identity_authority/raw_text_cannot_satisfy_query_authority.rs` | `&str` / `String` â†’ authority |
| `tests/ui/query_identity_authority/reporting_accessor_cannot_feed_query_authority.rs` | `*_for_reporting()` â†’ compose/admission |

Wire a dedicated test runner (mirror bridge):
`tests/phase_boundaries_query_identity_authority_compile_fail.rs` â†’
`tests/ui/query_identity_authority/*.rs`.

#### Phase 2B implementation slices (agent order)

**Slice 2B-0 â€” manifests only (no call-site fixes)**

1. Extend `phase_one_root_break_targets.rs` with the concrete APIs above.
2. Create all `tests/ui/query_identity_authority/*.rs` fixtures and capture
   `.stderr` expectations once `worth-query` compiles far enough for trybuild
   to execute the fixtures.
3. Add `phase_boundaries_query_identity_authority_compile_fail.rs` (fixtures
   should **pass** compile-fail before the break; they guard restorations).

2B-0 is preparation, not repair. If this slice runs after a root cut has already
made the workspace red, keep the red state and finish the structural prep. Do
not restore removed roots, add compatibility shims, or repair downstream call
sites to reduce the error count.

2B ledger rows use this schema:

```text
id:
compiler_error:
file:
symbol:
broken_root:
frontier:
attempted_category:
required_category:
owning_repair_phase:
closure_route:
blocked_downstream_crates:
```

2B-0 DX target:

```text
A repair agent should be able to run one command, see a categorized root-break
failure, map it to a 2B ledger row, and know whether the fix is typed evidence,
authority admission, bridged readmission, terminal projection quarantine, or
digest evidence derivation.

No agent should need to infer whether a string identity path is allowed. If a
text path is allowed, the API name must say value, reporting, projection, or
terminal. It must not say identity or authority.
```

2B-0 verification is structural while the workspace is intentionally red:

```powershell
cargo fmt --all
rg -n "WORTHQueryEvidenceIdentityEncoder::field_identity|WORTHQueryEvidenceIdentity::as_str|hash_parts|BridgeIdentityEvidence::from_external_authority" crates/worth-query/src/identity_authority/phase_one_root_break_targets.rs
rg --files crates/worth-query/tests/ui/query_identity_authority
rg -n "worth_query_identity_phase_one_compile_fail_targets" crates/worth-query/tests/phase_boundaries_query_identity_authority_compile_fail.rs
cargo test -p worth-query --test phase_boundaries_query_identity_authority_compile_fail --no-run
```

The `cargo test --no-run` command may fail while `worth-query` is red from 2B
root cuts. That failure is a frontier note, not permission to restore removed
roots.

2B-0 QA note: the fixture sources and runner exist, but `.stderr` expectation
capture is blocked while the crate fails before trybuild execution. Do not fake
these files from stale or synthetic diagnostics; capture them after the current
Query frontier compiles far enough to run the trybuild harness.

**Slice 2B-1 â€” Frontier B hard break (workspace RED; no fixes)**

1. Privatize or remove `WORTHQueryEvidenceIdentityEncoder::field_identity` and
   string-based `optional_identity` / `field_identity_sequence`.
2. Split replacement APIs, e.g.:
   - `field_authority_identity` â†’ `&WORTHQueryEvidenceIdentity` or
     `&QueryAuthorityIdentity<â€¦>` only
   - `field_projection_identity` â†’ `QueryProjectionIdentity<â€¦>` (terminal
     modules only)
   - `field_digest_evidence` â†’ `QueryDigestIdentityEvidence<â€¦>`
3. Remove public `WORTHQueryEvidenceIdentity::as_str` and `impl AsRef<str>`.
4. Run `cargo check -p worth-query 2>&1` â†’ record **full** ledger in
   `bridge_truth_identity_exposure_report.md` with new failure ids prefixed
   `2B-`.
5. **Stop.** Do not fix call sites in the same change.

**Slice 2B-2 â€” Frontier C** (after 2B-1 ledger exists)

Break truth-ID external-label constructors and round-trip
`evidence_identity()` â†’ compose paths. Re-run check; extend ledger.

**Slice 2B-3 â€” Frontier D** (bridge crate)

Align bridge manifest open roots; re-run `cargo check --workspace`.

**Slice 2B-4 â€” Frontier E**

Privatize `hash_parts` and `Canonical*Digest::from_parts`; certification modules
go red last â€” expected.

**Slices 2B-5+ â€” top-down repair (Phases 3â€“7 content, compiler-led)**

Fix ledger rows top-down: truth ID admission â†’ basis binding â†’ subscription â†’
causal â†’ workflow/domain_capabilities â†’ effect_lifecycle â†’ certification harness.
Each fix must include compile-fail guard or typed replacement; no dual APIs.

#### Phase 2B-1 / 2B-3 / 2B-4 status

- [x] Frontier B roots are cut and remain removed/restricted.
- [x] Frontier D bridge evidence surface roots are cut and remain
  removed/restricted.
- [x] Frontier E digest folklore roots are cut and remain removed/restricted.
- [x] `cargo check -p worth-query --lib` is red at the Query evidence/feeder
  frontier with QA-expanded digest converter discovery.
- [x] `cargo check --workspace` is red at the same `worth-query` frontier.
- [x] Raw transcripts are captured in
  `plans/worth-query/frontier2_query_root_break_cargo_check.txt` and
  `plans/worth-query/frontier2_workspace_root_break_cargo_check.txt`.
- [x] `2B-B-*` ledger rows are recorded in
  `plans/worth-query/bridge_truth_identity_exposure_report.md`.
- [x] QA-expanded root cuts removed the remaining bridge projection flattening
  root and restricted public evidence composition/export to crate scope.
- [x] QA-expanded root cuts removed digest-wrapper `evidence_identity()`
  converters so digest evidence cannot re-enter Query composition through a
  crate-wide helper.
- [x] QA-expanded root cuts made `BridgeIdentityEvidence::is_empty()`
  crate-private so public bridge evidence does not retain string-like facade
  predicates.
- [x] Bridge compile-fail coverage guards both public bridge evidence text
  projection and public bridge evidence emptiness predicates.
- [x] Bridge compile-fail coverage includes public `BridgeIdentityEvidence`
  string-facade rejection.
- [x] 2B-5 repaired the first workflow/domain-preview cluster with typed
  bridge boundary witnesses and owner-specific digest evidence helpers.
- [x] After 2B-5, `cargo check -p worth-query --lib` remains intentionally red
  with 504 errors; the next exposed frontier is view-shape/effect/runtime
  bridge composition, not the repaired workflow/domain-preview cluster.
- [x] 2B-5 QA tightened `bridge_trust_boundary()` to preserve tag-specific
  marker kinds for the repaired workflow/domain-preview and writeback bridge
  families; future bridge-family slices must add their own marker kinds before
  they may use this boundary export.
- [x] No downstream call-site repairs or compatibility shims are part of
  2B-1/2B-3/2B-4.

#### Frontier discovery protocol (do not miss a parent)

After each break slice:

1. Run `cargo check -p worth-query 2>&1` (or `--workspace` when unblocked).
2. Record **unique error file count**. Parent break: expect **â‰¥ 50 files** in
   `worth-query` for Frontier B. If &lt; 20, the break was too shallow.
3. For each failure, ledger: `broken_api`, `tier`, `attempted_category`,
   `required_category`, `owning_phase`, `blocked_crates`.
4. If crate X cannot compile far enough to show its errors, log
   `blocked_crates: [X]` and fix upstream first â€” **do not** restore APIs.
5. Grep is **verification only after break**, not the discovery tool. Root
   definitions such as public `field_identity(AsRef<str>)` must be zero; red
   call sites may remain only as compiler-led ledger failures until their owning
   repair slice closes them.

#### Phase 2B done when

- [x] `phase_one_root_break_targets.rs` lists every concrete API in this section.
- [x] All `tests/ui/query_identity_authority/*` fixtures exist and run in CI.
- [x] Frontier B break landed; `cargo check -p worth-query` was red for category
  reasons (not missing imports from unrelated work).
- [x] Compiler ledger `2B-*` rows cover the full red graph with owning phases.
- [x] Frontier Câ€“E breaks landed (C: external-token truth ID admission; D/E:
  bridge + digest folklore roots cut and repaired in `worth-query`).
- [x] No public `field_identity(AsRef<str>)`, no public
  `WORTHQueryEvidenceIdentity: AsRef<str>`, no dual `String` + typed authority
  storage pattern in new code.
- [x] Repair slices 2B-5+ green `worth-query --lib` and `--lib --tests` under
  category types only (downstream workspace crates explicitly deferred).
- [x] Phase 7 QA re-run returns `CLEARED` under Phase 7 hard bar **after** 2B
  repair (2026-06-15 post-2B pass; see compiler ledger header).

#### Phase 2B agent prompt (copy verbatim)

```text
Milestone 9.6 Phase 2B only. Read Mechanical Breakage Rules and the Phase 2B
section in plans/worth-query/milestone-9.6-bridge-truth-identity-lowering.md.

You are installing evidence/feeder ROOT breaks, not fixing feeder bundles.

Order:
1. Extend identity_authority/phase_one_root_break_targets.rs (concrete APIs).
2. Create tests/ui/query_identity_authority/* + phase_boundaries test wiring.
3. Frontier B: break WORTHQueryEvidenceIdentity encoder string ingress and
   public as_str/AsRef<str>. Do NOT fix call sites in the same commit.
4. cargo check -p worth-query; write 2B-* rows to bridge_truth_identity_exposure_report.md.
5. Stop when red. Next slice fixes ledger top-down.

Banned: fixing subscription/causal/domain_capabilities files before step 3;
adding typed fields beside _for_reporting: String; restoring APIs to green
downstream crates; marking Phase 2B done with <50 error files in worth-query.
```

### Phases 3-9 â€” top-down authority repair

Per phase: start at the first compiler frontier, replace the lower-authority
path with an authority-category type, then run the narrowest meaningful crate
checks before moving downstream. The repair order follows source truth, not the
shortest path to a green workspace.

**Required every fix phase:** update the compiler ledger and historical matrix
rows owned by that phase in the same PR. A row closes only through an
authority-category fix, compile-fail guard, terminal projection quarantine, or
named deferred owner milestone.

**Forbidden every fix phase:** incremental refactoring, parity tests, round-trip
string comparisons, keeping old behavior behind `cfg`, "prove identical envelopes
across repeated export," or landing ergonomic helpers that hide the authority
witness.

**Phase 3** â€” `worth-relational/src/presentation/bridge/`, `facade.rs`;
source-truth admission and owner witness production

**Phase 4** â€” `worth-runtime-bridge/src/identity.rs`, facade exports; bridge
carriers preserve authority category and expose only quarantined projections

**Phase 5** â€” `contracts.rs`, runtime backends, source adapters, and harness
adapters under `worth-query`; Query intake must reject projection/digest/external
tokens unless owner revalidation occurs

- [x] Query bridge-truth compile-fail lane executes and passes once the crate can
  compile far enough to run trybuild.
- [x] Bridge-backed runtime assembly requires a typed current-snapshot authority
  (`WORTHQueryRuntimeSnapshotIdentityAdapter`) instead of relying on erased source
  adapter tokens or a silent unavailable snapshot basis.
- [x] Preview stale-basis proof exercises backend-owned snapshot authority, not
  the removed `WORTHQueryRuntimeSourceAdapter::snapshot_token()` folklore path.
- [x] Signal routing fail-closed proof exercises the ordinary
  bridge-backed `write -> signal_sink.route_write_receipt` path with an
  authority-less mutation receipt.

**Phase 6** â€” `memory_workspace/`, write receipt surfaces, read receipts,
write surfaces, and `shared_read_pins/`; receipts store authority/evidence
categories, not text that later milestones must reinterpret

**Phase 7** â€” projection re-entry purge across evidence `from_bridge`,
intent/receipt/inspection surfaces, runtime/backend receipts, causal
inspection/materialization, live subscription/runtime session feeders,
workflow/domain-capability/effect-lifecycle feeders, and bridge causal
retained-mapping feeders

Phase 7 is not "more strings." It is projection re-entry: evidence, digest,
reporting, and retained bridge text have been able to re-enter composition,
lookup, admission, routing, comparison, and coherence checks. It is closed only
when every ordinary covered surface and every same-class upstream or adjacent
feeder that can mint, lower, inspect, retain, route, or report bridge
truth/query evidence uses the authority-category boundary. This includes
`subscription/`, `workflow/`,
`domain_capabilities/`, `effect_lifecycle/`, `runtime/inspection/causal/`,
`runtime/backend/receipts.rs`, `runtime/live_subscription.rs`,
`runtime/runtime_sessions.rs`, and `worth-runtime-bridge` causal-envelope
retained mapping/receipt feeders. No Phase 7 work may be certified complete by
row-scoped scans alone.

Phase 7 acceptance criteria:

- No authoritative Phase 7 production surface stores bridge truth/query evidence
  as `String`, `Arc<str>`, or `&str` while also treating it as identity.
- `WORTHQueryEvidenceIdentity` is evidence unless explicitly admitted by the
  owning authority; it does not become authority because its representation is
  stable.
- Digest/reporting accessors are allowed only as explicitly named projections
  such as `*_for_reporting`, never as the internal source of composition,
  lookup, admission, routing, comparison, or coherence.
- Runtime/session/live-subscription/workflow/domain-capability/effect-lifecycle
  feeders must pass typed `WORTHQueryEvidenceIdentity`,
  `WORTHQuerySnapshotIdentity`, `BridgeIdentityEvidence`, or domain-specific
  typed artifact handles into downstream constructors instead of re-wrapping
  display strings.
- Bridge causal retained mapping must compose retained record evidence through
  typed retained-mapping evidence parts, not `hash_parts(...)` or external
  authority strings disguised as typed identity.
- Bridge evidence must not be reconstructed from `as_str()`,
  `from_external_authority`, external authority strings, or Query reporting
  labels outside owner-controlled admission.
- Compile-fail gates reject projection, digest, external token, bridged identity,
  wrong kind, and raw text substitutions for Phase 7 authority APIs.
- Cursor Phase 7 QA (see **Phase 7 QA gate** below) must return `CLEARED` before
  Phase 7 is done. Phase 8 milestone work may not begin until Phase 7 QA is
  `CLEARED`, even if Phase 8 path rows are already code-complete.

**Phase 7 feeder bundles** (fix order within Phase 7; may run in parallel if paths
do not overlap):

| Bundle row | Primary paths |
|------------|---------------|
| subscription / live / sessions | `runtime/live_subscription.rs`, `runtime/backend/receipts.rs` (`SubscriptionActivationReceipt`), `runtime/runtime_sessions.rs`, `subscription/` |
| workflow / domain_capabilities | `workflow/lowering/writeback.rs`, `domain_capabilities/canonical_runtime/workflow/`, `domain_capabilities/payloads/workflow_semantics.rs`, `domain_capabilities/authoring/workflow.rs` |
| canonical_runtime artifacts | `domain_capabilities/canonical_runtime/continuity.rs`, `support.rs`, `artifacts.rs`, `invariant_capability.rs` |
| effect_lifecycle | `effect_lifecycle/` (normalization â†’ lowering â†’ `execution_bridge.rs`) |
| causal inspection (query) | `runtime/inspection/causal/request.rs`, `identity.rs`, `materialization/` |
| bridge causal envelope | `worth-runtime-bridge/src/diagnostics/causal_envelope/evidence_reference.rs`, `binding.rs`, `retained_mapping/digest_basis.rs`, `retained_mapping/retained_artifact_digest/` |

**Phase 7 done when**

- [x] All matrix rows with `Fix Phase = 7` are `Fixed`, including feeder bundles
  572â€“577 (not path rows alone)
- [x] Local gates clean for Phase 7 surfaces (at minimum):
  `cargo check -p worth-query --lib`, `cargo check -p worth-runtime-bridge --lib`,
  targeted tests for causal envelope, lower_runtime, identity_boundary as touched
- [x] Phase 7 QA gate returns **`CLEARED`** (hostile review â€” see below)
- [x] Matrix header records Phase 7 QA date and `CLEARED`

**Phase 8** â€” `worth-topo/.../write_authority.rs`, `bridge_source.rs`,
`TopologyStaticSignalSink` must call `RuntimeBridge::route` with typed commit
identity

Phase 8 path rows in the matrix may show `Fixed` when worth-topo code is
landed, but **Phase 8 is milestone-blocked** until Phase 7 QA is `CLEARED`.
Do not treat Phase 8 as complete for sequencing or closeout until then.

**Phase 9** â€” `hostile_certification*`, causal support; zero
`commit_identity().rsplit('-').parse()`; downstream crates and harnesses that
consume the typed receipt/query facade must compile without string shims before
Phase 10 closeout.

### End-to-End Trace Map â€” Phase 7 Through Phase 9

Use this trace map before changing row status. The compiler failure ledger is
the authoritative discovery record; these lanes keep the end-to-end pipeline
visible so fixes do not stop at the first local compiler error.

**Trace classification rules**

- `Phase 7 blocker`: an ordinary `worth-query` or `worth-runtime-bridge`
  authority path composes, routes, compares, recovers, mints, or admits from a
  string/digest/reporting projection instead of typed evidence.
- `Phase 8 blocker`: production `worth-topo` consumes Query or bridge identity
  through string commit/snapshot/entity/route authority instead of typed
  carriers.
- `Phase 9 blocker`: harness, certification, compat, or downstream code still
  teaches or depends on old string authority. Legitimate HTTP/JSON/display text
  is allowed only as terminal compatibility projection.
- `Allowed projection`: explicit `*_for_reporting()` or compatibility display
  output backed by typed internal fields and never fed back into authority.

| Lane | Phase | Authoritative roots | Pipeline to trace | Current blockers / watchpoints |
|------|-------|---------------------|-------------------|--------------------------------|
| Upstream relational and signal roots | 2A/3/7/8/9 | relational bridge snapshot/commit/record parts, grouped truth row identities, signal branch basis identities, signal merge compatibility basis, host output identity tokens | relational runtime IDs -> bridge truth identities -> grouped/read artifacts; signal branch/snapshot/output tokens -> branch basis -> merge compatibility/support -> Query/runtime subscription feeders | Phase 2A gate required before Query. Relational bridge parse helpers currently recover native IDs through typed bridge payload extractors, but this must be compiler-guarded. Signal is not bridge-truth-rooted; signal output/domain tokens and signal proof digests must remain signal proof/domain evidence, never bridge/query truth identity. |
| Declaration identity | 7/8/9 | `WORTHQueryAdmittedWorldBasis`, canonical declaration artifacts, `WORTHQueryDeclarationProgressionPayload`, foundational evidence, route plans | admitted handle/world basis -> checked declaration -> legality -> progression -> foundational evidence -> route plan -> receipt/envelope -> bridge continuation lowering | Phase 8 watchpoint: `application/declaration_evidence/artifact.rs` compares retained-world handle identity through reporting accessors. Keep `application/declaration_bridge_routing/lower_identity.rs` sealed; query truth IDs may be derived only from typed Query evidence, not caller display strings. |
| Workflow binding and preview | 7 | `WorkflowContextBinding`, workflow declaration reports, workflow runtime semantics, preview foundation artifacts, bridge preview session identities | payload authoring -> declaration/admitted-plan target -> request + target binding -> runtime semantics -> workflow binding -> declaration admission -> lowering/preview materialization | Phase 7 fixed in the detailed ledger: workflow/domain checks now use typed declaration, basis, target-binding, and outcome identities. Remaining digest/reporting accessors are terminal projection watchpoints for QA, not open authority rows. |
| Canonical runtime materialization | 7 | domain-capability contribution phase wrappers, target bindings, `WORTHQueryCanonicalRuntimeMaterialization`, typed payload identities | payload + typed target binding -> requested/eligible/admitted contribution -> materialization-ready contribution -> canonical runtime artifact/support/continuity/explanation/invariant evidence | Scoped lane is mostly typed. Watch projection re-entry through `target_digest()`, `binding_digest()`, materialization/report digests, source labels, and payload feeders. Include `continuity_correspondence.rs`, `aftermath.rs`, and `explanation.rs` in future QA. |
| Effect execution | 7/9 | `EffectAuthoringBasis`, workflow binding/declaration, `LoweredEffectExecutionPlan`, `ExecutedEffectPlan`, `EffectExecutionReceipt` | authoring basis -> normalized intent -> eligibility/admission -> authority-scoped plan -> workflow lowering -> relational/bridge execution -> typed receipt -> envelope/oracle/certification | Phase 7 fixed in the detailed ledger: batch admission and effect execution coherence compare typed scoped-basis and lower-runtime binding identities. Phase 9 remains a harness/certification cleanup watchpoint only where rows explicitly remain open. |
| Causal inspection | 7/9 | causal inspection identity wrappers, observation receipts, bridge envelopes/bindings, materialization receipts/proofs | observation receipt -> causal anchor/reference resolution -> request/admission -> bridge envelope -> materialization/readmission proof -> query causal artifact -> exploration/proof/certification projections | Phase 7 watchpoint: remaining `*_digest()` aliases and wrapper `as_str()` accessors can look authoritative beside typed identities. Phase 9 remains open in causal tests/support fixtures. |
| Subscription and session lifecycle | 7/9 | subscription declarations/admissions, activation receipts, active lanes, continuation evidence, live installation/session identities | declaration/admission -> bridge lowering -> basis binding -> signal strategy -> activation -> active lane -> attachment -> delivery/continuation/closeout -> runtime session/live installation | Phase 7 fixed in the detailed ledger: active lane, registry, continuation, live-read shape, support, diagnostic, and runtime-session identities now carry typed evidence/digest categories with raw/digest substitutions compile-failed where applicable. Phase 9 remains open only for named harness/downstream consumers. |
| Bridge retained evidence | 7/9 | `BridgeIdentityEvidence`, bridge causal references/bindings/envelopes/receipts, retained mapping evidence parts, Query causal reference artifacts | Query causal reference -> bridge causal reference -> retained mapping lookup -> bridge binding/envelope/receipt -> Query materialized causal reference artifact | Phase 7 fixed in the detailed ledger: retained mapping lookup adapters revalidate retained evidence into typed bridge identities before lookup, and legacy raw lookup APIs are no longer authority inputs. Remaining `as_str()` uses are terminal diagnostic/export watchpoints unless tied to an open Phase 9 row. |
| Worth-topo production downstream | 8 | relational commit/snapshot/record parts, bridge truth identities, Query commit/snapshot/entity identities | relational parts -> bridge truth identity -> Query typed receipt/read identity -> worth-topo write/read adapters -> diagnostic or JSON projection | Phase 8 fixed in the detailed ledger: production write/read/query-runtime rows now preserve typed commit/snapshot/entity authority and quarantine `identity.id` as explicit projection. Remaining worth-topo work is Phase 9 certification/harness cleanup only. |
| Phase 9 downstream and harness cleanup | 9 | relational IDs, bridge truth carriers, Query typed mutation/read receipts | typed lower identity -> bridge/Query typed receipt -> harness/downstream adapter -> terminal report/compat projection | Phase 9 open: worth-server adapters, hadwiger-research, and worth-ui still contain string-authority consumers. Query harness/certification and worth-topo certification rows are fixed or verified stale; `worth-kernel` and `WORTH-kernel` had no current blocker in this sweep. |

### Compiler Failure Ledger And Historical Trace Inventory

This section began as the Collapse Matrix. It is retained as historical trace
inventory, but it is no longer the primary discovery mechanism. From this
rewrite forward, row status can close only when it maps to compiler breakage,
a compile-fail guard, terminal projection quarantine, or a named deferred owner
milestone. Do not collapse these rows back into broad folder claims.

#### Upstream Relational And Signal Root Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Relational bridge snapshot identity | `SnapshotHandle`, `SnapshotId`, `VersionId` | `bridge_snapshot_identity_for_handle`, `bridge_snapshot_identity_for_commit`, `parse_bridge_snapshot_identity` | bridge snapshot display string | Phase 2A typed-seal gate | Current code recovers snapshot/version through `TruthSnapshotIdentity::relational_snapshot_parts()`. Add compiler guard so parse helpers cannot fall back to display text. |
| Relational bridge commit identity | `CommitId` | `TruthCommitIdentity::from_relational_commit_id` -> `parse_bridge_commit_identity` | bridge commit display string | Phase 2A typed-seal gate | Current code recovers commit through `TruthCommitIdentity::relational_commit_id()`. Add compiler guard so `commit-*` text cannot become relational authority. |
| Relational record identity | `RecordRef`, `EntityId`, `RelationId` | `record_ref_identity` -> `RelationalBridgeRecordIdentityParts` -> `record_ref_from_identity_parts` | `RelationalRowIdentity::as_str()` diagnostic label | Phase 2A typed-seal gate / projection quarantine | Row display label must remain diagnostic; authority is record identity parts. Add guard against `entity:*` / `relation:*` text re-entry. |
| Relational patch envelope identity | publication bundle / canonical commit envelope | `publication_patch_to_bridge_envelope` -> `BridgeCommittedPatchEnvelopeIdentity` | branch/patch/snapshot display labels | Upstream covered | Keep envelope built from typed commit, patch position, snapshot, and branch identities |
| Grouped truth row set | bridge snapshot read packet/result | `materialize_relational_authoritative_row_set` -> `RelationalAuthoritativeRowSetArtifact` | row-set digest, request-key error labels | Upstream covered with projection watchpoint | Digest is artifact evidence; request key is error reporting only |
| Grouped projection | grouped row set + contract | `grouped_projection.rs::RelationalGroupedProjectionArtifact` implements `GroupedProjectionSource` | `GroupedProjectionMemberSource::row_identity() -> &str` at `grouped_projection.rs:67` | Projection quarantine | Exact projection site identified; downstream must not parse this row string back into authority |
| Grouped truth canonical digest | typed snapshot + typed row parts | `row_set_digest`, `grouped_projection_digest` | prefixed SHA strings | Allowed artifact digest | Canonical digest may certify artifact contents, not replace snapshot/record identity |
| Signal branch identity | `SignalBranchId`, `SignalSnapshotId`, branch posture, restore posture | `SignalBranchBasisIdentity` -> `SignalBranchBasisArtifact` | branch component, snapshot component, head component, restore component, basis digests | Phase 2A signal proof-lane gate | Valid signal authority lane through `worth_proof::Artifact`; must stay distinct from bridge truth identity lane. Decide whether a dedicated `worth-signal` identity-category scaffold is required. |
| Signal branch trust-boundary bridge | `SignalBranchBasisArtifact` | `bridge_signal_branch_basis_trust_boundary` -> `BoundaryBridgedSignalBranchBasisArtifact` | basis digest reporting | Phase 2A signal proof-lane gate | Boundary-bridged signal basis requires revalidation; do not consume digest as current authority or bridge/query identity. |
| Signal merge compatibility basis | signal branch basis identity + scoped merge proof + strategy witness | `compatibility/readmission.rs::build_compatibility_basis` -> `SignalMergeCompatibilityBasis` | declaration, admitted-scope, strategy-witness digests copied into basis | Exact signal audit site | Exact constructor identified; digest fields are proof components and must not become bridge/query identity authority |
| Signal merge compatibility witness | compatibility fact inventory | `SignalMergeCompatibilityWitness::new` / replay decode | compatibility digest | Allowed proof digest | Digest validates fact inventory equality only; not a bridge/query identity |
| Signal output identity | host-provided `OutputIdentity` | `data/output.rs::NodeEvaluationResult::with_output_identity` -> `branching/merge_runtime.rs::resolve_identity_matches` -> `IdentityCorrespondenceRecord` | opaque string token + stable hash; `target_index.get(&source_output_identity)` | Phase 2A external/domain token gate | Host/domain equivalence token is used for signal merge correspondence only; never promote this token into query/bridge truth identity or relational entity/relation authority. |
| Signal partition/detail tokens | host-provided `PartitionToken`, changed regions, subscriptions | partition interner -> scoped invalidation/reuse matching | partition/detail strings | Phase 2A external/domain token gate | Allowed for signal domain scoping; keep separate from relational record/entity identity and bridge/query truth identity. |
| Signal identity matcher registry | `IdentityMatcherDescriptor`, typed IDs/names/policies | frozen registry -> selected matcher -> lowered merge plan | descriptor/registry digests | Upstream signal proof evidence | Registry digests certify selected strategy; not entity or branch identity |
| Signal scoped merge proof | normalized scope + scoped candidates | `merge/scoped_proof.rs::ScopedMergeProofPacket` -> `compatibility/readmission.rs::compare_retained_inputs` and `inspection/support_witness.rs::compare_retained_support_inputs` | declaration/admitted/skipped/no-op scope digests compared directly | Exact signal audit site | Exact digest-comparison sites identified; acceptable only if scoped proof remains the proof authority and never becomes query/bridge identity |
| Query subscription signal handoff | Query subscription/runtime session identity + signal strategy/basis | subscription activation -> active lane -> signal invalidation/support | signal basis/strategy digest-looking fields | Cross-lane blocker | Query must carry typed Query/bridge identities and treat signal digest fields as signal proof evidence only |
| Worth-topo via signal-derived read freshness | signal invalidation/session proof + Query typed read identity | signal domain event -> Query read execution/read views | output identity / partition token strings | Phase 8 watchpoint | Worth-topo must not use signal domain tokens as relational entity/relation anchors |

#### Declaration Identity Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Declaration entry orchestration | `WORTHQueryAdmittedConfiguredDomainHandle` | `orchestrate_declaration_entry` -> `worth_query_lower_declaration_entry_orchestration_on_handle` | orchestration transcript digests | Phase 7 covered | Keep as typed pipeline root |
| World basis identity | `WORTHQueryAdmittedWorldBasis` | typed `handle_identity()` and `basis_lifecycle_support_identity()` | `handle_identity_for_reporting()`, `basis_lifecycle_support_for_reporting()` | Phase 8 watchpoint | Reporting accessors may stay only if equality checks use typed identity |
| Canonical declaration identity | `WORTHQueryCanonicalDeclarationArtifact` | canonical declaration artifact -> progression payload | canonical digest rendering | Phase 7 covered | Keep digest as declaration artifact identity, not bridge truth identity |
| Progression payload | retained legality evidence + world basis | `WORTHQueryDeclarationProgressionPayload` -> checked outcome | `hash_parts` progression digest | Phase 7 watchpoint | Document digest as progression proof carrier only |
| Foundational evidence | `WORTHQueryDeclarationFoundationalEvidenceInput` | admitted progression -> foundational evidence | `handle_identity_digest()` | Fixed | Foundational evidence world admission now compares the subject's retained `WORTHQueryEvidenceIdentity` handle against the expected admitted world-basis handle identity when the subject carries typed world basis; digest strings remain only for denial detail/reporting compatibility. Gates: `cargo test -p worth-query application::declaration_evidence --lib` (`12 passed`), `cargo check -p worth-query` |
| Route planning | admitted progression + foundational evidence | `WORTHQueryDeclarationRoutePlanInput::admitted` -> checked route plan | route report strings | Phase 9 covered | Keep route plan typed; no Phase 9 blocker found in lane |
| Bridge continuation lowering | declaration envelope + bridge contract | `declaration_bridge_routing/lower.rs` -> bridge bindings | `AspectValue::String(envelope.declaration_digest())` | Projection quarantine | Payload display only; do not use as route authority |
| Query truth identity mint | Query evidence | `query_truth_*_identity` in `lower_identity.rs` | stable hash positions | Re-entry risk | Keep sealed; forbid caller display strings as input |
| Contribution-composed composition | composed contribution evidence | `WORTHQueryContributionComposedComposition` | aggregate ordering by `as_str()` | Projection quarantine | Sorting sealed identities is allowed only for deterministic aggregate projection |

#### Workflow Binding And Preview Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Workflow context binding | `WorkflowContextBinding` | source/query/basis/binding identities | `binding_digest()`, `query_for_reporting()`, `basis_for_reporting()` | Fixed | Binding carries typed source/query/basis/binding identities and post-merge/workflow materializers consume typed accessors for authority; digest/reporting accessors remain terminal projection. Gates: `cargo test -p worth-query workflow::tests::binding --lib` (`6 passed`), runtime-preflight workflow materializer gate (`4 passed`) |
| Workflow declaration | `QueryWorkflowDeclaration` | binding + request -> admission report | `declaration_digest()` | Fixed | Admission report retains typed declaration and binding identities; declaration digest text is used for relational provenance/reporting correlation only, not as authority. Gates: `cargo test -p worth-query workflow::tests::binding --lib` (`6 passed`), `cargo test -p worth-query workflow::tests::inspection --lib` (`4 passed`) |
| Mutation authority binding | workflow lowering | `MutationAuthorityBinding` | `binding_digest()` | Fixed | `MutationAuthorityBinding` carries `WORTHQueryEvidenceIdentity` through mutation lowering and composes the lowering identity with the typed authority binding; `binding_digest()` is a reporting projection only. Gates: `cargo test -p worth-query workflow::tests::lowering --lib` (`7 passed`), `cargo test -p worth-query domain_capabilities::canonical_runtime_workflow_lowering_tests --lib` (`7 passed`) |
| Workflow payload identity | workflow payload | `WORTHQueryWorkflowContributionPayload` | `payload_digest()` | Fixed | Workflow contribution payload owns `payload_identity()` and the preview materializer now preserves request-family evidence through the bridge preview declaration identity instead of collapsing through bridge evidence scope projection. Gate: `cargo test -p worth-query domain_capabilities::canonical_runtime_workflow_preview_tests --lib` (`7 passed`) |
| Preview canonical query | preview binding + request | preview canonical query identity -> `CanonicalQueryDigest` | canonical digest label | Phase 7 covered | Keep derived from typed evidence identity |
| Preview validated query | canonical query identity | validated query identity -> `ValidatedQueryDigest` | validated digest label | Phase 7 covered | Keep derived from typed evidence identity |
| Preview declaration identity | payload + binding + request + preview session | `BridgePreviewSessionDeclarationIdentity` | declaration digest identity | Phase 7 covered | Use bridge evidence identity, not external authority text |
| Unsupported preview denial | payload posture + target kind | denial receives request identity | denial message | Phase 7 covered | Keep request identity typed in denial constructor |
| Post-merge inspection | workflow declaration + merge outcome | `inspect_post_merge_outcome` | `query_for_reporting()` / `basis_for_reporting()` equality | Fixed | Post-merge inspection compares `declaration.binding().query_identity()` and `basis_identity()` against the authoritative outcome's typed source identities; reporting strings are emitted only into rows after authority equality succeeds. Gate: `cargo test -p worth-query workflow::tests::inspection --lib` (`4 passed`) |

#### Canonical Runtime Materialization Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Contribution request | `WORTHQueryDomainCapabilityContribution<P,T>` | request identity into eligible/admitted phases | request digest/reporting | Phase 7 covered | Keep request identity typed |
| Target binding | `WORTHQueryDomainCapabilityTargetBinding` | target identity + binding identity | `target_digest()`, `binding_digest()` | Projection watchpoint | Future composition must consume identities, not digest accessors |
| Materialization-ready proof | admitted contribution | `WORTHQueryMaterializationReadyDomainCapabilityContribution` | proof/report labels | Phase 7 covered | Keep as required carrier into materializers |
| Generic canonical artifact | contribution + target + payload | `WORTHQueryCanonicalRuntimeMaterialization::new` | materialization digest | Phase 7 covered | Compose typed target/binding/request/payload/materialization evidence |
| Support materialization | support payload + target binding | support artifact/support rows | support report digest | Projection watchpoint | Include payload feeder in QA |
| Invariant capability materialization | invariant/capability payload | invariant capability artifact | program/breadth digests | Projection watchpoint | Consume `program_identity()` / `breadth_identity()` where available |
| Continuity materialization | continuity payload | continuity artifact | source labels | Projection watchpoint | Labels are source labels only, not parsed identity |
| Continuity correspondence | correspondence payload | correspondence artifact | correspondence report labels | Projection watchpoint | Add file to Phase 7 feeder row coverage |
| Aftermath materialization | aftermath payload | aftermath artifact | source label fallback | Projection watchpoint | Prefer source evidence identity when present |
| Explanation materialization | explanation payload + bridge/query evidence | explanation artifact | requested evidence family join string | Projection watchpoint | Family-set string is descriptive only |
| Workflow semantics payload | workflow runtime semantics | preview/writeback/lowering materializers | payload/report digests | Cross-lane feeder | Keep tied to workflow lane blockers |

#### Effect Execution Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Authoring basis | `EffectAuthoringBasis` | capability/scoped basis identities | capability/scoped basis reporting | Phase 7 covered | Keep typed basis identity methods as source |
| Normalized intent | raw effect intent | `NormalizedEffectIntent` | `capability_digest()`, `scoped_basis_digest()`, `expected_lower_runtime_binding_digest()`, `normalized_digest()` | Re-entry risk | Projection accessors must not drive admission/coherence |
| Eligibility/admission | normalized intent | eligibility/admission output | denial strings | Phase 7 covered | Keep typed normalized identity |
| Batch admission | normalized batch items | mixed-basis check | `scoped_basis_digest()` and `expected_lower_runtime_binding_digest()` comparisons | Fixed | `AdmittedEffectBatch` admission compares `NormalizedEffectIntent::scoped_basis_identity()` and `expected_lower_runtime_binding_identity()` directly; digest accessors remain terminal reporting. Gates: `cargo test -p worth-query effect_lifecycle::tests::batch::admission --lib` (`7 passed`), `cargo check -p worth-query` |
| Lowered effect plan | admitted effect | `LoweredEffectExecutionPlan` | plan reporting | Phase 7 covered | Keep plan identity typed |
| Execution authority | relational/bridge runtime | `EffectExecutionAuthority` -> executed artifact | runtime error messages | Phase 7 covered | Do not use diagnostic strings as authority |
| Executed plan | lowered plan + authority artifact | `ExecutedEffectPlan` | artifact reporting | Phase 7 covered | `executed_authority_artifact_identity` composes typed evidence |
| Execution receipt | executed plan | `EffectExecutionReceipt` | `*_for_reporting()` | Phase 7 covered | Reporting only |
| Oracle verification | execution receipt + retained bridge records | oracle verification identities | bridge digest wrappers | Phase 9 watchpoint | Certification/reporting only unless reused for execution |
| Closeout certification | seeded/phase4/closeout rows | closeout bundle digests | `hash_parts` report digests | Phase 9 watchpoint | Allowed as certification evidence if not execution authority |

#### Causal Inspection Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Observation receipt | query/write/read observation | causal anchor/reference set | observation reporting | Phase 7 covered | Keep typed observation identities |
| Causal request | target + anchor + requested families | `CausalInspectionRequestIdentity` | `request_digest()` alias | Phase 7 watchpoint | Alias must be projection-only or renamed |
| Reference resolution | reference artifacts | `QueryCausalEvidenceReferenceArtifact` | `reference_for_reporting()` | Phase 7 covered with watchpoint | Do not feed reporting reference into request input |
| Admission proof | request + decision trace | admission receipt/outcome identities | decision trace strings | Phase 7 covered | Keep typed decision identity |
| Bridge envelope assembly | runtime bridge diagnostics | `BridgeCausalExplanationEnvelope` | envelope reporting | Cross-lane feeder | Consumes bridge retained-evidence lane |
| Materialized detail | readmission proof + evidence references | materialized detail identity | materialization report labels | Phase 7 covered | Compose typed reference receipt identities |
| Denied detail | denial + target/result identities | denied artifact detail identity | denial reason | Phase 7 covered | Denial reason remains message only |
| Bridge-backed artifact | bridge envelope + built artifact | causal/artifact identities | `bridge_envelope_for_reporting()`, `causal_identity_for_reporting()` | Phase 7 watchpoint | Reporting names are clear; digest aliases still need review |
| Exploration | causal artifact | decision/integrity traces | reporting trace fields | Phase 7 covered | Exploration may report, not recompose authority |
| Certification rows | causal inspection tests/support | row digests from retained reference evidence | certification digest strings | Fixed before Phase 2B-B-007 | Follow-up QA expanded this from row digest slots to the full causal certification chain: hostile rows, reference collection, named slots, row-digest sets, boundary audit, representative matrix, proof-shape, certification scope, and certification bundle sealed through `WORTHQueryEvidenceIdentity` composition rather than `hash_parts` over reporting strings. Historical note: query-basis bridge binding previously used `field_bridge_identity` for retained `BridgeIdentityEvidence`; Phase 2B-B-007 supersedes that root, so the repair must replace it with typed bridge evidence carry, owner readmission, or terminal projection quarantine. |

#### Subscription And Session Lifecycle Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Subscription declaration | declaration/admission evidence | subscription input/admission | declaration reporting | Phase 7 covered | Keep typed evidence identity |
| Bridge lowering | subscription declaration | bridge declaration and basis binding | lowering diagnostics | Phase 7 covered | Keep typed source identities |
| Activation input | admitted subscription + checkpoint | `SubscriptionActivationInput` | checkpoint reporting | Phase 7 covered | Activation receipt uses typed checkpoint identity |
| Active lane admission | activation/admission/query/bridge/signal identities | `ActiveSubscriptionLaneAdmission` | digest-looking accessors | Fixed | Admission stores activation/admission/query/bridge/basis/checkpoint/signal values as `WORTHQueryEvidenceIdentity` fields and exposes digest-looking methods only as reporting projections; typed identity accessors remain available for authority composition. Gate: `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib` (`12 passed`) |
| Active lane registry | active lane + attachment | registry keys/handles | stringified keys | Fixed | Registry indices use `ActiveSubscriptionLaneDigest` and `SubscriptionConsumerAttachmentDigest` typed wrappers backed by `WORTHQueryEvidenceIdentity`, not raw display strings; handles retain typed basis/checkpoint identities. Gate: `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib` (`12 passed`) |
| Consumer attachment | lane handle + consumer | `SubscriptionConsumerAttachment` | consumer/delivery cursor labels | Phase 7 covered | Labels can remain consumer/display data |
| Delivery window | lane + attachment + sequence | `QueryDeliveryWindow` | sequence/report labels | Phase 7 covered | Typed lane/attachment identities feed composition |
| Delivery batch | work packet + patch group + receipt | `QueryDeliveryBatch` | patch/report labels | Phase 7 covered | Keep patch group identity typed |
| Continuation evidence | source/target/basis/checkpoint/authority | `SubscriptionContinuationEvidence::new` | `impl Into<String>` endpoint inputs | Fixed | Continuation admission and internal constructor require `WORTHQueryEvidenceIdentity` for source, target, basis, checkpoint, and authority endpoints; lifecycle endpoint wrapping is typed before terminal report strings are produced. Gate: `cargo test -p worth-query subscription::tests::active::active_continuation --lib` (`4 passed`) |
| Closeout | close request + lane/attachment/checkpoint | lifecycle closeout identity | closeout kind labels | Phase 7 covered | Kind labels are descriptive only |
| Runtime live installation | session setup | `WORTHQueryRuntimeLiveSubscriptionInstallation` | view/policy labels | Phase 7 covered | Counter/source identities are typed |
| Runtime backend receipt tests | typed receipt construction | test receipts | external label rejection fixture | Phase 9 fixed/narrowed | Matrix row now fixed for raw string fields |

#### Bridge Retained Evidence Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Query causal reference | Query causal evidence reference | `CausalEvidenceReferenceDigest` with optional bridge authority | reference reporting | Phase 7 feeder | Do not construct from Query reporting strings |
| Builder bridge conversion | Query reference -> bridge reference | `builder_bridge.rs` | bridge reference labels | Phase 7 covered | Must preserve typed bridge evidence |
| Bridge evidence reference | bridge causal protocol | `BridgeCausalEvidenceReferenceIdentity` | `*_for_reporting()` | Phase 7 covered | Reporting only |
| Bridge evidence binding | retained bridge record/reference | `BridgeCausalEvidenceBinding` | owner/family labels | Phase 7 covered | Labels are shape fields only |
| Retained mapping digest basis | retained record evidence | retained mapping identity parts | digest basis labels | Phase 7 watchpoint | Split into direct matrix subrow |
| Route history preview retained mapping | retained route preview evidence | legacy lookup key bridge | `reference_identity.as_str()` lookup | Phase 7 watchpoint | Audit string-keyed bridge lookup as compatibility only |
| Planning checkpoint retained mapping | retained planning evidence | legacy lookup key bridge | `from_reference_evidence(...).as_str()` | Phase 7 watchpoint | Audit string-keyed bridge lookup as compatibility only |
| Source structural stream retained mapping | retained stream evidence | legacy lookup key bridge | string-keyed bridge lookup | Phase 7 watchpoint | Audit string-keyed bridge lookup as compatibility only |
| Envelope identity | bridge envelope | `BridgeCausalEnvelopeIdentity` | envelope reporting | Phase 7 covered | Keep bridge typed identity |
| Envelope receipt | bridge receipt | `BridgeCausalEnvelopeReceipt` | receipt reporting | Phase 7 covered | Reporting only |
| Query materialization import | bridge binding -> Query reference artifact | removed `field_bridge_identity` root | Query `for_reporting()` | Phase 7 reopened by 2B-B-007 | Replace bridge evidence projection flattening with typed bridge evidence carry, owner readmission, or terminal projection quarantine. |

#### Worth-Topo Production Downstream Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Write authority commit | relational commit parts | `WORTHQueryCommitIdentity::from_relational_commit_id` | commit diagnostic label | Phase 8 covered | Keep row fixed |
| Write authority snapshot | relational snapshot parts | `WORTHQuerySnapshotIdentity::from_relational_snapshot` | snapshot diagnostic label | Phase 8 covered | Keep row fixed |
| Mutation receipt | commit/snapshot/entity typed parts | `WORTHQueryMutationReceipt::from_authoritative_parts` | receipt report labels | Phase 8 covered | Keep row fixed for commit/snapshot |
| Query rows / deltas | topology row identity | `WORTHQueryEntityIdentity` plus explicit `identity.id` projection | endpoint/reporting labels | Phase 8 fixed | Typed row authority remains `WORTHQueryEntityIdentity`; endpoint labels are explicit terminal projections and Query read materialization no longer indexes by evidence digest |
| Entity parse support | formatted entity/relation label | typed relational record parts where authority is needed | `entity:*` / `relation:*` | Phase 8 fixed | Production authority paths use typed relational record identity sources; remaining formatted labels are projection/reporting edges |
| Relation endpoint lowering | relation endpoints | typed `WORTHQueryEntityIdentity` / relational parts | endpoint label projection | Phase 8 fixed | Existing graph relation endpoints require relational record parts and refuse evidence-digest fallback |
| Patch matching | patch entity identity | typed record identity | projection label compare at compatibility edge | Phase 8 fixed | Patch matching uses typed record identity where authority is needed; label comparison remains terminal compatibility only |
| Bridge source support | bridge/query identities | typed commit/snapshot/record parts extraction | evidence-label projection | Phase 8 covered | Keep row fixed if no string authority re-entry |
| Bridge source reads | bridge request/read packets | typed branch/commit/snapshot/record accessors | read packet labels | Phase 8 covered | Keep row fixed |
| Runtime binding snapshot | runtime binding state | `current_snapshot_identity()` | empty-state diagnostic | Phase 8 covered | Keep row fixed |
| Declaration initialization | typed read basis | declaration metadata | mismatch detail labels | Phase 8 covered | Keep row fixed |
| Source adapter | runtime source adapter | `WORTHQueryRuntimeSnapshotIdentityAdapter` | none | Phase 8 covered | Keep row fixed |
| Historical read basis | historical snapshot | `WORTHQuerySnapshotIdentity` | evidence-label projection | Phase 8 covered | Keep row fixed |
| Read execution query shape | user/read anchor | explicit projected `identity.id` label backed by typed query row identity | string anchor predicate | Phase 8 fixed | Anchor projection is terminal row selection data; rows without explicit `identity.id` are not indexed by evidence digest |
| Read family execution | family anchors | read family execution predicates plus typed row authority | string anchor predicate | Phase 8 fixed | Family execution preserves anchors through explicit row projection labels and typed retained rows |
| Row decode | retained rows | explicit `identity.id` projection with typed relational fallback | formatted entity/relation label | Phase 8 fixed | Decode prefers explicit projection labels and reports retained identity inventory on denial; no evidence-digest re-entry |
| Handle reads | public read session | handle read anchor inputs routed into typed query-runtime rows | anchor label | Phase 8 fixed | Query-runtime proof gate covers handle/read certification callers; remaining harness string consumers are Phase 9 rows |
| Read proof report | executed snapshot | typed executed snapshot identity | diagnostic label | Phase 8 covered | Keep row fixed |
| Static signal sink | typed receipt route | bridge route identity construction | route diagnostics | Phase 8 covered | Keep row fixed |
| Bridge certification | relational truth constructors | proof rows | evidence labels | Phase 8 covered / Phase 9 tests separate | Keep production-adjacent row fixed, tests remain Phase 9 |

#### Phase 9 Downstream And Harness Cleanup Lane

| Trace | Root authority | Carrier / edge | Projection points | Classification | Matrix action |
|-------|----------------|----------------|-------------------|----------------|---------------|
| Runtime transcript intent | relational commit/snapshot | transcript runtime fixtures | external label commit/snapshot authority | Fixed | Transcript intent receipts and derived patches now use deterministic relational-backed fixture identities; runtime API stabilization test gate passes |
| Transcript authority | typed relational constructors now present | transcript authority fixtures | old formatted transcript strings | Fixed | Current scan finds no formatted external-authority commit/snapshot constructors in runtime API stabilization fixtures |
| Aspect API finalization | Query mutation receipt | certification row digest | typed commit/snapshot evidence identity | Phase 9 fixed | Covered by detailed fixed row: aspect API finalization rows derive digest material from typed commit/snapshot evidence and targeted scan found no receipt commit string re-entry |
| Public bridge runtime support | bridge runtime test support | mutation receipts / hostile cert digests | terminal public bridge labels | Phase 9 fixed | Covered by detailed fixed rows for public bridge support, bridge parity, and hostile receipt/artifact digest helpers |
| Lower-runtime routing fixtures | typed evidence/relational truth constructors | certification fixtures | old raw truth constructors | Phase 9 fixed | Keep fixed |
| Causal inspection tests | bridge harness labels | causal write/read/support fixtures | typed causal/bridge evidence plus terminal report labels | Phase 9 fixed | Covered by detailed fixed causal rows; remaining proof-shape and certification strings are terminal evidence/report projections |
| Effect lifecycle seeded support | typed relational snapshot handles | seeded support fixtures | old patch from commit text | Phase 9 fixed | Keep fixed |
| Milestone-eight harness | bridge harness labels | patch/head/snapshot/branch fixtures | typed relational/bridge fixtures plus terminal labels | Phase 9 fixed | Covered by detailed fixed harness rows and focused worth-query gates recorded in the row 1051 closeout evidence |
| Projection consumption tests | bridge harness labels | projection facts tests | typed truth snapshot/branch/commit evidence | Phase 9 fixed | Covered by detailed fixed projection-consumption rows; read/write extraction validates typed commit/snapshot evidence |
| Query basis lifecycle tests | bridge harness labels | basis lifecycle tests | typed patch/head commit evidence | Phase 9 fixed | Covered by detailed fixed query basis lifecycle rows and retained-live/live-read result-shape gates |
| Intent admission bridge fixtures | bridge certification fixtures | patch identity derivation | typed bridge/query evidence identities | Phase 9 fixed | Covered by detailed fixed intent-admission and declaration-bridge-routing rows; note contradiction resolved by typed causality/route identity lowering |
| Intent admission runtime/read fixtures | certification labels | placeholder receipts/read fixtures | typed commit/snapshot/read evidence with terminal labels | Phase 9 fixed | Covered by detailed fixed generic inspection, runtime read, and live receipt rows |
| Hostile journal gap count | relational commit identity | hostile helper | old `rsplit('-')` parse | Phase 9 fixed | Row now fixed |
| Hostile receipt/artifact digest | write receipts/artifacts | hostile digest helpers | typed receipt/artifact snapshot/commit evidence | Phase 9 fixed | Covered by detailed fixed hostile helper row; targeted helper scan found no remaining commit/snapshot string composition |
| Native patch envelope fixture | bridge fixture | patch/snapshot/branch/entity fixture | typed relational truth constructors | Phase 9 fixed | Covered by detailed fixed relational/bridge fixture rows; native patch envelopes now use typed relational constructors |
| Runtime backend receipt tests | typed mutation receipts | signal routing tests | old raw string fields | Phase 9 fixed | Row now fixed; residual lives in fixture rows |
| Worth-topo bridge tests | bridge route tests | topology bridge test routes | bridge diagnostic evidence-label projections | Phase 9 fixed | Verified narrowed row: bridge tests route with relational truth constructors; diagnostic string projections stay terminal. Gate: `cargo test -p worth-topo projection::runtime_boundary::bridge::tests --lib` (`5 passed`). |
| Worth-topo read proof harness | topology read certification | historical read target | typed snapshot identity plus diagnostic projection | Phase 9 fixed | Harness uses `workspace.snapshot_identity()` and asserts typed executed snapshot identity. Gate: `cargo test -p worth-topo certification::projection_closeout::tests::topology_reads --lib` (`61 passed`). |
| Worth-topo derived chain | topology derived-chain certification | inspection/write receipt assertions | typed commit identity comparison | Phase 9 fixed | Derived-chain inspection compares typed `WORTHQueryCommitIdentity` handles; remaining surface names are derived-view labels. Gate: `cargo test -p worth-topo certification::projection_closeout::tests::derived_chain --lib` (`2 passed`). |
| Hadwiger research | test write authority | typed `WORTHQueryMutationReceipt` constructor | terminal Hadwiger report strings | Phase 9 fixed | Research graph invariant fixtures now use typed Query commit/snapshot/entity identities and typed lower-runtime envelope/source identity accessors; gate `cargo test -p hadwiger-research --test research_graph_invariants` passed (`10 passed`). |
| WORTH UI todo truth | UI truth state | task truth/snapshot routing | explicit snapshot projection label | Phase 9 fixed | Todo truth now routes mutations through typed `WORTHQueryEntityIdentity`, exposes board state as `TodoSnapshotProjection`, and treats the combined snapshot label as terminal UI diagnostics. Gate: `cargo check -p worth-ui` passed. |
| worth-server compat request | HTTP JSON request | compat request parsing | `entity_identity` JSON strings | Phase 9 fixed | Compat update/delete request parsing admits entity identity only from canonical relational bridge record text via `RelationalBridgeRecordIdentityParts::from_bridge_entity_identity(...)` before constructing `WORTHQueryEntityIdentity`; raw HTTP labels remain external compatibility input and cannot satisfy authority. Gates: `cargo check -p worth-server`, `cargo test -p worth-server --test compat_http_phase_three` (`8 passed`) |
| worth-server mutation result | direct/compat mutation response | result digest | `receipt.commit_identity()` | Phase 9 fixed | Single-mutation response result digests now project from typed commit evidence identity (`commit_evidence_identity().as_str()`), while inspection assertions compare typed commit/snapshot handles. Gates: `cargo test -p worth-server --test WORTH_native_facade_entry` (`62 passed`), `cargo test -p worth-server --test compat_http_phase_three` (`8 passed`) |
| worth-server query execution | handoff workspace | compatibility precondition | `workspace().snapshot_token()` | Phase 9 fixed | Compat mutation precondition observes the workspace typed snapshot identity and projects it only as the terminal compatibility validator label; direct read/state paths use explicit reporting accessors instead of the removed snapshot-token trait seam. Gate: `cargo check -p worth-server` |
| worth-server test adapters | runtime test adapters | snapshot adapter + mutation receipt fixtures | `snapshot_token() -> String`, formatted receipts | Phase 9 fixed | Server test adapters no longer implement the removed `snapshot_token`/`support_evidence` string shims; mutation receipt fixtures use typed commit, snapshot, entity, and subscription-support evidence identities. Gates: `cargo test -p worth-server --test WORTH_native_facade_entry` (`62 passed`), compat phase three/four (`8 passed` each) |
| worth-server integration tests | direct mutation/projection tests | inspection/read receipt assertions | snapshot/commit string equality | Phase 9 fixed | WORTH-native integration tests compare typed receipt/inspection identities or explicit evidence projections only at result/reporting edges; the backend-verified assertion denial fixture now requires a canonical relational bridge record identity instead of falling back from a raw task label. Gate: `cargo test -p worth-server --test WORTH_native_facade_entry` (`62 passed`) |
| Worth-kernel / WORTH-kernel | local/display formatting only in sweep | no identity-lowering authority found | ordinary local strings | Out of scope | Do not add blocker without new evidence |
| Subscription replay tests | bridge replay tests | truth identities from string literals | deferred replay fixture labels | Phase 10 | Close in Phase 10: migrate `subscription/replay_tests.rs` to typed relational constructors; no separate owner milestone |

### Phase 10 â€” closure (zero-deferral closeout)

Phase 10 closes Milestone 9.6 only when **every item below is met**. Nothing from
Phase 9 or earlier closeout passes may remain silently deferred.

**Workspace and QA**

- [ ] Phase 7 QA gate `CLEARED` (recorded in compiler ledger/header)
- [ ] Phase 9 hostile QA `CLEARED` on `query-repair` (gate paths + residual folklore)
- [ ] `cargo check --workspace` green
- [ ] Full compile-fail matrix green (see closeout doc Verification Gates â€” all
  `phase_boundaries_*` suites, not the fast Phase 9 subset alone)
- [ ] Hostile QA pass on full 9.6 bar (code inspection, not tests alone)

**worth-topo Phase 9 compile-fail extension (required â€” was deferred from Phase 9)**

- [ ] Add `query_runtime_phase_nine` trybuild manifest mirroring Phase 8
  (`phase_eight_compile_fail_targets.rs` pattern)
- [ ] Extend folklore inventory: scan previously excluded harness paths
  (`PHASE_EIGHT_EXCLUDED_FOLKLORE_PATHS`) with Phase 9 forbidden patterns
  (`WORTHQueryMutationReceipt {`, harness authority folklore)
- [ ] Add compile-fail UI fixtures (e.g. mutation receipt struct literals,
  terminal projection authority misuse)
- [ ] Wire `phase_boundaries_query_runtime_phase_nine_compile_fail.rs` + Cargo.toml

**worth-runtime-bridge subscription replay (required â€” was deferred to non-existent owner milestone)**

- [ ] Migrate `src/subscription/replay_tests.rs` from label-based
  `truth_identity_fixtures::{truth_snapshot_fixture, truth_branch_fixture}` to
  typed relational constructors (`truth_snapshot`, `truth_branch`,
  `from_relational_*`)
- [ ] Close matrix row **Subscription replay tests** in this milestone
- [ ] Add compile-fail or folklore guard on replay test paths if label-mint
  patterns remain reachable

**worth-spatial certification (postponed â€” separate optimization agent)**

- [ ] Triage and optimize `cargo test -p worth-spatial --test public_api_contract`
  (serial gate `--test-threads=1`; lib 72/72 green; harness perf/flake)
- Owner: worth-spatial agent â€” does **not** block worth-query WS-6+
  failures (boolean evidence ledger, evidence-ledger receipts, honesty guards,
  workload vocabulary)
- [ ] Distinguish 9.6 harness fallout vs pre-existing drift; fix either way
  before closeout

**Documentation and ledger**

- [ ] Update `phase-9-discovery-ledger.md` â€” remove open trybuild row; record
  Phase 10 execution
- [ ] Append `query-repair` closeout section to
  `milestone-9.6-bridge-truth-identity-closeout.md` with gate evidence
- [ ] Compiler Failure Ledger: all in-scope rows closed (no unnamed deferrals)
- [ ] Milestone status -> `Closed` only after all Phase 10 boxes above are checked

## Phase 7 QA gate

Phase 7 closes only on **`CLEARED`** from a hostile QA pass â€” not when path rows
flip to `Fixed` or when `identity_boundary` regex scans report zero residue.

Run QA only after feeder-bundle local fixes and gates are clean. Use a separate
agent or `composer-2.5-fast` with **code inspection**, not tests alone.

**Hard bar (summary):** every ordinary Phase 7 surface and every same-class
upstream/adjacent feeder uses the authority-category boundary internally.
Projection, digest evidence, external token, bridged identity, wrong kind, and
raw representation cannot satisfy current authority. String output is allowed
only as explicitly named `*_for_reporting` projections backed by typed internal
fields and never fed back into composition, lookup, admission, routing,
comparison, or coherence.

**Return format (exactly one):**

- `CLEARED:` â€” concise evidence Phase 7 satisfies the hard bar.
- `NOT CLEARED:` â€” numbered blockers with file paths and violation kind.

**QA prompt (copy verbatim):**

```text
You are doing skeptical QA for WORTH Query milestone 9.6 Phase 7 only. Use the
current workspace as authoritative.

Authoritative spec:
plans/worth-query/milestone-9.6-bridge-truth-identity-lowering.md
(Phase 7 section + feeder bundle table + matrix rows Fix Phase = 7)

Hard bar: Phase 7 is CLOSED only when every ordinary covered surface and every
same-class upstream/adjacent feeder uses the authority-category boundary
internally. Projection, digest evidence, external token, bridged identity, wrong
kind, and raw representation cannot satisfy current authority. No hash_parts,
string join, format digest folklore, dual lower-authority/authority lanes, or
cfg/test escape hatch in production. String projections acceptable only as
*_for_reporting or equivalent explicit projection names backed by typed internal
fields and never re-entering authority.

Inspect these feeder bundles (matrix rows 572â€“577):
- subscription/live/session/backend receipts
- workflow + domain_capabilities workflow lowering
- domain_capabilities canonical_runtime continuity/support/artifacts
- effect_lifecycle spine
- runtime/inspection/causal request + materialization
- worth-runtime-bridge diagnostics/causal_envelope retained_mapping

Inspect code, not just tests. Be hostile. Return exactly one of:
- CLEARED: with concise evidence
- NOT CLEARED: numbered blockers with paths and why each violates the hard bar

Do not complain about Phase 9 harness/fixture folklore unless it is a
same-class upstream feeder for Phase 7 production surfaces.
```

**After QA:** if `NOT CLEARED`, add or update compiler ledger rows for any new
blockers, fix feeder bundles, re-run local gates, re-run QA. Do not mark feeder
bundles 572-577 closed until QA is `CLEARED`.

## Closure Gate

Closed only when:

1. **Phase 2B done-when satisfied** (evidence/feeder roots broken, Query
   identity trybuild suite landed, ledger `2B-*` rows closed).
2. No relational export formats truth IDs to display strings at ordinary paths.
3. No Query adapter or receipt surface accepts or exposes truth identity as `String`.
4. Production signal sink routes through bridge with typed commit identity.
5. Journal-order helpers do not parse commit identity text.
6. Phase 2A + 2B hard gates still enforced â€” no restored public string constructors,
   no public `field_identity(AsRef<str>)`, no public
   `WORTHQueryEvidenceIdentity: AsRef<str>`.
7. No incremental-refactor debt: no dual string/typed paths, no `cfg` folklore,
   no parity fixtures left in tree.
8. Compiler Failure Ledger and historical trace inventory have no open
   in-scope rows; every deferred row names an owner milestone.
9. Phase 7 QA gate returned **`CLEARED`** after Phase 2B repair and is recorded
   in the compiler ledger/header.

## Out of scope (other milestones)

`subscription/`, `workflow/`, `domain_capabilities/`, `effect_lifecycle/`, live
subscription/runtime session feeders, and causal inspection/materialization
feeders are **not** out of scope when they feed or consume a Phase 7 bridge
truth/query evidence boundary. Same-class upstream or adjacent feeders are Phase
7 work. Fix `worth-kernel`, `worth-server`, etc. only when matrix lists them; no
compat shims.

## Goal-Mode Loop Prompt

```text
Spec: plans/worth-query/milestone-9.6-bridge-truth-identity-lowering.md

Read Mechanical Breakage Rules first. They override convenience.

1. Find first incomplete phase from repo state.
   - Phase 1 incomplete: authority categories, kind markers, root break list, or
     compiler ledger schema are missing.
   - Phase 2A incomplete: truth-routing root breaks not landed or Phase 2A gates
     not verified.
   - Phase 2B incomplete: evidence/feeder root breaks not landed (see Phase 2B
     section), Query `tests/ui/query_identity_authority/*` missing, workspace
     green after only bundle-local fixes, or `field_identity(AsRef<str>)` still
     public. **This is the active frontier until Phase 2B done-when is satisfied.**
   - Phase 7 incomplete: any Fix Phase = 7 feeder bundle row Open (572-577), any
     projection re-entry route remains, or Phase 7 QA not CLEARED â€” path rows
     Fixed alone are insufficient.
   - Phase 8 blocked: do not start Phase 9 downstream harness cleanup claiming
      Phase 8 done until Phase 7 QA is CLEARED (Phase 8 path rows may already
      show Fixed in matrix).
   - Phase 3-6, 9+: fix compiler ledger rows per phase; workspace may be red
     until Phase 10.
2. Phase 1: define root authority categories, kind markers, root API breaks,
   compiler ledger schema, and trybuild plan. Do not fix call sites. Extend
   phase_one_root_break_targets.rs with concrete Tier 3â€“6 APIs from Phase 2B.
3. Phase 2A: truth-routing root breaks (closed). Phase 2B: install evidence/feeder
   root breaks per Phase 2B slices; run `cargo check -p worth-query` (then
   workspace as unblocked); write 2B-* ledger rows. Workspace RED = success. Do
   not fix downstream compile errors to make the break smaller. Do not fix feeder
   bundles before parent encoder break.
4. Phases 3-6, 9: work from the first upstream compiler frontier; replace the
   lower-authority path with an authority-category type and update ledger rows
   in the same change. No incremental refactoring. No parity tests.
5. Phase 7: fix projection re-entry feeder bundles (table in spec); path rows may
   be parallel across non-overlapping bundles. Run Phase 7 QA gate; stay on
   Phase 7 until CLEARED. Do not mark feeder bundles closed until QA CLEARED.
6. Phase 8: only after Phase 7 QA CLEARED (worth-topo may already be code-complete).
7. Phase 10: workspace green + compile-fail gates + closeout doc.

Banned always: incremental refactoring, parity/round-trip tests during refactor,
deprecated shims, dual APIs, cfg folklore, incremental validation suites,
restoring public string ctors, rebuilding authority from projection/digest/raw
text, and closing Phase 7 from row-scoped scans alone.
```

## Compiler Failure Ledger And Historical Matrix

> **Phase 1 scan status:** `Closed` â€” agent scan completed on 2026-06-11;
> Cursor QA omissions corrected on 2026-06-11.
>
> **Phase 7 QA status:** `CLEARED` post-Phase-2B (2026-06-15) â€” hostile re-pass
> after evidence/feeder repair: no production `field_identity` ingress; Query +
> bridge compile-fail suites pass; Phase 7 certification lanes
> (`causal_inspection`, `runtime_certification`, `query_basis_lifecycle`) pass.
> Downstream crate repair remains Phase 8+.
>
> **Phase 7 QA blockers (2026-06-09 pass 5):**
> 1. `subscription/evidence_identities.rs::lifecycle_certification_bundle_identity` â€” lifecycle delivery auxiliaries (performance, attachment, delivery_window, work_packet, closeout, etc.) still composed via `field_identity(&str)` from certification sequence-projection strings, not typed handles.
> 2. `subscription/evidence_identities.rs::{active_lane_identity,certification_activation_bundle_identity}` â€” `query_declaration_for_reporting` embedded via `field_identity` while typed `query_declaration_identity` exists upstream (dual string/typed lane on same field).
> 3. `runtime/live_subscription.rs::live_subscription_source_digest_evidence` â€” installation/counter evidence still wraps `counters.digest()` and other string sources through `field_identity(source_digest, â€¦)` after typed counter `evidence_identity()` exists on the subscription spine.
> 4. `domain_capabilities/canonical_runtime/{artifacts,support,invariant_capability}.rs` â€” `canonical_runtime_request_identity` / `support_request_identity` still string-wrap `request_digest` with `field_identity` while target/binding use `field_evidence_identity` on the same materialization compose.
> 5. `domain_capabilities/canonical_runtime/workflow/{lowering,preview}.rs` â€” denial/preview materialization paths still use `field_identity(target, target_digest)` beside typed binding paths.
> 6. `effect_lifecycle/{planning,receipt,batch,authoring_basis}.rs` â€” production spine retains `hash_parts` compatibility digests and `field_identity` on `admitted_digest()` / `counters.digest()` / receipt strings beyond the normalized/lowering slice that pass 4 fixed.
> 7. `runtime/inspection/causal/materialization/artifacts/bridge_backed.rs` â€” dual authoritative accessors on typed fields (`artifact_identity()` + `artifact_digest()`, `causal_identity` + `causal_identity_digest()`, `bridge_envelope_digest()` beside `_for_reporting` siblings).
> 8. `runtime/inspection/causal/materialization/exploration.rs` â€” exploration path still calls `query_admission_digest()` / `bridge_envelope_digest()` after receipt API rename to `*_for_reporting()`, leaving mixed projection dialect.
> 9. `workflow/foundation.rs::{workflow_scope_digest_identity,preview bind helpers}` â€” binding-scope and preview-session evidence still compose via `field_identity` on raw string digests; only primary source/query/basis helpers were elevated to typed `field_evidence_identity`.
>
> **Phase 7 QA blockers (2026-06-09 pass 3â€“4 â€” resolved on targeted spine):**
> 1. ~~`subscription/performance_receipt.rs`~~ â€” typed `performance_receipt_identity`; reporting via `performance_receipt_for_reporting()`.
> 2. ~~`subscription/evidence_identities.rs`~~ â€” auxiliaries use `field_evidence_identity` for diagnostics, support, counters, future_selection, performance.
> 3. ~~`SubscriptionLifecycleCertificationBundle`~~ â€” dual typed `*_digest()` aliases removed; `_for_reporting` + `*_identity()` accessors only.
> 4. ~~`causal/materialization/receipt.rs`, `bridge_backed.rs`~~ â€” `*_for_reporting()` projection API aligned with proof.rs.
> 5. ~~`domain_capabilities/payloads/*`~~ â€” payload composition via `WORTHQueryEvidenceIdentity::compose`.
> 6. ~~`canonical_runtime/{artifacts,support,invariant_capability}.rs`~~ â€” target/request wired with `field_evidence_identity`.
> 7. ~~`effect_lifecycle/normalized.rs`~~ â€” capability/scoped-basis from typed `EffectAuthoringBasis` identities.
> 8. ~~`effect_lifecycle/lowering.rs`~~ â€” plan/artifact via `field_evidence_identity`.
> 9. ~~`workflow/foundation.rs`~~ â€” context source/query/basis accept typed identities.
>
> **Phase 8 milestone status:** `Closed` â€” Phase 7 QA is `CLEARED`, production worth-topo entity/relation/read-anchor blockers are fixed, and remaining non-production replay cleanup is explicitly deferred to the subscription replay typed identity milestone.
>
> **Last updated:** 2026-06-15
>
> **Superseding 2026-06-15 status:** Milestone 9.6 is closed. Phase 8 production rows are fixed, the final QA-loop pass is `CLEARED`, and the worth-topo query-runtime gate closed the production entity/relation/read-anchor blockers found by the 2026-06-13 trace sweep. Remaining replay cleanup is explicitly deferred to the subscription replay typed identity milestone.
>
> This matrix is historical trace inventory, not the primary discovery
> authority. The compiler failure ledger is authoritative after the Law 42 root
> breaks. Each fix phase updates owned rows in the same PR, but a row closes only
> by authority-category fix, compile-fail guard, terminal projection quarantine,
> or named deferred owner milestone. Feeder bundle rows (572-577) require Phase
> 7 QA `CLEARED` before closure.
>
> **Slice-10 clean scan note:** `worth-kernel` and `WORTH-kernel` were scanned
> on 2026-06-11; no in-scope ordinary bridge truth-routing `Truth*Identity` or
> `WORTHQueryMutationReceipt` string collapse path was found. If the Phase 2
> exposure report surfaces a `worth-kernel` or `WORTH-kernel` compile break, add
> the concrete row before continuing the fix phases.

| Fix Phase | Crate | Path | Pattern | Status | Notes |
|-----------|-------|------|---------|--------|-------|
| 2 | worth-runtime-bridge | `src/identity.rs`, `src/facade/exports_core.rs`, `input/envelope/core.rs`, `snapshot/token.rs` | `TruthCommitIdentity`, `TruthPatchIdentity`, `TruthBranchIdentity`, and `TruthSnapshotIdentity` are public aliases of `BridgeIdentity<Tag>`, whose public `new`, `as_str`, `Display`, and `PartialEq<&str>` expose typed truth IDs as arbitrary text. | Fixed | Gate landed in Phase 2; internal lawful mint and typed storage remain Phase 4 work |
| 2 | worth-query | `memory_workspace/mod.rs` | `WORTHQueryMutationReceipt`, `WORTHQueryMutationDelta`, and `WORTHQueryLivePatch` expose `commit_identity`, `snapshot_token`, and `entity_identity` as public `String` fields. | Fixed | Gate landed in Phase 2; internal typed receipt replacement remains Phase 6 work |
| 2 | worth-query | `runtime/backend/contracts.rs` | `WORTHQueryRuntimeBackend::snapshot_token()` and `WORTHQueryRuntimeSourceAdapter::snapshot_token()` return `String`, and initialization helpers accept `snapshot_token: &str`. | Fixed | Gate landed in Phase 2; adapter implementation drift remains Phase 5/6 work |
| 3 | worth-relational | `presentation/bridge/identities.rs` | `record_ref_identity` formats relational `EntityId`/`RelationId` as `entity:*`/`relation:*`, `bridge_snapshot_identity_for_binding` formats snapshots as `relational-snapshot:*:version:*`, and parse helpers recover native IDs by splitting those strings. | Fixed | Relational birth now returns typed `RelationalBridgeRecordIdentityParts`; snapshot/commit recovery uses bridge-owned typed extractors |
| 3 | worth-relational | `presentation/bridge/patch_envelopes.rs` | `publication_patch_to_bridge_envelope` mints `TruthCommitIdentity::new(format!("commit-*"))`, `TruthPatchIdentity::new(format!("patch-*"))`, and takes branch/snapshot identities as `impl Into<String>`. | Fixed | Publication mint now accepts native `BranchId` and typed snapshot identity, and uses bridge-owned relational constructors |
| 3 | worth-relational | `presentation/bridge/patch_envelopes.rs` | `publication_bundle_to_bridge_envelope` and `commit_envelope_to_bridge_envelope` call `bridge_snapshot_identity_for_*().as_str().to_string()` before rewrapping the value as `TruthSnapshotIdentity`. | Fixed | Bundle/commit envelope lowering carries `TruthSnapshotIdentity` directly |
| 3 | worth-relational | `presentation/bridge/runtime_source/branch_heads.rs` | `TruthBranchIdentity.as_str().to_string()` becomes relational `BranchId`, and branch-head errors compare/report string branch identity. | Fixed | Branch recovery uses `relational_branch_id()` and diagnostics avoid opaque bridge payload access |
| 3 | worth-relational | `presentation/bridge/runtime_source/committed_patches.rs` | `request.commit_identity().as_str()` feeds `parse_bridge_commit_identity`, which strips `commit-` and parses `CommitId`. | Fixed | Commit recovery uses `TruthCommitIdentity::relational_commit_id()` through the relational bridge helper |
| 3 | worth-relational | `presentation/bridge/runtime_source/snapshot_authority.rs` | `parse_bridge_snapshot_identity` splits `TruthSnapshotIdentity.as_str()` on `:` to recover `SnapshotId` and `VersionId`. | Fixed | Snapshot authority uses `relational_snapshot_parts()` and reports native snapshot/version values |
| 3 | worth-relational | `presentation/bridge/snapshot_reading.rs` | Snapshot reader calls `parse_bridge_record_identity(read.entity_identity())` and reports snapshot/record identity through string accessors. | Fixed | Read path requires bridge-carried typed relational record parts and reports native record labels after typed conversion |
| 3 | worth-relational | `presentation/bridge/runtime_source/continuity_lineage.rs` | Continuity lineage converts branch identity with `.as_str().to_string()`, parses prior slice `entity_identity()` text, formats `lineage:*`, and reuses `record_ref_identity` for resolved records. | Fixed | Branch/lineage/resolved record minting now uses bridge-owned relational typed constructors |
| 3 | worth-relational | `presentation/bridge/test_catalog.rs` | `PublicationBridgeCatalog` accepts branch/snapshot identities as `impl Into<String>`, indexes committed patches and snapshots by `Truth*Identity.as_str().to_string()`, and services requests by erased commit/branch/snapshot identity text. | Fixed | Catalog accepts native branch plus typed snapshot and indexes by typed bridge identities |
| 3 | worth-relational | `grouped_truth/canonical_digest.rs` | Grouped-truth row-set and grouped-projection digests encode `TruthSnapshotIdentity.as_str()` into canonical digest bytes, treating typed snapshot identity as arbitrary display text. | Fixed | Canonical digest basis encodes typed relational snapshot parts and typed relational row identities instead of bridge payload text |
| 3 | worth-relational | `facade.rs` | Public `bridge` facade re-exports the string-collapsing bridge helpers as the supported relational bridge API. | Fixed | Public bridge helpers now expose typed relational signatures and typed snapshot identity constructors |
| 3 | worth-runtime-bridge | `relational_identity.rs`, `input/envelope/canonical.rs`, `snapshot/packet.rs` | Relational bridge export needed a typed carrier through bridge patch items and snapshot reads, otherwise relational consumers would keep parsing `entity_identity()` text. | Fixed | Bridge carries `RelationalBridgeRecordIdentityParts` beside compatibility entity text for Phase 3 ordinary relational paths; Phase 4 owns removing/further sealing generic bridge storage |
| 3 | worth-runtime-bridge | `routing/surfaces.rs`, `routing/lowering/slices.rs`, `routing/planning/canonical.rs`, `continuity/requests/prior_slice.rs`, `facade/runtime/continuity_planning.rs` | Route planning and continuity planning previously carried only erased entity text into snapshot reads and prior slices. | Fixed | Planned route, subscription slice, snapshot read, and continuity prior-slice surfaces preserve typed relational record parts through the relational spine |
| 4 | worth-runtime-bridge | `src/identity.rs` | Generic `BridgeIdentity<Tag>` stores only `Arc<str>` and permits arbitrary text construction/exposure for every truth identity tag. | Fixed | Core constructor/exposure are crate-private, debug is opaque, and equality/hash/order use typed payload semantics instead of display text when typed payloads exist |
| 4 | worth-runtime-bridge | `input/envelope/core.rs`, `snapshot/token.rs` | Truth identity type aliases inherit `BridgeIdentity<Tag>` public text constructors rather than nominal constructors from relational artifacts. | Fixed | Public truth identity construction is via typed relational bridge constructors/extractors; public string mint/access is compile-fail guarded |
| 4 | worth-runtime-bridge | `src/source/async_declaration/writeback/staging.rs` | Async writeback staging falls back to `TruthCommitIdentity::new("bridge-async-writeback-missing-truth")` and `TruthSnapshotIdentity::new("bridge-async-writeback-missing-snapshot")` in production staging when admitted writeback input omits truth identities. | Fixed | Staging now fails closed on missing authoritative truth basis instead of minting placeholder truth identities |
| 4 | worth-runtime-bridge | `src/facade/exports_core.rs` | Facade re-exports truth identity aliases and request/standard-path types so downstream crates can keep constructing truth IDs from strings. | Fixed | Facade can still name typed truth handles but downstream crates cannot mint/expose them through string constructors/accessors |
| 4 | worth-runtime-bridge | `src/facade/tests/`, `src/harness/`, `src/builder/tests/`, `src/input/envelope/construction_tests.rs` | Bridge-internal tests and harnesses construct `Truth*Identity::new("...")` and `Truth*Identity::new(format!(...))` as ordinary fixture setup. | Fixed | Ordinary bridge fixtures use typed fixture/relational helpers and typed in-memory source keys; the only raw constructor is a named malformed validation artifact quarantine |
| 5 | worth-query | `runtime/backend/contracts.rs` | Phase 2 removed the public `snapshot_token() -> String` and declaration-initialization `snapshot_token: &str` gates; Phase 5 must replace stale adapter implementations and call sites with typed snapshot routing rather than restoring string methods. | Fixed | Runtime internals now route through `current_snapshot_identity()`; the default is an explicit unavailable authority marker, while concrete test/runtime backends that own state provide typed snapshot evidence. Text projection remains only at Phase 6-owned compatibility surfaces |
| 5 | worth-query | `runtime/backend/contracts.rs::WORTHQueryRuntimeSignalSinkAdapter` | Signal sink adapter default methods build `SignalInvalidationRoutingReceipt::from_mutation_receipt(receipt)` and `SignalInvalidationBoundaryReceipt::from_mutation_receipt(...)`, making string commit/snapshot receipt routing the ordinary adapter fallback. | Fixed | Default routing now returns `Result` and fails closed unless the mutation receipt carries bridge-authored authority |
| 5 | worth-query | `runtime/backend/bridge_backed.rs` | Bridge-backed backend still contains stale call sites such as `&self.snapshot_token()` after Phase 2 removed the string trait method; Phase 5 must route typed snapshot authority instead. | Fixed | Bridge-backed backend no longer implements or calls the erased snapshot-token trait method; bridge-backed assembly now requires the typed `WORTHQueryRuntimeSnapshotIdentityAdapter` seam instead of silently installing an unavailable snapshot authority |
| 5 | worth-query | `runtime/tests/support/adapters/`, `runtime/tests/support/stateful_bridge_runtime/`, bridge-backed assembly fixtures | Adapter fakes and fixtures construct `Truth*Identity::new("...")`, return string snapshot tokens, and compare receipt string fields; known feeder paths include `runtime/tests/support/stateful_bridge_runtime/` and bridge-backed fixture adapters. | Fixed | Phase 5 adapter fixtures no longer restore deleted adapter snapshot-token methods or string-fed bridge-authority helper calls; stale-basis proof now uses backend-owned typed snapshot authority instead of source-adapter folklore, write-path signal routing rejects authority-less receipts, and broader harness receipt/envelope folklore remains explicitly owned by Phase 9 rows |
| 6 | worth-query | `basis/mod.rs` | `ResolvedSnapshotIdentity` stores `snapshot_token: String`, exposes `snapshot_token() -> &str`, and hashes `format!("snapshot:{}", self.snapshot_token)`. | Fixed | Resolved basis proof now derives from `ResolvedSnapshotIdentity` typed evidence identity and `BasisDigest::from_evidence_identity(...)`; no production `BasisDigest::from_parts(...)` remains |
| 6 | worth-query | `query_basis_lifecycle/intent.rs` | `RawBasisSelector` carries branch, snapshot, commit, and preview identities as `String`, and `compute_raw_digest` formats `commit_identity:*` / `snapshot_identity:*`. | Fixed | Raw basis selectors carry `RawBasisIdentity` typed handles backed by `WORTHQueryEvidenceIdentity` or `BridgeIdentityEvidence`; raw digest composition uses `WORTHQueryEvidenceIdentity` scope fields |
| 6 | worth-query | `query_basis_lifecycle/binding.rs` | Bridge lower-runtime evidence references store record, selector, route, continuity, subscription, and snapshot identities as `String` and format them into digest parts. | Fixed | Bridge lower-runtime evidence references store `BridgeIdentityEvidence`; binding digest composition is isolated in `binding_evidence.rs` and composes typed evidence identity |
| 6 | worth-query | `memory_workspace/workspace.rs` | `snapshot_token()` returns `String`, receipts use `format!("commit-*")`, and delete/update APIs accept `entity_identity: &str`. | Fixed | Memory workspace now exposes `snapshot_identity() -> WORTHQuerySnapshotIdentity`, receipts carry `WORTHQueryCommitIdentity`/`WORTHQuerySnapshotIdentity`, and update/delete APIs require `WORTHQueryEntityIdentity` |
| 6 | worth-query | `memory_workspace/runtime_identity.rs` | `snapshot_token_from_runtime` stringifies `TruthSnapshotIdentity`, uses a string sentinel for empty state, formats entity IDs as `entity:*`, and parses entity strings back to `EntityId`. | Fixed | Runtime identity helpers now construct typed relational snapshot/entity handles and recover native `EntityId` from typed relational record parts |
| 6 | worth-query | `runtime/surface/mutation/write_receipt/mod.rs` | `WORTHQueryWriteReceipt` wraps `WORTHQueryMutationReceipt` and stores declared/target entity identity as `Option<String>`. | Fixed | Write receipt wraps typed mutation receipts, caches typed evidence identities, and stores declared/target entity handles as `WORTHQueryEntityIdentity` |
| 6 | worth-query | `runtime/surface/mutation/write_receipt/accessors.rs` | Public `commit_identity() -> &str`, `snapshot_token() -> &str`, `declared_entity_identity() -> Option<&str>`, and `target_entity_identity() -> Option<&str>` expose truth identity as text. | Fixed | Write receipt accessors expose `WORTHQueryCommitIdentity`, `WORTHQuerySnapshotIdentity`, `WORTHQueryEntityIdentity`, and explicit evidence identities instead of string truth IDs |
| 6 | worth-query | `runtime/surface/mutation/write_receipt/helpers.rs` | Retained assertion evidence takes `snapshot_token: &str`, so assertion verification is still keyed by an erased snapshot token. | Fixed | Existing-truth assertion verification now passes `WORTHQuerySnapshotIdentity` into `WORTHQueryVerifiedExistingTruthAssertion` and `WORTHQueryVerifiedAssumptionSet` |
| 6 | worth-query | `runtime/surface/mutation/write_receipt/preview.rs` | Preview write receipts accept `snapshot_token: String`, synthesize `preview_write_receipt_identity(...)` strings as commit/entity identities, and copy binding resolved entity identities by `to_string()`. | Fixed | Preview write receipts now accept typed snapshot evidence, compose `WORTHQueryCommitIdentity::preview`, and build preview/entity delta handles through `WORTHQueryEntityIdentity` |
| 6 | worth-query | `runtime/surface/mutation/batch_receipt.rs` | Batch receipts derive `write_commit_identity` by iterating `WORTHQueryWriteReceipt::commit_identity` and seal a string-derived batch digest. | Fixed | Batch receipt digest composes `WORTHQueryEvidenceScope::BatchWriteReceipt` from each component receipt's typed commit evidence identity |
| 6 | worth-query | `runtime/surface/mutation/command.rs` | Mutation commands store insert/update/delete `entity_identity` and verification `resolved_entity_identity` as `String`, then expose declared entity identity as owned text. | Fixed | Mutation command entity fields and declared identity accessors use `WORTHQueryEntityIdentity`; graph/batch builders require typed entity handles |
| 6 | worth-query | `runtime/mutation/binding/existing_truth.rs` | Existing-truth bindings store resolved entity identity as `String` and seal binding/denial digests from formatted authoritative/resolved/collection text. | Fixed | Existing-truth target/binding constructors require `WORTHQueryEntityIdentity`; resolved target accessors return typed handles and digests use typed evidence projection |
| 6 | worth-query | `runtime/shared_read.rs`, `runtime/shared_read_pins/`, `runtime/runtime_authoritative_mutation_routing.rs`, `runtime/read_composition_runtime.rs` | Shared read/generation paths capture, retire, and compare `snapshot_token` strings from write receipts and runtime snapshot tokens, including generation digests formatted as `shared-read-generation:{snapshot_token}`. | Fixed | Shared-read generation IDs, registry capture/retire, stale-basis checks, authoritative mutation routing, and read-composition runtime materialization checks now carry `WORTHQuerySnapshotIdentity`; generation digests use `WORTHQueryEvidenceScope::SharedReadGeneration` |
| 6 | worth-query | `runtime/error.rs::SharedReadStaleBasis` | Shared-read stale-basis errors carry `snapshot_token: String` and render that erased snapshot token into runtime error messages. | Fixed | `SharedReadStaleBasis` carries `WORTHQuerySnapshotIdentity`, and stale-basis construction preserves the captured typed handle |
| 6 | worth-query | `runtime/surface/live_read_receipt.rs`, `runtime/surface/unified_inspection_receipt.rs`, `runtime/surface/derived_inspection_receipt.rs`, `runtime/surface/derived_materialization_receipt.rs`, `runtime/surface/existing_truth_probe_receipt.rs` | Read, inspection, materialization, and probe receipts store `snapshot_token: String` and expose it through `snapshot_token() -> &str`. | Fixed | Receipt spine now stores typed snapshot identity/evidence handles and exposes typed snapshot evidence; Phase 7 intent/inspection adapters remain separately tracked |
| 6 | worth-query | `runtime/surface/live_artifact_bundle.rs`, `runtime/surface/derived_materialization_bundle.rs`, `runtime/workspace_queries.rs` | Live and derived bundles store `snapshot_token: String`; workspace query aggregation collects `receipt.snapshot_token().to_string()` from read results and builds bundle digests from `format!("snapshot:{snapshot_token}")`. | Fixed | Live/derived bundles retain `WORTHQuerySnapshotIdentity`; bundle digests compose typed snapshot evidence instead of formatted token text |
| 6 | worth-query | `runtime/surface/read_receipt_construction.rs`, `runtime/surface/read_composition.rs` | Read receipt construction and composition pass `snapshot_token: String`, expose snapshot tokens by `&str`, and mix snapshot text into read/composition digests. | Fixed | Read construction/composition routes typed snapshot identity through read receipts and removes `snapshot_token()` compatibility accessors |
| 6 | worth-query | `runtime/surface/verified_assumption_set.rs` | Verified assumptions accept `snapshot_token: &str`, store `assumption_snapshot_token: String`, and seal assumption digests from formatted snapshot text. | Fixed | Verified assumptions store `WORTHQuerySnapshotIdentity`, cache only canonical evidence projection for legacy access, and derive assumption snapshot digest from the typed evidence identity |
| 6 | worth-query | `runtime/runtime_reads_programs.rs` | Runtime read programs expose `snapshot_token(&self) -> String`, record write receipt commit identities as strings, and format replay/live/derived trace identifiers from text. | Fixed | Runtime snapshot token API removed; traces record typed commit identities and only project evidence strings at trace-label boundaries |
| 6 | worth-query | `projection_consumption/extraction/write_receipt.rs`, `projection_consumption/extraction/mod.rs` | Projection consumption validates source identity against `receipt.commit_identity()`, records write fact source identity as `receipt.commit_identity().to_string()`, copies resolved target entity identity by `str::to_string`, and passes read receipt `snapshot_token()` text into extraction context. | Fixed | Write/read extraction validates against typed commit/snapshot evidence, carries target/resolved identities as `WORTHQueryEntityIdentity`, and removes receipt snapshot-token extraction |
| 6 | worth-query | `projection_consumption/source/constructors.rs`, `projection_consumption/source/mod.rs`, `projection_consumption/consumed/facts.rs`, `projection_consumption/contracts.rs` | Projection source and consumed-fact contracts preserve `source_identity`, `entity_identity`, and receipt-derived identities as strings, then seal projection contract/fact digests from formatted source/entity identity text. | Fixed | Projection source identities use typed source handles; consumed entity/target/relation identity facts carry typed entity handles and digest from typed evidence identities |
| 6 | worth-query | `runtime/computed/surface.rs`, `runtime/computed/routing.rs`, `runtime/computed/refresh_context.rs` | Computed view patches and refresh contexts store `commit_identity` / `snapshot_token` as `String` and bind receipt commit strings into derived patch state. | Fixed | Computed patch/refresh surfaces retain `WORTHQueryCommitIdentity` and typed snapshot evidence; no Phase 6 `commit_identity: String` / `snapshot_token: String` remains |
| 6 | worth-query | `runtime/effect/delivery.rs`, `runtime/effect/routing.rs`, `runtime/runtime_intents.rs` | Effect deliveries and routing compare or clone receipt `commit_identity` strings into delivery state and pending intent matching. | Fixed | Effect delivery/routing uses typed commit identities for matching and delivery state; no receipt commit string clone remains in the Phase 6 slice |
| 6 | worth-query | `runtime/delivery.rs`, `runtime/state.rs` | Delivery and runtime state format patch identities and status details from receipt `commit_identity`, `declared_entity_identity`, and downstream string digests. | Fixed | Delivery/state surfaces route typed receipt/entity identities and reserve formatted details for display-only diagnostics |
| 6 | worth-query | `runtime/workspace.rs`, `runtime/workspace_submission.rs`, `runtime/runtime_declarations.rs` | Workspace-facing mutation APIs accept `entity_identity: impl Into<String>` and expose `snapshot_token() -> String` through runtime boundaries. | Fixed | Workspace mutation APIs require typed entity identities and runtime/workspace `snapshot_token()` APIs were removed |
| 6 | worth-query | `declarative_live.rs` | Public declarative live query session APIs accept `snapshot_token: impl Into<String>` and feed that erased token into `ResolvedSnapshotIdentity::new(...)` for live session basis declaration. | Fixed | Declarative live basis intake no longer accepts erased snapshot token strings in the Phase 6 surface |
| 7 | worth-query | `intent_admission/handoffs/bindings/mod.rs` | Intent handoff bindings store `trigger_commit_identity: String`, expose it as `&str`, compare it against pending delivery commit text, and seal handoff binding digests with formatted `commit:{pending_delivery.commit_identity()}` parts. | Fixed | Effect-triggered execution binding stores `WORTHQueryCommitIdentity`, compares typed commit handles, and hashes pending delivery from commit evidence identity |
| 7 | worth-query | `intent_admission/eligibility/seeds/generic_inspection.rs` | Generic inspection seeds build inspection labels and seed digests from `receipt.commit_identity()` / `receipt.snapshot_token()` text for write receipts and other receipt-derived admission evidence. | Fixed | Generic inspection seeds compose `GenericInspectionIntentSeed` evidence identities and use receipt/commit/snapshot evidence handles instead of receipt token text |
| 7 | worth-query | `intent_admission/eligibility/seeds/mutation.rs` | Mutation admission seeds format declared entity identity text as `entity:{declared_entity_identity}` and include binding/resolved symbolic identity strings in authoritative mutation intent input digests. | Fixed | Mutation intent and batch seed digests compose dedicated evidence identity scopes; hostile delimiter tests cover typed entity evidence and component seed composition |
| 7 | worth-query | `application/declaration_bridge_routing/lower.rs` | Declaration bridge lowering mints `TruthBranchIdentity`, `TruthCommitIdentity`, and `TruthSnapshotIdentity` from formatted query declaration/basis digests such as `query-branch:*`, `query-commit:*`, `query-snapshot:*`, and passes them into `BridgeTruthViewSelector` / `BridgeRouteRequest`. | Fixed | Declaration bridge lowering uses bridge-owned typed truth constructors and stable numeric relational commit/snapshot identity derivation instead of raw truth string constructors |
| 7 | worth-query | `application/declaration_bridge_routing/lower.rs::lower_writeback_declaration` | Writeback declaration lowering builds `TruthCommitIdentity::new(format!("query-trigger:{...}"))` and `TruthSnapshotIdentity::new(...)` from query causality/evaluation/basis digests before bridge writeback execution. | Fixed | Writeback declaration lowering routes through typed causality/route identities plus typed query truth commit/snapshot helpers; declaration bridge routing tests cover the path |
| 7 | worth-query | `effect_lifecycle/execution_bridge.rs` | Effect lifecycle bridge execution constructs `TruthCommitIdentity` from causality digest text and `TruthSnapshotIdentity` from evaluation snapshot / basis digest strings before calling `RuntimeBridge::execute_admitted_writeback`. | Fixed | Effect lifecycle bridge writeback execution now uses typed policy/causality/route identity constructors and stable typed truth commit/snapshot derivation |
| 7 | worth-query | `effect_lifecycle/execution_relational_scalar.rs` | Relational scalar execution compares expected `runtime_snapshot_token()` text against `current_branch_snapshot_token()` and parses branch IDs from binding digest text for freshness checks. | Fixed | Workflow mutation bindings now carry typed `WORTHQuerySnapshotIdentity` and `BranchId`; scalar execution consumes those typed handles directly, removes binding-digest branch parsing, and regression tests cover the boundary plus branch-scoped execution |
| 7 | worth-query | `continuation_pipeline/execution/readmission.rs` | Continuation readmission copies `request.commit_identity().to_string()` and bridge selector commit/snapshot identities via `.as_str().to_string()` into `WORTHQueryPreparedContinuationBasisWitness`. | Fixed | Prepared continuation basis witnesses and readmission observations now retain typed `WORTHQueryEvidenceIdentity` handles; bridge commit/snapshot paths derive typed Query evidence identities and tests guard against digest-string helpers returning |
| 7 | worth-query | `runtime/bridge_mutation_lowering.rs` | Bridge lowering APIs accept `resolved_target_entity_identity: Option<&str>` and rebuild continuity/naming evidence from string target identities. | Fixed | Bridge mutation lowering accepts typed `WORTHQueryEntityIdentity`, lowers only relational-record handles through `BridgeHistoricalResolvedRecordIdentity::from_relational_record`, rejects authored Query evidence strings for native bridge target slots, and regression tests cover both paths |
| 7 | worth-query | `runtime/surface/naming_mutation_evidence.rs` | `from_bridge` and `from_intent` copy attachment, prior/target authoritative, resolved entity, and collection identities into `String` fields with `.to_string()`. | Fixed | Naming evidence now stores typed mutation authority/collection handles plus typed resolved entity identity; bridge evidence is enriched with query-native typed target context when bridge-native identity cannot encode it |
| 7 | worth-query | `runtime/surface/continuity_mutation_evidence.rs` | `from_bridge` copies prior/successor authoritative identities, resolved target entity identity, lineage, and continuity digests into `String` fields; `from_intent` still hashes string successors. | Fixed | Continuity evidence now stores typed authority, target collection, resolved entity, and mutation evidence digest handles; intent digests compose `WORTHQueryEvidenceIdentity` instead of `hash_parts(...)` |
| 7 | worth-query | `runtime/surface/symbolic_target_reference_evidence.rs` | Symbolic target reference evidence stores `symbol`, `resolved_entity_identity`, and optional collection as `String` copied from bridge/reference inputs. | Fixed | Symbolic target reference evidence stores typed symbol, resolved entity, and collection handles; bridge bundles require a typed query-context fallback rather than accepting raw resolved-identity strings |
| 7 | worth-query | `runtime/surface/symbolic_aspect_resolution_evidence.rs` | Symbolic aspect resolution evidence stores `resolved_entity_identity: String` via `impl Into<String>` and exposes it as `&str`. | Fixed | Symbolic aspect resolution evidence stores typed symbol/collection handles and a `WORTHQueryEntityIdentity`; batches with symbolic aspect references use query-side typed resolution instead of backend atomic string resolution |
| 7 | worth-query | `runtime/surface/graph_composition_resolution_map.rs`, `runtime/surface/graph_composition_evidence.rs` | Graph composition resolution maps store `resolved_entity_identity: String`, then graph composition evidence seals symbolic-resolution digests from formatted `entry.resolved_entity_identity()` text. | Fixed | Graph composition resolution maps retain typed symbols, target collections, and entity identities; graph symbolic-resolution digests compose typed evidence identities |
| 7 | worth-query | `runtime/surface/mutation_evidence/binding.rs`, `runtime/surface/mutation_evidence/target.rs` | Mutation evidence binding/target artifacts store authoritative and resolved entity identities as `String` copied from bridge binding bundles and expose those values through `&str` accessors. | Fixed | Mutation binding/target evidence stores typed authority, collection, digest, and entity handles and exposes typed accessors; display strings are explicit edge projections only |
| 7 | worth-query | `runtime/surface/mutation_evidence/provenance.rs`, `runtime/surface/mutation_evidence/causality.rs` | Mutation provenance/causality evidence copies bridge contract, writeback, feedback, causality, route, evaluation, and truth-view digests into string fields with `to_string()`. | Fixed | Mutation provenance/causality evidence stores `WORTHQueryMutationEvidenceDigest` handles for bridge digest inputs and exposes typed digest accessors |
| 7 | worth-query | `runtime/surface/mutation_evidence/batch_digest_helpers.rs` | Batch mutation-evidence digest helpers format declared/resolved entity identity text from target, binding, symbolic reference, continuity, and naming evidence into batch digest parts. | Fixed | Batch mutation-evidence helpers compose aggregate `WORTHQueryEvidenceIdentity` values from typed target, binding, symbolic, naming, continuity, provenance, and causality evidence handles |
| 7 | worth-query | `runtime/inspection/unified/write_receipt.rs`, `runtime/inspection/unified/component.rs` | Write receipt inspections copy `commit_identity`, `snapshot_token`, and entity identities into string fields. | Fixed | Write receipt/component inspections retain typed commit, snapshot, declared entity, and target entity handles; text projection is limited to explicit evidence/reporting edges |
| 7 | worth-query | `runtime/inspection/unified/batch_write.rs` | Batch write receipt inspection collects `commit_identities: Vec<String>` from `entry.commit_identity().to_string()` and preserves batch receipt identity as text in the inspection artifact. | Fixed | Batch write inspection retains typed commit identity handles across entries and components instead of string collections |
| 7 | worth-query | `runtime/inspection/unified/write_receipt/digest.rs`, `runtime/inspection/unified/batch_write_digest.rs` | Digest components compose identity fields named `commit_identity`, `snapshot_token`, and `entity_identity` from string accessors. | Fixed | Write/batch receipt digest helpers compose typed receipt evidence identities and only flatten values inside explicit evidence encoder sequences |
| 7 | worth-query | `runtime/intent/branch.rs`, `runtime/inspection/intent.rs` | Authoritative/effect/preview intent receipt inspection identities now compose typed receipt, commit, snapshot, trigger-commit, basis, and admission identities; the remaining intent inspection gap is branch/basis snapshot routing that still carries `basis_snapshot_token` text and adjacent non-receipt inspection surfaces that still lower typed identity too early. | Fixed | Branch intent receipts and inspections carry typed basis snapshot identities and compose branch/basis inspection evidence through `field_evidence_identity` |
| 7 | worth-query | `runtime/inspection/causal/receipt.rs` | Causal inspection receipts copy `inspection.commit_identity()` and `inspection.snapshot_token()` into evidence references/tags as text rather than retaining typed write/read receipt handles. | Fixed | Causal write receipt consumers use typed commit, snapshot, and entity evidence identities until the bridge evidence-reference boundary |
| 7 | worth-query | `runtime/inspection/causal/materialization/` | Causal materialization receipts and proofs seal query admission, anchor, bridge receipt, and materialization identities through string-formatted digest parts, leaving no typed bridge truth handle boundary for receipt-derived evidence. | Fixed | Causal materialization receipt, proof, reference, performance, denial, and temporal digests compose `WORTHQueryEvidenceIdentity`; materialization fixtures preserve requested typed snapshot identity |
| 7 | worth-query | `runtime/intent/receipt.rs`, `runtime/intent/effect_triggered.rs` | Intent receipts now carry typed commit/snapshot evidence identities and effect-triggered receipts compose the nested authoritative intent receipt plus typed trigger commit evidence identity instead of copying write/effect receipt identity strings. | Fixed | Intent route receipts carry typed receipt identity |
| 7 | worth-query | `runtime/intent/provenance.rs`, `runtime/intent/provenance_identity.rs`, `runtime/intent/receipt.rs`, `runtime/intent/receipt_identity.rs`, `runtime/intent/effect_triggered.rs`, `runtime/inspection/preview/intent_receipt.rs`, `runtime/inspection/preview/intent_receipt_identity.rs` | Intent provenance now accepts typed snapshot evidence identities for authoritative/effect-triggered write-backed receipts, shared snapshot-token callers must pass through an explicit typed evidence adapter, authoritative/effect-triggered intent receipt digests compose nested write receipt and provenance identities, and preview intent receipt inspection digests compose typed basis/admission/receipt identities. | Fixed | Intent provenance and receipt identity boundary |
| 7 | worth-query | `runtime/intent/denial.rs` | Intent denial evidence clones `execution.mutation_receipt().snapshot_token` into `Option<String>`, exposes it as `Option<&str>`, and includes it as a `snapshot_token` evidence identity tag. | Fixed | Denial evidence and denial inspection retain `WORTHQuerySnapshotIdentity` plus typed snapshot evidence identity; no `snapshot_token()` denial accessor remains |
| 7 | worth-query | `runtime/intent/failure.rs` | Intent execution failure evidence clones `execution.mutation_receipt().snapshot_token` into a `String`, exposes it as `&str`, and seals failure digests with formatted `snapshot:{snapshot_token}` text. | Fixed | Failure evidence stores typed snapshot identity/evidence and composes `IntentExecutionFailureEvidence` instead of formatted snapshot text |
| 7 | worth-query | `runtime/intent/execution.rs` | Intent execution placeholder outcomes synthesize `WORTHQueryMutationReceipt` with `commit_identity: String::new()` and `snapshot_token: snapshot_token.into()` for invariant-violation executions. | Fixed | Placeholder/noop/invariant execution constructors require typed commit/snapshot handles at the boundary and no longer mint receipt identity from strings |
| 7 | worth-query | `runtime/inspection/feedback.rs` | Feedback inspection stores `trigger_commit_identity: String`, accepts trigger commit as `&str`, exposes it as `&str`, and seals feedback graph digests from formatted `trigger-commit:{trigger_commit_identity}`. | Fixed | Feedback inspection carries typed trigger commit evidence identity through graph and inspection identities; effect intent inspection asserts the wrapper does not collapse to write commit identity |
| 7 | worth-query | `runtime/backend/receipts.rs` | `SignalInvalidationRoutingReceipt` stores `commit_identity`/`snapshot_token` as `String`, formats digest inputs as `commit:{commit_identity}` and `snapshot:{snapshot_token}`, and compares against string receipt fields. | Fixed | Signal invalidation routing receipt stores and drift-checks typed commit/snapshot handles, exposes only typed `receipt_identity()`, and the lower-runtime signal boundary consumes that typed identity rather than a receipt-digest string accessor |
| 7 | worth-query | `runtime/runtime_writes.rs` | Runtime writes use `backend.snapshot_token()` for synthetic receipts and pass `&receipt.commit_identity` / `&receipt.snapshot_token` into intent execution provenance. | Fixed | Runtime writes now feed synthetic assertion receipts from `current_snapshot_identity()` and preserve typed commit/snapshot evidence through write provenance |
| 7 | worth-query | `runtime/runtime_read_intents.rs`, `runtime/runtime_unified_inspection_intents.rs` | Runtime read and unified-inspection intent routers clone `backend.snapshot_token()` into read/inspection receipts and propagate receipt `snapshot_token()` text into intent evidence. | Fixed | Live read/materialized posture now receives typed snapshot evidence identity; unified inspection surfaces are clean under the row erasure scan |
| 7 | worth-query | `runtime/runtime_sessions.rs` | Runtime session setup passes `backend.snapshot_token()` into session basis and subscription setup, carrying erased snapshot text across session boundaries. | Fixed | Runtime session basis setup carries `WORTHQuerySnapshotIdentity`; row scan found no remaining snapshot-token projection in sessions |
| 7 | worth-query | `runtime/effect/inspection.rs` | Effect inspection formats `delivery.commit_identity()` into feedback-phase inspection digests, preserving trigger commit identity as text in effect inspection evidence. | Fixed | Effect inspection identity composition moved to `runtime/effect/inspection_identity.rs` and composes delivery trigger commit evidence identity directly |
| 7 | worth-query | `runtime/preview/mod.rs`, `runtime/preview/basics.rs`, `runtime/preview/workflow_ops.rs`, `runtime/preview/session_execution.rs`, `runtime/preview/mutation_ops.rs` | Preview sessions store `basis_snapshot_token: String`, compare promotion snapshot token strings, create preview write receipts from `runtime.snapshot_token()`, and record preview trace write receipts from `receipt.commit_identity().to_string()`. | Fixed | Preview route, promotion, closeout, and execution paths carry typed snapshot/source evidence identities; trace write receipt recording keeps typed `WORTHQueryCommitIdentity` |
| 7 | worth-query | `preview/scoped.rs` | Scoped preview inspection routes preview observation basis through `basis().identity().snapshot_token()` and uses erased snapshot text as the scoped basis label. | Fixed | Scoped preview now derives `RawBasisIntent::runtime_snapshot` from typed snapshot identity and compares typed `NormalizedBasisSubject`, not scope-label text |
| 7 | worth-query | `runtime/preview/evidence/promotion.rs`, `runtime/preview/evidence/closeout.rs`, `runtime/preview/evidence/execution.rs` | Preview evidence artifacts store basis/promotion snapshot tokens and preview commit identity as `String` and seal them into evidence identities. | Fixed | Promotion/closeout/execution evidence store typed snapshot and source evidence identities; token/commit-string accessors were removed |
| 7 | worth-query | `runtime/inspection/preview/outcome.rs` | Preview outcome inspection stores preview and target basis snapshot tokens as `String`, exposes them as `&str`, and includes them as evidence identity tags. | Fixed | Preview outcome inspection stores typed basis, closeout, residue, rebinding, and snapshot identities; digest getters are report projections only |
| 7 | worth-query | `runtime/branch.rs`, `runtime/intent/branch.rs` | Branch and intent branch sessions store basis snapshot tokens as `String` and expose them by `&str`. | Fixed | Branch basis and branch intent receipts carry typed `WORTHQuerySnapshotIdentity` and typed receipt/evidence identities |
| 7 | worth-query | `runtime/runtime_batch_writes.rs`, `runtime/runtime_helpers.rs`, `runtime/runtime_probe_routing_intents.rs`, `runtime/runtime_inspection_materialization_intents.rs` | Runtime helper paths synthesize aggregate commit identities with `format!` from child receipt commit strings and propagate string snapshot tokens through batch/probe/materialization flows. | Fixed | Batch/helper/materialization flows compose typed commit/snapshot identities; materialization bundle consistency compares typed snapshot handles and helper budget digest uses `RuntimeSubscriptionBudget` evidence scope |
| 7 | worth-query | `view_shape_live/grouped_execution.rs` | Fixed: grouped execution compares bridge snapshot identity through `WORTHQueryEvidenceIdentity`, grouped bridge row-set materialization preserves typed relational record identity for projection parity, and grouped fixtures derive query basis identity from the same typed bridge snapshot instead of matching display labels. | Fixed | View-shape grouped execution snapshot boundary |
| 7 | worth-query | `lower_runtime_routing/adapters/runtime_backend.rs`, `lower_runtime_routing/plans/mod.rs` | Fixed: write-authority lower-runtime routing binds mutation commit evidence as `WORTHQueryEvidenceIdentity`, signal invalidation subjects compose from typed routing receipt/commit/snapshot identities, `WORTHQueryLowerRuntimeCapabilityRequest` requires `WORTHQueryLowerRuntimeSubjectIdentity`, and `WORTHQueryLowerRuntimeRoutePlan` now requires `WORTHQueryLowerRuntimeRouteSubjectIdentity` instead of accepting raw route-subject strings. | Fixed | Production lower-runtime routing adapter |
| 7 | worth-runtime-bridge | `src/diagnostics/records/route_entry.rs`, `src/diagnostics/state/` | Fixed: route diagnostics now carry `BridgeRouteRecordEntityIdentity` (`RelationalRecord` or `TruthSurface`) with a hard-broken constructor/accessor API; route diagnostic state indexes route, invalidation, continuity, and source commit lookups by typed bridge/truth identities; JSON export performs explicit diagnostic-label projection instead of treating canonical identity as `String`. | Fixed | Adjacent bridge diagnostic feeder |
| 7 | worth-query | `subscription/`, `runtime/live_subscription.rs`, `runtime/runtime_sessions.rs`, `runtime/backend/receipts.rs` | Subscription activation/live-installation/runtime-session feeders store activation, declaration, basis, signal, support, counter, and budget identities as strings or digest-only wrappers before feeding runtime receipt and inspection surfaces. | Fixed | Active lane signal strategy now carries `WORTHQueryEvidenceIdentity`, continuation admission now requires typed source/target/basis/checkpoint/authority evidence, and raw continuation identity substitution has compile-fail coverage. Bridge-parity manual witness now carries typed query declaration, bridge declaration, basis, signal, and activation identities; bridge-parity width, receipt, counters, failure, comparison, and explanation artifacts now compose `WORTHQueryEvidenceIdentity` values, with digest-named accessors demoted to reporting projections. Runtime certification scope compares support/parity/lifecycle declaration and bridge source identities with typed drift checks; runtime certification bundle, coverage width/receipt, hostile coverage evidence, certification counters, coverage rows, coverage matrices, variation sets, certified coverage handles, and bridge-parity links now carry typed evidence identities where upstream artifacts expose them. Lifecycle certification now preserves view-shape and basis-posture identities into the bundle, coverage rows store typed basis/view-shape identities, and coverage variation sets compose typed evidence identity sequences instead of digest string sets. Runtime subscription activation adapters now return typed support evidence identities, and `SubscriptionActivationReceipt::from_activation` no longer re-admits support from raw strings. Runtime live subscription budget verification now compares `WORTHQueryEvidenceIdentity` values directly; the legacy-named test helper no longer returns a raw budget string. Preview lifecycle certification now compares closeout/isolation/discard/handoff basis, checkpoint, preview epoch, and source identities with typed drift checks instead of `*_for_reporting()` strings. Lifecycle certification context policy, tenant-basis, relationship-proof, view-shape, and basis-posture values now carry typed evidence identities into bundle composition. Preview discard/promotion closeout and certification evidence now store residue/isolation/performance as typed identities, with `*_digest()` accessors demoted to reporting projections and compile-fail fixtures updated for the removed raw residue field. Active lane admission and opened lane records now retain typed activation/admission/query-declaration/bridge-declaration identities, and lifecycle certification source coherence uses typed drift checks rather than digest equality. Scale counter snapshots and scale slope reports now retain typed activation/admission/counter/snapshot identities, compose slope identity with `field_evidence_identity`, and certification validates scale source coherence via typed drift checks. Activation/admission query declaration coherence and runtime family coverage support/parity/lifecycle source alignment now use typed evidence drift checks instead of projection string equality. Diagnostic evidence/stage traces now retain typed source identities, no longer rebuild trace sources from projection text, and admitted/denied/runtime-certification trace source validators compare typed identity values. Admitted and denied diagnostic bundles now retain typed support-report and lifecycle-certification references, and runtime certification scope/coverage validation uses typed drift instead of digest text for those bundle references. Active lifecycle certification now requires a typed delivery-window witness, compares lane/attachment/delta/lowering/work-packet/delivery/acknowledgement/continuation/closeout sources with typed drift, and continuation reports retain typed checkpoint identity. Active registry lookup now indexes lanes by `ActiveSubscriptionLaneDigest` and attachments by `SubscriptionConsumerAttachmentDigest`; registry registration/closeout no longer admits `&str` keys, and remaining text access in that slice is terminal denial/reporting projection. Live-read result-shape authority now flows from declarative canonicalization into `WORTHQueryRuntimeLiveSubscriptionInstallation` and `WORTHQueryLiveReadReceipt` as `CanonicalResultShapeDigest` plus evidence identity; test-only live receipts no longer accept raw shape strings, and the legacy `view_shape_digest()` accessor projects the canonical digest rather than the live-view family identity. Live declaration admission receipts now retain typed `DeclarativeLiveViewShape` for drift checks, with shape text quarantined behind `view_shape_for_reporting()` and target collection exposed only as a declaration/reporting label. Active lifecycle closeout now maps attachment-not-active and attachment-lane-mismatch through explicit lifecycle denial kinds instead of comparing projected `source_digest()` text to attachment digest text. Subscription support profiles now require typed source evidence identities and compose profile source via `field_evidence_identity` instead of source digest text. Subscription support reports now retain typed source identity while keeping `source_digest()` terminal reporting-only. Runtime certification scope now matches admitted diagnostic bundles through typed identity drift checks, and coverage variation sets deduplicate evidence identities by scope/scheme/token rather than projection text alone. Runtime live subscription attachment requests now carry typed delivery cursor seed identity, and runtime installation passes activation receipt identity instead of activation reporting text. Latest focused gates: `cargo check -p worth-query --lib`, `cargo test -p worth-query projection_consumption::tests::retained_live --lib`, `cargo test -p worth-query runtime::tests::live --lib`, `cargo test -p worth-query runtime::tests::live_receipts --lib`, `cargo test -p worth-query subscription::tests::active::active_lifecycle --lib`, `cargo test -p worth-query subscription::tests::active::active_sharing --lib`, `cargo test -p worth-query subscription::tests::active::active_delivery --lib`, `cargo test -p worth-query runtime::tests::live --lib`, `cargo test -p worth-query subscription::tests::active::active_closeout --lib`, `cargo test -p worth-query subscription::tests::admission --lib`, `cargo test -p worth-query subscription::tests::diagnostics --lib`, `cargo test -p worth-query subscription::tests::support --lib`, `cargo test -p worth-query subscription::tests::runtime_certification --lib`, `cargo test -p worth-query subscription::tests::runtime_certification_closure_support --lib`, `cargo test -p worth-query subscription::tests::active::active_continuation --lib`, `cargo test -p worth-query subscription::tests::runtime_certification:: --lib`, `cargo test -p worth-query --test phase_boundaries_compile_fail`, `cargo test -p worth-query runtime::tests::session_label_outputs --lib`, and the final row 1051 targeted projection/digest re-entry scans. Closed after the final scan found no non-terminal subscription/runtime-session projection or digest re-entry in the targeted authority paths; remaining text hits are terminal/reporting accessor plumbing or typed attachment constructor calls. Runtime session lowering helpers were split into `runtime/runtime_session_lowering.rs`, keeping touched runtime session files under the workspace Rust line cap. |
| 7 | worth-query | `workflow/`, `workflow/lowering/`, `domain_capabilities/payloads/workflow_semantics.rs`, `domain_capabilities/authoring/workflow.rs`, `domain_capabilities/canonical_runtime/workflow/` | Workflow/domain-capability lowering stores authority binding, basis, causality, runtime-preflight scope, preview declaration, and target binding evidence as digest strings before lowering mutation/writeback/preview bridge evidence. | Fixed | `inspect_post_merge_outcome` now verifies declaration/outcome coherence through typed workflow query and basis identities rather than `query_for_reporting()`, `basis_for_reporting()`, or outcome source digest text. Domain capability materialization admission now compares typed target binding identities instead of `binding_digest()` text. Workflow inspection identity helpers were split into `workflow/inspection/identities.rs`, keeping touched workflow files under the Rust line cap. Latest focused gates: `cargo check -p worth-query --lib`, `cargo test -p worth-query workflow::tests::inspection --lib`, `cargo test -p worth-query domain_capabilities::canonical_runtime_workflow_inspection_tests --lib`, `cargo test -p worth-query domain_capabilities::canonical_runtime_support_workflow_tests --lib`, `cargo test -p worth-query domain_capabilities::materialization_tests --lib`, and targeted workflow/domain production scans for reporting/digest equality and composition. Digest/projection accessors remain terminal report-row projections only. |
| 7 | worth-query | `domain_capabilities/canonical_runtime/continuity.rs`, `domain_capabilities/canonical_runtime/support.rs`, `domain_capabilities/canonical_runtime/artifacts.rs`, `domain_capabilities/canonical_runtime/invariant_capability.rs`, `domain_capabilities/canonical_runtime/continuity_correspondence.rs`, `domain_capabilities/canonical_runtime/aftermath.rs`, `domain_capabilities/canonical_runtime/explanation.rs` | Canonical runtime materialization and support artifacts derive continuity/support/materialization/invariant identities from target binding digest strings and `hash_parts(...)` instead of typed contribution target evidence. | Fixed | Scoped canonical runtime lane composes typed target/binding/request/payload/materialization evidence. Follow-up scan no longer carries an open "remaining risk" note; any future projection accessor hit must open a new row with a named owner rather than living inside a Fixed row. |
| 7 | worth-query | `effect_lifecycle/` | Effect lifecycle normalization, lowering, batch lowering, and execution bridge paths preserve lower-runtime authority binding and workflow mutation/writeback evidence as strings before calling workflow lowering and bridge execution. | Fixed | Batch admission coherence now compares typed scoped-basis and expected lower-runtime binding identities through `NormalizedEffectIntent::scoped_basis_identity()` and `expected_lower_runtime_binding_identity()` instead of digest accessors. Latest focused gates: `cargo check -p worth-query --lib`, `cargo test -p worth-query effect_lifecycle::tests::batch::admission --lib`, `cargo test -p worth-query effect_lifecycle::tests::batch --lib`, and targeted `effect_lifecycle/` scans for the removed batch digest comparisons. Remaining bridge-oracle runtime diagnostic record digest matching is carried forward to the causal/bridge retained diagnostics rows rather than row 1054. |
| 7 | worth-query | `runtime/inspection/causal/`, `runtime/inspection/causal/materialization/` | Causal inspection request failures, materialization receipts/proofs, bridge references, and receipt-derived evidence can collapse typed query/bridge evidence into formatted strings or value sequences. | Fixed | Causal redaction certification now compares typed causal/artifact identities instead of reporting strings, representative matrix validation uses typed artifact-kind enums instead of string labels, and materialization tests compare Query observation identity to Query subject identity rather than bridge-exported evidence labels. Remaining proof-shape digest comparisons are terminal certification evidence. Latest focused gates: `cargo check -p worth-query --lib`, `cargo test -p worth-query runtime::tests::causal_inspection::certification --lib`, `cargo test -p worth-query runtime::tests::causal_inspection::materialization --lib`, and targeted causal projection/composition scans. |
| 7 | worth-runtime-bridge | `src/diagnostics/causal_envelope/`, `src/diagnostics/causal_envelope/retained_mapping/` | Fixed: retained mapping lookup adapters now revalidate retained bridge evidence into typed bridge identities before lookup; source/materialization, source failure, structural, stream checkpoint, stream replay, and replay-by-checkpoint diagnostic lookup APIs no longer accept raw `&str`; stream checkpoint authority uses `CheckpointTokenIdentity` with explicit terminal reporting projection. | Fixed | `cargo check -p worth-runtime-bridge --lib`, `cargo check -p worth-query --lib`, and `cargo test -p worth-runtime-bridge retained_mapping --lib` pass; retained-artifact scans found no remaining `from_reference_evidence(...).as_str()` lookup bridge, and the fixture compiler fallout exposed by the retained-mapping gate was replaced with validated typed fixture constructors rather than compatibility shims. |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters/write_authority.rs` | Production write authority builds `WORTHQueryMutationReceipt` with `format!("commit-*")` and `bridge_snapshot_identity_for_commit(...).as_str().to_string()` for single and batch writes. | Fixed | Write authority now builds mutation receipts from `WORTHQueryCommitIdentity` / `WORTHQuerySnapshotIdentity` derived from relational commit parts. Milestone-blocked until Phase 7 QA CLEARED |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters/write_support.rs`, `query_rows.rs` | Topology query rows/deltas format and parse `entity:*`/`relation:*` identities as strings for mutation targets and live rows. | Fixed | Topology query rows now carry typed `WORTHQueryEntityIdentity` authority plus an explicit terminal `identity.id` projection label for endpoint/reporting use; retained row lookup and query-runtime support resolve that projection label without re-admitting evidence digest text. Query read materialization now indexes rows only by explicit `identity.id` projection and no longer falls back to `row.identity().to_string()` evidence. Focused gates: `cargo check -p worth-query --lib` and `cargo test -p worth-topo query_runtime --lib` (`64 passed`) recorded in `goal_mode_current_query_lib_check_after_worth_topo_identity_hardening_verify.txt` and `goal_mode_worth_topo_query_runtime_test_latest.txt`. |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters/write_authority/write_lowering.rs`, `projection/runtime_boundary/query_runtime/adapters/write_authority/patch_matching.rs` | Relation endpoint lowering and patch matching parse/compare formatted entity or relation labels to decide production write authority. | Fixed | Existing endpoints lower through typed `WORTHQueryEntityIdentity` patch matches and existing-target authority helpers; graph relation endpoint authority now requires relational record parts and refuses to fall back to `evidence_identity().to_string()`. Symbolic graph batches now keep invariant-complete relation programs atomic even when same-batch symbolic aspect references are present, so shell/wire/loop authority is admitted as a complete graph rather than command-by-command projection text. Focused gate: `cargo test -p worth-topo query_runtime --lib` (`64 passed`). |
| 8 | worth-topo | `projection/runtime_boundary/read_execution/query_shape.rs`, `projection/runtime_boundary/read_execution/family_execution.rs`, `projection/runtime_boundary/read_execution/row_decode.rs`, `projection/read_views/domain/handle_reads.rs` | Read execution anchors, family execution, retained-row decode, and handle-read inputs select production records from string identity anchors. | Fixed | Production read anchors now survive the Query runtime boundary because source rows project explicit `identity.id` labels and read materialization joins on those labels instead of evidence identities. Retained-row decode and topology row lookup prefer the explicit projected identity and only use typed relational parts as a fallback for already-typed rows; missing anchors report the retained projection inventory for diagnosis. Focused gate: `cargo test -p worth-topo query_runtime --lib` (`64 passed`), including topology read certification, radial/successor relation-update reads, shell/wire/loop mutation programs, and runtime posture rows. |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters/bridge_source_support.rs` | Topology bridge source parses `commit-*`, `relational-snapshot:*:version:*`, and `entity:*`/`relation:*` strings back into relational IDs. | Fixed | Bridge source support extracts relational commit, snapshot, and record parts from typed bridge/query identities |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters/bridge_source.rs` | Bridge source calls `request.commit_identity().as_str()`, compares branch/snapshot identity text, and reads snapshot packets by parsing `read.entity_identity()`. | Fixed | Bridge source now resolves branch/commit/snapshot/record authority through typed relational payload accessors |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters/binding.rs` | `TopologyRuntimeBinding::snapshot_token()` mints erased snapshot text by calling `bridge_snapshot_identity_for_commit/handle(...).as_str().to_string()` and falling back to a string sentinel for empty state. | Fixed | Runtime binding exposes `current_snapshot_identity() -> WORTHQuerySnapshotIdentity` and preserves typed relational snapshot parts |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters/declaration_initialization.rs` | Declaration initialization accepts `snapshot_token: &str`, compares it against `bridge_snapshot_identity_for_handle(...).as_str()`, and reports mismatch details using snapshot identity text. | Fixed | Declaration initialization no longer accepts or compares snapshot-token text; metadata derives from typed read basis |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters.rs::TopologyRuntimeSourceAdapter` | Source adapter implements `snapshot_token() -> String` by forwarding binding snapshot text. | Fixed | Source adapter implements `WORTHQueryRuntimeSnapshotIdentityAdapter` and returns typed snapshot identity |
| 8 | worth-topo | `projection/runtime_boundary/read_execution/basis_context.rs` | Historical read execution stores `HistoricalSnapshot { snapshot_token: String }`, passes snapshot token by `&str` into preflight/materialization, and builds `QueryBasisContextRequest::historical_snapshot(snapshot_token)` from erased text. | Fixed | Historical read execution stores `WORTHQuerySnapshotIdentity`; evidence-label projection is explicit at lower compatibility edges |
| 8 | worth-topo | `projection/read_views/domain/handle_reads.rs` | Read-handle entry copies `workspace.snapshot_token().to_string()` into `TopologyReadExecutionTarget::historical_snapshot(...)` as an owned erased snapshot token. | Fixed | Handle reads pass `workspace.snapshot_identity()` into historical execution targets |
| 8 | worth-topo | `projection/read_views/domain/read_proof/report.rs`, `projection/read_views/domain/read_proof/report_surface.rs` | Read-proof reports store `executed_snapshot_token: Option<String>` from `receipt.snapshot_token().to_string()` and expose the executed snapshot token as `Option<&str>`. | Fixed | Read-proof reports store typed executed snapshot identity and expose diagnostic labels only through an explicit projection accessor |
| 8 | worth-topo | `projection/runtime_boundary/query_runtime/adapters.rs::TopologyStaticSignalSink` | Static signal sink uses the default `WORTHQueryRuntimeSignalSinkAdapter::build_signal_invalidation_routing_receipt` path, which derives `SignalInvalidationRoutingReceipt` from string commit/snapshot receipt fields instead of a typed bridge route identity. | Fixed | Static signal sink now routes typed receipts through bridge route identity construction before boundary receipt lowering |
| 8 | worth-topo | `certification/bridge.rs` | Bridge certification routes with `TruthCommitIdentity::new(format!("commit-{commit_id}"))`, branch identity from raw branch text, and stores route/snapshot/history identities as strings in proof rows. | Fixed | Bridge certification uses relational truth identity constructors and explicit evidence-label projections for proof rows. All Phase 8 rows: code may be landed; milestone sequencing blocked until Phase 7 QA CLEARED |
| 9 | worth-query | `correspondence/`, `historical/`, `view_shape_live/` test bridge fixtures | Test sources minted patch/head identities from request commit or branch evidence text and compared fixture snapshots by evidence label. | Fixed | Fixtures now use typed relational commit/patch fixture positions and typed snapshot-handle comparison; targeted scans reject commit/branch-derived patch/head folklore |
| 9 | worth-query | `harness/` test bridge/effect fixtures | Harness preflight/resolved-basis fixtures no longer accept or lower raw snapshot token text; `runtime_preflight_with_snapshot_identity`, `runtime_basis`, and `store_basis` require `WORTHQuerySnapshotIdentity`, and ordinary harness callers use relational snapshot fixture handles. Row scan confirms remaining harness `snapshot_token`/formatted bridge-harness truth IDs belong to later explicit rows. | Fixed | Harness folklore replacement |
| 9 | worth-query | `harness/fixtures/effect_authorities.rs`, `harness/fixtures/preview_bridge.rs` | Shared harness fixtures compute runtime snapshot tokens as strings and mint patch/head/snapshot/branch truth identities from request commit, branch text, or formatted preview snapshot seeds. | Fixed | Shared harness fixtures now expose typed `WORTHQuerySnapshotIdentity` handles, mint bridge patch/head/snapshot/branch identities through explicit relational constructors, and compare preview snapshots by typed identity rather than evidence-label text |
| 9 | worth-query | `harness/runtime_api_stabilization/transcript_runtime.rs`, `harness/runtime_api_stabilization/transcripts.rs`, `harness/runtime_api_stabilization/transcript_maintainer.rs`, `harness/runtime_api_stabilization/transcript_session_proofs.rs` | Runtime API stabilization transcript intent receipts and derived patches no longer construct commit/snapshot authority from formatted transcript labels; transcript helpers were split so touched files remain under the Rust line cap. | Fixed | `cargo check -p worth-query --lib` and `cargo test -p worth-query runtime_api_stabilization --lib` pass; targeted runtime API stabilization scan finds no remaining `from_external_authority_label(format!(...))` commit/snapshot constructors. |
| 9 | worth-query | `harness/runtime_api_stabilization/transcript_runtime/transcript_authority.rs` | Transcript authority write fixtures use typed relational commit/snapshot constructors rather than formatted `transcript-commit:*` / `transcript-snapshot:*` strings. | Fixed | Covered by `cargo test -p worth-query runtime_api_stabilization --lib` and the targeted runtime API stabilization fixture scan. |
| 9 | worth-query | `harness/aspect_api_finalization_certification/rows.rs`, `harness/aspect_api_finalization_certification/rejections.rs` | Aspect API finalization certification rows now derive single-write receipt digest material from typed commit/snapshot evidence identities instead of `commit_identity().to_string()`, and rejection helpers were split so touched files remain under the Rust line cap. | Fixed | `cargo check -p worth-query --lib` and `cargo test -p worth-query aspect_api_finalization --lib` pass; targeted aspect API finalization scan found no remaining `receipt_digest: *.commit_identity().to_string()` or commit/snapshot `field_identity` re-entry in the certification rows. |
| 9 | worth-query | `tests/support/public_bridge_runtime/`, `tests/support/public_bridge_runtime/hostile_certification.rs` | Public bridge runtime test support now constructs mutation receipts from relational commit/snapshot identities, bridge patch envelopes derive patch/snapshot/branch identities from relational commit payloads, and hostile certification composes receipt evidence identities without string projection re-entry. | Fixed | `cargo test -p worth-query --test public_bridge_runtime_bootstrap` and `cargo test -p worth-query --test milestone_9_7_phase_10_hostile_certification` pass; targeted public bridge runtime scans found no remaining formatted external commit/snapshot constructors, bridge harness-label patch/snapshot/branch reconstruction, or receipt evidence `as_str()` composition in that support tree. |
| 9 | worth-query | `lower_runtime_routing/certification/surface/fixtures/` | Lower-runtime routing certification fixtures construct `WORTHQueryMutationReceipt` literals with string commit/snapshot fields and mint `Truth*Identity::new(...)` / patch identities from commit or branch text across core and phase-six fixtures. | Fixed | Lower-runtime certification fixture tree now composes typed evidence/relational truth identities; Phase 7 inventory scans cover the fixture/support feeder paths for `hash_parts(` and bridge harness-label regressions. |
| 9 | worth-query | `runtime/tests/causal_inspection/` | Causal inspection tests and support now route through a shared relational commit authority helper, derive materialization patch/head/snapshot identities from typed relational parts, and build writeback native causality from relational commit/snapshot authority instead of `query-trigger:*` or bridge evidence text. | Fixed | `cargo test -p worth-query causal_inspection --lib` passes; directory-wide causal inspection scan found no remaining `from_bridge_harness_label`, `query-trigger`, `evidence_identity().as_str()` slot suffixes, or raw `Truth*Identity::new` constructors. `materialization/support.rs` was split with `materialization/references.rs` so touched files stay under the Rust line cap. |
| 9 | worth-query | `effect_lifecycle/certification/seeded/support.rs` | Seeded effect lifecycle certification support derives patch identity text from `commit_identity.as_str()` and constructs snapshot/branch truth identities from raw fixture strings. | Fixed | Seeded effect lifecycle support returns typed relational snapshot handles and constructs bridge patch/snapshot/branch identities with typed relational constructors |
| 9 | worth-query | `harness/milestone_eight_certification/` | Milestone-eight certification harness now derives patch/head/snapshot/branch truth identities and grouped row requests from deterministic relational commit, branch, snapshot, and record parts instead of request evidence text, bridge harness labels, external snapshot labels, or coarse `result:*` row strings. | Fixed | `cargo test -p worth-query milestone_eight_certification --lib` passes; targeted harness scan found no remaining bridge harness-label truth constructors, external snapshot authority labels, raw `Truth*Identity::new` constructors, or coarse snapshot read requests. The first focused run exposed untyped relational row requests, which were fixed by carrying `RelationalBridgeRecordIdentityParts` through grouped snapshot reads and patch targets. |
| 9 | worth-query | `projection_consumption/tests/`, `query_basis_lifecycle/tests/`, `query_basis_lifecycle/projection/support.rs` | Projection consumption and query-basis lifecycle fixtures now derive branch, commit, snapshot, patch, lineage, record, and row-read identities from deterministic relational authority helpers instead of bridge harness labels, external authority labels, formatted patch/head text, or coarse snapshot row strings. | Fixed | `cargo test -p worth-query projection_consumption --lib` passes 101 tests; `cargo test -p worth-query query_basis_lifecycle --lib` passes 39 tests. Targeted scan found no remaining row-scoped bridge harness-label constructors, external commit/snapshot authority labels, raw `Truth*Identity::new` constructors, or coarse snapshot read requests; line-cap scan for touched projection/query-basis directories is clean. |
| 9 | worth-query | `intent_admission/certification/fixtures/bridge.rs` | Intent admission bridge certification fixtures now derive patch and snapshot truth identities from the relational commit payload and use a relational branch identity instead of bridge harness-label patch/snapshot/branch strings. | Fixed | `cargo test -p worth-query intent_admission::certification --lib` passes; targeted fixture scan found no remaining bridge harness-label reconstruction, commit identity text formatting, or raw `Truth*Identity::new(...)` fixture path. |
| 9 | worth-query | `intent_admission/certification/fixtures/runtime.rs`, `intent_admission/certification/fixtures/read.rs`, `intent_admission/certification/fixtures/mod.rs` | Intent admission runtime/read certification fixtures now derive placeholder commit/snapshot and read-basis snapshot identities from deterministic relational-backed fixture positions instead of formatted `certification-*` labels, external snapshot labels, or `snapshot_token: &str` basis inputs. | Fixed | `cargo test -p worth-query intent_admission::certification --lib` passes; targeted fixture scan found no remaining formatted certification commit/snapshot helper calls, external-authority snapshot labels, `snapshot_token: &str` read-basis input, or fixture snapshot string re-entry. |
| 9 | worth-query | `runtime/tests/support/bridge/hostile_certification.rs::hostile_journal_gap_count` | Hostile journal helper calls `receipt.commit_identity().rsplit('-').next().and_then(|suffix| suffix.parse::<usize>().ok())`. | Fixed | Exact `rsplit('-')` ban is fixed; helper now uses typed relational commit identity extraction |
| 9 | worth-query | `runtime/tests/support/bridge/hostile_certification.rs::hostile_write_receipt_digest`, `runtime/tests/support/bridge/hostile_certification.rs::hostile_published_artifact_digest` | Hostile certification digest helpers compose typed receipt commit/snapshot evidence and published artifact snapshot evidence directly instead of sealing projected receipt/artifact strings. | Fixed | `cargo test -p worth-query hostile_certification --lib` passes; targeted helper scan found no remaining `commit_identity().as_str`, `snapshot_token()`, or artifact snapshot evidence `as_str()` composition in the hostile receipt/artifact digest helpers. |
| 9 | worth-query | `runtime/tests/support/bridge/fixture.rs::native_patch_envelope` | Bridge fixture support now derives patch and snapshot truth identities from the relational commit payload and uses a relational branch identity instead of commit evidence-label patch text or raw snapshot/branch fixture strings. | Fixed | `cargo test -p worth-query hostile_certification --lib` and `cargo test -p worth-query runtime::tests::live --lib` pass; targeted fixture scan found no remaining bridge harness-label patch/snapshot/branch reconstruction or commit evidence `as_str()` patch derivation. |
| 9 | worth-query | `runtime/backend/receipts.rs` tests | Signal routing tests construct `WORTHQueryMutationReceipt { commit_identity: "commit-1".to_string(), snapshot_token: "snapshot-1".to_string(), ... }` and assert string equality. | Fixed | Raw string receipt fields are gone; residual authority-less external-label rejection coverage belongs to fixture-specific rows |
| 9 | worth-relational | `presentation/bridge/bridge_source_tests/` | Bridge source tests mint `TruthCommitIdentity::new(format!("commit-*"))`, compare branch/snapshot identities by `as_str()`, and route committed patch requests from formatted commit text. | Fixed | Moved into Phase 3 because relational bridge-source certification is part of the ordinary relational spine; tests now use relational typed commit/branch/snapshot/record constructors and extractors |
| 9 | worth-topo | `projection/runtime_boundary/bridge/tests.rs` | Bridge tests call `.route(TruthCommitIdentity::new(format!(...)))` and compare route/record identities as strings. | Fixed | Verified stale row: tests now route with `TruthCommitIdentity::from_relational_commit_id`, compare typed snapshot identities, and keep route/invalidation/snapshot/history strings as terminal diagnostic evidence-label projections. Gate: `cargo test -p worth-topo projection::runtime_boundary::bridge::tests --lib` (`5 passed`). |
| 9 | worth-topo | `certification/support/read_proof_harness.rs`, `certification/projection_closeout/tests/topology_reads/` | Topology read-proof certification harnesses copy `workspace.snapshot_token().to_string()` into historical read execution targets and assert executed snapshot tokens as string values. | Fixed | Verified stale row: historical read proof harness uses `workspace.snapshot_identity()` and topology read tests assert `executed_snapshot_identity()` typed equality. Gate: `cargo test -p worth-topo certification::projection_closeout::tests::topology_reads --lib` (`61 passed`). |
| 9 | worth-topo | `certification/projection_closeout/tests/derived_chain.rs` | Derived-chain certification asserts inspection `commit_identity()` against write receipt commit identity strings and carries topology surface identities as string fixtures. | Fixed | Verified stale row: derived-chain inspection compares typed commit identities; derived topology surface names remain terminal view labels. Gate: `cargo test -p worth-topo certification::projection_closeout::tests::derived_chain --lib` (`2 passed`). |
| 9 | hadwiger-research | `tests/research_graph_invariants.rs`, `src/research_graph_invariants/requests.rs`, `src/agent_advisory/operations.rs` | Test write authority builds `WORTHQueryMutationReceipt` with `commit_identity: commit_identity.to_string()` and `snapshot_token: format!("{commit_identity}:snapshot")`; downstream Hadwiger code still called removed digest accessors on lower-runtime boundary envelopes and grouped compositions. | Fixed | Research graph invariant fixtures now construct `WORTHQueryMutationReceipt` through typed `WORTHQueryCommitIdentity`, `WORTHQuerySnapshotIdentity`, and `WORTHQueryEntityIdentity` values derived from deterministic relational fixture parts; lower-runtime boundary source and envelope reporting strings project from typed identity accessors only at Hadwiger report edges; grouped contribution report strings use typed composition identity. Gate: `cargo test -p hadwiger-research --test research_graph_invariants` (`10 passed`). |
| 9 | worth-ui | `src/todo/truth.rs`, `src/showcase.rs` | Todo truth state stores `snapshot_token: String`, synthesizes workspace snapshot text from child snapshot tokens, and routes mutations through string entity identities. | Fixed | Todo truth now routes update/delete by typed `WORTHQueryEntityIdentity`, exposes board snapshot state as explicit `TodoSnapshotProjection`, and projects the combined snapshot label only for UI diagnostics. Gate: `cargo check -p worth-ui` passed. |
| 9 | worth-server | `surfaces/compat_http/mutation_execution/schema.rs` | Compat HTTP mutation requests parse `entity_identity` and `resolved_entity_identity` as JSON strings and build canonical request digests from formatted identity text. | Fixed | Compat request JSON remains an external-token boundary, but update/delete authority now requires canonical relational bridge record text admitted through `RelationalBridgeRecordIdentityParts::from_bridge_entity_identity(...)` before building `WORTHQueryEntityIdentity`; non-canonical raw labels deny before authority construction. Gate: `cargo test -p worth-server --test compat_http_phase_three` (`8 passed`) |
| 9 | worth-server | `WORTH_native/direct/mutation.rs`, `surfaces/compat_http/mutation_execution/response.rs` | Server mutation result digests use `receipt.commit_identity()` as a string result digest for single mutations. | Fixed | Single-mutation responses project `commit_evidence_identity().as_str()` as terminal result digest; typed commit authority remains available only through typed receipt/inspection accessors. Gates: `cargo test -p worth-server --test WORTH_native_facade_entry` (`62 passed`), `cargo test -p worth-server --test compat_http_phase_three` (`8 passed`) |
| 9 | worth-server | `surfaces/compat_http/mutation_execution/query_execution.rs` | Compatibility precondition observes `handoff.workspace().snapshot_token()` as a string basis digest. | Fixed | Compatibility precondition now reads `workspace().snapshot_identity()` and converts it only to the terminal HTTP validator label; the erased `snapshot_token()` backend/source seam remains absent. Gate: `cargo check -p worth-server` |
| 9 | worth-server | `tests/support/direct_context_runtime.rs`, `tests/support/query_handoff/runtime.rs`, `tests/support/query_handoff/runtime_mutation_support.rs`, `tests/support/compat_http/phase_three_runtime.rs`, `tests/support/compat_http/phase_four_runtime.rs` | Server test adapters implement `snapshot_token(&self) -> String` and construct `WORTHQueryMutationReceipt` with formatted commit/snapshot strings. | Fixed | Adapters provide typed snapshot/session/support evidence identities, mutation receipts are built from typed commit/snapshot/entity identities, and split helper files keep touched tests within the 400-line cap. Gates: `cargo test -p worth-server --test WORTH_native_facade_entry` (`62 passed`), compat phase three/four (`8 passed` each) |
| 9 | worth-server | `tests/WORTH_native/direct_mutation.rs`, `tests/WORTH_native/direct_projection.rs` | WORTH-native integration tests compare result/inspection digests to `receipt.commit_identity()`, assert `inspection.snapshot_token() == receipt.snapshot_token()`, and consume direct projection read receipt snapshot tokens as strings. | Fixed | Direct integration assertions now compare typed commit/snapshot handles or explicit evidence/projection accessors, and the backend-verified assertion denial fixture rejects fake raw entity labels by requiring canonical relational bridge identity input. Gate: `cargo test -p worth-server --test WORTH_native_facade_entry` (`62 passed`) |
| â€” | worth-runtime-bridge | `src/subscription/replay_tests.rs` | Subscription replay tests mint truth identities from string literals/formatted commit and patch text, but subscription replay is outside this milestone's ordinary truth-routing spine. | Fixed | Phase 10: migrated to typed relational constructors; folklore guard in `tests/subscription_replay_folklore_guard.rs` |

**Historical note:** rows below the header were produced by the earlier Phase 1
scan and remain useful trace material. They are not sufficient for closure after
this rewrite. Any newly discovered row must close through the compiler ledger
rules: authority-category fix, compile-fail guard, terminal projection
quarantine, or named deferred owner milestone.

### Other Artifacts

| Artifact | When | Path |
|----------|------|------|
| Compiler failure ledger / exposure report | Phase 2 | `plans/worth-query/bridge_truth_identity_exposure_report.md` â€” first `cargo check --workspace` failure inventory after root authority breaks; classify attempted category, required category, owner phase, and closure route |
| Compile-fail gates | Phase 2+ | `worth-runtime-bridge/tests/ui/`, `worth-query/tests/ui/`, downstream/harness UI fixtures for forbidden substitutions |
| Closeout | Phase 10 | `plans/worth-query/milestone-9.6-bridge-truth-identity-closeout.md` |
