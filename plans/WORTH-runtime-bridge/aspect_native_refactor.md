# WORTH Runtime Bridge Aspect-Native Refactor Spec

> Status: Active implementation spec
>
> Roadmap parent: [worth_runtime_bridge_roadmap.md](worth_runtime_bridge_roadmap.md)
>
> Vision parent: [worth_runtime_bridge_vision.md](worth_runtime_bridge_vision.md)
>
> Bridge certification companion: [test-requirements.md](test-requirements.md)
>
> Platform carriage successor:
> [WORTH Signal Milestone 13.1](../WORTH_signal/milestone-13.1-plan.md)
> ([implemented cross-runtime closeout](../WORTH_signal/milestone-13.1-closeout.md))
>
> Foundational authority companion: [aspect-contracts-values-and-authoritative-state](../../crates/worth-foundational/docs/aspect-contracts-values-and-authoritative-state/README.md)
>
> Primary architectural driver: hard-break `worth-runtime-bridge` onto
> `worth-foundational` aspect contracts, masks, authoritative state, and
> authoritative patch carriers without preserving JSON or string-shaped
> authority paths.

## Purpose

This spec is the reference document for making `worth-runtime-bridge`
aspect-native.

The bridge already has strong protocol surfaces: committed patch envelopes,
fine-grained route planning, source contracts, subscription families,
writeback families, diagnostics, replay, and certification bundles. Those
surfaces are not enough if the values, targets, masks, and authority evidence
that move through them still depend on bridge-local string labels, raw snapshot
bytes, digest-only writeback payloads, or JSON-shaped diagnostic truth.

The refactor is a hard break. The final crate must read as though
`worth-runtime-bridge` was designed around `worth-foundational` aspect
contracts from the beginning. Historical engineering documents may describe
the migration. Production code, public facades, retained diagnostics,
certification fixtures, and harness helpers must not preserve signs of a
prior JSON/raw-byte/string-label/digest-only authority model.

## Cross-Runtime Granular Invalidation Boundary

Runtime Bridge is the installed correspondence owner between committed
Relational change meaning and lower-runtime consumers. In the completed
granular invalidation path it owns:

- matching one committed semantic change to installed dependencies while
  keeping aspect, change kind, field path, and record locality correlated
- preserving exact locality or reporting an explicitly admitted widening
- delivering authoritative direct truth separately from optional evidence of
  Signal work that actually ran
- rejecting stale graph, runtime-generation, snapshot, mapping, or allocation
  bindings during restore and reinstallation
- reporting candidate probes, matched dependencies, widening, rejection,
  direct delivery, and performed-delivery cost rows at the Bridge boundary

The Bridge does not mint Query maintenance, consumer disclosure, or Signal
execution authority. Signal remains the owner of performed recomputation.
Query consumes the installed delivery, admits application consequences, and
publishes the current query-shaped result. A semantic scope or private lookup
key can narrow candidate selection, but neither is authority and neither may be
replaced by a physical shard, region, or worker identity.

The application-facing contract is documented in
[Granular Live Invalidation](../../workspaces/worth-query/crates/worth-query/docs/runtime-surfaces/granular-live-invalidation.md).

Hard-break rules:

- no legacy JSON production authority path
- no compatibility fallback, shim, alternate constructor, or coexistence path
  where native aspect contracts are required
- no duplicate bridge-local aspect value model
- no route authority derived from free-form `surface_label` strings
- no snapshot core carrying untyped `aspect_bytes`
- no writeback authority input where a digest replaces a typed aspect effect
- no diagnostics or counters whose JSON projection is the semantic source
- no public or test helper vocabulary that teaches callers to encode aspect
  truth as bytes, labels, payload digests, JSON values, or foreign-format bags
- no "legacy", "compat", "fallback", "raw", "payload", "bytes", or
  string-label authority names in production modules unless the module is an
  explicitly external ingress/export boundary and cannot influence native
  authority

Non-native projection may exist only as terminal external I/O: adapter ingress
from non-WORTH hosts, harness report export, or presentation rendering. It
must be named as an external boundary, must immediately lower into or derive
from native carriers, and must be unable to participate in source, routing,
writeback, replay, diagnostics, or certification authority. It is not a
fallback path, compatibility shim, or alternate authority lane.

## Governing Source Summaries

The following documents were read to shape this spec. These summaries are part
of the contract so later implementation batches can verify that code still
matches the intended pressure.

### `MENTALITY.md`

The work must solve the hard authority problem first. Aspect-native means
mechanical enforcement through types, constructors, masks, phase outputs, and
certification bundles, not narrative discipline. Authority comes first,
derived explanation second.

### `arch_laws.md`

The bridge must preserve contractual facades, phase-typed proof packets,
typed failures, and authority/derived separation. Planning, lowering,
delivery, materialization, writeback, and diagnostics are separate boundary
crossings. Later phases consume proof-bearing outputs from earlier phases.

### `composition_laws.md`

Files and functions must expose one responsibility each. Large refactors should
split along domain seams while replacing semantics, not perform cosmetic file
churn. Any cleanup in this refactor must make aspect authority easier to see
and harder to bypass.

### `domain_structure_laws.md`

Directory topology must be an authority map. Aspect read contracts,
committed-patch targets, routing masks, writeback intents, diagnostics
artifacts, and facade surfaces are different subdomains and must not collapse
into catch-all files.

### `perf_laws.md`

Aspect-native work must preserve bounded plan/execute separation. Masks must
bound projection, mutation, diagnostics, and writeback breadth. Counters must
prove breadth and rejection behavior rather than hiding cost behind logs.

### Foundational Aspect Docs

`worth-foundational` owns aspect contracts, scalar and struct values,
canonical field paths, masks, authoritative state, authoritative patch
envelopes, validation/admission flow, identities/locators, compatibility
lowering, digest preparation, and grouped public lanes.

For the bridge this means:

- native aspect contracts and values are semantic input
- masks are not optional decorations
- authoritative patches and state carriers are the only authority-shaped
  mutation/read basis
- JSON compatibility is terminal lowering or ingress translation only
- canonical basis and digest preparation must use foundational carriers where
  possible instead of bridge-local string reconstruction

### Runtime Bridge Vision And Roadmap

The bridge coordinates truth and compute without becoming either runtime. It
owns protocol boundaries, causality transfer, route artifacts, source
admission, writeback handoff, diagnostics, and certification. It must not own
relational truth semantics or signal scheduling semantics.

This aspect-native refactor therefore replaces bridge authority carriers while
preserving the bridge's role: consume, lower, route, certify, and hand off
native aspect truth without redefining it.

### Runtime Bridge Milestones 2, 7, 12, And 13

Milestone 2 requires deterministic aspect mapping and fine-grained slice
routing. Milestone 7 requires bridge-owned source contracts and packetized
truth-backed reads. Milestone 12 requires bridge-mediated writeback to preserve
effect meaning through an authority boundary. Milestone 13 requires
end-to-end causality, typed failures, and offline-sufficient certification
bundles.

Aspect-native closeout is the cross-cutting retrofit that makes those milestone
claims honest under foundational contracts.

## Goal

Make every production bridge path that carries truth value, changed surface,
projection breadth, mutation intent, writeback effect, diagnostic target, or
certification identity use foundational aspect contracts, masks,
authoritative-state carriers, and authoritative patches as its semantic basis.

## Adversarial Constraint

The refactor must survive this hostile condition:

> A long-lived bridge with mixed snapshot reads, fine-grained aspect routing,
> grouped truth projections, subscription lifecycles, diagnostics-tier
> variation, writeback family admission, replay after restart, and hostile
> harness lanes must produce the same native aspect values, target identities,
> mask-bounded route artifacts, writeback effect identities, failure classes,
> and certification bundles every time, while JSON, string labels, raw bytes,
> and digest-only payloads remain unable to influence production authority.

If any supported production path:

- decodes snapshot truth through `serde_json` after source admission
- uses raw `aspect_bytes` as the read-result contract
- derives route authority from prefixed free-form labels such as `field:name`
- accepts committed patch items without a native aspect locator or field path
- uses writeback `domain_payload_digest` as the proposed effect instead of
  digesting a typed effect carrier
- lets diagnostics JSON define canonical truth
- widens masks or projections without a typed admitted reason
- requires host logs to explain an aspect-native mismatch

then the refactor is not complete.

## Aspect-Native Definition

A runtime-bridge path is aspect-native only when all of the following are true:

- It carries `AspectContract`, `AspectKey`, `AspectLocator`,
  `AspectFieldLocator`, `AspectFieldPath`, `AspectValue`, `StructAspectValue`,
  foundational masks, or authoritative patch/state carriers as semantic data.
- It distinguishes authority carriers from derived diagnostics,
  presentations, and external I/O projections.
- It validates contract/value/mask compatibility before materialization,
  routing, delivery, writeback admission, or certification export.
- It computes canonical basis from native carriers or explicit canonicalization
  helpers rather than from ad hoc string concatenation of labels.
- It exposes typed failures for unsupported contract, value, mask, target,
  patch, and writeback-effect shapes.
- It allows JSON only at an explicitly named external-ingress or terminal-export
  boundary that cannot be selected as an authority path.
- It contains no production fallback branch that can choose "legacy" behavior
  after native admission fails.
- It does not preserve old authority vocabulary in public names, test helper
  names, diagnostic record fields, or certification bundle fields. A name that
  still says `bytes`, `payload`, `surface_label`, `domain_payload`,
  `legacy`, `compatibility`, or `fallback` in a production authority slice is a
  failed migration unless the file is explicitly an external I/O boundary.

### Eradication Standard

This refactor is not complete when the native path works. It is complete only
when the old path cannot be found, selected, reconstructed, or taught by the
crate.

Every implementation batch must therefore delete or quarantine all touched
legacy evidence:

- delete old constructors rather than deprecating them
- delete old public exports rather than aliasing them
- delete tests that assert legacy shape and replace them with native artifact
  assertions
- delete foreign-format helpers from production paths rather than renaming them
- delete diagnostic fields that imply old authority even if their values are
  now derived from native carriers
- delete fallback branches that recover from native rejection by widening to
  raw bytes, labels, JSON, or digest-only payloads

The only allowed historical evidence is in roadmap, spec, closeout, migration,
or audit documents. Current production code and current tests must describe the
final native architecture, not the migration history.

## Legacy Eradication Baseline

This section names legacy authority patterns that must be annihilated wherever
they remain. It is not permission to preserve them until a late cleanup pass.
If a later implementation batch discovers one of these patterns in a new
subsystem, the batch must move back to that seam immediately and delete the
legacy path before continuing forward.

Many bridge protocol surfaces are already phase-typed and deterministic. That
does not lower the bar. A deterministic legacy-shaped artifact is still legacy
if it teaches the crate to recover truth from JSON, bytes, labels, arbitrary
digests, foreign-format shims, or host-local report maps.

### Snapshot Read Values

Any source path that decodes snapshot bytes by attempting JSON or UTF-8
recovery is forbidden production authority. A snapshot result contract that
exposes `aspect_bytes` or a similarly raw carrier forces downstream
materialization to recover aspect meaning from bytes rather than receiving a
native value carrier admitted under a contract and mask. Such paths must be
deleted, not adapted.

### Patch And Routing Targets

Any patch item modeled as `entity_identity + aspect_key + surface_label` is a
legacy bridge-local encoding. Any route classifier that treats prefixes such as
`field:`, `region:`, or `facet:` as authority is a legacy parser. Production
patch items must carry native aspect targets: aspect locator, optional field
locator/path, surface kind, and mask basis. If external parsing remains for
foreign input, it must terminate before production routing and leave no
fallback branch.

### Snapshot Packet Breadth

Any snapshot request or subscription slice that carries `surface_label` as
breadth authority is legacy. Packet breadth must be declared with native masks
and contract-backed targets before materialization. Coarse widening is allowed
only as a typed admitted widening class with retained proof, never as
label-driven or foreign-input-driven widening.

### Grouped Truth And Row Materialization

Row-set and grouped truth materializers must be contract/mask validators and
canonical row assemblers, not decoders. If a grouped truth path has to recover
meaning from request keys, raw bytes, JSON, or string labels, the native value
contract failed upstream and the old path must be deleted.

### Writeback Mapper Boundary

Any writeback mapper or facade helper that accepts an arbitrary
`domain_payload_digest`, `effect_digest`, or equivalent string as proposed
effect authority is legacy. A digest is evidence, not the proposed effect. The
mapper envelope must carry a typed bridge writeback intent/effect carrier
backed by aspect patch or authoritative mutation carriers, with digest derived
from that carrier.

### Diagnostics And Certification

Diagnostics and harness certification may export JSON only as terminal report
rendering. Any production diagnostic record, counter artifact, failure
classification, or certification bundle that stores JSON as the source of
meaning must be rewritten as typed artifacts with JSON projection methods.
Harness fixtures must not preserve legacy input construction merely because the
export format is JSON.

## Non-Goals

- Do not redesign relational authority semantics.
- Do not redesign signal dependency ownership or scheduling.
- Do not make the bridge a second aspect-authority runtime.
- Do not split files only for aesthetics.
- Do not preserve legacy JSON or string paths for production callers.
- Do not add a broad abstraction layer that hides masks, contracts, or costs.
- Do not add foreign-format shims to avoid updating callers. Callers must move
  to native carriers or live behind an explicitly external boundary.
- Do not leave migration-era tests, aliases, or helper names in place as
  documentation of the old path. Historic docs record history; current code
  records architecture.

## Target Directory Skeleton

This is the expected destination shape. Names may refine during implementation,
but the authority seams must remain visible.

```text
crates/worth-runtime-bridge/src/
  source/
    aspect_read_contract/
      request.rs
      result.rs
      value.rs
      validation.rs
      materialization.rs
    grouped_truth_projection/
      contract.rs
      lanes.rs
      rows.rs
      validation.rs
  input/
    committed_patch/
      envelope.rs
      target.rs
      normalization.rs
      validation.rs
  routing/
    aspect_targets/
      target_derivation.rs
      mask_projection.rs
      subscription_slice_lowering.rs
      admitted_widening.rs
  writeback/
    aspect_intent/
      declaration.rs
      effect.rs
      mapper_envelope.rs
      candidate.rs
      authority_request.rs
  diagnostics/
    aspect_artifacts/
      source.rs
      routing.rs
      writeback.rs
      masks.rs
      certification.rs
    terminal_export/
      json.rs
  facade/
    runtime/
      source/
      routing/
      writeback/
  external_io/
    foreign_ingress/
      json_lowering.rs
    terminal_report_export/
      json_projection.rs
```

Cleanup rule: split only while replacing an authority seam. For example,
splitting `source/aspect_values.rs` is required because its responsibility
must change from JSON decoding to native value validation. Splitting a stable
typed counter file is not required unless it currently makes JSON or digest
authority hard to remove.

Directory rule: names such as `foreign`, `ingress`, `export`, or `json` are
permitted only under an external I/O branch that cannot be imported by native
authority modules. Avoid `compatibility` in new runtime-bridge production
topology; it is too easy to misread as a supported coexistence contract. If
source, routing, writeback, replay, diagnostics authority, or certification
authority imports an external I/O module, the boundary has failed.

## Phases

### Phase 1: Crate-Wide Legacy Authority Eradication Gate

Choke point:

- public facade exports
- bridge harness fixtures and support helpers
- diagnostics and certification bundle field names
- foreign-input, fallback, raw, bytes, payload, and string-label seams that
  can still be reached from production code

New production shape:

- Every native authority slice has one native construction path and no legacy
  alternate path.
- External non-native input/output code lives behind explicitly named
  ingress/export modules and cannot be imported by source, routing, writeback,
  replay, diagnostics, or certification authority.
- Tests build native aspect contracts, locators, masks, values, authoritative
  state, patches, and writeback intents directly; tests do not teach callers to
  synthesize authority from JSON, raw bytes, labels, or digest strings.
- Public exports prefer final native names and do not preserve migration-era
  aliases.

Deletion/refactor:

- Delete all production fallback branches whose purpose is "try native, then
  recover through legacy".
- Delete public helper APIs that accept raw bytes, surface labels, JSON values,
  domain payload digests, or foreign-format bags for authority-bearing fields.
- Delete or move foreign-input lowering modules out of authority import paths.
- Delete diagnostic/certification field names that imply legacy authority, even
  when the value is currently native-derived.

Acceptance evidence:

- Repository scan finds no production authority occurrence of `aspect_bytes`,
  `surface_label`, `domain_payload`, `legacy`, `fallback`, `compatibility`,
  or `serde_json::Value` outside explicit external I/O or historic docs.
- Native source, routing, writeback, replay, diagnostics, and certification
  tests construct native carriers rather than legacy fixtures.
- No facade method can admit an authority-bearing operation from raw JSON,
  raw bytes, string labels, or arbitrary digest strings.

#### Phase 1 Completion Runbook

This is the only active Phase 1 plan. Every older Phase 1 board below is audit
history, not an execution worklist.

Current status, updated 2026-06-02: Phase 1 is `closed`. The strict reopen
queue is empty after closing the diagnostics public identity lookup row and the
facade/current-test authority helper identity constructor row. Future Phase 1
work may reopen only from one named later-phase concern that proves a current
executable old authority shape under the blocker threshold below.

Phase 1 has one responsibility: annihilate current production or current-test
authority paths that still admit, retain, prove, diagnose, certify, replay, or
teach JSON/raw-byte/label/arbitrary-digest/string-collection authority. It is
not cleanup, file splitting, vocabulary polishing, or broad scan work.

##### Phase 1 Finish Cockpit

This is the only active Phase 1 operator plan. Every older Phase 1 board below
is audit history, not an execution worklist.

Phase 1 is closed by default. It reopens only when a named, current, executable
old-authority seam can still construct, admit, retain, diagnose, certify,
replay, or teach JSON/raw-byte/string-label/arbitrary-digest/string-collection
authority. Cleanup-only topology, broad file size concerns, weak names, and
general discomfort do not reopen Phase 1 unless they hide that exact authority
seam.

Current tuple:

```text
mode=P1-closed
strict_reopen_queue=empty
live_concern=none
unclassified_confidence_hits=0
open_row=none
decision=0 Phase 1 blockers
last_lockback=2026-06-02
```

Operational rule: prove once, decide once, then stop. Do not turn scan output
into a backlog. Do not edit source before one complete blocker row exists. If a
candidate remains ambiguous after direct caller/test-radius inspection, promote
that candidate to the one row and annihilate it instead of spending another
turn searching.

###### Phase 1 Structured Finish Sprint

Use this sprint when the team asks to finish, re-check, or make Phase 1 more
efficient. It is intentionally smaller than the historical boards: one status
lock, three proof gates, one decision, and optionally one row. Anything outside
that shape is later-phase work unless it is required by the row.

Sprint invariant:

```text
Phase 1 is about executable old authority only.
No cleanup lane exists.
No compatibility or test shim is allowed.
No source edit is allowed before the decision step names exactly one blocker row.
```

Execution packets:

1. `P1-status`: record the current tuple and strict reopen queue before looking
   at code. If the queue is empty, the default outcome remains closed.
2. `P1-authority-wall`: run the authority vocabulary scan once and classify by
   callable seam, not by match count. Output one row per API/file family.
3. `P1-json-wall`: run the JSON/import-wall scans once. Output either
   `terminal`, `foreign_ingress`, or one native-authority import violation.
4. `P1-constructor-wall`: inspect only callable public, facade, harness,
   current-test, and UI-fixture seams from the previous two packets. Do not
   inspect unrelated helpers.
5. `P1-decision`: choose exactly one of:
   `0 blockers` or `1 blocker row`. If a candidate is still ambiguous, it is
   the one blocker row; do not keep searching.
6. `P1-row`, only if needed: delete the old authority shape vertically through
   its first choke point, direct callers, tests, diagnostics/certification
   surfaces, and compile-fail fixtures. No aliases, no old helper preservation,
   no compatibility fallback.
7. `P1-proof`: run only the proof commands required by the decision. Planning
   or classification-only closeout does not get cargo tests.
8. `P1-lockback`: update this spec and memory with the tuple, classification
   ledger, row proof if any, and the next legal action.

Closeout output must be one compact table:

| Packet | Evidence | Classification | Next action |
| --- | --- | --- | --- |
| `P1-status` | Tuple and queue. | `closed` or `row-open` | Continue or inspect row. |
| `P1-authority-wall` | Grouped callable seam hits. | `blocked`, `canonical`, `foundational`, `presentation`, `test-proof`, or `historic` | None unless blocked. |
| `P1-json-wall` | JSON/export/import-wall radius. | `terminal`, `foreign_ingress`, or `blocked` | None unless blocked. |
| `P1-constructor-wall` | Direct callable constructor/helper radius. | `dismissed` or `blocked` | None unless blocked. |
| `P1-decision` | Blocker count. | `0 blockers` or one row name | Stop or execute row. |

If `P1-decision` is `0 blockers`, Phase 1 is done for the current code state and
the next efficient action is the active later phase. If `P1-decision` names one
row, the plan for that row must be written before editing and must include the
new directory skeleton only when the row earns a split by authority boundary,
terminal I/O quarantine, proof-retention separation, or line-cap enforcement.

###### Phase 1 Completion Board

Run this board only when the team asks what remains in Phase 1 or when a later
phase exposes a named Phase-1-shaped concern. `P1-0` through `P1-4` are
read-only. Source edits are legal only in `P1-5` after the row packet is
complete.

| Step | Gate | Work | Output | Edit rule |
| --- | --- | --- | --- | --- |
| `P1-0` | Status lock | Copy the current tuple and strict reopen queue before any scan. | Tuple copied into the turn plan. | No edits. |
| `P1-1` | Authority wall | Run the authority vocabulary scan once. Group hits by file/API, not match count. | Classification ledger for candidate files/APIs. | No edits. |
| `P1-2` | JSON/import wall | Run the JSON scan and native-authority import-wall scan once. | Terminal/export/foreign-ingress classification or one row candidate. | No edits. |
| `P1-3` | Constructor wall | Inspect only callable public/facade/harness/current-test/UI candidates from `P1-1` and `P1-2`. | Every callable construction seam dismissed or promoted. | No edits. |
| `P1-4` | Decision | Count blockers. Choose `0 blockers` or exactly one blocker row. | No-row closeout statement or filled blocker-row packet. | Planning only. |
| `P1-5` | Row annihilation | Delete the old authority shape through the first choke point, direct callers, tests, and proof fixtures. | Old API/field/helper/import gone; native carrier/proof in place; no alias or shim. | Row edits only. |
| `P1-6` | Proof | Prove the no-row decision or source-changing row. | Focused proof transcript, residue scans, line-cap result, diff hygiene. | Row-local fixes only. |
| `P1-7` | Lockback | Record evidence in this spec and memory. | Tuple restored to `P1-closed`; queue empty or next explicit concern named by later phase. | No edits except docs/memory. |

Legal endings:

- `0 blockers`: record classifications, leave Phase 1 closed, and continue the
  active later phase.
- `1 blocker row closed`: rewrite the row vertically, prove it, record it, and
  return Phase 1 to closed.

Illegal endings:

- multiple plausible rows queued for later
- cleanup-only splits or rename-only passes
- compatibility shims, test shims, aliases, or old helper preservation
- cargo tests for planning-only classification
- repeated broad searches after `P1-4`

###### Phase 1 Classification Ledger

Use this ledger during `P1-1` through `P1-4`. One ledger row means one file,
callable API, or terminal boundary, never one grep match.

| Gate | Path/API | Classification | Why it is or is not executable old authority | Direct radius inspected | Row |
| --- | --- | --- | --- | --- | --- |
| `P1-1` | `input/envelope/construction_tests.rs`, `routing/canonicalization_tests.rs`, `subscription/certification/historical_basis.rs` | `canonical` / `foundational` | These hits assert native digest-shaped canonical mechanics or read typed subscription semantic digest evidence from certification bundles. They do not admit caller-supplied old authority. | Direct files only. | None |
| `P1-2` | `harness/**/terminal_report_export/**`, `harness/tests/**/terminal_report_export/**` | `terminal` | `serde_json::Value` / `json!` hits are report/export/capture topology. Native source, routing, writeback, diagnostics, subscription, certification, and facade authority paths do not import terminal JSON/export modules as authority input. | Directory-level import-wall over native authority branches. | None |
| `P1-3` | Snapshot value fixtures, expected field-target basis helpers, policy row display label, writeback certification digest helper, sealed UI placeholders, digest-prefix assertions | `presentation` / `canonical` / `test-proof` | Snapshot helpers take typed `TruthSnapshotIdentity` and string aspect values; field-target helpers compute expected canonical basis; policy row labels are terminal matrix presentation; private writeback digest helper is fed only static certification digest domains; sealed UI placeholders prove raw `&str` authority is uncallable; digest-prefix assertions check native canonical artifacts. | Direct caller radius for `writeback_harness_digest`, `writeback_harness_error_digest`, and `route_policy_row`. | None |

Known current non-blockers:

- `facade/tests/causal_envelope/retained_mapping.rs` is a 401-line structural
  hygiene target, not a Phase 1 authority row. Fix it only inside a coupled
  later aspect/topology batch or dedicated line-cap pass.
- `serde_json::Value` / `json!` belongs to terminal report/export or terminal
  capture topology under the harness.
- `&str` helper hits open a row only when they admit or retain authority from
  raw identity/digest/label/JSON text. They do not open a row when they carry
  aspect scalar values, presentation labels, expected canonical-basis strings,
  or static private proof domains.

Classifications:

- `blocked`: executable production/facade/harness/current-test authority seam.
- `terminal`: report/export/capture JSON or human presentation only.
- `foreign_ingress`: explicit external input lowering before native authority.
- `foundational`: documented `worth-foundational` carrier/accessor such as
  `payload()` or `bytes()` on foundational values.
- `canonical`: digest/canonical-basis mechanics derived from named native
  carriers, not caller-supplied authority.
- `presentation`: assertion text, display text, current-test prose, or
  human-readable detail rendered from native artifacts.
- `test-proof`: sealed compile-fail or privacy proof that cannot construct
  authority.
- `historic`: archived docs only.

###### Phase 1 Gate Commands

These are the only default Phase 1 proof scans. Add a scan only if a complete
blocker row names a new authority shape.

```powershell
rg -n 'aspect_bytes|surface_label|domain_payload|legacy|fallback|compatibility|shim|raw_|payload_|bytes_|from_digests|request_digests|retained_.*digests|Vec<String>|Vec<&str>|BTreeSet<String>|HashSet<String>|VecDeque<String>' crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g '*.rs'
rg -n 'serde_json::Value|serde_json::Map|json!\(|compatibility\(\)|lower_json|JsonCompatibility' crates/worth-runtime-bridge/src -g '*.rs'
rg -n 'pub fn [A-Za-z0-9_]+\([^)]*&str|pub\(crate\) fn [A-Za-z0-9_]+\([^)]*&str|fn [A-Za-z0-9_]*(patch|snapshot|commit|route|authority|declaration|identity|target|writeback|causal|merge)[A-Za-z0-9_]*\([^)]*&str' crates/worth-runtime-bridge/src/facade crates/worth-runtime-bridge/src/harness crates/worth-runtime-bridge/tests -g '*.rs'
rg -n 'todo!\(|unimplemented!\(|sealed_authority_placeholder::<&str>|:sha256:|effect:sha256|route:sha256|truth-state:sha256|truth-view:sha256|strategy:sha256' crates/worth-runtime-bridge/src/facade crates/worth-runtime-bridge/src/harness crates/worth-runtime-bridge/tests -g '*.rs'
rg -n 'terminal_report_export|terminal_projection|foreign_ingress|serde_json' crates/worth-runtime-bridge/src/source crates/worth-runtime-bridge/src/routing crates/worth-runtime-bridge/src/writeback crates/worth-runtime-bridge/src/diagnostics crates/worth-runtime-bridge/src/subscription crates/worth-runtime-bridge/src/facade -g '*.rs'
```

Run the commands in order. Stop after `P1-4` if every hit is classified and no
blocker row exists. If a hit is ambiguous after direct caller/test-radius
inspection, make it the one blocker row and type it; do not start another scan
loop.

###### Blocker Threshold

A hit opens Phase 1 source work only when it is current and executable:

- Production, facade, harness, diagnostics, certification, replay, or current
  test code can construct, admit, retain, diagnose, certify, replay, or teach
  authority from JSON, raw bytes, labels, arbitrary digest strings, foreign
  bags, fake proof placeholders, or string identity collections where typed
  carriers exist.
- Public or crate-wide API accepts an old authority shape for source, routing,
  writeback, replay, diagnostics, certification, or retained proof lookup.
- Current tests or UI fixtures preserve fake authority literals, old public
  names, `todo!`, `unimplemented!`, or old helper construction.
- Native authority code imports terminal/foreign JSON export code as input to
  source, routing, writeback, replay, diagnostics, or certification meaning.
- The row-touched topology would remain over 400 lines or deletion-resistant
  unless split along native authority, proof retention, diagnostic/denial, or
  terminal-export responsibility.

Non-blockers are not work: terminal report/export JSON, foreign ingress before
native admission, presentation strings, human assertion text, semantic labels
that cannot construct authority, foundational accessors, canonical digest
mechanics over named native basis, derived digest projections, and historical
docs.

###### Concern Packet

Create this packet before inspecting a named `P1-concern` beyond the named
file. If any field is missing, the concern is too vague and Phase 1 stays
closed.

```text
Phase 1 concern:
- Suspected seam:
- Why this might feed authority:
- Named file(s) to inspect:
- Direct caller radius:
- Direct current-test/UI fixture radius:
- Terminal/export boundary to inspect, if any:
- Stop condition for dismissal:
```

Concern radius is hard-limited to the named file, direct callers, direct tests,
and direct terminal/export module if the seam may cross an I/O wall. Do not add
a second seam during the same concern.

###### Blocker Row Packet

No Phase 1 source edit may begin until every field is filled.

```text
Phase 1 blocker row:
- Blocker name:
- Concern source:
- Exact executable old authority shape:
- Native replacement carrier/proof:
- Foundational surfaces used:
- First choke point to replace:
- How far back this batch must move to delete upstream old authority:
- Direct production/facade/harness/diagnostics/certification/replay scope:
- Current-test or compile-fail shim that would still teach the old shape:
- New directory skeleton, if required by authority/import-wall/line cap:
- Legacy names/constructors/fields/modules to delete, not alias:
- Residue scans that must go clean:
- Focused proof commands, each with a 10 minute timeout:
```

Valid replacements are foundational carriers, masks, locators, authoritative
state/patches, typed identities, typed denials, typed evidence packets, or
sealed proof carriers. "Rename it" is not a replacement.

Required skeleton when a row earns a split:

```text
<owning-subsystem>/
  mod.rs                          # aggregation/facade only
  <native-authority-family>.rs     # proof-bearing native carrier or admission path
  <proof-retention-family>.rs      # retained certification/diagnostic evidence
  <denial-or-diagnostic-family>.rs # typed failure context when the row touches failure
  terminal_report_export.rs        # JSON/report projection only, never authority input
```

Forbidden skeleton names remain forbidden even in tests: `helpers`, `common`,
`legacy`, `compat`, `shim`, `raw`, `old`, `new`, and cleanup-only folders.

###### Row Execution Order

1. Replace the admitting or retaining choke point first.
2. Delete old constructors, parsers, helpers, fields, aliases, fake-current-test
   support, and terminal-to-authority imports outright.
3. Update direct facade, harness, diagnostics, certification, replay, current
   tests, and UI fixtures in the same batch only where coupled to the row.
4. Add or update hostile tests and compile-fail proof when public or sealed
   construction changed.
5. Split only row-touched files whose responsibility or line cap would otherwise
   preserve the old seam.
6. Run focused proof commands with 10 minute cargo timeouts.
7. Lock back by recording classifications, proof commands, residue scans, and
   the closed finish tuple.

Default proof commands after a source-changing row:

```text
cargo fmt -p worth-runtime-bridge
cargo check -p worth-runtime-bridge --tests
cargo test -p worth-runtime-bridge <focused-row-filter> -- --nocapture
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
```

Skip the compile-fail test only if the row does not touch public/sealed type
boundaries. Do not run cargo tests for planning-only Phase 1 changes.

No-row proof commands:

```text
cargo fmt -p worth-runtime-bridge
git diff --check -- plans/worth-runtime-bridge/aspect_native_refactor.md crates/worth-runtime-bridge
```

Do not run cargo tests for no-row planning or classification-only changes. For
a documentation-only runbook edit, `cargo fmt` is unnecessary unless Rust code
changed; use scoped `git diff --check` on the spec as the proof.

###### Strict Reopen Queue

| Row | Status | Stop condition |
| --- | --- | --- |
| `P1-R1 diagnostics public identity lookup` | Closed 2026-06-02. | Public/facade/harness diagnostics lookups no longer accept raw route, invalidation, workload, history, merge, preview-session, bulk, or adjacent writeback retained-record identity text; callers pass typed bridge identity carriers; compile, focused tests, row scans, and lockback passed. |
| `P1-R2 facade/current-test authority helper identity constructors` | Closed 2026-06-02. | Facade writeback support helpers require typed declaration and causality identities; causal-envelope preview helpers require typed preview identity carriers; subscription preview helpers consume a typed preview identity packet; retained writeback-chain helper consumes a typed writeback-chain input packet; local mutation-bundle helpers require typed causality, effect, and idempotence identities; row-touched files are under 400 lines; exact row residue scans and focused tests pass. |

Closed row evidence packet:

```text
Phase 1 row:
- Blocker name: diagnostics public identity lookup raw text inputs.
- Old authority shape: public/facade/harness diagnostics lookups accepted `&str`
  identities for route, invalidation, workload, historical, merge, preview,
  bulk, and adjacent writeback retained records.
- Native replacement: typed bridge identity carriers, including
  `BridgeRouteIdentity`, `BridgeInvalidationIdentity`,
  `BridgeWorkloadIdentity`, `BridgeHistoricalEvaluationRecordIdentity`,
  `BridgeHistoricalEvaluationFailureIdentity`, `BridgeMergeRecordIdentity`,
  `BridgePreviewSessionIdentity`, `PreviewExecutionRecordIdentity`,
  `BridgePreviewDiscardRecordIdentity`, and writeback retained-record identity
  carriers for admission, execution, mapper envelope/input/record, and replay.
- Choke point: `diagnostics/facade/query/*`, `diagnostics/state/*`,
  `diagnostics/handle*`, `facade/standard_path/diagnostics.rs`,
  `facade/runtime/speculation.rs`, and direct facade/harness/query callers.
- How far back this moves: direct facade, harness, pricing, retained mapping,
  query fixture, standard-path, and replay callers pass typed identities; no
  caller `as_str()` shim remains at a public retained-identity lookup boundary.
- Direct production/facade/harness scope: diagnostics facade query modules,
  diagnostics retained state query modules, diagnostics handles, standard-path
  diagnostics facade, runtime speculation replay, retained causal mapping, and
  direct caller tests/support.
- Current-test detox: current tests construct, retain, or reuse native identity
  carriers; compile-fail tests prove raw `&str` calls are gone without teaching
  fake authority construction.
- New directory skeleton: no split. Touched files are already below the line cap;
  split only if a lockback gate proves a row-touched file now hides a second
  authority/import-wall blocker.
- Out of scope: read-only digest string selectors on already-typed diagnostic
  records, terminal JSON report keys, canonical digest mechanics, and human
  presentation strings that cannot feed authority.
- Row scan: public diagnostics/speculation/writeback retained-identity lookups
  containing `&str`, caller `.as_str()`, string literals, or fake current-test
  placeholders at the public lookup/replay boundary.
- Focused proof: `cargo fmt -p worth-runtime-bridge -p worth-query`,
  `cargo check -p worth-runtime-bridge --tests`,
  `cargo check -p worth-query --tests`,
  `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture`, plus focused diagnostics/speculation/writeback/causal-envelope
  tests affected by the row.
```

Closeout proof commands, after R1 and any exact adjacent row are closed:

```text
cargo fmt -p worth-runtime-bridge -p worth-query
cargo check -p worth-runtime-bridge --tests
cargo check -p worth-query --tests
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
cargo test -p worth-runtime-bridge speculation -- --nocapture
cargo test -p worth-runtime-bridge diagnostics_explanations -- --nocapture
cargo test -p worth-runtime-bridge writeback -- --nocapture
cargo test -p worth-runtime-bridge causal_envelope -- --nocapture
```

Every cargo command uses a 10 minute timeout. Add more focused tests only if a
lockback gate opens a new exact row. Do not run broad tests for planning-only
Phase 1 changes.

Residue scans for `P1-G1` through `P1-G3`:

```text
rg -n 'replay_preview_bundle\(\s*"|replay_preview_bundle\([^\n]*as_str\(|route_record_for_route_identity\([^\n]*as_str\(|historical_record_for_record_identity\([^\n]*as_str\(|bulk_record_for_workload_identity\([^\n]*as_str\(|preview_.*_for_session_identity\([^\n]*as_str\(|writeback_.*for_identity\([^\n]*&str|writeback_.*for_identity\([^\n]*as_str\(' crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests crates/worth-query/src -g '*.rs'
rg -n 'pub fn .*&str|pub\(crate\) fn .*&str' crates/worth-runtime-bridge/src/diagnostics/handle.rs crates/worth-runtime-bridge/src/diagnostics/handle_history.rs crates/worth-runtime-bridge/src/diagnostics/facade/query crates/worth-runtime-bridge/src/diagnostics/state crates/worth-runtime-bridge/src/facade/standard_path crates/worth-runtime-bridge/src/facade/runtime/speculation.rs -g '*.rs'
rg -n 'todo!\(|unimplemented!\(|sealed_authority_placeholder::<&str>\(\).*replay_preview_bundle|replay_preview_bundle\(\s*"' crates/worth-runtime-bridge/tests/ui crates/worth-runtime-bridge/src crates/worth-query/src -g '*.rs'
rg -n 'serde_json::Value|json!' crates/worth-runtime-bridge/src -g '*.rs'
```

Lockback evidence for `P1-R1`:

- Row-local raw identity scans found no raw string replay calls, no caller
  `.as_str()` diagnostics/speculation/writeback retained-identity lookup calls,
  and no stale public `&str` signatures in the row choke points.
- The only row-local `&str` fixture hit is
  `writeback_diagnostics_lookup_requires_record_identity.rs`, which is an
  intentional compile-fail proof that raw string lookup no longer type-checks.
- Runtime-bridge UI fixtures under the row scope contain no `todo!()` or
  `unimplemented!()` placeholders.
- Production JSON scan found no `serde_json::Value` or `json!` in native
  diagnostics, facade, source, routing, writeback, or speculation authority
  directories. Remaining JSON hits are classified under terminal report/export
  projection topology.
- Row-touched files are under the 400-line cap; the largest touched file is
  `facade/runtime/speculation.rs` at 374 lines.
- Proof commands passed with 10 minute command timeouts:
  `cargo fmt -p worth-runtime-bridge -p worth-query`,
  `cargo check -p worth-runtime-bridge --tests`,
  `cargo check -p worth-query --tests`,
  `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture`, `cargo test -p worth-runtime-bridge speculation --
  --nocapture`, `cargo test -p worth-runtime-bridge diagnostics_explanations
  -- --nocapture`, `cargo test -p worth-runtime-bridge writeback --
  --nocapture`, and `cargo test -p worth-runtime-bridge causal_envelope --
  --nocapture`.

The JSON scan is a classification gate, not a work queue. It opens work only
when a native authority module imports or stores JSON as proof rather than
terminal report/export projection.

Blocker threshold:

| Hit kind | Open a row only if | Otherwise classify as |
| --- | --- | --- |
| `payload()` / `bytes()` | The value is bridge-owned raw snapshot/writeback/source authority after native admission. | Foundational accessor or canonical binary content projection. |
| `as_bytes()` | The bytes are caller-supplied authority or foreign-format recovery. | Canonical digest mechanics from named native basis. |
| `serde_json::Value` / `json!` | JSON is retained before terminal export or imported by native authority. | Terminal report/export/capture projection. |
| `String`, `Vec<String>`, maps | The collection admits, retains, proves, diagnoses, certifies, replays, or teaches authority where a typed carrier exists. | Presentation, semantic label, digest projection, or bounded report key. |
| Public `&str` lookup input | The input selects retained authority/proof/certification identity that already has a typed carrier. | Read-only digest/projection accessor or terminal report lookup. |
| `legacy`, `compatibility`, `fallback`, `shim`, `raw`, `payload`, `bytes`, `surface_label`, `domain_payload` | The spelling names a current production/test authority lane or public helper. | Historical docs, terminal external I/O, canonical mechanics, or human assertion text. |
| Fake digest-shaped literals | The literal constructs authority or proof. | Assertion against derived digest prefix, provided no construction path accepts it. |

Default answer to "what is left in Phase 1?": nothing executable is open. Phase
1 is closed; only a named later-phase concern that proves a current executable
old authority shape can reopen it.

Efficiency rules for Phase 1:

- Do not use broad scans as a work queue. Broad scans are proof gates only.
- Do not split files for cleanliness unless the split also deletes a blocker,
  quarantines terminal I/O, or restores the line cap in touched code.
- Do not rename old vocabulary unless the same row deletes the old executable
  authority shape.
- Do not patch tests with compatibility helpers. Tests must construct native
  carriers or sealed compile-fail placeholders.
- Do not open a second row while the first row is unresolved.
- Do not run cargo tests for planning-only Phase 1 changes.

##### Active Ledger

| Item | Current state | Efficient next action |
| --- | --- | --- |
| Strict reopen queue | Empty. | Continue the active later phase. Do not run Phase 1 scans or edits without a named later-phase concern. |
| Current concern | None. | If later work finds a suspected old authority seam, fill a concern packet before inspection. |
| Current open row | None; `P1-R1 diagnostics public identity lookup` and `P1-R2 facade/current-test authority helper identity constructors` are closed. | No Phase 1 source work. |
| Closeout proof | Passed for R1/R2 lockback and the latest no-row decision. | Re-run only for a fresh closeout report or after a new reopened row closes. |

##### Superseded Finish Notes

The `Phase 1 Finish Cockpit` above is the only active finish plan. Older lane,
sprint, concern, and row-loop sections below remain as archived guardrails and
definitions only. Do not treat them as a second execution board.

Phase 1 has no cleanup backlog. Large files, broad helper topology, and ugly
names are Phase 1 work only when they hide or preserve a proven legacy authority
path, import-wall violation, fake current-test shim, or touched-file line-cap
failure. Otherwise they belong to the later phase that actually changes the
aspect authority seam.

##### Archived Phase 1 Closeout Certification Sprint

This older sprint is retained only as audit history. Use `Phase 1 Finish
Cockpit` and its `P1-0` through `P1-7` board steps for execution. The
archived `P1-C*` names below must not be treated as a second active board.

1. `P1-C0 Status lock`: Read this runbook and confirm the strict reopen queue is
   empty. If no named concern exists, do not inspect arbitrary files before the
   proof gates.
2. `P1-C1 No-shim scan`: Run the crate/current-test residue scan from
   `Phase 1 Gate Commands`. Classify every hit into `authority-blocker`,
   `public-api-blocker`, `test-shim-blocker`, `terminal-io`,
   `canonical-mechanics`, or `presentation-only`.
3. `P1-C2 JSON/import wall`: Run the JSON/API scan and import-wall scan. JSON
   may remain only under `terminal_report_export`, terminal capture/projection,
   or explicitly foreign I/O topology. If a native authority module imports JSON
   projection as proof construction, open one blocker row.
4. `P1-C3 Public/test constructor wall`: Inspect only hits from the no-shim scan
   that are public, facade-visible, harness-visible, or current-test fixtures.
   The stop condition is either "sealed/native or terminal-only" or a complete
   blocker row. Do not chase old words in docs or human presentation.
5. `P1-C4 Line-cap and topology restraint`: Run the runtime-bridge Rust line-cap
   scan. Split only files touched by a blocker rewrite or files already proven
   to hide an authority/import-wall violation. No cleanup-only split belongs to
   Phase 1.
6. `P1-C5 Proof commands`: If no blocker row opened, run only the closeout proof
   commands listed below. If a row opened, rewrite that row vertically first,
   then run the row proof plus closeout proof.
7. `P1-C6 Lockback`: Record the sprint result in this ledger. If no row opened,
   Phase 1 remains closed. If a row opened and closed, the strict reopen queue
   must return to empty before later-phase work resumes.

Sprint output must be a table with exactly these columns:

| Gate | Result | Evidence | Action |
| --- | --- | --- | --- |
| `P1-C1` | `clean`, `classified`, or `row-opened` | Scan summary and blocker count. | `none` or row name. |
| `P1-C2` | `clean`, `classified`, or `row-opened` | Import-wall/JSON path summary. | `none` or row name. |
| `P1-C3` | `clean`, `classified`, or `row-opened` | Public/test constructor summary. | `none` or row name. |
| `P1-C4` | `clean` or `row-opened` | Line-cap/topology summary. | `none` or row name. |
| `P1-C5` | `passed` or `failed` | Cargo/check proof commands. | `none` or row name. |

Efficiency rule: do not edit during `P1-C1` through `P1-C4`. The first edit is
allowed only after a complete row packet exists. That row owns production,
facade/harness, current tests, UI fixtures, residue scan, focused tests, and
lockback in one vertical batch.

##### Concern Packet

Create this packet before any Phase 1 inspection beyond reading the named file.
If any field is missing, the concern is too vague and Phase 1 stays closed.

```text
Phase 1 concern:
- Suspected seam:
- Why this might feed authority:
- Named file(s) to inspect:
- Direct caller radius:
- Stop condition for dismissal:
```

Concern inspection has a hard radius: the named file, direct callers, direct
tests, and direct terminal/export module if the seam may cross an I/O wall. Do
not add a second seam during the same concern.

##### Blocker Definition

Reopen Phase 1 only for a current production path, public/facade path, harness
helper, current test, or UI compile-fail fixture that still uses one of these as
authority rather than terminal projection or explicit foreign I/O:

- `serde_json::Value`, `json!`, or JSON maps as retained meaning.
- Raw bytes, payload bytes, aspect bytes, or byte recovery as value meaning.
- Surface labels, aspect labels, route prefixes, or foreign labels as target
  meaning.
- Arbitrary digest strings, caller-provided digest labels, or digest-only
  writeback/proposed-effect construction.
- Raw string collections for identities, grouping keys, route members, lineage,
  counters, certification rows, or replay evidence.
- Current tests or compile-fail fixtures that teach fake native authority
  literals, old public names, `todo!`, or `unimplemented!`.

Do not reopen Phase 1 for foundational `payload()`/`bytes()` access, canonical
`as_bytes()` hashing over named native bases, terminal report JSON, or human
presentation text unless that surface feeds authority.

##### Row Packet

No Phase 1 source work may begin until every field is filled.

```text
Phase 1 row:
- Blocker name:
- Old authority shape:
- Native replacement:
- Choke point:
- How far back this moves:
- Direct production/facade/harness scope:
- Current-test detox:
- New directory skeleton:
- Out of scope:
- Row scan:
- Focused proof:
```

The row must name one seam, not a subsystem. The native replacement must be a
foundational carrier, mask, locator, authoritative state/patch, typed identity,
typed denial, typed evidence packet, or sealed proof carrier. "Rename it" is not
a valid replacement. `New directory skeleton` must be `no split` unless the same
edit deletes the blocker, quarantines terminal I/O, or restores the line cap.

##### Execution Loop

| Packet | Work | Stop condition |
| --- | --- | --- |
| `P1-0 Status Lock` | Read this runbook and active ledger. | Empty queue keeps Phase 1 closed. |
| `P1-1 Concern Triage` | Fill the concern packet and inspect only that radius. | Dismissed, later-phase-owned, or row-ready. |
| `P1-2 Row Readiness` | Fill the row packet completely. | Complete row exists, or no edit is allowed. |
| `P1-3 Vertical Rewrite` | Delete the old authority shape through direct production, facade/harness, current tests, and UI fixtures. | No alias, fallback, compatibility shim, fake test path, or old construction remains in row scope. |
| `P1-4 Structural Restraint` | Split only as declared in the row skeleton. | New files/folders are named by authority, lifecycle, proof, or terminal-boundary responsibility. |
| `P1-5 Row Proof` | Run the row scan, focused tests, package check, compile-fail if needed, line-cap check, and scoped diff hygiene. | Row closes or is corrected before any new row starts. |
| `P1-6 Lockback` | Record the closure evidence in this ledger. | Strict reopen queue is empty again. |

##### Classification Table

| Classification | Meaning | Required action |
| --- | --- | --- |
| `authority-blocker` | Production/facade/harness authority depends on JSON, bytes, labels, arbitrary digests, old vocabulary, or raw string collections. | Rewrite with foundational carriers, masks, locators, authoritative state/patches, typed denials, typed identities, or sealed proof carriers. |
| `public-api-blocker` | Public or crate-wide helper accepts old authority input or preserves old constructor shape. | Delete it or make invalid construction uncallable with sealed native construction and compile-fail proof. |
| `test-shim-blocker` | Current tests teach old construction, fake authority literals, old names, `todo!`, or `unimplemented!`. | Delete the shim and construct native carriers or sealed placeholders. |
| `terminal-io` | JSON/report/capture code renders typed artifacts and cannot feed authority. | Keep under terminal export/capture topology and prove the import wall. |
| `canonical-mechanics` | Foundational payload/bytes access, canonical hashing, digest-prefixing, or counter bytes derive from named native basis carriers. | Classify only unless caller-supplied authority is present. |
| `presentation-only` | Human/report text derives from typed artifacts and is never accepted as authority. | Classify only unless it crosses back into authority. |

##### Closeout Certification Commands

Run these only after a code-changing reopened row or when producing a fresh
Phase 1 closeout report. Use a 10 minute timeout per cargo command:

```text
cargo fmt -p worth-runtime-bridge
cargo check -p worth-runtime-bridge --tests
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
```

Also run the row's focused test family, row-local residue scan, runtime-bridge
Rust line-cap scan, terminal JSON/import-wall scan if JSON topology moved or the
team requested a fresh closeout report, and scoped `git diff --check`.

Do not run cargo tests for Phase 1 planning or documentation-only updates.

##### Historical Phase 1 Finish Protocol

This closed protocol is retained as audit evidence for the completed Phase 1
restart. It is not the active execution loop. Use the
`Phase 1 Finish Cockpit` above for any future reopened Phase 1-shaped
concern.

The completed restart executed Phase 1 as five serial gates. A later gate did
not start until the earlier gate was either closed or had produced an exact
blocker row.

| Gate | Name | Purpose | Allowed edits | Stop condition |
| --- | --- | --- | --- | --- |
| `P1-F0` | Bounded rebaseline | Run the Phase 1 scans once and classify hits. | Spec/workboard only, unless a scan reveals a precise executable blocker. | Every hit is either in a blocker row or classified as terminal I/O, canonical mechanics, sealed compile-fail proof, or presentation-only. |
| `P1-F1` | Exact authority blocker rewrite | Delete one blocker row at a time from production plus direct callers/tests. | Native carrier/proof introduction, old constructor/accessor deletion, coupled test/facade update, line-cap split only if touched files require it. | The blocker term no longer exists as an authority path; any remaining spelling is classified and harmless. |
| `P1-F2` | Test-shim detox for touched rows | Ensure current tests do not teach the old path removed by `P1-F1`. | Replace fake literals or old helper calls with native carriers or sealed compile-fail placeholders. | Focused tests prove the native construction path; compile-fail proof exists when a public/sealed boundary changed. |
| `P1-F3` | Terminal JSON/import wall proof | Prove JSON remains terminal-only and cannot feed authority. | Only move or rename modules if a native authority module imports terminal I/O. | Source, routing, writeback, diagnostics, subscription, certification, and facade authority paths do not import terminal JSON/export/capture modules. |
| `P1-F4` | Closeout proof | Run final formatting, checks, line-cap scan, no-shim scan, and diff hygiene. | No cleanup edits. Fix only a newly proven blocker from the gate. | Phase 1 closure statements are true and recorded. |

Historical operational batch order:

1. Run `P1-F0` once and populate the active workboard.
2. If no rows are open, run `P1-F3` and `P1-F4`, record closure, and leave Phase
   1.
3. If rows are open, execute only the first row through `P1-F1` and `P1-F2`.
4. Run the row scan and focused proof for that row; mark it closed only after
   both pass or have written non-blocker classifications.
5. Repeat from the next open row without re-running the broad manifest.
6. After the final row, run `P1-F3` and `P1-F4` once and stop.

This plan deliberately optimizes editing time over search time: broad scans
only create the queue; row-local reads and edits consume the queue.

##### Phase 1 Active Workboard

| Order | Row | Files | Classification | Native replacement | Coupled proof |
| --- | --- | --- | --- | --- | --- |
| 1 | Bulk canonical workload request members | `src/routing/planning/bulk/types/request_surfaces.rs`, `src/routing/planning/bulk/planner/workload_pipeline.rs`, `src/harness/tests/planning/bulk_workload/canonical_summary.rs` | Closed `authority-blocker` | Replaced `Arc<str>` member collections and `workload_segment_digests` with typed member identities. Existing route, subscription-slice, commit, snapshot, and branch identities now flow through the canonical request directly; bulk-scoped continuity member, truth-view member, and workload segment identities cover derived member categories. Segment identity is digest-shaped from private native basis and no longer exposes `segment|commit=...` text. | `cargo fmt -p worth-runtime-bridge`; `cargo test -p worth-runtime-bridge bulk_workload -- --nocapture` (10 passed); `cargo check -p worth-runtime-bridge --tests`; targeted scan has no old public member/accessor shape, with only the private `bulk-workload-segment` digest-basis row and a negative assertion for `segment|commit=`. |
| 2 | Historical continuity lineage resolved keys | `src/adapter/continuity_lineage.rs`, `src/continuity/resolution.rs`, `src/harness/tests/planning/continuity/planning_requests.rs`, plus direct constructor callers in builder/facade/harness fixtures | Closed `authority-blocker` | Replaced `canonical_resolved_lineage_keys` and `canonical_resolved_record_keys` raw string sets with `BridgeHistoricalResolvedLineageIdentity` and `BridgeHistoricalResolvedRecordIdentity`. Historical lineage authority now accepts typed identity vectors, stores typed identity slices, enforces canonical ordering/duplicate denial on typed carriers, and renders `as_str()` only for digest-basis preparation. The old resolved-key accessors and resolved-key error variants are gone. Coupled tests construct typed identities directly; no resolved-key conversion helper remains. | `cargo fmt -p worth-runtime-bridge`; `cargo test -p worth-runtime-bridge continuity -- --nocapture` (48 passed); `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`; targeted old-key/raw-vector scan clean; touched line-cap scan clean; scoped `git diff --check` had only existing LF/CRLF warnings. |
| 3 | Phase 1 closeout proof | Whole `worth-runtime-bridge` crate | Closed proof gate | No cleanup edits were made after Row 2. The exact gate scans classify remaining hits as terminal I/O, canonical mechanics, sealed compile-fail proof, counter construction mechanics, or presentation-only. JSON hits are under terminal report/capture/export topology; authority import-wall scan has no hits; no unresolved resolved-key/string-collection authority row remains. | `cargo fmt -p worth-runtime-bridge`; `cargo test -p worth-runtime-bridge continuity -- --nocapture` (48 passed); `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`; whole runtime-bridge Rust line-cap scan clean; targeted resolved-key/raw-vector residue scan clean; import-wall scan clean; broad no-shim scan classified as terminal JSON/export, canonical digest mechanics, foundational `payload()`/`bytes()` accessors, `digest_input_bytes` counter proof, `from_values` counter constructors, or presentation-only assertions; scoped `git diff --check` had only existing LF/CRLF warnings. |

Strict reopen rows, recorded 2026-06-02:

| Order | Row | Files | Classification | Native replacement | Coupled proof |
| --- | --- | --- | --- | --- | --- |
| 4 | Bulk packet source and continuity identities | `src/routing/planning/bulk/types/packet_families.rs`, `src/routing/planning/bulk/types/reductions.rs`, `src/routing/planning/bulk/types/route_packets.rs`, `src/routing/planning/bulk/planner/member_identities.rs`, `src/routing/planning/bulk/planner/packet_reduction.rs`, `src/routing/planning/bulk/planner/admission.rs`, `src/routing/planning/bulk/planner/support.rs`, direct bulk workload/packet-reduction tests | Closed `authority-blocker` | Deleted raw `Arc<str>` packet fields, raw tuple grouping, raw source locality sets, and raw continuity-output grouping for branch/commit/snapshot/continuity authority. Routing, truth-view, continuity, and reduced artifacts now carry `TruthBranchIdentity`, `TruthCommitIdentity`, `TruthSnapshotIdentity`, `BulkTruthViewMemberIdentity`, `BulkContinuityMemberIdentity`, route identities, and subscription-slice identities as authority carriers. String accessors remain projection only. The shared bulk member identity derivation lives in `planner/member_identities.rs`; no cleanup-only split was performed. | `cargo fmt -p worth-runtime-bridge`; `cargo test -p worth-runtime-bridge packet_reduction -- --nocapture` (11 passed); `cargo test -p worth-runtime-bridge bulk_workload -- --nocapture` (11 passed); `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`; row scan found no raw packet source fields, raw continuity fields, raw tuple maps, or raw source `BTreeSet::<Arc<str>>` groupings under `routing/planning/bulk`; line-cap scan clean; scoped `git diff --check` had only existing LF/CRLF warnings. |
| 5 | Structural fingerprint snapshot carrier | `src/structural/fingerprints.rs`, direct structural tests | Closed `authority-blocker` | Replaced stored `snapshot_identity: Arc<str>` and the arbitrary string constructor argument with `TruthSnapshotIdentity`. `StructuralFingerprint::snapshot_identity()` now returns the typed identity; snapshot text is explicit projection through `snapshot_identity_text()` and canonical-basis rendering only. Direct structural/facade tests construct and assert typed snapshot identities; no compatibility alias, string constructor, or test shim remains. No topology split was required because touched files stayed under the line cap. | `cargo fmt -p worth-runtime-bridge`; `cargo test -p worth-runtime-bridge structural -- --nocapture` (54 passed); `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`; row scan found no `snapshot_identity: Arc<str>` or weak `snapshot_identity_proof` accessor under structural/facade structural tests; touched line-cap scan clean; scoped `git diff --check` had only existing LF/CRLF warnings. |
| 6 | Speculation promotion/reuse branch bindings | `src/speculation/contracts.rs`, direct speculation tests | Closed `authority-blocker` | Replaced `truth_branch_identity: Arc<str>` and `signal_branch_identity: Arc<str>` in promotion admissibility and preview reuse equivalence proofs with `TruthBranchIdentity` and `BridgeSignalBranchIdentity` cloned from the declaration branch binding. Matching now compares typed branch carriers; canonical basis renders `as_str()` only for digest preparation. Added direct proof tests for promotion and reuse retained typed branch identities. The pricing-shock speculation proof now compares delivered fanout targets as a set because delivery order is not the contract under that standard-path assertion. | `cargo fmt -p worth-runtime-bridge`; `cargo test -p worth-runtime-bridge speculation -- --nocapture` (29 passed); `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`; row scan found no raw truth/signal branch identity fields in `speculation/contracts.rs`; touched line-cap scan clean; scoped `git diff --check` had only existing LF/CRLF warnings. |
| 7 | Strict Phase 1 closeout proof | Whole `worth-runtime-bridge` crate | Closed proof gate | No cleanup edits were made during closeout. Remaining broad no-shim hits are classified as foundational accessor (`payload()`/`bytes()` on validated/foundational carriers), canonical digest mechanics (`as_bytes()` and digest prefixes derived from named native canonical bases), digest-input counter proof, terminal report/export/capture JSON, or presentation-only test assertions. Import-wall scan proved terminal JSON/export/capture modules are not imported by source/routing/writeback/diagnostics/subscription/facade authority paths. No new executable authority blocker row remains. | `cargo fmt -p worth-runtime-bridge`; `cargo test -p worth-runtime-bridge structural -- --nocapture` (54 passed); `cargo test -p worth-runtime-bridge speculation -- --nocapture` (29 passed); `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`; import-wall scan clean; whole runtime-bridge line-cap scan clean; broad no-shim scan classified with no blocker rows; scoped `git diff --check` had only existing LF/CRLF warnings. |

Do the rows in order. Do not split unrelated god files while executing the
workboard. If a touched file is over 400 lines, split by the new native
responsibility only; do not introduce `helpers`, `common`, `utils`, `legacy`,
`compat`, or `fallback` topology.

Phase 1 closeout status, updated 2026-06-02: rows 4-7 are closed and the strict
reopen queue is empty. Future Phase 1-shaped findings must become exact
blocker rows inside the phase that discovers them. Do not reopen broad Phase 1
archaeology after this proof.

##### Phase 1 Row 2 Closeout Record

This source-changing batch is closed for the current manifest.

Batch target:

- Native authority seam being replaced: historical continuity lineage authority
  currently admits resolved lineage and record evidence as raw `Arc<str>` key
  collections.
- How far back this batch must move to delete upstream legacy: only the
  historical lineage authority constructor, its continuity-resolution consumer,
  direct facade exports if the new carriers need export, and coupled continuity
  planning tests. Do not reopen unrelated subscription continuation,
  certification digest-list, stream lowered-member, source record, or structural
  presentation-list hits unless the row-2 coupling search proves they feed the
  same authority seam.
- Files expected to rewrite:
  `src/adapter/continuity_lineage.rs`,
  `src/continuity/resolution.rs`,
  `src/harness/tests/planning/continuity/planning_requests.rs`, plus direct
  facade/export or error-kind files only if compilation requires the hard break.
- New directory skeleton for this slice: no new directory is planned. If
  `continuity_lineage.rs` crosses the 400-line cap while deleting the blocker,
  split it into `adapter/continuity_lineage/{mod.rs,resolved_identity.rs}` so
  resolved identity admission is separate from topology/digest authority.
- Aspect-native carriers introduced: typed historical resolved-lineage identity
  and typed historical resolved-record identity, both canonical-orderable,
  duplicate-denied, and rendered with `as_str()` only for digest-basis
  preparation or presentation.
- Legacy carriers removed: `canonical_resolved_lineage_keys`,
  `canonical_resolved_record_keys`, `Vec<Arc<str>>` constructor arguments for
  resolved authority, and `&[Arc<str>]` resolved-key accessors.
- Public/test/harness shims deleted: tests must construct the typed identities;
  no helper may accept raw resolved-key strings and convert them for caller
  convenience.
- Residue scan terms this batch must clear:
  `canonical_resolved_lineage_keys`,
  `canonical_resolved_record_keys`,
  `DuplicateResolvedLineageKeys`,
  `NonCanonicalResolvedLineageKeys`,
  `DuplicateResolvedRecordKeys`,
  `NonCanonicalResolvedRecordKeys`,
  and `BridgeHistoricalLineageAuthority::try_new` calls with raw string
  vectors.
- Certification and compile checks completed with 10-minute timeouts:
  `cargo fmt -p worth-runtime-bridge`;
  `cargo test -p worth-runtime-bridge continuity -- --nocapture` (48 passed);
  `cargo check -p worth-runtime-bridge --tests`;
  `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`
  (1 passed);
  targeted residue scan clean;
  touched-file line-cap scan clean;
  scoped `git diff --check` reported only existing LF/CRLF warnings.

##### Phase 1 Batch Template

Every Phase 1 code batch must be expressible as:

```text
1. name the blocker row
2. name the native carrier/proof that replaces it
3. delete the old constructor/path/helper in production and tests
4. add compile-fail or hostile proof if a public/test authority seam changed
5. run only the focused verification for touched proof families
6. record the row as closed
```

Rows that cannot name a native replacement are not ready to edit. Rows whose
replacement is "rename it" are invalid; Phase 1 deletes old authority shapes,
not old spellings.

##### Phase 1 Gate Commands

```powershell
rg -n "legacy|fallback|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|serde_json::Value|json!|from_value|to_value|Vec<String>|Vec<&str>|BTreeSet<String>|HashSet<String>|VecDeque<String>|from_digests|digest.*label|label.*digest|todo!|unimplemented!" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
rg -n "serde_json::Value|json!|serde_json::Map|from_value|to_value|StructuredValue::Json|SnapshotPayload" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
rg -n "terminal_report_export|json_projection" crates/worth-runtime-bridge/src/source crates/worth-runtime-bridge/src/routing crates/worth-runtime-bridge/src/writeback crates/worth-runtime-bridge/src/diagnostics crates/worth-runtime-bridge/src/subscription crates/worth-runtime-bridge/src/facade -g "*.rs"
rg -n "fallback|legacy|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|route:sha256|effect:sha256|strategy:sha256|truth-state:sha256|truth-view:sha256|todo!|unimplemented!" crates/worth-runtime-bridge/src/facade/tests crates/worth-runtime-bridge/src/harness/tests crates/worth-runtime-bridge/tests/ui -g "*.rs"
Get-ChildItem -Recurse crates/worth-runtime-bridge/src,crates/worth-runtime-bridge/tests -Filter *.rs | ForEach-Object { $lines=(Get-Content $_.FullName | Measure-Object -Line).Lines; if($lines -gt 400){ "$lines $($_.FullName)" } }
```

End-of-gate verification, only after a code-changing blocker rewrite:

```powershell
cargo fmt -p worth-runtime-bridge
cargo check -p worth-runtime-bridge --tests
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
git diff --check -- crates/worth-runtime-bridge plans/worth-runtime-bridge/aspect_native_refactor.md
```

Add focused tests for the touched proof family, using a 10-minute timeout for
each cargo command. Do not run cargo tests for docs-only plan changes.

#### Historical Phase 1 Final Operating Plan

This subsection is historical evidence from a completed reopened Phase 1 audit.
It is superseded by `Phase 1 Completion Runbook` above. Do not use it as a live
worklist.

Phase 1 is not a general cleanup pass. Work belongs here only if it deletes an
executable legacy authority seam, proves an old construction path uncallable,
quarantines terminal external I/O, or fixes a touched line-cap violation while
doing one of those things.

Current bounded manifest, recorded 2026-06-02:

| Area | Current result | Decision |
| --- | --- | --- |
| Runtime-bridge line cap | Clean. | No Phase 1 split is required unless a touched file crosses the cap. |
| JSON/API scan | `serde_json::Value`, `json!`, and `serde_json::Map` hits are under terminal report export/capture topology; `from_values` hits are subscription counter constructors, not serde lowering. | Keep JSON terminal-only and verify the import wall after edits. |
| Current-test no-shim scan | Only canonical `as_bytes()` digest mechanics and `digest_input_bytes` counter proof references. | No broad test detox pass remains; update tests only if a blocker rewrite touches them. |
| Broad non-terminal vocabulary scan | Clean after Gate 1. `harness/adapter/adapter_impl/policy/shared_artifacts.rs` no longer exposes `combined_digest(label, values)`. | Policy certification digest identity is now owned by named proof-basis functions backed by a closed artifact enum. |

Execution gates:

| Gate | Start point | Required action | Stop condition |
| --- | --- | --- | --- |
| 1. Policy certification digest-basis hard break | Complete. `harness/adapter/adapter_impl/policy/{certification_digest_basis.rs,shared_artifacts.rs,certification_execution.rs,terminal_report_export/json_projection.rs}` | Deleted generic `combined_digest(label, values)` from non-terminal policy certification. Added closed `PolicyCertificationDigestArtifact` selection plus named digest-basis functions for provenance policy equivalence, replay, diagnostics, rejection failures, rejection diagnostics, ambient-leak policy/replay/diagnostics, semantic route policy, and empty terminal provenance report projection. | `rg -n "combined_digest\(|digest.*label|label.*digest" crates/worth-runtime-bridge/src/harness/adapter/adapter_impl/policy -g "*.rs"` has no hits. |
| 2. Import and JSON wall | `harness/**/terminal_report_export/**` plus source/routing/writeback/diagnostics/subscription/facade authority trees | Confirm JSON renders typed artifacts only and native authority trees do not import terminal JSON projection modules. If a terminal file is constructing authority-shaped proof, move that construction into the native policy subsystem before rendering. | JSON/API hits are terminal-only; authority import-wall scan has no hits. |
| 3. Current-test shim confirmation | `facade/tests`, `harness/tests`, `tests/ui` | Do not reopen broad test cleanup. Run the bounded no-shim scan after Gate 1; edit tests only if Gate 1 changed a proof family or the scan finds executable fake authority. | Remaining hits are canonical digest mechanics, counter byte accounting, sealed compile-fail proof, or terminal presentation only. |
| 4. Phase 1 proof | Entire `worth-runtime-bridge` crate | Run formatting, focused policy certification tests if Gate 1 changes code, package check, compile-fail proof if public/sealed APIs changed, line-cap scan, import-wall scan, no-shim scan, and scoped diff check. Use 10-minute timeouts for cargo commands. | Spec records the final classification and Phase 1 stops. Any later Phase 1-shaped blocker must be fixed inside the phase that creates it, not through another open-ended restart. |

Planned Gate 1 skeleton if code changes are needed:

```text
crates/worth-runtime-bridge/src/harness/adapter/adapter_impl/policy/
  shared_artifacts.rs
    - bundle admission and row construction only
    - no generic digest-domain helper
  certification_execution.rs
    - orchestrates certification suites
    - asks named proof-basis functions for certification digests
  certification_digest_basis.rs
    - closed policy certification artifact enum or named evidence packets
    - policy/replay/diagnostics/failure/empty-provenance basis construction
```

Allowed carriers:

- `PolicyCertificationDigestArtifact` as a closed enum if the digest family is
  fixed and all callers provide typed evidence.
- Named evidence packets such as
  `PolicyCertificationReplayDigestBasisEvidence` when an artifact needs several
  native source fields.
- Existing native policy artifacts:
  `AdmittedBridgePolicyContract`, `LoweredBridgeExecutionPolicy`,
  `BridgePolicyProvenanceRecord`, `BridgePolicyReplayBundle`,
  `BridgeRoutePlanningPolicy`, `BridgePolicyRejection`, and typed policy
  resolution rows.

Forbidden in this closeout:

- generic `label: &str` digest-domain parameters outside terminal projection
- caller-supplied digest row lists as proof
- renaming `combined_digest` without replacing its proof boundary
- splitting policy files unless the split houses typed digest-basis authority or
  is needed for the touched line cap
- reopening Phase 1 because broad scans show canonical `as_bytes()` hashing,
  foundational `payload()` accessors, or terminal JSON rendering

Verification commands:

```powershell
cargo fmt -p worth-runtime-bridge
cargo test -p worth-runtime-bridge policy_certification -- --nocapture
cargo test -p worth-runtime-bridge policy -- --nocapture
cargo check -p worth-runtime-bridge --tests
rg -n "combined_digest\(|digest.*label|label.*digest" crates/worth-runtime-bridge/src/harness/adapter/adapter_impl/policy -g "*.rs"
rg -n "serde_json::Value|json!|serde_json::Map|from_value|to_value" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
rg -n "terminal_report_export|json_projection" crates/worth-runtime-bridge/src/source crates/worth-runtime-bridge/src/routing crates/worth-runtime-bridge/src/writeback crates/worth-runtime-bridge/src/diagnostics crates/worth-runtime-bridge/src/subscription crates/worth-runtime-bridge/src/facade -g "*.rs"
rg -n "fallback|legacy|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|route:sha256|effect:sha256|strategy:sha256|truth-state:sha256|truth-view:sha256|todo!|unimplemented!" crates/worth-runtime-bridge/src/facade/tests crates/worth-runtime-bridge/src/harness/tests crates/worth-runtime-bridge/tests/ui -g "*.rs"
Get-ChildItem -Recurse crates/worth-runtime-bridge/src,crates/worth-runtime-bridge/tests -Filter *.rs | ForEach-Object { $lines=(Get-Content $_.FullName | Measure-Object -Line).Lines; if($lines -gt 400){ "$lines $($_.FullName)" } }
git diff --check -- crates/worth-runtime-bridge plans/worth-runtime-bridge/aspect_native_refactor.md
```

Cargo commands in this list must use a 10-minute timeout. Do not run them until
the planned Gate 1 implementation is complete.

Gate 1 proof, recorded 2026-06-02:

| Proof | Result |
| --- | --- |
| `cargo fmt -p worth-runtime-bridge` | Passed. |
| `cargo test -p worth-runtime-bridge policy_certification -- --nocapture` | Passed: 4 tests. |
| `cargo test -p worth-runtime-bridge policy -- --nocapture` | Passed: 61 tests. |
| `cargo check -p worth-runtime-bridge --tests` | Passed. |
| Policy digest-label residue scan | No hits. |
| JSON/API scan | Terminal export/capture topology plus `subscription/counters::from_values` false positives only. |
| Authority import-wall scan | No hits. |
| Current-test no-shim scan | Canonical `as_bytes()` and `digest_input_bytes` proof references only. |
| Runtime-bridge Rust line-cap scan | Clean. |

Remaining Phase 1 closeout action:

- Run scoped diff hygiene after this spec update.
- If clean, Phase 1 is closed for the current manifest. Continue with the next
  aspect-native phase target instead of reopening broad Phase 1 archaeology.

#### Historical Phase 1 Efficient Finish Board

This is historical evidence from an earlier Phase 1 finish board. It is
superseded by `Phase 1 Completion Runbook` above.

Phase 1 is not a cleanup phase. It is only about annihilating executable legacy
authority evidence. Splitting, renaming, or topology work belongs in Phase 1
only when it removes a legacy authority seam, quarantines terminal external I/O,
proves an old public/test construction path uncallable, or keeps a touched file
under the workspace line cap.

The efficient loop is:

1. Rebaseline once.
   - Run one bounded manifest scan over `worth-runtime-bridge` production,
     current tests, and compile-fail fixtures.
   - Write blocker rows here; do not repeat broad discovery until a later edit
     changes one of those rows.
2. Rewrite blocker rows, not scan matches.
   - A blocker is any current production or test path that constructs, stores,
     or teaches authority from JSON, raw bytes, payload-shaped values,
     surface/aspect labels, fallback/compat/shim vocabulary, arbitrary digest
     strings, or raw string collections.
   - Replace blockers with sealed native carriers derived from foundational
     contracts, locators, values, masks, authoritative patches/state, or named
     canonical-basis evidence.
3. Quarantine terminal I/O.
   - JSON may remain only under explicit terminal export/capture topology.
   - Native source, routing, writeback, diagnostics, subscription,
     certification, replay, and facade authority must not import terminal JSON
     projection modules.
4. Detox tests after production authority is clean.
   - Current tests must build native carriers directly or through narrow
     sealed fixture support.
   - Compile-fail fixtures may mention removed APIs only to prove the old path
     is gone; they must not preserve executable fake legacy construction.
5. Close Phase 1 with proof.
   - Run format, package check, compile-fail proof, line-cap scan, import-wall
     scan, no-shim scan, and any focused tests for files changed while fixing
     blockers.
   - Every remaining match must be classified as terminal I/O, presentation
     text, foundational accessor, canonical digest mechanics, or historic docs.

Current 2026-06-02 status:

| Gate | Status | If it fails, start here | Required fix |
| --- | --- | --- | --- |
| Rebaseline manifest | Complete | The first file reported by the bounded scan, not another crate-wide search loop. | Add one blocker row per file, then edit from the rows. |
| Public authority API eradication | Complete | `facade`, `input`, `snapshot`, `routing`, `writeback` public constructors. | Delete old public constructors/aliases and update callers to native carriers in the same batch. |
| Authority collection eradication | Complete | Files with `Vec<String>`, `Vec<&str>`, string sets, digest lists, or `from_digests` authority helpers. | Replace with named proof sets or canonical-basis carriers whose constructors own native evidence. |
| Terminal JSON wall | Complete | Non-terminal JSON/API hits and imports from native authority trees into terminal export modules. | Move authority construction into native modules; leave JSON as rendering only. |
| Current-test no-shim eradication | Complete | `facade/tests`, `harness/tests`, and `tests/ui`. | Delete executable old examples; use sealed native proof construction and compile-fail proofs for removed paths. |
| Closeout proof | Complete | Any failed verification command or unclassified scan hit. | Fix the owning blocker row and rerun only the relevant proof plus the final closeout scans. |

Do not reopen open-ended Phase 1 work while this board is green. If Phase 2+
implementation creates a fresh Phase 1-shaped blocker, fix that blocker inside
the phase that created it and update this board only if the closeout proof
changes.

#### Phase 1 Closeout Execution Plan

Phase 1 is closeout-ready only after the remaining work is executed as
eradication gates, not broad cleanup. File splitting is allowed only when it
removes or quarantines one of the authority seams below, or when a touched file
would otherwise violate the workspace line cap.

Execution discipline for the rest of Phase 1:

- Start each batch from the named files below, not from a crate-wide search.
  Use broad scans only to verify the batch and discover the next bounded seam.
- Do not split files unless the split removes a legacy authority shape,
  quarantines terminal external I/O, or is required because the touched file
  exceeds the workspace line cap.
- Do not rename legacy concepts as a first move. Replace the construction
  boundary with a typed native carrier, update consumers, then delete the old
  carrier/API/test fixture.
- Classify every remaining scan hit into exactly one status: blocker,
  terminal external I/O, presentation/domain data, canonical digest mechanics,
  or historic documentation. Anything classified as blocker must be rewritten
  in the same batch before moving on.
- Prefer narrow proof carriers over generic collections. If the value is an
  identity set, candidate set, request log, equivalence member set, or digest
  basis, the type name must say which proof it carries and its constructor must
  be at the authority boundary that proves it.

Phase 1 finish protocol:

Do not continue Phase 1 as an open-ended search. Execute the remaining work as
the four closed passes below. Each pass starts from the named files, performs
the listed verification scans, and updates this section before moving on.

Phase 1 operator checklist:

1. Rebaseline once, then stop rediscovering. Run the Gate 1 scans, write one
   manifest row per file, and classify each row as blocker, terminal export,
   presentation/domain data, canonical digest mechanics, foundational accessor,
   or historic documentation. Do not run another broad search until a later
   edit changes the manifest.
2. Close authority collections first. Start from the Gate 2 files and delete
   any caller-supplied string list, digest list, identity set, request log, or
   ambiguous collection that can participate in authority. Replace it with a
   sealed proof carrier whose name says which native evidence it freezes.
3. Prove JSON is terminal-only before touching tests. Start from the Gate 3
   terminal export files. If a JSON module constructs authority-shaped records,
   move that construction into the native subsystem and leave JSON as rendering
   only. Then run the authority-tree import wall.
4. Detox current tests after production authority is clean. Start from
   `facade/tests`, `harness/tests`, and `tests/ui`. Delete old executable
   examples rather than preserving them behind test helpers. Compile-fail
   fixtures must prove sealed native construction, not teach legacy inputs.
5. Close with proof, not vibes. Run the Gate 5 commands, line-cap scan,
   import-wall scan, no-shim classification scan, focused touched-family tests,
   and scoped `git diff --check`. Phase 1 is closed only when every remaining
   hit has a written non-blocking classification.

Efficiency rule: if a task does not delete a blocker, quarantine terminal I/O,
prove a native authority boundary, or keep a touched file under the line cap,
it does not belong in Phase 1. Defer it.

Each gate uses this same decision table:

1. If a hit constructs or stores authority from a string, JSON value, raw byte
   buffer, arbitrary digest, or old vocabulary, rewrite it immediately as a
   typed native proof carrier.
2. If a hit is domain text, presentation text, or terminal external I/O, leave
   it only when the file path and names make that non-authority status obvious.
3. If a hit is canonical digest mechanics, keep it only when a named basis
   carrier owns the rows and callers cannot inject arbitrary basis strings.
4. If classification is ambiguous, treat it as a blocker and type it.

#### Historical Phase 1 Completion Control Board

This board is historical closeout evidence. The single operational source of
truth is `Phase 1 Completion Runbook` above. Do not reopen Phase 1 because this older
table mentions unfinished work unless a fresh row packet proves the current
worktree has a blocker.

Phase 1's purpose is legacy authority annihilation, not general cleanup. It is
finished only when current production code, facade APIs, current tests, and
compile-fail fixtures no longer expose an executable path that constructs
source, route, writeback, diagnostic, certification, replay, or subscription
authority from JSON, raw bytes, payload-shaped values, surface/aspect labels,
compatibility/fallback/shim vocabulary, arbitrary digest strings, or raw string
collections.

Efficiency rules:

- Run the rebaseline manifest once per Phase 1 closeout attempt, then edit from
  named blocker rows. Do not keep rediscovering the same residue.
- Every non-final implementation batch must delete a blocker, quarantine a
  terminal I/O boundary, or prove an invalid old construction path uncallable.
- Split files only when the split removes a legacy authority seam, houses a
  native proof carrier, quarantines terminal I/O, or keeps a touched file under
  the 400-line cap.
- Classify ambiguous hits as blockers and type them. No old-path vocabulary gets
  a benefit-of-the-doubt exception in current code or current tests.

Completion gates:

| Gate | Status | Start point | Required action | Proof |
| --- | --- | --- | --- | --- |
| 0. Rebaseline manifest | Complete | `src`, current tests, compile-fail fixtures | One residue pass, one line-cap pass, then write blocker rows here. | Manifest below recorded 2026-06-02. |
| 1. Public authority API eradication | Complete | `facade`, `input`, `snapshot`, `routing`, `writeback` | Remove public/crate-wide constructors that admit authority from JSON/raw bytes/payload text/labels/arbitrary digests/fallback names. | No public blocker rows in the current manifest; compile-fail suite passed. |
| 2. Current-test shim eradication | Complete | `facade/tests`, `harness/tests`, `tests/ui` | Remove executable current-test examples of old authority construction; compile-fail fixtures may mention removed APIs only to prove they are gone. | No-shim scan has only canonical digest mechanics and sealed-proof compile-fail references. |
| 3. Terminal I/O quarantine | Complete | `harness/**/terminal_report_export`, JSON hits outside terminal topology | Keep JSON as report/capture projection from typed artifacts only; enforce import wall from native authority trees. | JSON/API scan and authority import-wall scan passed. |
| 4. Digest/string collection proof | Complete | Manifest rows for digest lists/string collections | Retain digest strings only as derived evidence from named native basis carriers; replace authority collections with proof sets. | Current manifest has no raw string-collection authority rows. |
| 5. Closeout proof | Complete | Gates 0-4 | Run final cargo/check/scans and record classifications. | Proof table below recorded 2026-06-02. |

Current-worktree manifest, recorded 2026-06-02:

| Scan area | Classification | Closeout decision |
| --- | --- | --- |
| Line cap | `clean` | No Phase 1 split is required for line-cap reasons. |
| Import wall | `clean` | Source, routing, writeback, diagnostics, subscription, and facade authority trees do not import terminal report export or JSON projection modules. |
| JSON APIs | `terminal-io` plus `canonical-mechanics` false positives | `serde_json::Value`, `json!`, and `serde_json::Map` hits are under harness terminal report export/capture topology. `from_value`/`to_value` hits are subscription counter `from_values` constructor names, not serde lowering. |
| Old vocabulary | `canonical-mechanics` | No `legacy`, `fallback`, `compatibility`, `compat`, `shim`, `aspect_bytes`, `surface_label`, `aspect_label`, `domain_payload`, `from_digests`, fake `*:sha256:*` test literals, `todo!`, or `unimplemented!` blockers remain. Native digest-family prefixes are derived from typed carriers. |
| Raw/payload/bytes names | `canonical-mechanics` | Remaining hits are foundational `payload()` proof-artifact accessors, foundational binary value `bytes()` canonicalization, `as_bytes()` hashing over named canonical basis rows, and `digest_input_bytes` counters. |
| String collections | `clean` | No `Vec<String>`, `Vec<&str>`, `BTreeSet<String>`, `HashSet<String>`, `VecDeque<String>`, or `&[String]` authority residues remain under current runtime-bridge source/tests. |

Closeout proof, recorded 2026-06-02:

| Proof | Result |
| --- | --- |
| `cargo fmt -p worth-runtime-bridge` | Passed. |
| `cargo check -p worth-runtime-bridge --tests` | Passed. |
| `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture` | Passed. |
| JSON/API scan | Terminal report export/capture modules plus `subscription/counters::from_values` false positives only. |
| Import-wall scan | No hits across source, routing, writeback, diagnostics, subscription, and facade authority trees. |
| No-shim vocabulary scan | Canonical digest mechanics, foundational `payload()`/`bytes()` accessors, native digest-family prefixes, and digest-input counters only. |
| Runtime-bridge line-cap scan | Clean. |
| Scoped `git diff --check` | Passed with existing CRLF warnings only. |

Phase 1 is closed. If a later phase creates a fresh Phase 1-shaped blocker, fix
that blocker inside the phase that created it. Do not restart open-ended Phase 1
searching.

Next work continues at Phase 3/Phase 4 target and mask semantics, not Phase 1
cleanup, unless one of the completion gates above fails on a fresh edit.

#### Historical Phase 1 Current Finish Plan

This subsection is historical evidence from an earlier bounded closeout loop.
It is superseded by `Phase 1 Completion Runbook` above.

1. Run the five exact gates in this subsection once.
2. Write every blocker into the current blocker manifest.
3. Edit only from blocker files, not from repeated broad archaeology.
4. Split files only when the split houses native proof, quarantines terminal
   I/O, or fixes a touched line-cap violation.
5. Stop Phase 1 when all remaining hits are classified below and the proof
   commands pass.

The adversarial constraint for the loop is precise: current production and
current tests must not teach or execute any authority path based on JSON,
payload/byte recovery, surface/aspect labels, legacy/fallback/compat/shim
branches, arbitrary digest strings, or raw string collections.

##### Gate 1: Rebaseline Manifest

Run the residue and line-cap commands once at the start of the closeout batch:

```powershell
rg -n "legacy|fallback|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|serde_json::Value|json!|from_value|to_value|Vec<String>|Vec<&str>|from_digests|digest.*label|label.*digest|todo!|unimplemented!" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
Get-ChildItem -Path crates/worth-runtime-bridge/src,crates/worth-runtime-bridge/tests -Recurse -Filter *.rs | ForEach-Object { $c=(Get-Content $_.FullName).Length; if($c -gt 400){ '{0}:{1}' -f $c,$_.FullName } }
```

Blocker classification:

- `authority-blocker`: any hit in source, routing, snapshot, writeback,
  diagnostics, subscription, facade, replay, merge, policy, speculation, or
  certification production code that admits, stores, routes, or proves truth
  through a foreign-format, label, byte, raw collection, arbitrary digest, or
  fallback branch.
- `test-shim-blocker`: any current test or UI fixture that constructs fake
  native authority from literals, preserves old public names, uses `todo!` or
  `unimplemented!` placeholders, or teaches a deleted legacy shape.
- `terminal-io`: JSON/report/export/capture code that renders from typed native
  artifacts and cannot feed authority. This must live under terminal export or
  terminal capture topology.
- `canonical-mechanics`: digest hashing over named native canonical-basis
  carriers, foundational `payload()` accessors, foundational binary
  `bytes()` value preparation, and `digest_input_bytes` counters.
- `presentation-only`: domain text or report text derived from typed native
  artifacts, never accepted as authority input.

Ambiguous hits are blockers. Do not spend time arguing them safe.

##### Gate 2: Authority Blocker Rewrite

For every `authority-blocker`, perform a vertical hard break in one batch:

- Replace loose strings, labels, byte recovery, arbitrary digest rows, or JSON
  maps with foundational carriers: `AspectKey`, `AspectLocator`,
  `AspectFieldLocator`, `AspectValue`, `AspectContract`, foundational masks,
  authoritative state, or authoritative patch artifacts.
- Delete public and test constructors that could synthesize proof without the
  proving boundary.
- Preserve failure evidence as typed error context with native locators, value
  families, masks, and canonical bases.
- If the touched file exceeds the line cap, split by authority responsibility,
  not by helper convenience.

Allowed split skeletons:

- `input/<responsibility>/{mod.rs,target_admission.rs,patch_basis.rs,denials.rs}`
- `routing/<responsibility>/{mod.rs,eligibility.rs,target_identity.rs,diagnostics.rs}`
- `snapshot/<responsibility>/{mod.rs,contract.rs,validated_value.rs,denials.rs}`
- `writeback/<responsibility>/{mod.rs,effect_intent.rs,authority_request.rs,proof_basis.rs,denials.rs}`
- `diagnostics/<responsibility>/{mod.rs,retention.rs,masked_projection.rs,accessors.rs}`

Do not create `helpers`, `common`, `utils`, `legacy`, `compat`, or fallback
directories.

##### Gate 3: Terminal JSON Wall

Run the JSON/API and import-wall checks after Gate 2 edits:

```powershell
rg -n "serde_json::Value|json!|serde_json::Map|from_value|to_value" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
rg -n "terminal_report_export|json_projection" crates/worth-runtime-bridge/src/source crates/worth-runtime-bridge/src/routing crates/worth-runtime-bridge/src/writeback crates/worth-runtime-bridge/src/diagnostics crates/worth-runtime-bridge/src/subscription crates/worth-runtime-bridge/src/facade -g "*.rs"
```

Any non-terminal JSON authority hit is a blocker and must move behind native
typed artifact construction. JSON may render typed artifacts; it may not build
or decide them.

##### Gate 4: Current-Test Shim Eradication

Run the bounded current-test scan:

```powershell
rg -n "fallback|legacy|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|route:sha256|effect:sha256|strategy:sha256|truth-state:sha256|truth-view:sha256|todo!|unimplemented!" crates/worth-runtime-bridge/src/facade/tests crates/worth-runtime-bridge/src/harness/tests crates/worth-runtime-bridge/tests/ui -g "*.rs"
```

Tests must construct native proof through the same production proof boundaries
or through narrow sealed fixture support scoped to the test module. Compile-fail
fixtures should prove deleted old surfaces are uncallable without demonstrating
fake authority construction.

##### Gate 5: Closeout Proof

Run only after Gates 1-4 have no blocker rows:

```powershell
cargo fmt -p worth-runtime-bridge
cargo check -p worth-runtime-bridge --tests
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
```

Also run focused tests for any proof family changed during Gate 2 or Gate 4,
the runtime-bridge line-cap scan, the Gate 3 import-wall scan, the Gate 4
current-test scan, and scoped `git diff --check`.

Phase 1 is finished only when:

- no blocker rows remain in the manifest
- all remaining hits are classified as terminal I/O, canonical mechanics, or
  presentation-only
- runtime-bridge Rust code and tests are under the line cap except explicit
  workspace allowlist entries
- package check and compile-fail proof pass
- the spec records the final classification and the exact proof commands

If later Phase 2+ work creates a new Phase 1-shaped blocker, fix that blocker
inside the phase that created it. Do not reopen open-ended Phase 1 searching.

Current active gates:

| Gate | Status | Exact start point | Action | Done evidence |
| --- | --- | --- | --- | --- |
| 1. Rebaseline + source-artifact index | Complete | `subscription/certification/source_artifact_index{.rs,/}` | Split the over-cap source artifact index into typed evidence, record, whole-index basis, and per-kind basis modules. Delete anonymous local digest-list assembly. | `source_artifact_index` line-cap clean, focused subscription certification/source/compile-fail tests passed, touched residue scan clean except classified foundational/canonical mechanics. |
| 2. Authority collection residue | Complete | Scan only `src` and current runtime-bridge tests for raw string/digest collections. Start edits from blocker files, not from a second broad archaeology pass. | Replaced the broad retained causal mapping digest bucket with `retained_artifact_digest/{planning_checkpoint,route_history_preview,source_structural_stream,writeback}` so the dispatcher selects evidence families while artifact modules own typed digest-basis projection. | Gate 2 scan has one classified non-blocker: `subscription/certification/historical_basis.rs` reads `BridgeSubscriptionCertificationSemanticDigests::subscription_basis_digest()` from a typed certification bundle. No caller-supplied digest/string collection authority remains in the Gate 2 manifest. |
| 3. JSON terminal/import wall | Complete | `harness/adapter/**/terminal_report_export/**`, `harness/tests/**/terminal_report_export/**`, plus any non-terminal JSON hit from Gate 2. | Ensure JSON renders only from typed native records. If JSON constructs authority-shaped records, move construction into the native subsystem and leave JSON as terminal projection. | JSON API hits are confined to terminal export/capture topology, except `subscription/counters::from_values` false positives. Authority import-wall scan across source, routing, writeback, diagnostics, subscription, and facade returned no hits. |
| 4. Current-test no-shim eradication | Complete | `facade/tests/**`, `harness/tests/**`, `tests/ui/**` | Delete executable examples of old paths in tests: fake `*:sha256:*` authority values, raw/payload/aspect-label helpers, compatibility/fallback/shim names, and `todo!`/`unimplemented!` placeholders. | Bounded current-test scan found only canonical `as_bytes()` digest mechanics and `digest_input_bytes` counter proof references. Compile-fail fixtures pass and prove sealed native construction without fabricated legacy authority. |
| 5. Closeout proof | Complete | Entire `worth-runtime-bridge` crate. | Run final format/check/focused tests, line-cap scan, import-wall scan, no-shim scan, and write the final classification table. | Final proof has no blocker rows: package check and compile-fail tests pass, runtime-bridge source line-cap scan is clean, import wall is clean, and remaining broad no-shim hits are classified as foundational/canonical/terminal/presentation mechanics. |

Gate 2 exact residue command:

```powershell
rg -n "Vec<String>|Vec<&str>|BTreeSet<String>|HashSet<String>|VecDeque<String>|&\[String\]|request_digests|equivalence_members|record_digests|from_digests|synthetic_|retained_.*digests" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
```

Gate 2 closeout classification:

- `diagnostics/causal_envelope/retained_mapping` now separates retained
  artifact family dispatch from per-family digest projection. The stale
  `retained_digests` bucket name is gone, and all retained mapping files are
  under the workspace line cap.
- `subscription/certification/historical_basis.rs` remains as
  `canonical-digest-mechanics`: it compares two
  `BridgeSubscriptionCertificationSemanticDigests` bundle accessors to prove
  retained versus latest-unretained basis drift. It does not admit or store a
  caller-supplied digest collection.

Gate 3 exact JSON/API command:

```powershell
rg -n "serde_json::Value|json!|serde_json::Map|from_value|to_value" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
```

Gate 3 import-wall command:

```powershell
rg -n "terminal_report_export|json_projection" crates/worth-runtime-bridge/src/source crates/worth-runtime-bridge/src/routing crates/worth-runtime-bridge/src/writeback crates/worth-runtime-bridge/src/diagnostics crates/worth-runtime-bridge/src/subscription crates/worth-runtime-bridge/src/facade -g "*.rs"
```

Gate 4 exact no-test-shim command:

```powershell
rg -n "fallback|legacy|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|route:sha256|effect:sha256|strategy:sha256|truth-state:sha256|truth-view:sha256|todo!|unimplemented!" crates/worth-runtime-bridge/src/facade/tests crates/worth-runtime-bridge/src/harness/tests crates/worth-runtime-bridge/tests/ui -g "*.rs"
```

Gate 5 verification commands, each cargo command with a 10 minute timeout:

```powershell
cargo fmt -p worth-runtime-bridge
cargo check -p worth-runtime-bridge --tests
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
```

Gate 5 also runs the focused test families touched by Gates 2-4, the
runtime-bridge Rust line-cap scan, and scoped `git diff --check`.

Active Phase 1 closeout classification:

- JSON APIs are terminal-only: `serde_json::Value`, `json!`, and
  `StructuredValue::Json` are under terminal report export or terminal snapshot
  capture topology. The only non-terminal-looking Gate 3 hits are
  `subscription/counters::from_values` constructor names, not serde conversion.
- The terminal import wall is clean: source, routing, writeback, diagnostics,
  subscription, and facade authority trees do not import `terminal_report_export`
  or `json_projection`.
- Current tests have no executable legacy/shim path. Remaining Gate 4 matches
  are canonical digest `as_bytes()` mechanics in mirror helpers and the
  `digest_input_bytes` cost/counter proof.
- Broad Phase 1 scan hits are classified as foundational `payload()` accessors,
  foundational binary value `bytes()` preparation, named canonical digest basis
  hashing, native digest-family prefixes derived from typed carriers, terminal
  export/capture JSON, and typed pricing ranked-material presentation evidence.
- Runtime-bridge source line-cap scan is clean.
- Verification evidence: `cargo fmt -p worth-runtime-bridge`,
  `cargo check -p worth-runtime-bridge --tests`, `cargo test -p
  worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`,
  Gate 3 JSON/API scan, Gate 3 import-wall scan, Gate 4 current-test no-shim
  scan, broad no-shim classification scan, runtime-bridge line-cap scan, and
  scoped `git diff --check` with only the existing CRLF warning on retained
  causal mapping.

Phase 1 is closed in the active control plan. Future Phase 1 references are
historical evidence only; new implementation work resumes from Phase 2 unless
a later edit creates a fresh blocker.

Historical Phase 1 strict restart board:

Historical efficiency restart overlay:

This overlay records the completed stricter "no shims anywhere, including
tests" reset. It is retained as audit evidence only. The active operating plan
is `Phase 1 Completion Runbook` above.

Efficiency rules for the remaining Phase 1 work:

- Run one bounded rebaseline at the start of a batch, write down the blocker
  files, then edit from those exact files. Do not keep re-running broad scans
  between small edits unless a change creates a new authority seam.
- If a scan hit is ambiguous, treat it as a blocker. Do not spend time arguing
  a maybe-safe string list into safety.
- Split only when the split gives a proof carrier a real home, quarantines
  terminal I/O, or fixes a touched line-cap violation.
- Tests are part of the same architecture. A test helper that teaches raw
  labels, fake digests, JSON authority construction, or legacy shape is a
  blocker even if the production path is clean.

Batch 1: rebaseline and source-artifact-index closeout.

- Start files:
  - `crates/worth-runtime-bridge/src/subscription/certification/source_artifact_index.rs`
  - `crates/worth-runtime-bridge/src/subscription/certification/source_artifact_index/**`
    if the split already exists by the time this batch runs.
- Why this comes first:
  - The current file is over the 400-line cap, so any further touch must split
    it.
  - The file owns subscription certification source-artifact digest/index
    evidence; if any local digest-list assembly still looks like authority, it
    must become a named typed basis carrier.
- Required skeleton if touched:

```text
crates/worth-runtime-bridge/src/subscription/certification/source_artifact_index/
  mod.rs
  artifact_evidence.rs
  artifact_record.rs
  index_basis.rs
  kind_index_basis.rs
```

- Aspect-native requirements:
  - `BridgeSubscriptionSourceArtifactEvidence` owns scenario/role/lane/family
    evidence.
  - `BridgeSubscriptionSourceArtifactRecord` owns the admitted source-artifact
    record basis.
  - `BridgeSubscriptionSourceArtifactIndexBasis` owns whole-index canonical
    basis.
  - `BridgeSubscriptionSourceArtifactKindIndexBasis` owns per-kind digest
    projection; no local anonymous `Vec`/`join` should read as a caller-fed
    digest-list authority path.
- Done means:
  - No file in the split exceeds 400 lines.
  - No public constructor admits arbitrary artifact digest basis rows.
  - Subscription certification tests still prove record de-duplication,
    counters, and digest drift through typed evidence.

Batch 1 progress:

- Split the over-cap flat
  `subscription/certification/source_artifact_index.rs` into
  `source_artifact_index/{mod.rs,artifact_evidence.rs,artifact_record.rs,index_basis.rs,kind_index_basis.rs}`.
- `artifact_evidence.rs` owns the typed artifact kind/scenario/role taxonomy
  and derives source artifact identity/digest only from those typed evidence
  fields plus optional reference-workload lane/family evidence.
- `artifact_record.rs` owns admitted source-artifact input and materialized
  record derivation; public construction still goes through
  `BridgeSubscriptionSourceArtifactInput::from_evidence`.
- `index_basis.rs` owns whole-index canonical basis from materialized records.
- `kind_index_basis.rs` owns per-kind semantic-source digest projection, so
  `BridgeSubscriptionSourceArtifactIndex::artifact_kind_digest` no longer
  assembles an anonymous local digest list.
- Added `BridgeGroupedBindingValueFamily` to the public facade export cluster
  beside `BridgeGroupedTruthViewError`, and adjusted grouped source tests to
  match through the facade export. This fixed the warning-clean check without
  leaving a private-module-only typed denial surface.
- Verified with `cargo fmt -p worth-runtime-bridge`,
  warning-clean `cargo check -p worth-runtime-bridge --tests`,
  `cargo test -p worth-runtime-bridge subscription_certification --
  --nocapture`, `cargo test -p worth-runtime-bridge
  subscription::certification -- --nocapture`, `cargo test -p
  worth-runtime-bridge source -- --nocapture`, and `cargo test -p
  worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`.
  Line-cap scan is clean; the source-artifact-index authority-collection
  residue scan is clean; touched no-shim scan reports only canonical
  `as_bytes()` digest mechanics and the already-classified foundational
  `payload()` accessor in grouped source truth.

Batch 2: authority-collection residue gate.

- Start from the Batch 1 scan manifest only. Expected terms:
  - `Vec<String>`, `Vec<&str>`, `BTreeSet<String>`, `HashSet<String>`,
    `VecDeque<String>`, `&[String]`, `record_digests`,
    `request_digests`, `equivalence_members`, `retained_*digests`,
    `from_digests`, `synthetic_`.
- Required action:
  - Rewrite blockers into sealed carriers named for the proof they freeze.
  - Classify remaining hits only as `presentation-domain`,
    `canonical-digest-mechanics`, `terminal-export`, or
    `foundational-accessor`.
- Done means:
  - No authority path stores caller-supplied digest/string collections.
  - Remaining collections are visibly terminal display/domain text or typed
    proof carriers.

Batch 3: JSON terminal/import wall.

- Start files:
  - `crates/worth-runtime-bridge/src/harness/adapter/**/terminal_report_export/**`
  - `crates/worth-runtime-bridge/src/harness/tests/**/terminal_report_export/**`
  - any non-terminal JSON hit from the Batch 2 scan manifest.
- Required action:
  - JSON modules may render only from typed native records.
  - If a JSON module constructs authority-shaped records, move construction
    into the native subsystem first and leave JSON as terminal projection.
  - Authority trees must not import `terminal_report_export` or
    `json_projection`.
- Done means:
  - `serde_json::Value`, `json!`, maps, and JSON conversion APIs exist only in
    terminal export/capture or explicit foreign-ingress topology.

Batch 4: no-test-shim current surface.

- Start files:
  - `crates/worth-runtime-bridge/src/facade/tests/**`
  - `crates/worth-runtime-bridge/src/harness/tests/**`
  - `crates/worth-runtime-bridge/tests/ui/**`
- Required action:
  - Delete executable examples of old paths in tests, including fake
    `*:sha256:*` authority inputs, raw/payload/aspect-label helpers,
    compatibility/fallback/shim naming, and `todo!`/`unimplemented!`
    placeholders.
  - Compile-fail fixtures must prove sealed native construction using sealed
    placeholders, not by fabricating old authority values.
- Done means:
  - Tests teach only the final aspect-native API.
  - Any remaining old-looking term is written into the closeout
    classification and is non-executable/historic/terminal.

Batch 5: closeout proof.

- Required verification, with a 10 minute timeout per cargo command:

```powershell
cargo fmt -p worth-runtime-bridge
cargo check -p worth-runtime-bridge --tests
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
```

- Also run focused tests for every proof family touched by Batches 1-4.
- Required scans:

```powershell
rg -n "Vec<String>|Vec<&str>|BTreeSet<String>|HashSet<String>|VecDeque<String>|&\[String\]|request_digests|equivalence_members|record_digests|from_digests|synthetic_" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
rg -n "serde_json::Value|json!|serde_json::Map|from_value|to_value" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
rg -n "fallback|legacy|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|route:sha256|effect:sha256|strategy:sha256|truth-state:sha256|truth-view:sha256|todo!|unimplemented!" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
```

- Phase 1 is complete only when every remaining scan hit has one written
  classification and there are no `blocker` rows.

| Pass | Status | Purpose | Next action | Verification gate |
| --- | --- | --- | --- | --- |
| 0 | Complete | Re-establish strict restart discipline. | Phase 1 is only about annihilating legacy authority evidence. Structure work is allowed only when it deletes or quarantines a concrete legacy/native-boundary seam or keeps a touched file under the line cap. | Read-gate docs and line-cap scan. |
| 1 | Complete | Rebaseline the current crate with exact residue manifests. | The current manifest is below. Do not repeat broad discovery before Gate 3 unless a later edit creates new residue. | Gate 1 scan set completed; no authority blocker found outside the Gate 3 snapshot-capture decision. |
| 2 | Complete | Delete remaining authority-bearing collection or digest-list leftovers. | Gate 2 blockers are closed. Remaining collection hits are typed pricing ranked-material evidence or terminal presentation. | Focused proof-family tests plus authority-collection residue scan completed. |
| 3 | Complete | Prove JSON is terminal-only and cannot be imported by authority modules. | The ambiguous root terminal snapshot helpers were split out of `json_projection.rs` into `terminal_snapshot_capture.rs`, leaving `json_projection.rs` as `serde_json::Value` report projection only. | JSON API scan plus import-wall scan completed. |
| 4 | Complete | Re-run no-test-shim facade/current-test eradication. | Bounded current-test scan found no executable JSON/raw-byte/string-label/arbitrary-digest authority examples. Remaining hits are canonical digest byte mechanics and the digest-input counter proof. | Compile-fail boundary suite and no-shim residue scan completed. |
| 5 | Complete | Phase 1 closeout proof. | Phase 1 is closed. Continue with Phase 2 native snapshot read contract work. | `cargo fmt`, `cargo check --tests`, focused tests, compile-fail, line-cap scan, import-wall scan, no-shim classification scan, and scoped `git diff --check` completed. |

Current restart manifest:

| Gate | Files | Classification | Required next action |
| --- | --- | --- | --- |
| 1/2 | `harness/tests/pricing_support/**`, `harness/tests/pricing_showcase_support/**`, pricing shock tests | `presentation-domain` / typed evidence | No edit unless a later change turns `PricingShockRankedMaterialDamageSet` back into a raw list authority carrier. Current `ranked_materials_by_damage` hits are typed simulation evidence and terminal assertions. |
| 3 | `harness/adapter/terminal_report_export/json_projection.rs` | `terminal-export` | `json_projection.rs` now renders only `serde_json::Value` from typed runtime-bridge records. Harness snapshot capture value construction lives separately in `terminal_snapshot_capture.rs` and is imported only by the harness adapter capture path. |
| 3 | `harness/adapter/adapter_impl/**/terminal_report_export/*json_projection.rs`, `harness/tests/**/terminal_report_export/*.rs` | `terminal-export` | Spot-check by module path and import wall: these modules remain under harness terminal export/test terminal export topology and are not imported by authority trees. |
| 3 | source, routing, writeback, diagnostics, subscription, facade authority trees | `import-wall-clean` | The import-wall scan has no hits for `terminal_report_export|json_projection`; keep this as a closeout proof. Omit nonexistent `src/replay` from the command unless that directory is later introduced. |
| 4 | `facade/tests`, `harness/tests`, `tests/ui` | `complete` | Bounded no-test-shim scan found only `Sha256::digest(...as_bytes())` canonical digest mechanics and `digest_input_bytes` counter proof references. No current-test helper preserves executable legacy/fallback/raw/payload/bytes/surface-label/domain-payload authority. |
| 5 | `src`, `tests` | `phase-1-closed` | Current crate-local Rust line-cap scan is empty. Final no-shim classification found only foundational accessors, canonical digest mechanics, typed pricing presentation evidence, and terminal export/capture JSON. |

#### Strict Restart Gate 1: Rebaseline Manifest

This gate exists to prevent another search-heavy loop. It produces a bounded
manifest first, then every later pass starts from exact files.

Run these scans and classify each hit in a small table before editing:

```powershell
rg -n "Vec<String>|Vec<&str>|BTreeSet<String>|HashSet<String>|VecDeque<String>|&\[String\]|request_digests|equivalence_members|record_digests|read_packet_aspect_value_texts|ranked_materials_by_damage" `
  crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests `
  -g "*.rs"

rg -n "serde_json::Value|json!|serde_json::Map|from_value|to_value" `
  crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests `
  -g "*.rs"

rg -n "fallback|legacy|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|from_digests|route:sha256|effect:sha256|strategy:sha256|truth-state:sha256|truth-view:sha256|todo!|unimplemented!" `
  crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests `
  -g "*.rs"
```

Gate 1 output must be a manifest with one row per file, not one row per match.
The classification values are:

- `blocker`: executable authority, public/test construction, or diagnostics
  proof still shaped by JSON, raw bytes, labels, arbitrary digest text, old
  vocabulary, or an ambiguous generic collection.
- `terminal-export`: JSON or display text rendered from typed native evidence
  under an explicit `terminal_report_export` or `terminal_report_export` child
  path, with no authority module importing it.
- `presentation-domain`: domain-facing text or ordered presentation data whose
  producer and consumer names make non-authority status obvious.
- `canonical-digest-mechanics`: digest computation over a named native
  canonical-basis carrier; callers cannot inject arbitrary digest basis rows.
- `foundational-accessor`: `payload()` or `bytes()` calls on
  `worth-foundational` proof/value carriers, not bridge-local payload or byte
  authority.
- `historic-doc`: migration/spec wording only, never current production or
  current tests.

#### Strict Restart Gate 2: Authority Collection Leftovers

Start from these files if Gate 1 still reports collection/digest-list hits:

```text
crates/worth-runtime-bridge/src/subscription/certification/source_artifact_index.rs
crates/worth-runtime-bridge/src/source/record_digests.rs
crates/worth-runtime-bridge/src/source/records.rs
crates/worth-runtime-bridge/src/facade/tests/causal_envelope/retained_mapping_support.rs
```

Required decision:

- If `record_digests` or `entries: &[String]` is only a local rendering of a
  named native basis carrier, rename/split only if the current name teaches an
  arbitrary digest-list authority path.
- If callers can supply digest entries, replace the list with a proof carrier
  such as `SubscriptionSourceArtifactRecordEvidenceSet` or
  `ExpectedRetainedCausalDigestBasis`, and seal construction at the boundary
  that proves the evidence.
- If a source digest helper is derived from
  `ContractValidatedAspectArtifact`, source materialization records, or
  `SnapshotReadContract`, keep it only behind a name that says which native
  basis it freezes.

Allowed skeleton if splitting is required:

```text
crates/worth-runtime-bridge/src/
  subscription/certification/source_artifact_index/
    mod.rs
    record_evidence.rs
    digest_basis.rs
  source/
    record_digest_basis.rs
```

Do not split these files merely to reduce grep noise. Split only if the
authority proof carrier needs a named home or a touched file crosses the line
cap.

Strict restart Gate 2 progress:

- Closed the remaining authority-collection blockers found by the Gate 1
  manifest. `BridgeSubscriptionSourceArtifactIndex` now derives its top-level
  canonical basis through private
  `BridgeSubscriptionSourceArtifactIndexBasis` and
  `BridgeSubscriptionSourceArtifactIndexRecordEvidence` carriers instead of
  assembling a local `record_digests` string.
- Narrowed retained causal-envelope bulk-planning failure digest basis so
  production and test mirrors accept typed `BridgeBulkPlanningFailure` records,
  not arbitrary retained-record digest strings.
- Renamed the source packet-set digest helper module from
  `record_digests` to `packet_set_digest_basis`, and renamed its entry points
  away from `synthetic_*` so the touched source materialization path describes
  native observation-derived packet-set basis instead of fake digest authority.
- Residue scan for `Vec<String>`, `Vec<&str>`, `BTreeSet<String>`,
  `HashSet<String>`, `VecDeque<String>`, `&[String]`, `request_digests`,
  `equivalence_members`, `record_digests`,
  `from_retained_record_digests`, `synthetic_`,
  `read_packet_aspect_value_texts`, and `ranked_materials_by_damage` now has
  no authority-collection blockers. Remaining hits are pricing
  `PricingShockRankedMaterialDamageSet` domain evidence and terminal
  presentation/export assertions.
- Import-wall scan for `terminal_report_export|json_projection` across source,
  routing, writeback, diagnostics, subscription, and facade authority trees
  returned no hits.
- Verified with `cargo fmt -p worth-runtime-bridge`,
  `cargo test -p worth-runtime-bridge causal_envelope -- --nocapture`,
  `cargo test -p worth-runtime-bridge subscription_certification --
  --nocapture`, `cargo check -p worth-runtime-bridge --tests`, the
  crate-local line-cap scan, and scoped `git diff --check` with only existing
  CRLF warnings.

#### Strict Restart Gate 3: JSON Terminal Wall

Start from JSON files found by Gate 1, especially:

```text
crates/worth-runtime-bridge/src/harness/adapter/terminal_report_export/json_projection.rs
crates/worth-runtime-bridge/src/harness/adapter/adapter_impl/**/terminal_report_export/*json_projection.rs
crates/worth-runtime-bridge/src/harness/tests/pricing_support/terminal_report_export/*.rs
crates/worth-runtime-bridge/src/harness/tests/pricing_showcase_support/terminal_report_export/*.rs
```

Required proof:

- Terminal export modules may render `serde_json::Value` only from already
  typed native artifacts.
- Terminal export modules must not construct snapshot/source/routing/writeback
  authority records as a side effect of rendering a report.
- Source, routing, writeback, diagnostics, subscription certification authority,
  and facade modules must not import terminal export modules.

If a JSON module still constructs authority-shaped records, move the authority
record construction into the native subsystem first and leave the JSON module
as a projection only.

Import-wall scan:

```powershell
rg -n "terminal_report_export|json_projection" `
  crates/worth-runtime-bridge/src/source `
  crates/worth-runtime-bridge/src/routing `
  crates/worth-runtime-bridge/src/writeback `
  crates/worth-runtime-bridge/src/replay `
  crates/worth-runtime-bridge/src/diagnostics `
  crates/worth-runtime-bridge/src/subscription `
  crates/worth-runtime-bridge/src/facade `
  -g "*.rs"
```

Strict restart Gate 3 progress:

- Split terminal harness snapshot capture construction out of
  `harness/adapter/terminal_report_export/json_projection.rs` into
  `harness/adapter/terminal_report_export/terminal_snapshot_capture.rs`.
  The JSON projection module now returns only `serde_json::Value` report
  projections from typed bridge records.
- Renamed the capture helpers to terminal snapshot capture value vocabulary and
  aliased the external `worth_harness::SnapshotPayload` type at the boundary so
  runtime-bridge names do not teach payload authority.
- Re-ran the import-wall scan across source, routing, writeback, diagnostics,
  subscription, and facade authority trees; it returned no hits.

#### Strict Restart Gate 4: No-Test-Shim Current Surface

Start from:

```text
crates/worth-runtime-bridge/src/facade/tests/
crates/worth-runtime-bridge/src/harness/tests/
crates/worth-runtime-bridge/tests/ui/
```

Current tests must not preserve executable examples of the old architecture.
The following are blockers even in tests:

- arbitrary `*:sha256:*` digest literals used as authority input
- JSON values used to manufacture source, route, writeback, diagnostic, or
  certification truth before terminal export
- helper names containing `legacy`, `compatibility`, `fallback`, `shim`,
  `raw`, `payload`, `bytes`, `surface_label`, `aspect_label`, or
  `domain_payload` for native authority
- compile-fail fixtures that show old constructors by fabricating fake native
  values instead of using sealed placeholders
- `todo!` or `unimplemented!` in current runtime-bridge test surfaces

Strict restart Gate 4 progress:

- Bounded no-test-shim scan over `facade/tests`, `harness/tests`, and
  `tests/ui` found no executable old-authority examples. Remaining matches are
  canonical digest mechanics in test mirror basis helpers and the
  `digest_input_bytes` counter proof.
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture` passed, proving current sealed/private boundary fixtures still
  compile-fail as expected without legacy construction examples.

Allowed skeleton if splitting is required:

```text
crates/worth-runtime-bridge/src/
  facade/tests/support/<native proof family>.rs
  harness/tests/<native proof family>/
crates/worth-runtime-bridge/tests/ui/<sealed native proof fixture>.rs
```

#### Strict Restart Gate 5: Closeout Proof

Run only after Gates 1-4 are closed:

```powershell
cargo fmt -p worth-runtime-bridge
cargo check -p worth-runtime-bridge --tests
cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture
```

Also run focused tests for every proof family touched by Gates 2-4, each with a
10 minute timeout. Finish with:

```powershell
$over = Get-ChildItem crates/worth-runtime-bridge/src,crates/worth-runtime-bridge/tests -Recurse -Filter *.rs | ForEach-Object { $count = (Get-Content $_.FullName | Measure-Object -Line).Lines; if ($count -gt 400) { "$count $($_.FullName)" } }
rg -n "fallback|legacy|compatibility|compat|shim|raw|payload|bytes|aspect_bytes|surface_label|aspect_label|domain_payload|from_digests|route:sha256|effect:sha256|strategy:sha256|truth-state:sha256|truth-view:sha256|todo!|unimplemented!" crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests -g "*.rs"
git diff --check -- plans/worth-runtime-bridge/aspect_native_refactor.md crates/worth-runtime-bridge/src crates/worth-runtime-bridge/tests
```

Phase 1 is complete only if every remaining scan hit is written into this
section with its classification and no `blocker` rows remain.

Pass A: authority-bearing string collections.

- Stop condition: every authority-bearing string collection below is deleted or
  replaced by a typed proof carrier. Remaining string collections must be
  classified as presentation/domain data in Pass B.
- Allowed split rule: split only when the authority seam needs an earned
  sub-boundary or the touched file would exceed the workspace line cap.
- Residue terms: `Vec<String>`, `Vec<&str>`, `BTreeSet<String>`,
  `HashSet<String>`, `VecDeque<String>`, `&[String]`, `commit_identities`,
  `lifecycle_digests`, `retained_candidates`, `request_digests`,
  `equivalence_members`, and `record_digests`.
- Allowed skeleton if splitting is required:

```text
crates/worth-runtime-bridge/src/
  structural/
    fingerprints/
      mod.rs
      record_value_evidence.rs
      equivalence_member_evidence.rs
      canonical_basis.rs
  diagnostics/causal_envelope/retained_mapping/
    retained_digest_basis.rs
```

| Order | Gate | Status | Exact files | Required edit | Proof target |
| --- | --- | --- | --- | --- | --- |
| A1 | Stream command windows | Complete | `harness/adapter/adapter_impl/stream/*` | Command text terminates in `command_ingress`; execution consumes `NativeStreamCommitWindow` and `TruthCommitIdentity`. | Stream tests plus stream string-list residue scan. |
| A2 | Speculation churn lifecycle digests | Complete | `harness/adapter/adapter_impl/speculation/{churn.rs,churn_certification.rs,shared.rs,terminal_report_export/*}` | Lifecycle evidence remains `SpeculationPreviewLifecycleDigestSet`; terminal export only may render strings. | Speculation tests plus lifecycle-digest residue scan. |
| A3 | Structural retained candidates | Complete | `harness/adapter/adapter_impl/structural/certification_bundle.rs`, `harness/adapter/adapter_impl/structural/terminal_report_export/json_projection.rs`, `harness/adapter/adapter_impl/structural/typed_certification_tests.rs` | Retained candidates remain a typed candidate set before JSON export, with typed certification assertions proving non-empty unique retained evidence and counter alignment. | Structural tests plus retained-candidate residue scan. |
| A4 | Diagnostics preview session reservation index | Complete | `diagnostics/state/mod.rs`, `diagnostics/state/speculation.rs`, `diagnostics/facade/speculation.rs`, `facade/runtime/speculation.rs` | `reserved_preview_session_identities` stores `BridgePreviewSessionIdentity`, and reservation APIs require typed identities. | Diagnostics/speculation focused tests plus reservation-index scan. |
| A5 | In-memory writeback request digest log | Complete | `harness/fixtures/in_memory/writeback_surface.rs` and callers | Deleted the dead `request_digests` log and replaced formatted-string commit indexing with `RecordingTruthWritebackCommitKey`. | `cargo test -p worth-runtime-bridge writeback -- --nocapture`; request-digest and formatted authority-key scan. |
| A6 | Structural fingerprint equivalence members | Complete | `structural/fingerprints.rs`, direct structural tests | `StructuralFingerprint` now retains typed `StructuralFingerprintRecordValueEvidenceSet` and `StructuralFingerprintEquivalenceMemberSet` derived from validated snapshot values and read packets; the public facade exports the evidence carrier types and tests prove the top-level canonical basis retains both named evidence sets. | `cargo test -p worth-runtime-bridge structural -- --nocapture`; fingerprint string-list scan; line-cap scan. |
| A7 | Causal-envelope retained digest local vectors | Complete | `diagnostics/causal_envelope/retained_mapping/retained_digests.rs`, facade causal-envelope retained-mapping tests | Replaced local counter/failure digest vectors with `RetainedCausalMappingDigestBasis` and mirrored `ExpectedRetainedCausalDigestBasis`; public construction now names counter-value and retained-record-digest authority basis rows instead of generic string entries. | `cargo test -p worth-runtime-bridge causal_envelope -- --nocapture`; retained-digest local-vector scan. |

Pass B: domain/test presentation lists.

- Start condition: Pass A scans are clean for authority-bearing string
  collections.
- Stop condition: every remaining `Vec<String>` or `Vec<&str>` hit is a typed
  native carrier, terminal presentation/domain data, or a blocker fixed
  immediately.
- Expected split: none unless classification reveals an authority seam or a
  touched file exceeds the workspace line cap.
- Residue terms: `Vec<String>`, `Vec<&str>`, `BTreeSet<String>`,
  `HashSet<String>`, `VecDeque<String>`, `&[String]`.

Pass B execution order:

1. Close B2 before opening any new search surface.
2. Execute B3 from the two named files only.
3. Run the Pass B residue scan across the named B files and pricing support
   tree.
4. Mark Pass B complete only when every hit is either typed or explicitly
   presentation/domain-local by path and name.

| Order | Gate | Status | Exact files | Required edit or classification | Proof target |
| --- | --- | --- | --- | --- | --- |
| B1 | Subscription product/component/lane ID lists | Complete | `subscription/certification/manifest.rs`, `subscription/certification/manifest/workload_ids.rs`, `subscription/certification/ordering_hostility.rs`, facade subscription certification tests, `harness/tests/subscription_certification/suites_35_37.rs` | Wrapped declared product/component/lane IDs in typed workload ID and ID-set carriers before manifest sealing; runtime facade and tests now pass typed sets, and manifest sealing rejects empty declared IDs before canonical authority is produced. | `cargo test -p worth-runtime-bridge subscription_certification -- --nocapture`; `cargo test -p worth-runtime-bridge subscription::certification -- --nocapture`; B1 raw-list residue scan. |
| B2 | Pricing ranked material/provenance lists | Complete | `harness/tests/pricing_support.rs`, `harness/tests/pricing_support/simulation_evidence.rs`, `harness/tests/pricing_shock/support/provenance_records.rs`, pricing terminal exports and pricing-showcase terminal exports | Replaced ranked-material damage lists with `PricingShockRankedMaterialDamageSet`, replaced positional provenance text vectors with `PricingProvenanceAspectTextPacket`, renamed the packet reader to `read_pricing_provenance_aspect_text_packet`, and split simulation evidence out of `pricing_support.rs` after the typed carriers pushed the file over the line cap. Terminal JSON/markdown renders strings only from typed pricing evidence carriers. | `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge pricing_shock -- --nocapture`; B2 residue scan for raw ranked-material/provenance vectors and old helper names; touched-file line-cap scan. |
| B3 | Mapping/support test display lists | Complete | `mapping/freezing/tests.rs`, `harness/tests/support.rs`, merge harness callsites | Renamed the mapping assertion display helper to `sorted_context_mapping_id_assertion_pair` and returned a fixed `[&str; 2]` projection from typed frozen mapping context; changed `merge_declaration` to accept typed `TruthCommitIdentity` parents directly so harness ancestry authority is no longer introduced as `Vec<&str>`. | `cargo check -p worth-runtime-bridge --tests`; `cargo test -p worth-runtime-bridge mapping::freezing -- --nocapture`; `cargo test -p worth-runtime-bridge merge -- --nocapture`; Pass B residue scan; touched-file line-cap scan. |

Pass B residue command:

```powershell
rg -n "Vec<String>|Vec<&str>|BTreeSet<String>|HashSet<String>|VecDeque<String>|&\[String\]|read_packet_aspect_value_texts|ranked_materials_by_damage\[|ranked_materials_by_damage\.first\(" `
  crates/worth-runtime-bridge/src/harness/tests/pricing_support.rs `
  crates/worth-runtime-bridge/src/harness/tests/pricing_support `
  crates/worth-runtime-bridge/src/harness/tests/pricing_shock `
  crates/worth-runtime-bridge/src/harness/tests/pricing_showcase_support `
  crates/worth-runtime-bridge/src/mapping/freezing/tests.rs `
  crates/worth-runtime-bridge/src/harness/tests/support.rs `
  -g "*.rs"
```

Pass C: JSON quarantine and import wall.

- Start condition: Pass A and Pass B are closed.
- Stop condition: JSON appears only under explicit terminal export or foreign
  ingress paths, and those modules borrow typed artifacts instead of
  constructing authority.
- Required scan terms: `serde_json::Value`, `json!`, `serde_json::Map`,
  `from_value`, `to_value`, `terminal_report_export`, `json_projection`, and
  imports from authority modules into terminal export modules.
- Expected hotspots: harness adapter report emitters, writeback certification
  terminal projections, policy/source/structural certification reports,
  route-support exports, and diagnostics modules that still store JSON.
- Execution order:
  1. Scan JSON hits and classify by path: `terminal_report_export`,
     `foreign_ingress`, historic docs, or blocker.
  2. For any blocker outside terminal/foreign-ingress paths, move JSON rendering
     into a terminal export module and make the source module retain typed
     artifacts only.
  3. For any terminal export module that builds authority instead of borrowing
     typed artifacts, introduce the missing typed evidence carrier in the source
     module first, then render from it.
  4. Do not create compatibility shims or aliases; deleted JSON authority paths
     stay deleted.
- Allowed skeleton if splitting is required:

```text
crates/worth-runtime-bridge/src/
  external_io/
    foreign_ingress/
    terminal_report_export/
  diagnostics/
    terminal_export/
  harness/adapter/adapter_impl/
    terminal_report_export/
```

Pass D: no-shim facade and current-test residue gate.

- Start condition: Pass C JSON/import-wall scans are clean.
- Stop condition: remaining hits are classified as historic documentation,
  explicit external I/O, terminal presentation, canonical digest mechanics, or
  domain text. Any production or current-test authority hit is a blocker and is
  rewritten before closeout.
- Required scan terms: `legacy`, `compat`, `compatibility`, `fallback`, `shim`,
  `raw`, `payload`, `bytes`, `aspect_bytes`, `surface_label`,
  `domain_payload`, `from_digests`, `route:sha256`, `effect:sha256`,
  `strategy:sha256`, `truth-state:sha256`, `truth-view:sha256`, `todo!`, and
  `unimplemented!`.
- Public/test rule: no test helper may preserve an executable example of old
  JSON/raw-byte/string-label/arbitrary-digest authority. Privacy/compile-fail
  tests must use sealed native placeholders instead.
- Execution order:
  1. Run the no-shim residue scan over `crates/worth-runtime-bridge/src` and
     `crates/worth-runtime-bridge/tests`.
  2. Classify canonical digest prefix construction separately from arbitrary
     digest input; digest construction is allowed only when derived from named
     native basis carriers.
  3. Treat current-test fake authority examples as blockers, even when they are
     compile-fail fixtures.
  4. Run compile-fail tests if any public constructor, private field, facade
     method, or UI fixture changes.
  5. Close Phase 1 only when current code and current tests no longer reveal
     historical JSON/raw-byte/string-label/digest-only authority paths.
- Allowed skeleton if splitting is required:

```text
crates/worth-runtime-bridge/src/
  facade/
    runtime/
    tests/
      support/
  harness/tests/
    <native proof family>/
  tests/ui/
    <sealed native proof fixtures>
```

Pass D closeout classification:

- `canonical_basis.rs` and `snapshot/read_result.rs` retain direct
  foundational `Artifact::payload()` accessor calls. These are not
  runtime-bridge payload authority; they are proof-artifact accessors from
  `worth-foundational` used to reach canonical basis sequence and validated
  aspect value views.
- `subscription/signal_strategy.rs`, `writeback/effect/derived_effect.rs`, and
  `source/grouped_truth_view/digest_basis.rs` retain native digest-family
  prefixes derived from typed canonical basis carriers. They are not arbitrary
  digest-input constructors and do not admit caller-supplied digest authority.
- The line-cap closeout split moved subscription family registry tests into
  `subscription/family_registry_tests.rs` and moved writeback authority
  rejection retained-diagnostics assertions into the existing writeback test
  support module.

Closeout proof for Phase 1:

- `cargo fmt -p worth-runtime-bridge`
- `cargo check -p worth-runtime-bridge --tests`
- focused tests for every touched proof family, with a 10 minute timeout per
  command
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture` when public/type/privacy boundaries change
- crate-wide line-cap scan for `crates/worth-runtime-bridge/src` and
  `crates/worth-runtime-bridge/tests`
- crate-wide no-shim residue scan, with each remaining hit classified as
  historic documentation, explicit external I/O, terminal presentation, or a
  blocker
- `git diff --check`, accepting only pre-existing CRLF warnings

Phase 1 closeout classification:

- `canonical_basis.rs` and `snapshot/read_result.rs` retain foundational
  `payload()` accessors on proof/value artifacts. These are not
  runtime-bridge payload authority.
- `canonical_basis.rs` retains `value.bytes()` only for canonical
  `binary-content-digest` preparation of foundational opaque/binary value
  families.
- `*.as_bytes()` and `digest_input_bytes` hits are canonical digest mechanics
  and cost/counter proof surfaces over named canonical basis strings, not raw
  bytes as snapshot/source/route/writeback authority.
- Native digest-family prefixes such as
  `bridge-derived-writeback-effect:sha256`,
  `bridge-subscription-signal-strategy:sha256`, and
  `bridge-grouped-truth-view:sha256` are derived from typed native canonical
  basis carriers. They are not arbitrary digest-input constructors.
- `serde_json::Value`, `json!`, and `StructuredValue::Json` occur under
  harness terminal export or terminal snapshot capture topology. Import-wall
  scans prove source, routing, writeback, diagnostics, subscription, and facade
  authority trees do not import those terminal modules.
- `ranked_materials_by_damage` remains typed
  `PricingShockRankedMaterialDamageSet` simulation evidence and terminal
  presentation/export data, not a raw authority collection.

Phase 1 verified with:

- `cargo fmt -p worth-runtime-bridge`
- `cargo check -p worth-runtime-bridge --tests`
- `cargo test -p worth-runtime-bridge harness -- --nocapture`
- `cargo test -p worth-runtime-bridge snapshot -- --nocapture`
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture`
- Gate 3 import-wall scan
- Gate 4 no-test-shim scan
- final crate-wide no-shim classification scans
- runtime-bridge Rust line-cap scan
- scoped `git diff --check`

### Phase 2: Native Snapshot Read Contract

Choke point:

- `snapshot/packet.rs`
- `source/aspect_values.rs`
- `source/row_set.rs`
- `source/grouped_truth_view.rs`

New production shape:

- `SnapshotReadRequest` carries an aspect locator, optional aspect field
  locator/path, an explicit projection mask, and a truth-view basis.
- `SnapshotReadRecord` is replaced by a native result record carrying a
  contract-checked aspect value carrier.
- Broad read packets lower through mask-bounded native contracts.
- Row and grouped materializers consume native records and validate contract,
  mask, and lane membership.

Deletion/refactor:

- Delete production `decode_snapshot_aspect_bytes`.
- Delete production `aspect_bytes` accessors.
- Delete byte-to-JSON snapshot value decoding from source materialization.
- Keep JSON helpers only under explicitly external ingress/export modules whose
  tests prove they terminate before native authority.

Acceptance evidence:

- No production source materialization path calls `serde_json` to recover
  aspect values.
- Snapshot result validation rejects missing, duplicate, wrong-contract,
  wrong-mask, and unsupported-value records before materialization.
- Existing source parity and grouped truth certification continues through
  native value carriers.

Phase 2 progress:

- Source row-set materialization now retains
  `ContractValidatedAspectArtifact` on each `BridgeMaterializedFieldValue`
  instead of cloning a raw `SnapshotReadValue` back out of the validated
  snapshot record. Scalar convenience accessors derive from the retained
  validated artifact, and tests prove the stored canonical basis is recomputed
  from that retained proof carrier.
- `SnapshotReadValue` remains only at the untrusted `SnapshotReadRecord`
  reader-result boundary and inside snapshot contract validation. Static scans
  show no `SnapshotReadValue` or `read_value()` usage under
  `src/source`.
- Grouped truth materialization now distinguishes absent binding aspects from
  present-but-unsupported native value families. Identity and grouping bindings
  must resolve to retained contract-validated scalar artifacts; struct-valued
  bindings return typed `Unsupported*AspectValueFamily` errors carrying the
  binding role, row identity, aspect key, value family, and validated value
  canonical basis before grouped member rows are materialized.
- Phase 1 line-cap enforcement drift was closed while entering Phase 2: all
  stale `worth-runtime-bridge` entries were removed from the workspace Rust
  line-cap allowlist after a crate-local scan proved no runtime-bridge Rust
  file exceeds the cap.

Phase 2 closeout classification:

- `SnapshotReadValue` and `SnapshotReadRecord::read_value()` remain only in
  `snapshot/read_result.rs` and `snapshot/packet.rs`, where the untrusted
  snapshot reader result crosses into bridge validation. No `src/source`
  materializer imports or reads them.
- `payload()` calls in `snapshot/read_result.rs` and
  `source/grouped_truth_view.rs` are foundational
  `ContractValidatedAspectArtifact` accessors used to inspect a retained
  validated artifact family. They are not bridge payload authority or JSON
  recovery.
- `bytes`/`as_bytes()` hits in `src/source` and `src/snapshot` are canonical
  digest mechanics over named native basis strings. No `aspect_bytes` accessors
  or byte-to-JSON decoding remain.
- `serde_json::Value`, `json!`, `serde_json::Map`, `from_value`, and
  `to_value` have no hits under `src/source` or `src/snapshot`.
- The grouped unsupported-family taxonomy is public through the source facade:
  `BridgeGroupedTruthViewError` and `BridgeGroupedBindingValueFamily` can be
  matched by downstream callers without reaching through private modules.

Phase 2 verified with:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge source -- --nocapture`
- `cargo test -p worth-runtime-bridge snapshot -- --nocapture`
- `cargo check -p worth-runtime-bridge --tests`
- source/snapshot raw read-value and JSON residue scans
- source/snapshot no-shim vocabulary scan with only classified foundational
  accessors and canonical digest mechanics
- runtime-bridge Rust line-cap scan
- scoped `git diff --check`

### Phase 3: Native Patch Envelope And Aspect Targets

Choke point:

- `input/envelope/canonical.rs`
- `input/normalization.rs`
- `input/validation.rs`
- `routing/surfaces.rs`

New production shape:

- `BridgeCommittedPatchItem` becomes a native bridge patch target:
  entity/subject identity plus aspect locator, optional field locator/path,
  truth surface kind, and foundational mutation/projection mask basis.
- Prefix parsing is demoted to explicit foreign ingress only.
- Routing consumes normalized native targets, not string labels.
- Target identity and digest basis use foundational locators and field paths.

Deletion/refactor:

- Remove production `surface_label` as the authority input.
- Remove prefix classification from production route derivation.
- Replace `BridgePatchCoordinate` string surface coordinates with aspect target
  coordinates.
- Delete current-test fixtures that manufacture patch authority from
  string-labeled surfaces, except tests whose purpose is to prove external
  ingress lowering fails closed or lowers into native targets.

Acceptance evidence:

- Committed patch envelopes can represent field, relation-endpoint, region,
  partition, and facet targets without encoded strings.
- Unsupported target kinds fail through typed bridge errors.
- Route artifacts preserve native target identity only. Any original external
  label may appear solely in terminal external-ingress rejection diagnostics,
  not in admitted route artifacts.

Phase 3 progress:

- `BridgePatchCoordinate` has been deleted from the current runtime-bridge
  surface. Patch and routing error contexts now expose
  `BridgePatchTargetCoordinate`, retaining entity identity, foundational aspect
  locator, optional foundational field locator, native truth surface kind, and
  target canonical basis.
- `BridgePatchTargetCoordinate::new` is crate-private. Public callers can
  inspect native patch-target evidence retained on errors, but cannot fabricate
  target coordinates from arbitrary basis strings.
- Committed patch envelope construction and route eligibility denial now attach
  the same native patch-target coordinate to typed bridge errors, so
  unsupported locator/mask/surface failures and missing mapping failures carry
  aspect-native context instead of string surface coordinates.
- The mixed error-context bucket was split into
  `error/context/{mod.rs,mapping_freeze.rs,patch_routing.rs,delivery_replay_snapshot.rs}`.
  The split gives patch/routing target evidence, mapping-freeze evidence, and
  delivery/replay/snapshot lifecycle evidence separate homes while preserving
  the public `BridgeErrorContext` facade.
- Mapping target selector canonicalization no longer emits prefix-shaped
  `field:<path>` coordinates. `TruthPatchTargetSelector` now exposes native
  target-selector basis text such as `target-selector|kind=entity-field`
  with an explicit foundational field path, and routing/mapping-registry digest
  preparation consumes that basis instead of selector labels.
- Route eligibility now proves the whole-aspect target matrix through real
  committed-patch envelope construction, aspect registration, mapping lookup,
  and eligibility admission for relation-endpoint, region, partition, and facet
  targets. The admitted route entries retain native target basis and no
  prefix-parsed coordinates.
- Final route planning now proves the same target matrix after artifact
  construction. `BridgePlannedRoute::route_record_entries()` and the planned
  subscription-slice read packet retain native committed-patch target basis for
  relation-endpoint, region, partition, and facet targets; route identities and
  record entries reject `field:`, `surface_label`, and `aspect_label` residue.
- Unsupported target categories are unrepresentable through the public target
  constructors. The removed generic `aspect_surface` path and private
  patch-target-coordinate constructor remain compile-fail guarded, while
  private invalid target shapes fail through typed `UnsupportedTruthDeltaSurface`
  validation before route planning can build artifacts.

Phase 3 verified so far with:

- `cargo fmt -p worth-runtime-bridge`
- `cargo check -p worth-runtime-bridge --tests`
- `cargo test -p worth-runtime-bridge mapping::freezing -- --nocapture`
- `cargo test -p worth-runtime-bridge input::envelope -- --nocapture`
- `cargo test -p worth-runtime-bridge routing::eligibility -- --nocapture`
- `cargo test -p worth-runtime-bridge routing::planning::plan -- --nocapture`
- `cargo test -p worth-runtime-bridge routing::surfaces -- --nocapture`
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --nocapture`
- target-selector prefix residue scan over mapping/routing
- residue scan proving `BridgePatchCoordinate` and `patch_coordinate()` are
  absent from current source/tests
- touched-surface no-shim vocabulary scan
- runtime-bridge Rust line-cap scan
- scoped `git diff --check` with only existing CRLF warnings

Remaining Phase 3 closeout work:

- Phase 3 is closed for the current manifest. Continue with Phase 4
  mask-native routing and subscription slice semantics.

Phase 3 closeout proof, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge routing::planning::plan -- --nocapture`
  (1 passed)
- `cargo test -p worth-runtime-bridge routing::eligibility -- --nocapture`
  (2 passed)
- `cargo test -p worth-runtime-bridge routing::surfaces -- --nocapture`
  (5 passed)
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture`
- `cargo check -p worth-runtime-bridge --tests`
- target residue scan over `routing`, `input`, and `mapping` found only
  pre-existing classified field-target/negative-proof hits, not final route
  artifact foreign-label authority
- `BridgePatchCoordinate` / `patch_coordinate()` residue scan had no hits
- runtime-bridge Rust line-cap scan was clean
- scoped `git diff --check` had only existing CRLF warnings

### Phase 4: Mask-Native Routing And Subscription Slices

Choke point:

- `routing/canonicalization.rs`
- `routing/planning/*`
- `routing/lowering/slices.rs`
- `subscription/declaration/slice_intent.rs`

New production shape:

- Subscription declarations bind to aspect targets and masks.
- Route planning derives slice eligibility from target + mask + frozen
  registration truth.
- Lowered invalidation artifacts carry mask-bounded subscription targets.
- Widening requires a typed admitted-widening class and retained proof.

Deletion/refactor:

- Remove string surface matching as routing authority.
- Remove request-key construction that concatenates entity/aspect/surface text
  as the only identity basis.
- Preserve human-readable labels only as derived explanations.

Acceptance evidence:

- Field, lens, region, partition, and facet routes are driven by native target
  and mask equality/compatibility.
- Diagnostics-tier variation cannot alter route target identity.
- Coarse widening remains explicit, bounded, admitted, and counted.

Phase 4 progress:

- Subscription declaration intent and lowered subscription slices now derive
  native target basis through the same admitted committed-patch target shape.
  The shared path reconstructs `BridgeCommittedPatchTarget` from foundational
  aspect locator, optional field locator, projection mask, and native surface
  kind, then asserts the projection mask matches target law.
- `TruthDeltaSurfaceIdentity` is digest-shaped from a named basis containing
  entity identity, native surface kind, and a private
  `truth-delta-surface-target-mask:sha256:*` proof derived from the typed
  committed-patch target. It no longer embeds entity/aspect/surface/target text
  as the public identity string or reopens committed-patch target basis as
  downstream authority.
- Cross-surface tests prove declaration intent and lowered slices share the
  same committed-patch target/mask basis for field, relation-endpoint/lens,
  region, partition, and facet targets.
- Route surface tests prove surface identities are digest-shaped and do not
  embed native target basis text or human surface labels.
- Committed patch envelope digest identity is now `patch:sha256:*` derived
  from the native committed-patch target basis. The raw patch basis remains
  private digest input only, so route planning provenance can carry
  `patch-digest` without embedding entity/aspect/target basis text.
- Route entry identity is now `route-entry:sha256:*` derived from named route
  entry basis. Route basis and planning provenance retain route-entry digest
  identity instead of concatenating entity/aspect/native-target/mapping/signal
  scope text as the public route-entry identity.
- Final route artifact tests now assert the separation explicitly: route record
  target evidence retains native committed-patch target basis, while
  `TruthDeltaSurfaceIdentity` remains digest-shaped and does not embed the
  target basis.
- Bulk planning now carries `BridgeRouteIdentity` through canonical workload
  requests, routing packets, continuity remap packets, widening aggregation
  packets, reduced publications, and reduced widening artifacts. Route
  identity no longer collapses into raw `Arc<str>` authority inside the bulk
  packet/reduction chain; string projection remains only for digest-basis and
  presentation formatting.
- Bulk workload and packet-reduction tests now prove the typed route identity
  survives canonical request construction, packetization, continuity/widening
  origin tracking, and reduced route identity lists.
- Bulk parallel-preparation legality proof now carries typed
  `BulkPacketRegionIdentity` values through disjoint packet regions and
  admitted preparation partitions. Region identities are digest-shaped
  `bulk-packet-region:sha256:*` artifacts derived from route, truth-view, or
  continuity packet locality basis; the old `route-partition:*`,
  `truth-view-partition:*`, and `continuity-partition:*` concatenated keys no
  longer cross the admission proof boundary. Parallel legality now checks the
  typed packet-region identities for uniqueness before emitting
  `DisjointPacketRegionsCertified`.
- Bulk routing packet reduction now retains typed subscription-slice identities
  through `TruthDeltaRoutingPacket`, `InvalidationReductionPacket`, and
  `ReducedBridgePublication`. The old reduced-target string scope accessor is
  gone from the packet lane; `.as_str()` is used only while rendering canonical
  digest basis.
- Bulk reduction families are closed through `BridgeInvalidationReductionFamily`
  instead of stored string family names, and widening packets/reduced widening
  artifacts carry the admitted `BridgeMappingWideningClass` enum rather than a
  lowercased debug string. Packet-reduction tests assert enum equality and
  typed subscription-slice equality across routing packets, reduction packets,
  and reduced publication artifacts.
- Bulk widening bounded scopes now retain typed `TruthDeltaSurfaceIdentity`
  through route records, `WideningAggregationPacket`, reduction grouping, and
  `ReducedWideningAggregation`. The old string projection of the route-record
  surface identity no longer crosses the packet/reduction authority boundary;
  `.as_str()` remains only for digest-basis rendering and public read
  projection.
- Lowered invalidation targets now carry `BridgeInvalidationTargetIdentity`,
  backed by the shared `BridgeIdentity` tag system, plus native committed-patch
  target basis and truth-delta surface identity. Canonical invalidation and
  lowering-summary digests consume the typed target identity instead of a
  `signal_scope:routing_mode` string key, so two routes with the same signal
  scope but different typed truth-delta surface proofs no longer collapse at
  the lowered invalidation target boundary.
- Frozen mapping registrations now carry
  `BridgeFrozenMappingRegistrationIdentity`, backed by the shared
  `BridgeIdentity` tag system and derived once at mapping freeze from mapping
  id, native truth-scope selectors, snapshot read contract basis, signal scope,
  and routing mode. Route-entry and bulk mapping-registry digest bases consume
  that typed frozen-registration identity instead of rebuilding registration
  selector/signal/routing text in downstream routing proof code.
- Subscription declarations and lowered subscription slices now share a
  crate-local `BridgeSubscriptionSliceTargetIdentity` proof derived from entity
  identity, admitted committed-patch target basis, projection mask, native
  surface kind, and subscription slice kind. Declaration digests and lowering
  subscription-slice digest bases consume that typed target/mask identity
  instead of carrying the full committed-patch target basis as downstream
  identity input; native target basis remains retained only as target evidence.
  The proof identity is not exported as a public construction surface.
- Route-entry digest basis now consumes typed
  `TruthDeltaSurfaceIdentity` and `BridgeFrozenMappingRegistrationIdentity`
  proof values only. It no longer rebuilds route-entry identity from raw
  entity, aspect, native-target, mapping id, signal scope, or routing text.
  Native committed-patch target basis remains retained on route records as
  evidence for diagnostics and explanation, not as route-entry authority.
- Normalized truth-delta surfaces now retain the native
  `BridgeCommittedPatchTarget` carrier from the committed patch item instead
  of storing decomposed aspect key, field locator, surface kind, and projection
  mask evidence as parallel routing authority. Surface normalization is now a
  pure committed-patch-target derivation; aspect registrations influence
  fine-grained classification only. Lowered subscription slices consume the
  projection mask from the retained target rather than reconstructing it from
  field presence.
- Lowered invalidation target identity now consumes typed
  `TruthDeltaSurfaceIdentity` as the target/mask proof and no longer includes
  native committed-patch target basis text in its identity basis. The native
  target basis remains retained on `BridgeInvalidationTarget` as diagnostic
  evidence only; invalidation and lowering-summary digests continue to consume
  the digest-shaped target identity.
- Subscription slice target identity now consumes a private
  `subscription-target-mask:sha256:*` proof derived from the admitted
  `BridgeCommittedPatchTarget`, rather than accepting native committed-patch
  target basis text as a downstream identity input. Declaration intents and
  lowered subscription slices pass typed committed-patch target carriers into
  the proof boundary; native target basis remains retained only as target
  evidence and terminal/report projection data.
- Truth-delta surface identity now consumes a private
  `truth-delta-surface-target-mask:sha256:*` proof derived from the typed
  `BridgeCommittedPatchTarget`, rather than using native committed-patch target
  basis text as public surface-identity authority. Native target basis remains
  retained on `TruthDeltaSurface` only as diagnostic/explanation evidence; the
  surface identity basis consumes the digest-shaped target/mask proof plus
  entity identity and native surface kind.
- Snapshot read targets now carry `SnapshotReadTargetIdentity`, a typed
  `snapshot-read-target:sha256:*` proof derived at target construction from the
  snapshot read contract and native target evidence. Snapshot read request
  canonical basis, correlation ids, packet digest basis, and canonical read
  ordering consume the typed target identity rather than embedding native target
  basis text or reopening `snapshot-read-target|locator=...` evidence.
- Prior continuity subscription slices now retain the lowered subscription
  slice canonical proof and use it for prior-slice canonical basis, continuity
  request correlation ids, and logical deduplication. Native target basis
  remains retained only as evidence for reconstructing successor slice shape and
  test assertions, not as prior-slice authority.
- Route diagnostic entries now retain typed `BridgeCommittedPatchTarget`
  carriers for both normalized route target evidence and source patch target
  evidence. `target_canonical_basis()` and
  `source_target_canonical_basis()` are projection-only report accessors derived
  from those carriers; machine-readable route explanations can inspect native
  locators, masks, and target kind directly instead of trusting formatted target
  basis text.
- Patch and routing error coordinates now retain typed
  `BridgeCommittedPatchTarget` carriers instead of decomposed aspect locator,
  field locator, surface kind, and target-basis fields. Entity identity remains
  the coordinate owner, while aspect key, locator, field locator, surface kind,
  projection mask, and target canonical basis are derived from the retained
  committed-patch target. The coordinate constructor is crate-internal and the
  compile-fail proof uses a sealed native target placeholder rather than WORTHd
  target-basis strings.

Phase 4 lowered invalidation target proof, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge routing -- --nocapture` (37 passed)
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture` (1 passed)
- `cargo check -p worth-runtime-bridge --tests`
- targeted residue scan found no `canonical_target_order`,
  `CanonicalTargetView`, tuple-shaped invalidation target handoff, or old
  signal-scope/routing-mode target-key authority in routing.
- line-cap scan over routing and routing harness tests was clean.
- scoped `git diff --check` reported only existing LF/CRLF warnings.

Phase 4 frozen mapping registration identity proof, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo check -p worth-runtime-bridge --tests`
- `cargo test -p worth-runtime-bridge mapping::freezing -- --nocapture`
  (11 passed)
- `cargo test -p worth-runtime-bridge routing::canonicalization --
  --nocapture` (1 passed)
- `cargo test -p worth-runtime-bridge bulk_workload -- --nocapture`
  (11 passed)
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture` (1 passed)
- QA found and fixed the initial line-cap violation by moving the new frozen
  registration proof into `mapping/freezing/tests/registration_identity.rs`,
  and deleted the stale `AspectKeySelector::canonical_label()` helper left by
  removing downstream inline mapping-registry basis assembly.
- targeted residue scan now leaves signal-scope/routing-mode basis text only in
  typed frozen-registration identity construction and typed invalidation-target
  identity construction; route-entry and bulk mapping-registry bases consume
  typed identities.
- touched mapping/routing/bulk workload files remain under the 400-line cap;
  scoped `git diff --check` reported only existing LF/CRLF warnings.

Phase 4 latest proof, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge input::envelope -- --nocapture`
- `cargo test -p worth-runtime-bridge routing::canonicalization --
  --nocapture`
- `cargo test -p worth-runtime-bridge routing::planning::plan --
  --nocapture`
- `cargo check -p worth-runtime-bridge --tests`
- raw patch-basis residue scan over `input` and `routing` shows raw
  `patch|commit=...` only as private digest input plus the negative test
  assertion; no route/provenance entry embeds `committed-patch-target` basis.
- touched runtime-bridge files remain under the 400-line cap.

Phase 4 bulk route identity proof, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge packet_reduction -- --nocapture`
  (11 passed)
- `cargo test -p worth-runtime-bridge bulk_workload -- --nocapture`
  (10 passed)
- `cargo check -p worth-runtime-bridge --tests`
- targeted bulk route identity residue scan found no raw
  `route_identity: Arc<str>`, `originating_route_identity: Arc<str>`,
  `Arc<[Arc<str>]>` reduced route identity, string route-member accessor, or
  string route-identity accessor carriers under `routing/planning/bulk`.
- touched bulk planning and coupled planning-test files remain under the
  400-line cap.

Phase 4 bulk packet reduction identity proof, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo check -p worth-runtime-bridge --tests`
- `cargo test -p worth-runtime-bridge packet_reduction -- --nocapture`
  (11 passed)
- `cargo test -p worth-runtime-bridge bulk_workload -- --nocapture`
  (10 passed)
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture`
- targeted packet-reduction residue scan over `routing/planning/bulk` and
  `harness/tests/planning/packet_reduction` found no
  `subscription_slice_identity: Arc<str>`, `reduced_target_scope`,
  `reduction_family: Arc<str>`, `widening_class: Arc<str>`, or lowercased debug
  widening-class construction. The remaining `reduced_target_scope` text is
  historical milestone documentation only.
- touched bulk planning and packet-reduction test files remain under the
  400-line cap; scoped `git diff --check` reported only existing LF/CRLF
  warnings.

Phase 4 normalized surface target-carrier proof, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge routing -- --nocapture` (37 passed)
- `cargo check -p worth-runtime-bridge --tests`
- QA found and fixed two proof-boundary issues: the first edit left a no-op
  aspect-registry lookup inside normalization, and `TruthDeltaSurface` still
  duplicated `surface_kind` beside the retained target. Both were removed.
- Surface normalization now returns `NormalizedTruthDeltaSurfaceSet` directly
  instead of a fallible `Result`, because committed patch envelope admission
  owns target validation and registration matching happens later in
  fine-grained classification.
- Targeted residue scans found no registry-influenced
  `derive_normalized_truth_delta_surface_set(..., registry)` callsites, no
  `projection_mask_for_surface` reconstruction helper, and no routing tests
  preserving `expect("...normalize")` assertions for surface normalization.
- Routing line-cap scan was clean; scoped `git diff --check` reported only
  existing LF/CRLF warnings.

Phase 4 canonical lowering proof consumption, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge routing::canonicalization -- --nocapture`
  (3 passed)
- `cargo test -p worth-runtime-bridge routing::lowering -- --nocapture`
  (1 passed)
- `cargo check -p worth-runtime-bridge --tests`
- QA found and fixed a proof-strength gap: after production canonicalization
  stopped re-deriving registration and subscription slice keys, the first test
  pass proved only digest-basis non-leakage. A direct route-entry ordering proof
  now constructs two real native registrations through `validate_route_request`
  and asserts ordering follows the typed frozen registration identity after the
  normalized truth-delta surface proof.
- Route-entry canonical ordering now consumes
  `BridgeFrozenMappingRegistrationIdentity` instead of reopening mapping id,
  selector, target selector, signal scope, specificity, and routing-mode text.
- Subscription slice digest bases and lowering summaries now consume the
  retained `BridgeSubscriptionSlice::canonical_basis()` generated at lowering
  construction time instead of rebuilding a parallel key from slice target,
  snapshot read contract, and match-status pieces downstream.
- The now test-only `BridgeSubscriptionSlice::slice_target_identity()` accessor
  is gated behind `cfg(test)` so production does not retain a dead convenience
  surface after canonicalization switched to the retained slice basis.
- Targeted residue scans found no `canonical_subscription_slice_key`,
  `canonical_registration_order`, selector-order helper, target-selector-order
  helper, or downstream slice-target/read-contract key reconstruction in
  routing/subscription. Routing line-cap scan was clean; scoped
  `git diff --check` reported only existing LF/CRLF warnings.

Phase 4 invalidation target surface-proof consumption, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge routing::lowering::targets --
  --nocapture` (1 passed)
- `cargo test -p worth-runtime-bridge fine_grained_precision -- --nocapture`
  (3 passed)
- `cargo check -p worth-runtime-bridge --tests`
- QA found the lowered invalidation target identity was still reopening native
  committed-patch target basis text even though `TruthDeltaSurfaceIdentity`
  already carries the mask-bounded target proof. The identity basis now uses
  signal scope, routing mode, and typed truth-delta surface identity only.
- A direct lowered-target proof asserts that changing retained native target
  basis under the same surface proof does not change target identity, while
  changing the typed surface proof does. The fine-grained routing proof was
  renamed to assert identity variation through surface proof rather than
  through native target basis text.
- Targeted residue scan found no invalidation target identity basis that embeds
  `native_target_basis`/`committed-patch-target` text. Touched routing lowering
  and routing harness files remain under the 400-line cap; scoped
  `git diff --check` reported only existing LF/CRLF warnings.

Phase 4 subscription target-mask proof consumption, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge subscription::declaration --
  --nocapture` (17 passed)
- `cargo test -p worth-runtime-bridge routing::lowering -- --nocapture`
  (2 passed)
- `cargo test -p worth-runtime-bridge routing::canonicalization --
  --nocapture` (3 passed)
- `cargo check -p worth-runtime-bridge --tests`
- QA found and fixed an initial proof-boundary weakness: the first edit changed
  the function argument from raw target basis to `BridgeCommittedPatchTarget`,
  but the final slice identity basis still embedded the full target canonical
  basis internally. The final shape derives a private
  `BridgeSubscriptionTargetMaskIdentity` first, then derives
  `BridgeSubscriptionSliceTargetIdentity` from entity identity, that typed
  target/mask proof, and slice kind.
- Targeted residue scan found no `subscription_slice_target_identity(...)`
  caller passing native target-basis text, no `subscription-slice-target|...`
  basis embedding committed-patch target basis text, and no public/facade
  exposure of the private target-mask proof. Touched subscription/routing files
  remain under the 400-line cap; scoped `git diff --check` reported only
  existing LF/CRLF warnings.

Phase 4 truth-delta surface target-mask proof consumption, recorded
2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge routing::surfaces -- --nocapture`
  (6 passed)
- `cargo test -p worth-runtime-bridge routing::canonicalization --
  --nocapture` (3 passed)
- `cargo test -p worth-runtime-bridge routing::lowering -- --nocapture`
  (2 passed)
- `cargo test -p worth-runtime-bridge fine_grained_precision -- --nocapture`
  (3 passed)
- `cargo check -p worth-runtime-bridge --tests`
- QA found no structural blocker after the rewrite: the target/mask proof stays
  private to `routing/surfaces.rs`, no public facade construction surface was
  added, and no topology split was justified for the narrow proof handoff.
- Targeted residue scan found no `truth_delta_surface_identity(...)` caller
  passing native target-basis text and no `truth-delta-surface|...` basis
  embedding committed-patch target basis text. The only hit was the intended
  private `TruthDeltaSurfaceTargetMaskIdentityTag`. Routing line-cap scan was
  clean; scoped `git diff --check` reported only the existing LF/CRLF warning
  on `routing/surfaces.rs`.

Phase 4 snapshot read target proof consumption, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge snapshot -- --nocapture` (50 passed)
- `cargo test -p worth-runtime-bridge routing::canonicalization --
  --nocapture` (4 passed)
- `cargo test -p worth-runtime-bridge routing::planning::plan -- --nocapture`
  (1 passed)
- `cargo check -p worth-runtime-bridge --tests`
- QA found and fixed one proof-boundary cleanup issue: after request and
  ordering authority moved to `SnapshotReadTargetIdentity`, the old
  `SnapshotReadTarget::canonical_basis()` convenience surface was dead and could
  invite downstream re-opening of target basis. It was deleted.
- Snapshot read request canonical basis and canonical request ordering now
  consume typed `SnapshotReadTargetIdentity`; request/correlation/packet
  authority no longer embeds `native-target=` or `snapshot-read-target|locator=`
  text. Native target basis remains retained on `SnapshotReadTarget` only as
  evidence/projection.
- Targeted residue scan found no snapshot request basis embedding native target
  text and no `target().canonical_basis()` caller. The only
  `snapshot-read-target|contract=...native-target=...` hit is the private
  one-way target identity proof derivation. Touched snapshot/routing files
  remain under the 400-line cap; scoped `git diff --check` reported only
  existing LF/CRLF warnings.

Phase 4 continuity prior-slice proof consumption, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge continuity -- --nocapture` (49 passed)
- `cargo test -p worth-runtime-bridge routing::planning::plan -- --nocapture`
  (1 passed)
- `cargo check -p worth-runtime-bridge --tests`
- QA found and fixed two adjacent issues. First, `PriorSubscriptionSlice`
  canonical and logical dedup bases still reopened native target basis even
  though each prior slice is derived from a lowered `BridgeSubscriptionSlice`
  that already has a mask-native canonical proof. The basis now consumes the
  retained subscription-slice canonical proof. Second, one split-successor test
  asserted incidental successor ordering; it now asserts the successor set,
  which is the continuity contract.
- Targeted residue scan found no `prior-slice|...target=` or
  `prior-slice-logical|...target=` authority basis. Remaining
  `native_target_basis()` hits in the continuity slice are retained target
  evidence for successor reconstruction or negative assertions. Touched files
  remain under the 400-line cap; scoped `git diff --check` was clean.

Phase 4 snapshot read coordinate proof consumption, recorded 2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge delivery::snapshot -- --nocapture`
  (9 passed)
- `cargo test -p worth-runtime-bridge snapshot -- --nocapture` (50 passed)
- `cargo check -p worth-runtime-bridge --tests`
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture`
- QA found and fixed a public construction seam: `BridgeSnapshotReadCoordinate`
  exposed public constructors, and the subscription-slice constructor accepted
  raw target canonical-basis text. That made an error-coordinate proof look
  constructible from caller text even though delivery already had a typed
  `SnapshotReadTargetIdentity` proof.
- Snapshot read coordinates are now crate-internally constructed. Subscription
  slice coordinates retain `SnapshotReadTargetIdentity`; the old public raw
  target-basis constructor path is covered by
  `snapshot_read_coordinate_constructor_private` compile-fail proof.
- The delivery snapshot contract rejection proof now asserts that the retained
  read coordinate carries the planned snapshot target identity and that the
  coordinate proof is digest-shaped. `SnapshotReadTargetIdentity` is exported as
  the public read-only evidence type; native target basis remains retained only
  inside snapshot targets as internal evidence/projection.
- Targeted residue scan found no public snapshot-coordinate constructors, no
  `target_canonical_basis: Option<_>` snapshot coordinate storage, and no
  delivery handoff from `read.native_target_basis()`. Touched files remain under
  the 400-line cap; scoped `git diff --check` reported only existing LF/CRLF
  warnings.

Phase 4 patch/routing error-coordinate target proof consumption, recorded
2026-06-02:

- `cargo fmt -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge input::envelope -- --nocapture`
  (8 passed)
- `cargo test -p worth-runtime-bridge routing::eligibility -- --nocapture`
  (2 passed)
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail --
  --nocapture`
- `cargo check -p worth-runtime-bridge --tests`
- QA found and fixed one duplicate-evidence issue: the first coordinate rewrite
  retained a cached target canonical-basis projection beside the typed
  committed-patch target. The final coordinate stores only entity identity and
  `BridgeCommittedPatchTarget`; target canonical basis is returned as an owned
  projection derived from the target at accessor time.
- Patch construction and missing-route registration denials now retain native
  committed-patch target coordinates that expose foundational aspect locator,
  field locator, target kind, projection mask, and projection-only target basis.
  The UI compile-fail fixture proves external callers cannot construct patch
  target coordinates and no longer teaches WORTHd target-basis string input.
- Targeted residue scan found no old decomposed coordinate constructor, no
  cached `target_canonical_basis_projection`, and no `WORTHd-target-basis`
  fixture. Touched files remain under the 400-line cap; scoped
  `git diff --check` reported only existing LF/CRLF warnings.

Phase 4 final closeout readiness, recorded 2026-06-02:

- Bounded target/mask authority scan over routing, subscription, snapshot,
  continuity, delivery, diagnostics, and error surfaces found one remaining
  duplicate retained projection cache in route diagnostics:
  `BridgeRouteRecordEntry` stored `target_canonical_basis_projection` and
  `source_target_canonical_basis_projection` beside typed
  `BridgeCommittedPatchTarget` carriers.
- The route diagnostic record now stores only the typed target carriers and
  derives `target_canonical_basis()` / `source_target_canonical_basis()` as
  projection-only accessors at report time. No target-basis string survives as a
  parallel diagnostic record field.
- Remaining scan hits are classified as private one-way proof construction from
  typed carriers (`truth-delta-surface-target-mask`,
  `subscription-target-mask`, `snapshot-read-target`), retained
  diagnostic/evidence projection, terminal report/test assertions, or canonical
  basis rows that consume typed identities instead of native target basis.
- Verification: `cargo fmt -p worth-runtime-bridge`;
  `cargo test -p worth-runtime-bridge routing::planning::plan -- --nocapture`
  (1 passed); `cargo test -p worth-runtime-bridge explanations -- --nocapture`
  (6 passed); `cargo check -p worth-runtime-bridge --tests`; duplicate route
  projection-cache scan clean; touched files under the 400-line cap; scoped
  `git diff --check` reported only the existing LF/CRLF warning.
- Phase 4 is closed for the current manifest. Continue with Phase 5 grouped
  truth and query-facing projection contracts.

### Phase 5: Grouped Truth And Query-Facing Projection Contracts

Choke point:

- `source/row_set.rs`
- `source/grouped_truth_view.rs`
- grouped lane exports in `facade/exports_core.rs`

New production shape:

- Grouped truth view materialization is declared as projection over native
  aspect contracts and masks.
- Field values carry `AspectValue` or `StructAspectValue` plus field-path proof
  where the lane projects a struct member.
- Grouped lane identity uses foundational grouped public lane semantics where
  applicable.

Deletion/refactor:

- Remove any row assembly that infers field meaning from request key strings.
- Remove terminal JSON assumptions from grouped truth artifacts.

Acceptance evidence:

- Grouped truth certification can prove lane, field, mask, and value parity
  without inspecting JSON.
- Query-facing bridge exports retain native aspect values until an explicit
  consumer projection boundary.

Phase 5 progress, recorded 2026-06-02:

- `BridgeMaterializedFieldIdentity` derives from foundational locator and
  projection-mask canonical basis, and materialized rows key fields by that
  typed identity instead of request key strings.
- Grouped truth materialization no longer asks rows for the first field matching
  an aspect key. It consumes exactly one whole-aspect projection matching the
  foundational `AspectKey`, no field locator, and a whole-aspect
  `ProjectionMask`.
- Ambiguous grouped binding projections now fail through typed
  `AmbiguousIdentityAspect` or `AmbiguousGroupingAspect` errors with row,
  aspect, and matching-projection count evidence instead of silently selecting
  whichever projection sorts first.
- Grouped projection contracts and materialized lane values now retain the
  lane grouping aspect as foundational `AspectKey`; string accessors are
  read-only projection surfaces for downstream display/consumer boundaries.
- Invalid grouping aspect-key text is no longer a grouped truth materialization
  denial path. `GroupedProjectionSource` now requires foundational `AspectKey`
  carriers for grouping, identity binding, and grouping binding keys, making
  invalid key text unrepresentable at the projection-source boundary.
- Tests prove grouped truth rejects ambiguous whole-aspect grouping bindings,
  struct-valued bindings still fail before member materialization, row-set
  projections expose digest-shaped field identities plus locator/mask canonical
  basis evidence, and grouped projection sources cannot supply unvalidated
  aspect-key text.
- Verification: `cargo fmt -p worth-runtime-bridge`;
  `cargo test -p worth-runtime-bridge source -- --nocapture`; and
  `cargo check -p worth-runtime-bridge --tests` all passed with 10 minute
  command timeouts. Targeted residue scans found no `field_for_aspect_key`,
  direct `.fields().get(...)`, candidate-collection shim, or production
  string-stored grouped lane aspect in source/facade; touched files stayed
  under the 400-line cap, and scoped `git diff --check` reported only existing
  LF/CRLF warnings.
- The public `GroupedProjectionSource` trait hard-break is complete:
  `GroupedProjectionContractError` and `InvalidProjectionContract` are deleted,
  the direct runtime-bridge, worth-query, and worth-relational implementors now
  carry foundational `AspectKey`, and downstream checks consume typed
  historical-lineage identities and validated row-set values instead of old key
  or read-value shims.
- Verification: `cargo fmt -p worth-runtime-bridge -p worth-query -p
  worth-relational`; `cargo test -p worth-runtime-bridge source --
  --nocapture`; `cargo check -p worth-runtime-bridge --tests`; `cargo check -p
  worth-query --tests`; and `cargo check -p worth-relational --tests` all
  passed with 10 minute command timeouts. Targeted residue scans found no
  `GroupedProjectionContractError`, `InvalidProjectionContract`, string-returning
  `GroupedProjectionSource` binding methods, old historical lineage key
  accessors, `project_snapshot_read_value_for_consumption_json`, or bridge row
  `read_value()` calls in the touched seams. The touched over-cap relational
  bridge source test was split into `bridge_source_tests/{mod,support,lineage,
  publication}.rs`; the remaining touched over-cap correspondence test is an
  existing CI allowlisted file.
- Query-facing grouped planning/live projection surfaces now retain native
  `AspectKey` for grouping aspect authority across `ViewShapeDescriptor`,
  `DeclarativeLiveViewShape`, `WORTHQueryLiveViewBuilder`,
  `GroupedViewPlanningArtifact`, baseline materialization, delivery metadata,
  grouped execution lanes, desired state, and delta comparison. Bridge grouped
  binding proofs expose native aspect keys, and query grouped execution compares
  those native keys rather than reconstructed strings.
- Projection-consumption grouped machine facts now retain native `AspectKey` for
  grouped memberships and grouped relation endpoints. Relational and bridge
  grouped extraction pass native grouping keys into `ConsumedProjectionFactSet`;
  fact-set digest and certification oracle digests consume native key accessors;
  `grouping_aspect()` remains only a projection/read accessor.
- Verification: `cargo test -p worth-query projection_consumption --
  --nocapture`, `cargo check -p worth-query --tests`, and
  `cargo test -p worth-runtime-bridge source -- --nocapture` all passed with
  10 minute command timeouts. Targeted residue scans found no string-stored
  grouped aspect authority in projection consumption, view-shape planning/live,
  or runtime-bridge grouped contracts. Touched projection-consumption files stay
  under the 400-line cap; the only broad projection-consumption over-cap test
  file observed was pre-existing and untouched.
- Phase 5 is closed for the current grouped truth/query-facing projection
  manifest. Continue with Phase 6/Phase 7 reconciliation from current code and
  this spec; do not reopen Phase 5 unless later work exposes a concrete grouped
  aspect authority blocker.

### Phase 6: Aspect-Native Writeback Intent

Status:

- Phase 6A complete: writeback effect lowering now requires
  `BridgeWritebackEffectIntent`, backed by foundational
  `AuthoritativeRecordAspectPatch` and canonical patch-basis preparation.
- The former `domain_payload_digest` authority seam has been removed from
  production writeback mapper/effect/facade/diagnostic surfaces; retained
  diagnostics now name the derived `effect_intent_digest`.
- Adapter-bound writeback authority requests and receipts now retain the native
  `BridgeWritebackEffectIntent` carrier, not only effect-intent digest strings.
  Digest and canonical-basis fields remain projection/evidence accessors derived
  from that typed intent.

Choke point:

- `writeback/mapper.rs`
- `writeback/effect.rs`
- `facade/runtime/writeback.rs`
- `diagnostics/writeback.rs`

New production shape:

- Writeback mapper envelopes carry a typed aspect-native intent/effect:
  foundational patch envelope, mutation plan, or bridge-owned effect carrier
  whose fields are aspect contracts, locators, masks, and values.
- The former `domain_payload_digest` concept is deleted from native production
  naming. Any digest exposed downstream is named as a digest of the typed
  effect intent or authority artifact that produced it.
- Authority requests validate target contract, mutation mask, causality, and
  strategy compatibility before handoff.

Deletion/refactor:

- Digest-only proposed effect admission is removed from the production
  mapper/effect/facade path.
- Facade writeback lowering no longer accepts arbitrary domain payload/evidence
  digests as the effect authority.
- Adapter writeback authority requests and receipts cannot be constructed from
  effect digest strings alone; they are minted from native effect intent
  evidence and retain that carrier for authority consumers.
- Diagnostics fields that expose effect digests are projection/evidence fields
  derived from typed effect intent and patch canonical-basis carriers.

Acceptance evidence:

- Equivalent typed effects produce equal writeback digests.
- Digest collisions or arbitrary digest strings cannot create a valid
  writeback candidate.
- Writeback certification bundles include native effect, authority request,
  receipt, failure, and counter artifacts with JSON only as export.

Current closeout evidence:

- `TruthWritebackRequest` retains `BridgeWritebackEffectIntent` at the adapter
  authority boundary and exposes `effect_intent()` as the native read-only
  carrier. `effect_intent_digest()` and
  `effect_intent_patch_canonical_basis()` are projections from that carrier.
- `TruthWritebackReceipt` retains the same native effect intent copied from the
  producing request. Receipt digest, request digest, and authoritative artifact
  digest remain evidence projections, not construction authority.
- `BridgeWritebackExecutionRecord` retains optional typed
  `TruthWritebackRequest` and `TruthWritebackReceipt` authority carriers.
  Request and receipt digest accessors derive from those carriers for
  diagnostic/report projection only.
- `BridgeMutationProvenanceBundle` retains the same optional typed
  request/receipt authority carriers from the execution record, and
  `BridgeBatchMutationAuthorityBundle` counts retained typed authority request
  and receipt carriers rather than digest fields.
- `BridgeAdmittedWritebackExecutionReceipt` retains the typed admitted execution
  request and typed authority receipt. Its request and authority-receipt digest
  accessors are projections from those retained carriers, and
  `BridgeAdmittedWritebackExecution` no longer duplicates the authority receipt
  beside the sealed execution receipt.
- `BridgeWritebackLoopPreventionReport` retains native
  `BridgeWritebackFeedbackProvenance`, optional
  `BridgeWritebackFeedbackContext`, and `BridgeWritebackIdempotenceBasis`
  carriers. Current/incoming feedback, causality, and idempotence digest
  accessors derive terminal projections from those retained carriers instead of
  cached strings.
- Writeback safety-gate diagnostics retain typed loop-prevention and
  strategy-coherence reports first. Explanation digest accessors are projections
  from retained reports, so diagnostics cannot become an alternate authority
  store for safety-gate proof.
- Facade writeback authority-session and admitted-execution-contract tests prove
  request and receipt artifacts retain the same foundational authoritative patch
  carried by the lowered writeback effect.
- Duplicate-authority writeback certification now retains native
  `BridgeDerivedWritebackEffect`, `BridgeWritebackReplayBundle`,
  `BridgeWritebackLoopPreventionReport`, `BridgeWritebackIdempotenceBasis`,
  `BridgeWritebackAuthorityOutcome`, `TruthWritebackRequest`, and
  `TruthWritebackReceipt` artifacts in the typed certification matrix/report
  path. Digest and patch canonical-basis accessors are derived projections for
  terminal report export, not stored bundle truth.
- Authority-denial writeback certification now retains native
  `AdmittedBridgeWritebackContract`, `BridgeWritebackStrategyBasis`,
  `BridgeWritebackStrategyCoherenceReport`, `BridgeDerivedWritebackEffect`, and
  `BridgeWritebackIdempotenceBasis` evidence for each post-lowering denial, plus
  typed `TruthWritebackRequest` / `TruthWritebackReceipt` artifacts for
  authority-crossing merge rejections. The rejecting authority fixture records
  the typed rejected attempt, while denial digest and disposition accessors
  derive only terminal report projections from retained carriers.
- Feedback-loop writeback certification now retains native
  `BridgeDerivedWritebackEffect`, `BridgeWritebackReplayBundle`,
  `AdmittedBridgeWritebackContract`, `BridgeWritebackIdempotenceBasis`,
  `BridgeWritebackLoopPreventionReport`,
  `BridgeWritebackStrategyCoherenceReport`, optional
  `BridgeValidatedWritebackCandidate`, optional `TruthWritebackRequest` /
  `TruthWritebackReceipt`, changed-effect idempotence/failure, and rebuilt
  contract/effect/idempotence/loop-prevention/outcome/replay-bundle/receipt
  carriers. Digest and disposition accessors derive terminal projections from
  retained typed carriers; the duplicate loop-prevention digest/disposition
  cache was deleted.
- Admission-boundary writeback certification now retains native family and
  authority-boundary carriers for both projected and aspect families:
  `AdmittedBridgeWritebackContract`, `BridgeDerivedWritebackEffect`,
  `BridgeWritebackIdempotenceBasis`, `BridgeWritebackReplayBundle`, and
  `BridgeWritebackAuthorityOutcome`. Contract/effect/idempotence/replay/authority
  digest accessors derive terminal projections from those retained carriers
  rather than preserving digest-only family proof.
- Mapper-parity writeback certification now retains native projected/aspect
  `BridgeDerivedWritebackEffect` and `BridgeWritebackReplayBundle` carriers in
  each family row. Effect-intent, patch canonical-basis, causality,
  mapped-input, mapper-envelope, and replay digest accessors derive terminal
  projections from retained carriers instead of storing digest-only family
  truth.
- Replay-loop isolation certification now retains native family and isolation
  carriers across projected/aspect family rows, cross-family replay, same-family
  rebuild equivalence, changed-causality isolation, and feedback loop
  prevention: `BridgeDerivedWritebackEffect`,
  `BridgeWritebackIdempotenceBasis`, `BridgeWritebackReplayBundle`,
  `BridgeWritebackError`, `BridgeWritebackReplayRecord`,
  `BridgeWritebackExecutionRecord`, `BridgeWritebackFeedbackContext`, and
  `BridgeWritebackLoopPreventionReport`. Digest, equality, disposition, and
  decision-trace accessors derive terminal projections from those retained
  carriers.

### Phase 7: Typed Diagnostics, Counters, And Certification Bundles

Choke point:

- `diagnostics/*`
- `harness/adapter/*`
- certification bundle builders
- counter JSON helpers

New production shape:

- Diagnostics records are typed artifacts first.
- Counter snapshots are typed artifacts first.
- Certification bundles carry native bridge evidence and expose JSON only via
  terminal export helpers.
- Failure classes identify unsupported aspect contract, value carrier, target,
  mask, patch, source result, and writeback intent boundaries.

Deletion/refactor:

- Remove production `serde_json::Value` as stored diagnostic meaning.
- Remove harness-local JSON maps as the only source of bundle truth.

Acceptance evidence:

- Offline bundle sufficiency works from typed bundle artifacts.
- JSON export can be deleted or changed without changing certification truth.
- Diagnostics-tier variation changes retained explanation only.

Current closeout evidence:

- The duplicate-authority certification matrix and authority-boundary matrix are
  typed artifacts first: replay bundle, effect, authority request, and authority
  receipt are retained directly, while terminal JSON receives only derived
  digest projections. Loop-prevention and attempt reports retain typed
  loop-prevention, idempotence, outcome, replay-bundle, and receipt carriers.
  The focused typed certification proof asserts the retained effect intent is
  identical across matrix effect evidence, both authority requests, both
  receipts, and the retained replay bundle.
- The authority-denial certification matrix is typed evidence first for the
  denial lanes that have crossed lowering or authority: admitted contract,
  strategy-basis, strategy-coherence, lowered effect, and idempotence carriers
  are retained directly; merge-authority rejection retains request/receipt
  carriers; pre-authority feedback denials retain incoming
  `BridgeWritebackFeedbackContext`; and loop-prevention evidence retains the
  typed loop-prevention report while proving no request/receipt exists.
- The feedback-loop certification matrix is typed evidence first for
  convergence, loop prevention, changed-effect rejection, interleaving, and
  restart replay. It retains `BridgeWritebackCausalityBasis`,
  `BridgeWritebackFeedbackProvenance`, `BridgeWritebackFeedbackContext`,
  `BridgeRouteIdentity`, and `TruthCommitIdentity` carriers instead of cached
  route/causality/provenance/commit digest strings. Feedback-loop certification
  execution is split into origin, publication, replay, and restart proof-step
  modules; terminal digest and identity strings are derived only by projection
  accessors and terminal report export. The focused typed proof walks the matrix
  directly and proves effect, replay bundle, idempotence, loop-prevention,
  authority boundary, changed-effect, interleaved truth, restart replay,
  feedback-context, and boundedness projections derive from retained typed
  carriers rather than terminal JSON or digest-only storage.
- The admission-boundary certification matrix is typed evidence first for
  projected/aspect family admission and authority separation. The focused typed
  proof asserts family evidence and the admission/authority proofs derive
  contract, effect-intent, patch-basis, idempotence, replay, and authority
  commit projections from retained typed carriers.
- The mapper-parity certification matrix is typed evidence first for projected
  and aspect writeback families: the focused proof walks retained effect and
  replay-bundle carriers directly and verifies digest/canonical-basis accessors
  are projections rather than stored authority.
- The replay-loop isolation certification matrix is typed evidence first across
  every replay/isolation row: the focused proof walks retained family,
  cross-family replay, same-family rebuild, changed-causality, feedback, and
  loop-prevention carriers directly and verifies terminal digest/equality
  accessors are derived projections.
- The writeback execution/provenance boundary is typed evidence first: execution
  records, mutation provenance bundles, batch mutation authority counts, and
  admitted execution receipts retain typed request/receipt carriers. Digest
  accessors remain only terminal diagnostic/report projections derived from
  those carriers.
- Writeback diagnostic explanations are typed-retention wrappers first, not
  duplicate projection caches. Admission, candidate, outcome, execution, mapper
  record, mapper envelope, mapped family input, replay bundle, and replay record
  explanations retain the native artifact they explain and derive digest,
  canonical-basis, class, and counter accessors from that retained carrier.
  Direct diagnostics tests assert explanation-held artifacts match the runtime
  retained records/bundles before checking derived projections.

### Phase 8: Facade And Harness Detox

Choke point:

- `facade/exports_core.rs`
- `facade/runtime/*`
- `harness/tests/*`
- `harness/adapter/*`

New production shape:

- Public facade constructors expose native aspect contracts, masks, targets,
  values, patch carriers, source contracts, and writeback effect carriers.
- Raw external constructors are isolated under explicitly named ingress/export
  modules and are not re-exported as facade authority conveniences.
- Harness scenarios build native bridge inputs first and only project to JSON
  for report rendering.

Deletion/refactor:

- Remove public APIs that allow callers to encode patch surfaces, source
  results, writeback effects, or diagnostic truth as strings, bytes, JSON, or
  arbitrary digest strings.
- Remove tests whose only assertion is JSON shape rather than typed artifact
  meaning.

Acceptance evidence:

- Public exports make the native path the easy path.
- External I/O paths are visibly non-authoritative.
- Harness lanes prove native/source/routing/writeback parity after restart and
  diagnostics perturbation.

Current closeout evidence:

- Harness target selection is typed before execution. `BridgeHarnessAdapter`
  uses `BridgeHarnessTargetId` as its target identity, and `RunRecord`,
  `ReplayRecord`, and `SnapshotRecord` retain that typed target instead of a
  raw string selector.
- Route, history, source, merge, and structural harness target constructors
  require native identity carriers: `TruthCommitIdentity`, `TruthBranchIdentity`,
  `SourceDeclarationIdentity`, `MergeHistoryDeclarationIdentity`, and
  `StructuralIdentityDeclarationIdentity`. Tests no longer pass string literals
  into those constructors.
- Adapter-local target enums preserve the same typed declaration identities, so
  execution dispatch does not reparse or reinterpret terminal target text.
- Terminal target strings are isolated to
  `harness/adapter/target_id/terminal_projection.rs` for display/report naming.
  The sibling `constructors.rs` owns only typed constructor APIs, and `mod.rs`
  owns the native target enum.
- Writeback certification current tests no longer call `as_external_target()` or
  preserve `external_target` vocabulary. `writeback_certification.rs` builds
  `ExecutionRequest` values with typed `BridgeHarnessTargetId` targets and
  derives string material only through a local terminal-label packet used for
  fixture/request names.
- Merge facade, harness, adapter certification, and causal-retention fixture
  helpers no longer accept raw merge declaration IDs and mint declaration
  identity/authority artifact pairs internally. Callers now provide
  `MergeHistoryDeclarationIdentity` at the declaration boundary, and helper
  code derives authority artifact basis text only from that typed identity.
- Merge and structural harness certification bundles no longer retain
  declaration, contract, or branch/merge record authority as `String`
  projections. Merge certification now stores native declaration, contract, and
  record identities in the bundle/report layer and projects `.as_str()` only in
  terminal JSON export. Structural summary, identity-separation, and diff
  reports now retain native declaration, contract, and branch-comparison record
  identities directly; only retained candidate identities remain string-based in
  this seam because production structural reduction still exposes them as
  retained `Arc<str>` candidates rather than typed identity carriers.
- Policy harness certification matrices no longer retain declaration identity,
  lowered-policy identity, or truth-view resolution outcome as strings. Policy
  admitted/rejection rows now store native `BridgePolicyDeclarationIdentity`,
  route rows store the native lowered-policy identity carrier, and request
  policy matrices retain the full `BridgeTruthViewPolicyResolution` result for
  branch-local and historical checks. Terminal JSON export is now the only seam
  that renders those policy carriers to `"Admitted"` / `"Rejected"` or identity
  text. Typed certification tests prove exact declaration identities on admitted
  and rejection rows plus real lowered-policy identity retention on route rows.
- Pricing workload certification support no longer retains real runtime
  identity authority as `String` in its ordinary replay/failure/speculation
  evidence rows. `PricingReferenceBundle`, `PricingAspectBundle`,
  `PricingFailureBundle`, `PricingReplayBundle`, `PricingDiscardBundle`,
  `PricingPromotionBundle`, `PricingFanoutBundle`, `PricingRestartReplayBundle`,
  and `PricingCommitAttribution` now retain native
  `Truth{Branch,Commit,Snapshot}Identity`, `BridgeRouteIdentity`,
  `BridgeInvalidationIdentity`, `BridgeTruthViewSelectorIdentity`,
  `BridgeHistoricalEvaluationRecordIdentity`, `BridgeAspectRegistrationId`, and
  `BridgePreviewSessionIdentity` carriers through pricing support, pricing
  domain attribution, pricing-shock capture helpers, and direct proof lanes.
  Pricing terminal/export/report surfaces now render `.as_str()` only at the
  certification/showcase JSON and markdown boundaries. The one deliberate
  string-shaped holdout in this pricing seam is simulation-only branch labels
  from `simulation_capture.rs`, because those are synthetic scenario labels, not
  retained runtime authority carriers.
- Pricing merge and historical-provenance support no longer retain real runtime
  authority as `String` in their remaining merge/provenance rows.
  `PricingMergeBundle` now retains native `TruthSnapshotIdentity` and
  `BridgeAspectRegistrationId`; `PricingHistoricalProvenanceBundle` now retains
  native `TruthCommitIdentity` and `TruthSnapshotIdentity`. The coupled pricing
  certification evidence, pricing support terminal export, pricing showcase
  lineage/markdown/JSON/ML export helpers, and pricing-shock proof suites now
  project those carriers to strings only at terminal report and lineage-node
  boundaries. QA found and fixed one first-pass gap: the typed shock-commit
  comparison in `pricing_support/certification_evidence.rs` needed an explicit
  `TruthCommitIdentity` import to keep the proof row compilable.
- Pricing workload classification lanes no longer flatten real runtime enums
  into `String` before retention. `PricingAspectBundle` now stores native
  `TruthDeltaSurfaceKind`, `FineGrainedMatchStatus`, and
  `SubscriptionSliceKind`; `PricingWritebackBundle` now stores native
  `BridgeWritebackFamilyKind` and `BridgeWritebackStrategyClass`;
  `PricingMergeBundle` now stores native `BridgeMergeConsumptionClass`,
  `BridgeMergeRoutingOutcomeClass`, optional `BridgeMergePrecedenceStage`,
  optional `BridgeMergeDenialClass`, and native merged-route
  `FineGrainedMatchStatus`; `PricingTrustAttackBundle` now stores native
  `BridgePolicyRejectionKind`, `BridgePolicyFieldKind`,
  `BridgeRouteErrorKind`, `BridgeMergePrecedenceStage`, and
  `BridgeMergeDenialClass`. The coupled digest-basis builders, certification
  evidence, showcase lineage edges, suite/showcase/ML JSON projection, and
  pricing-shock proof suites now render those classifications only at terminal
  presentation boundaries. QA found and fixed one real issue in the row:
  `pricing_shock/support/capture_failures.rs` needed explicit imports for the
  new public enum carriers after the hard break.
- Policy, speculation, and stream adapter typed-certification fixtures no
  longer accept raw commit, patch, snapshot, or field identity text at their
  committed-patch/snapshot helper boundaries. The helpers require
  `TruthCommitIdentity`, `TruthPatchIdentity`, `TruthSnapshotIdentity`, and, for
  policy field targeting, foundational `FieldKey`; current tests construct
  native identity carriers explicitly before fixture admission.
- Shared harness and pricing-shock snapshot fixture helpers no longer accept
  raw snapshot identity text and mint `TruthSnapshotIdentity` internally.
  `snapshot`, `field_slice_snapshot`, `pricing_snapshot`, and
  `pricing_aspect_snapshot` require `TruthSnapshotIdentity` at the fixture
  boundary, so current harness tests construct native snapshot identities before
  source fixture admission.
- Pricing-domain snapshot export now follows the same rule. `PricingDomainWorld`
  snapshot fixture helpers require `TruthSnapshotIdentity` at the export
  boundary, and pricing-domain plus pricing-shock scenario callers construct the
  native snapshot identity before source fixture admission.
- Pricing-shock committed-patch fixture helpers no longer accept branch, commit,
  patch, or snapshot identity text and mint patch envelope identities
  internally. `pricing_patch` and `pricing_patch_items` require a
  `BridgeCommittedPatchEnvelopeIdentity`, and current callers construct that
  envelope from `TruthBranchIdentity`, `TruthCommitIdentity`,
  `TruthPatchIdentity`, and `TruthSnapshotIdentity` before patch fixture
  admission.
- Pricing-shock provenance snapshot identity rewrite helpers no longer accept
  raw snapshot identity text. `snapshot_with_identity` requires
  `TruthSnapshotIdentity`, and hostile conflicting-snapshot callers construct
  the native identity before rewriting fixture and read-result identity.
- Writeback adapter feedback patch fixtures no longer accept branch, commit,
  patch, or snapshot identity text and mint feedback committed-patch envelopes
  internally. `bridge_feedback_patch` requires a
  `BridgeCommittedPatchEnvelopeIdentity`, and the feedback-loop certification
  caller constructs that identity from `TruthCommitIdentity`,
  `TruthPatchIdentity`, `TruthSnapshotIdentity`, and `TruthBranchIdentity` before
  feedback metadata is attached.
- Adapter speculation shared helpers no longer accept preview declaration,
  binding, truth-branch, signal-branch, snapshot, or authoritative commit
  identity text. Discard, promotion, and churn certification callers construct
  `BridgePreviewSessionDeclarationIdentity`,
  `BridgeSpeculativeBranchBindingIdentity`, `TruthBranchIdentity`,
  `BridgeSignalBranchIdentity`, `TruthSnapshotIdentity`, and
  `TruthCommitIdentity` before entering the shared helper boundary.
- Speculation churn certification no longer retains lifecycle proof as replay
  digest strings. `SpeculationChurnCertification` retains
  `BridgePreviewReplayBundle` values through `SpeculationPreviewReplayBundleSet`,
  and branch-isolation rows retain typed `BridgePreviewSessionIdentity`,
  `TruthBranchIdentity`, `PreviewExecutionRecordIdentity`, and
  `BridgePreviewDiscardRecordIdentity` carriers. Terminal lifecycle/replay
  digest strings are derived only from retained replay bundles during harness
  report projection.
- Facade speculation and harness counter preview-basis helpers no longer accept
  raw suffix text and mint preview truth-view or structural identities
  internally. Current tests assemble typed `TruthBranchIdentity`,
  `TruthSnapshotIdentity`, `StructuralSchemaIdentity`, and
  `StructuralIdentityDeclarationIdentity` packets at the scenario boundary
  before calling `preview_session_basis` or `structural_basis`; the helper
  boundary now only consumes native carriers and derives canonical basis from
  them.
- Field-target expected canonical-basis helpers no longer accept raw field
  text. Harness explanation/routing tests plus input-envelope and routing
  surface tests construct foundational `FieldKey` values at the scenario
  boundary, use the same typed key for committed-patch target construction, and
  pass that `FieldKey` into expected target-basis assertions. Current tests no
  longer preserve a helper that teaches canonical committed-patch field targets
  as caller-supplied strings.
- Harness mapping-registration helpers no longer accept raw signal-scope text
  and mint `SignalInvalidationScope` internally. Bulk-planning identity tests,
  fine-grained routing tests, and pricing-shock runtime support now construct
  `SignalInvalidationScope` explicitly at the scenario/runtime-builder boundary
  before mapping registration helper admission. The helper boundary only
  consumes the native invalidation-scope carrier.
- Continuity certification/parity authority fixtures no longer accept branch or
  snapshot identity text and mint continuity authority bases internally. Shared
  certification helpers and the parity-local fixture require
  `TruthBranchIdentity` and `TruthSnapshotIdentity` at the authority boundary,
  so current tests construct native continuity identity carriers before
  registering lineage authority.
- Facade stream committed-patch fixture support no longer accepts branch,
  commit, patch, or snapshot identity text and mints patch envelope identities
  internally. `canonical_envelope` requires `TruthBranchIdentity`,
  `TruthCommitIdentity`, `TruthPatchIdentity`, and `TruthSnapshotIdentity`, and
  stream plus causal-retained mapping callers construct those native identities
  before stream window admission.
- Writeback adapter route and authority-denial causality helpers no longer
  accept raw commit identity text. `route_digest_for_commit` and
  `authority_denial_causality` require `TruthCommitIdentity`, and feedback-loop
  plus authority-denial certification callers construct the typed identity
  before route planning or causality evidence derivation.
- Writeback authority-denial certification no longer selects harness failure
  digest domains from caller-supplied text. The shared helper now accepts a
  closed private `WritebackHarnessErrorDigestDomain` enum, and the authority,
  merge-authority, unsafe-feedback, and contradictory-feedback denial rows use
  those finite variants rather than raw digest-domain strings.
- Writeback-certification feedback-loop, authority-denial, and mapper-parity
  rows no longer retain classification meaning as strings before terminal
  export. `FeedbackReplayBundleReport` and `FeedbackBoundednessProof` now keep
  native `BridgeWritebackStrategyClass`, `BridgeWritebackRetryDisposition`,
  `BridgeWritebackOutcomeClass`, and `BridgeWritebackErrorKind`;
  authority-denial rows keep native `BridgeWritebackErrorKind` plus a closed
  `AuthorityDenialBoundaryClass`; mapper-parity shadow-protocol rejection keeps
  native `BridgeWritebackErrorKind`. Terminal JSON projection is now the only
  seam that renders those carriers to text. QA also split
  `feedback_loop/terminal_projection_access.rs` by moving interleaved-truth
  accessors into
  `feedback_loop/terminal_projection_access/truth_interleaving_projection_access.rs`
  so all row-touched files stayed under the workspace line cap.
- Duplicate-authority writeback certification no longer retains truth-trigger,
  route, or causality meaning as stored strings. The matrix now keeps the
  native `BridgeWritebackCausalityBasis` and derives
  `causality_digest()`, `truth_trigger_digest()`, and `route_digest()` from
  that retained carrier. The duplicate certification executor also stopped
  flattening the first committed patch identity into a `String`; it now keeps a
  typed `TruthCommitIdentity`, derives the typed route identity first, and
  builds causality from those native carriers before lowering the effect.
- Replay-loop isolation no longer stringifies `BridgeWritebackLoopDisposition`
  inside certification accessors. Both `ReplayLoopFeedbackIsolation` and
  `FamilyExtensionLoopIsolation` now return the native
  `BridgeWritebackLoopDisposition`; only the replay-loop and family-extension
  terminal JSON projection helpers render that disposition to text.
- Replay-loop and family-extension replay-isolation rows no longer synthesize
  failure or decision-trace digests inside certification modules. The
  `cross_family_replay`, `same_family_equivalence`, and `changed_causality`
  artifacts under `replay_loop_isolation` and
  `family_extension/replay_isolation` now retain native error, replay-record,
  projected-bundle, changed-bundle, rebuilt-bundle, and execution-record
  carriers only; terminal JSON projection derives those failure/trace digests
  at the export boundary. The proof pass caught and removed one honest residue:
  replay-loop certification-evidence export still called the deleted
  `failure_digest()` method until that export seam was rewritten to use the new
  terminal projection helper.
- Family-extension mapper-parity and shadow-protocol rows no longer retain
  admission-record digest strings or synthesize mapper/shadow trace digests
  inside certification modules. `WritebackFamilyExtensionMatrixEvidence` now
  carries native `BridgeWritebackFamilyAdmissionRecord` values into
  `family_extension/{mod.rs,mapper_and_shadow.rs}`, the family evidence row
  retains the native admission record instead of a copied digest string, and
  mapper/shadow proof rows now retain native admission records, execution
  records, and `BridgeWritebackError` only. Terminal JSON projection in
  `terminal_report_export/family_extension_json_projection.rs` is now the only
  seam that derives the mapper-parity decision trace digest and
  shadow-protocol failure/decision digests. The proof pass caught one honest
  residue in top-level family-extension certification-evidence export still
  calling the deleted `failure_digest()` method until that export seam was
  redirected through the new terminal helper.
- Causal-envelope evidence references no longer accept raw reference identity
  text at the public/current-test boundary. `BridgeCausalEvidenceReference::new`
  requires `BridgeCausalEvidenceReferenceIdentity`, owner-specific constructors
  bind query, runtime-bridge, relational, and signal family before reference
  assembly, and the generic owner/family constructor is private. Runtime-bridge
  facade causal-envelope/writeback mapping tests and worth-query causal
  materialization/certification tests now construct typed reference identities
  before assembling references. The lower-runtime slot certification fixture
  split `slot_support/lower_runtime_slot_references.rs` because explicit native
  reference assembly would otherwise keep the touched slot-support file over the
  workspace line cap.
- Causal-envelope retained-mapping tests no longer rebuild retained runtime
  evidence by pairing family enums with record-identity strings at each
  assertion site. The retained-mapping support seam now owns the single sealed
  `runtime_bridge(family, identity)` assembly point, exposes typed helpers for
  route, bulk-planning, source, structural, continuity, stream, historical
  failure, and merge retained artifacts, and keeps missing-record denial rows
  as the only explicit family-plus-identity callsites. The digest-basis mirror
  moved into `retained_mapping_digest_support.rs`, returning the touched
  retained-mapping support topology to responsibility-sized files under the
  workspace line cap.
- The remaining causal-envelope preview and writeback mapping suites now follow
  the same rule. Shared causal-envelope helpers own the only sealed
  `runtime_bridge(family, identity)` assembly point for route, preview
  execution/discard/promotion, and writeback admission/mapper/envelope/mapped
  input/execution/replay retained artifacts; current tests no longer rebuild
  those references directly from family enums plus identity text at each
  callsite. The writeback scale harness also stopped preserving the retained
  chain as six raw identity strings and now retains the native admission,
  mapper-envelope, mapped-input, mapper-record, execution, and replay
  artifacts themselves before terminal projection. Remaining direct
  `runtime_bridge(...)` calls in the causal-envelope suite are limited to the
  sealed helper locations plus intentionally synthetic denial/evidence-owner
  rows that explicitly test reference validation behavior.
- worth-query batch mutation inspection no longer asks runtime-bridge for
  request/receipt digest counts. Query consumes
  `BridgeBatchMutationAuthorityBundle::authority_request_count` and
  `authority_receipt_count`, and its batch mutation evidence exposes those
  values as authority carrier counts. Digest fields remain only aggregate
  projections over native target, causality, provenance, naming, continuity,
  symbolic-reference, and existing-truth evidence.
- The row-touched over-cap harness files were split only where the typed
  snapshot fixture rewrite earned topology: history assertion helpers,
  diagnostics continuity source support, and sink-rejection slice diagnostics
  now live in responsibility-specific test modules rather than preserving broad
  scenario files over the line cap.
- Focused residue scans show no string-literal calls to route/history/source/
  merge/structural target constructors and no `String` harness execution target
  aliases. The `as_external_target` projection remains only in
  `harness/adapter/target_id/terminal_projection.rs`. Remaining `target` string
  fields are pricing presentation/support evidence, not execution authority.

## Forbidden Hybrids

These are not acceptable intermediate end states:

- native `AspectValue` produced by production JSON decoding of snapshot bytes
- `AspectKey` plus string `surface_label` treated as a native target
- route artifacts that carry masks but still match on labels
- writeback effects represented by digest strings plus diagnostics prose
- typed diagnostics that internally store arbitrary JSON maps as semantic data
- facade constructors that silently accept both native carriers and legacy
  strings for the same authority field
- foreign-input lowering modules imported by production source, routing, or writeback
  authority paths
- "native" constructors that accept old byte/label/digest inputs and lower
  them internally for caller convenience
- test support helpers that still manufacture authority from JSON, raw bytes,
  surface labels, or arbitrary digests outside explicitly named external I/O
  tests
- diagnostic or certification bundles whose field names preserve old authority
  concepts even after the stored values are native-derived
- public facade aliases that preserve legacy API shape for caller continuity

Temporary test adapters may use foundational external-lowering helpers only
when the test name and module path make the foreign I/O boundary explicit and
the test asserts that lowering terminates before native authority. They may not
be general harness conveniences.

## Must Ship

- native snapshot read request and result carriers
- aspect contract and mask validation before source materialization
- native committed patch target items
- mask-native route planning and slice lowering
- grouped truth projection over native values and field paths
- typed writeback intent/effect carriers
- typed diagnostics/counters/certification bundles with terminal JSON export
- facade APIs that make native carriers the default production surface
- hostile certification proving JSON, raw bytes, label strings, and digest-only
  payloads cannot become authority
- a final residue audit that proves production code and current tests no longer
  expose migration-era authority vocabulary except external I/O boundaries

## Must Preserve

- relational truth remains authoritative for state, patches, validation, and
  commit admission
- signal remains authoritative for dependency ownership and execution
- bridge remains a protocol boundary and proof-transfer runtime
- replay, diagnostics-tier invariance, and certification bundle sufficiency
- existing route/source/writeback phase separation
- bounded breadth through projection, mutation, and diagnostic masks
- historical migration explanation only in docs, never as a production
  caller-continuity contract

## Acceptance Evidence

The refactor is complete only when the bridge harness and code audit can prove:

- `serde_json` is absent from production authority paths except explicit
  external ingress/export modules.
- `aspect_bytes` is absent from the production snapshot read contract.
- `surface_label` is absent from production committed-patch and routing
  authority; any remaining label is explanatory or external-I/O-only.
- route targets are aspect locator, field locator/path, and mask based.
- source materialization consumes native aspect values admitted by contract.
- grouped truth and query-facing projections preserve native aspect carriers.
- writeback candidates cannot be admitted from digest-only payloads.
- diagnostics and counters are typed artifacts before JSON export.
- replay after restart preserves native source, route, writeback, failure, and
  certification identities.
- external I/O tests prove unsupported JSON/raw/string/digest shortcuts fail
  closed.
- no production or current-test authority helper preserves legacy names or
  fallback behavior outside explicit external I/O tests.
- deleting terminal JSON export code would not change native source, routing,
  writeback, replay, diagnostics, or certification meaning.

## Batch Plan Template

Every implementation turn against this spec must start by reading the required
docs and then writing a concrete batch plan using this shape:

```text
Batch target:
- Native authority seam being replaced:
- How far back this batch must move to delete upstream legacy:
- Files expected to delete or rewrite:
- New directory skeleton for this slice:
- Aspect-native carriers introduced:
- Legacy carriers removed:
- Public/test/harness legacy or foreign-format shims deleted:
- Residue scan terms this batch must clear:
- Certification or compile checks to run:
- Tests intentionally skipped because the change is trivial/docs-only:
```

Do not split a subsystem unless the split removes or quarantines one of the
forbidden authority carriers.

Do not declare a batch complete if any touched production path still exposes a
legacy-shaped fallback or foreign-format alias. If deleting that alias breaks
callers, update the callers in the same batch.

## Verification Policy

- For docs-only edits, run a lightweight diff or formatting sanity check.
- For source changes that alter production bridge semantics, run focused
  `worth-runtime-bridge` checks with a 10 minute timeout per command.
- When a source change touches shared query or relational bridge surfaces, also
  run the relevant focused `worth-query` and `worth-relational` checks with a
  10 minute timeout per command.
- Do not run broad workspace tests after trivial edits.

## Sequencing Notes

The highest-value implementation order is:

1. Crate-wide legacy authority eradication, because any remaining fallback,
   shim, alias, or legacy-named helper can keep teaching the old model after
   native carriers exist.
2. Native snapshot read contract, because it removes the most direct JSON/raw
   byte authority path.
3. Native committed patch targets, because routing cannot be fully
   aspect-native while patch items encode surfaces as strings.
4. Mask-native routing and subscriptions, because it consumes the new target
   carriers and removes label matching.
5. Grouped truth projection, because source reads and masks will now have the
   right value basis.
6. Writeback intent, because digest-only effects are the authority-boundary
   equivalent of raw snapshot bytes.
7. Diagnostics/certification detox, because typed production evidence should
   exist before terminal JSON export is cleaned.
8. Facade/harness detox, because public construction should be tightened after
   the native internal carriers are real.

This order is not permission to defer discovered legacy. If Phase 6 work finds
Phase 2 residue, the next batch moves back to Phase 2 and deletes it. The
sequence is a dependency map, not a broom closet for known old paths.

## Self-Check

This spec is not a cleanup wishlist. It is an authority-boundary replacement
plan.

The work is complete only when the bridge cannot accidentally recover truth
from JSON, bytes, labels, or arbitrary digests in production. The bridge should
instead consume and publish native aspect contracts, masks, authoritative
state/patch carriers, typed effects, typed failures, and typed certification
artifacts. Anything else is presentation, external I/O, or test scaffolding.

The stricter closeout check is: a new engineer should not be able to infer from
current production code, current facade APIs, or current certification fixtures
that the crate ever used JSON, raw bytes, surface labels, or digest-only
payloads as authority. If they can, the migration is not done.
