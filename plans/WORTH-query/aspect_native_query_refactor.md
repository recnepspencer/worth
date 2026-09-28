# WORTH Query Aspect-Native Query Refactor

> Status: Active implementation spec
>
> Roadmap parent: [worth_query_roadmap.md](worth_query_roadmap.md)
>
> Vision parent: [worth_query_vision.md](worth_query_vision.md)
>
> Certification companion: [test-requirements.md](test-requirements.md)
>
> Foundational authority companion:
> [aspect-contracts-values-and-authoritative-state](../../crates/worth-foundational/docs/aspect-contracts-values-and-authoritative-state/README.md)
>
> Primary architectural driver: hard-break `worth-query` internals onto
> `worth-foundational` contracts, locators, field paths, values, masks,
> authoritative state, and authoritative patch carriers while preserving
> Query's ergonomic authoring role.

## Purpose

Define the actual end-state for making `worth-query` aspect-native in the same foundational sense that now governs `worth-runtime-bridge` and `worth-relational`, while preserving Query's role as the daily-driver authoring and read surface.

This spec is the reference document for starting the Query aspect-native
rewrite. It is intentionally stricter than a migration note. Production code,
public facades, certification fixtures, retained diagnostics, and harness
helpers must finish as if Query was designed around foundational aspect
contracts from the beginning.

Hard-break rules:

- no production mutation authority carrier whose semantic value is
  `serde_json::Value`
- no production mutation authority carrier whose aspect identity is a
  free-form dotted `String`
- no backend existing-truth probe contract that returns `(String,
  serde_json::Value)` as the authority basis
- no read, materialization, retained-row, or projection-consumption path that
  must project native aspect values into JSON before core fact extraction
- no effect delivery or workflow substrate that treats JSON desired-aspect
  state as the proposed mutation effect
- no metadata, diagnostics, certification, replay, parity, or inspection row
  whose JSON projection is the retained semantic source
- no public or test helper vocabulary that teaches callers to hand Query raw
  JSON values, dotted aspect paths, arbitrary projection objects, or string
  field bags for authority-bearing work
- no fallback, compatibility, or coexistence branch where a native carrier is
  required
- no phase closeout that relies on text search as the primary proof that a
  legacy authority path is unreachable

External JSON may exist only in an explicitly named external ingress or
terminal projection boundary. That boundary must immediately lower into native
carriers or derive from native carriers, and native mutation, read,
materialization, effect, certification, replay, and inspection code must not
import it as an authority helper.

## Why this doc exists

The current public Query mutation facade is already safer than the old payload-first surface. `workspace.insert(...)`, `workspace.update(...)`, `workspace.delete(...)`, batch builders, and preview-facing authoring all teach aspect-oriented authoring instead of raw write commands. That work was the right stabilization move, but it did not finish the deeper migration.

Today, many core Query internals still treat authority, mutation lowering, materialization, and projection consumption as string-path plus `serde_json::Value` problems:

- `crates/worth-query/src/runtime/surface/mutation/command.rs` still exposes `WORTHQueryWriteCommand::UpdateAspect { aspect_path: String, value: serde_json::Value }`.
- `crates/worth-query/src/runtime/mutation/aspect.rs` still centers `WORTHQueryAspectValue` on string paths and JSON lowering.
- `crates/worth-query/src/runtime/workspace_queries.rs` still returns `Vec<serde_json::Value>` for derived materialization.
- `crates/worth-query/src/runtime/computed/surface.rs` still stores computed rows and patch payloads as `serde_json::Value`.
- `crates/worth-query/src/projection_consumption/extraction/row_materialization/row_like.rs` now feeds native aspect values into consumed field facts, but broader materialization rows still retain JSON compatibility surfaces.
- `crates/worth-query/src/memory_workspace/mod.rs` still stores external row projections as `serde_json::Value` beside aspect maps and exposes dotted-path JSON lookup.

So the open question is no longer "does Query mention aspects?" The open question is "which concrete subfolders and files still treat JSON or path strings as authority carriers, and what is the correct foundational target for each migration zone?" This doc answers that question by zone.

## Governing summaries

`MENTALITY.md`

Solve the hard substrate mismatch instead of polishing the comfortable facade. The migration must remove authority leakage and false-native seams even when the public API already feels nicer.

`arch_laws.md`

Authority must travel through explicit proof-bearing types. Construction, validation, lowering, and mutation admission cannot rely on ambient strings or loosely shaped payload carriers.

Law 41 is a primary design driver for this refactor, not background advice.
Query must encode what has been proven about a value in the type that leaves
each phase. A caller-authored aspect request, a validated aspect value, a
lowered mutation target, an admitted authoritative patch, an executed write
receipt, a retained materialization row, and a certified projection fact are
different proof states and must not share one catch-all representation.
Constructors for proof-carrying states must be sealed or private; fields must
be private; accessors must be read-only; and transition functions must consume
the prior proof type and produce the next proof type. A runtime check for a
proof state that a type could have guaranteed is a design failure in this
refactor.

`composition_laws.md`

The plan must match the code's responsibility boundaries. We should not declare "remove JSON from Query" as one monolithic task; we should rewrite each migration zone according to its real authority role.

`domain_structure_laws.md`

Folder topology should expose domain boundaries. Query is large enough that top-level folders hide important authority differences, so this spec must name subfolders and representative files.

`perf_laws.md`

The rewrite should lower once, validate once, and avoid repeated encode/decode churn between foundational values and JSON. Cheap-looking APIs must not hide expensive re-materialization or re-validation loops.

`worth_query_vision.md`

Query is supposed to be the platform's daily-driver authoring and read layer. Its read, write, preview, inspection, and live surfaces should share one coherent ontology instead of mixing aspect truth with ad hoc JSON carriers.

`worth_query_roadmap.md`

Query should declare intent once, lower once, and execute against canonical truth without stealing authority from relational, bridge, or foundational subsystems. That requires a stricter internal authority model than the current public DX freeze delivered.

`worth-foundational` JSON compatibility

Foundational is the source of aspect truth and the source of transitional JSON
compatibility. If a Query boundary receives JSON that semantically represents
aspect values, aspect state, or aspect patches, Query must route that input
through `worth_foundational::compatibility().json()`,
`JsonCompatibilityAspectInput`, `lower_json_aspect_value(...)`, or
`lower_json_record_aspect_state(...)` instead of defining a local JSON lowering
lane. Query-owned `serde_json` may only parse or print explicitly named
terminal documents whose semantic authority has already been moved into native
Query/foundational carriers. Native Query authoring must not be routed through
the Foundational JSON compatibility bridge merely for convenience.

`test-requirements.md`

This migration is not done when the happy-path facade compiles. It is done when certification, replay, parity, and adversarial harnesses all prove that foundational-native carriers are the actual substrate.

`aspect-api-finalization-plan.md` and `aspect-api-finalization-closeout.md`

These docs intentionally froze the public mutation API before the deeper substrate rewrite. They are prerequisites, not proof of completion.

## Arch Law 41 proof ladder

The Query migration must use Law 41 as the main pipeline architecture. The
goal is not merely to replace JSON with native values. The goal is to make
skipped validation, out-of-order lowering, weaker proof promotion, and
post-hoc authority reconstruction unrepresentable.

### Mutation proof states

The mutation spine should move through distinct, sealed proof-bearing states:

1. caller authoring input: ergonomic, may include string sugar, not authority;
2. parsed native target: aspect keys, locators, field paths, and raw desired
   value intent, but not yet validated;
3. validated desired aspect: value admitted against its contract and mask;
4. lowered mutation intent: desired aspect is attached to native target,
   naming, continuity, symbolic-reference, and metadata proof;
5. backend-admissible mutation request: existing-truth assertions and mutation
   masks have been checked and cannot be skipped;
6. authoritative patch/state request: foundational patch/state carriers exist
   and lower-authority authoring input is no longer retained as authority;
7. executed write receipt: execution proof, touched graph evidence, and native
   delta evidence are sealed behind receipt constructors.

Downstream code must accept the narrowest proof state it needs. No function
that requires a validated or admitted mutation may accept authoring input,
path strings, JSON values, or a weaker proof wrapper.

### Read and projection proof states

The read side should have its own proof ladder:

1. read authoring request: ergonomic selector or query expression;
2. validated projection request: contract-backed projection mask and native
   target set;
3. backend materialization request: bridge/relational backend can execute
   without recovering truth from JSON rows;
4. retained native row: native values and identities are retained as semantic
   truth;
5. consumed fact set: extraction has proven grouping/member/value facts from
   native row carriers;
6. certified projection artifact: oracle, replay, parity, and hostile checks
   consume native proof states, with JSON only as terminal export.

### Sealing requirements

Every proof-carrying Query type introduced by this refactor must follow these
rules:

- private fields;
- constructor visibility limited to the proving module or transition function;
- no `Default`, broad `From`, or public struct literal construction for proof
  states;
- read-only accessors that do not leak mutable internals or lower-authority
  reconstruction inputs;
- compile-fail coverage for public construction, weaker-proof promotion, and
  skipped-stage calls when the type is part of a public or cross-module
  authority boundary.

### Law 41 closeout test

For each migrated zone, ask: "Can a caller with a weaker value construct the
stronger proof type, skip one transition, or satisfy an admitted/executed API
with an earlier proof state?" If yes, that zone is not migrated even if it no
longer mentions JSON.

## Mechanical enforcement over search

Residue scans are useful discovery tools, not proof. This refactor must prefer
mechanical enforcement wherever Rust can carry the invariant.

The closeout order for every migrated slice is:

1. make invalid construction unrepresentable with private fields, sealed
   traits/tokens, phase-typed wrappers, and narrow constructors;
2. make invalid transition order uncallable by having each transition consume
   the prior proof state and produce the next proof state;
3. make lower-authority promotion fail at compile time with compile-fail tests
   or public facade contract tests;
4. make module topology enforce boundary visibility with private modules,
   `pub(in ...)`, crate-private constructors, and facade-only exports;
5. use residue scans only after those fences exist, to find names or helpers
   that still teach the old model.

Search output may never be the only evidence for a production authority claim.
A clean scan for `serde_json::Value`, `aspect_path`, or `WORTHQueryAspectValue`
does not prove the migration if a weaker value can still satisfy a stronger
API through `From`, `Default`, public fields, public constructors, trait
implementation escape hatches, or broad enum variants.

Mechanical evidence the implementation should prefer:

- compile-fail tests that attempt public struct literals for proof states;
- compile-fail tests that attempt weaker-proof promotion into admitted,
  retained, executed, or certified APIs;
- compile-fail tests that attempt to implement sealing traits or construct
  sealing tokens outside the proving module;
- type signatures where admitted/executed/certified functions cannot accept
  authoring input, raw backend rows, terminal JSON projections, or earlier
  proof wrappers;
- source-derived certification inventories instead of hand-maintained allowlist
  rows when an audit must prove exported surfaces;
- facade export tests that prove only final native/proof-bearing surfaces are
  publicly reachable.

Search can still support the work, but only as a follow-up diagnostic:

- find candidate legacy seams before choosing a slice;
- verify old names were deleted from touched files;
- catch accidental public vocabulary drift after mechanical tests already
  enforce the invariant.

## Adversarial constraint

Assume the current Query API is only "aspect-shaped" unless a surface can prove that:

1. authority enters through foundational contracts, locators, validated values, and authoritative patches;
2. internal mutation and materialization steps keep that truth in foundational carriers instead of re-expanding it into JSON;
3. production Query contains no JSON authority or projection shim except for a small, explicitly named allowlist that is justified by a hard external contract.

If a migration zone fails any of those three tests, it is not done.

The refactor must survive this hostile condition:

> A caller with only Query's public ergonomic API, bridge-backed execution,
> memory-workspace execution, preview/lifecycle paths, projection consumption,
> retained materialization, workflow capabilities, certification fixtures, and
> hostile tests must be unable to manufacture authority from JSON rows, dotted
> strings, arbitrary field bags, or projection objects. The same scenario must
> lower once into native foundational carriers, execute through those carriers,
> retain native evidence, and export JSON only after all authority decisions
> have already happened.

If any supported production path:

- accepts `WORTHQueryAspectValue` as path-plus-JSON authority;
- derives an authoritative mutation target from a dotted string after command
  construction;
- verifies existing truth from `(String, serde_json::Value)` pairs;
- materializes core derived facts from `Vec<serde_json::Value>`;
- computes fact digests from JSON instead of native canonical value encodings;
- delivers runtime effects as JSON payloads after a native value should exist;
- stores workflow desired-aspect state as external JSON;
- requires a test harness JSON helper to explain production authority;

then the refactor is not complete.

## What "foundational-native" means for WORTH Query

WORTH Query does not need to become a thin alias over raw `worth-foundational`, but its authority-bearing core must.

For this refactor, "foundational-native" means:

- aspect identity is represented by foundational keys, locators, field locators, or canonical field paths, not by free-form dotted strings as the source of truth;
- aspect values are represented by `AspectValue`, `StructAspectValue`, validated aspect artifacts, or authoritative state/patch artifacts, not by `serde_json::Value`;
- mutation intent lowers into authoritative patch/state vocabulary before runtime execution;
- inspection, preview, and read materialization expose clearly separated native truth surfaces versus external projections;
- JSON is banned from production Query by default. Any exception must be named, justified, and treated as an external contract rather than a general compatibility lane.

### Eradication standard

This refactor is not complete when native alternatives exist. It is complete
only when the old authority shape cannot be selected, reconstructed, or taught
by current production code and current tests.

Every implementation batch must therefore delete or quarantine touched legacy
evidence:

- delete old constructors rather than deprecating them;
- delete public exports that preserve path-plus-JSON command shape;
- delete or rewrite tests that assert JSON authority as the normal path;
- delete production helper modules that project native values into JSON for
  internal reuse;
- delete diagnostic, receipt, and certification fields that imply JSON or
  dotted-string authority even if their values are now derived from native
  carriers;
- delete fallback branches that recover from native rejection by widening to
  JSON, dotted strings, projection maps, or arbitrary digests.

The only allowed historical evidence is in roadmap, spec, closeout, migration,
or audit documents. Current production code and current tests must describe the
final native architecture.

## JSON ban and exception policy

The migration target is not "JSON at the edges." The migration target is "no production JSON in Query unless an exception is explicitly approved."

Former exception candidates that have now been quarantined to tests:

- `crates/worth-query/src/aspect_field_authoring/external_json_ingress.rs`
- `crates/worth-query/src/aspect_field_authoring/external_json_projection.rs`

Those files may survive only if Query truly owns an external JSON contract that cannot move elsewhere. If they are only legacy convenience shims, they should be deleted or moved out of production Query. Tests may temporarily retain JSON fixtures because the crate is large, but those tests must be marked as legacy coverage or compatibility-debt tests instead of teaching JSON as the normal authoring model.

### Production JSON allowlist contract

Any production `serde_json::Value` usage that survives a phase must be listed
in this document before the phase closes. Each row must name:

- owner module;
- external contract that requires JSON;
- why the JSON value cannot move into a terminal export or ingress module yet;
- native carrier that owns semantic authority before or after the JSON
  boundary;
- removal condition;
- tests proving the JSON boundary cannot participate in native authority.

An unlisted production `serde_json::Value` occurrence under
`crates/worth-query/src` is a blocker for closeout of the phase that touched
it. Test JSON is tolerated only with an explicit debt note while production
APIs are being replaced.

JSON-as-aspect-truth is not an allowlist category for Query. If an external
contract supplies JSON whose meaning is an aspect value/state/patch, the only
approved compatibility path is Foundational's JSON bridge. A Query production
file that imports `serde_json` for aspect compatibility must either disappear
or be a tiny external boundary that immediately delegates to
`worth_foundational::compatibility().json()` and returns native validated or
authoritative artifacts. The current Query codebase has no approved production
JSON-as-aspect compatibility boundary.

Current production allowlist:

| Module | Approved reason | Native authority carrier | Removal condition |
| --- | --- | --- | --- |
| `crates/worth-query/src/consumer_kit/support_snapshot/document/terminal_json_codec.rs` | External support snapshot documents are durable terminal artifacts; JSON is decoded only from `WORTHQueryExternalSupportSnapshotTerminalJsonDocument` and encoded only from validated native support snapshot documents | `WORTHQuerySupportSnapshotDocument` validates into `WORTHQuerySupportSnapshot`; native support snapshot rows own semantic authority before terminal export and after external ingress | Remove when support snapshot durability moves to a non-JSON external format or outside `worth-query`; `production_serde_json_is_confined_to_support_terminal_codecs` must be updated or fail |
| `crates/worth-query/src/consumer_kit/support_pinning/document/terminal_json_codec.rs` | External support pin contracts are durable terminal artifacts consumed from checked-in downstream support pins; JSON is decoded only from `WORTHQueryExternalSupportPinContractTerminalJsonDocument` and encoded only from sealed native support pin contracts | `WORTHQuerySupportPinContractDocument` validates into `WORTHQuerySupportPinContract`; native pin requirements and observed rows own semantic authority before terminal export and after external ingress | Remove when support pin durability moves to a non-JSON external format or outside `worth-query`; `production_serde_json_is_confined_to_support_terminal_codecs` must be updated or fail |

No other production JSON usage is pre-approved. The allowlist is mechanically
enforced by
`consumer_kit::support_snapshot::tests::runtime_boundary::production_serde_json_is_confined_to_support_terminal_codecs`.
The two surviving support terminal codecs are also mechanically barred from
becoming local aspect compatibility bridges by
`consumer_kit::support_snapshot::tests::runtime_boundary::support_terminal_json_codecs_do_not_become_aspect_compatibility_bridges`.

## Legacy authority baseline

These patterns are active blockers wherever they appear in production Query.
They are not permission to preserve them until a final cleanup pass.

### Mutation commands and aspect values

`WORTHQueryWriteCommand::UpdateAspect { aspect_path, value }`,
path-bearing delete variants, and `WORTHQueryAspectValue` are legacy
authority shapes when they carry dotted strings and JSON. The destination
shape must carry native mutation targets and `AspectValue` or authoritative
patch/state vocabulary. String authoring sugar may remain only before command
construction and must lower once into a native carrier.

Law 41 requirement: do not replace these with one broad "native command" bag.
The replacement must distinguish authored input, validated desired aspects,
lowered mutation intent, backend-admissible request, authoritative patch/state
request, and executed receipt as separate proof states.

### Existing-truth probes and backend contracts

Backend contracts that return `(String, serde_json::Value)` pairs force Query
to verify truth through a path/value bag. Existing-truth probes must return
native aspect facts, locators, field paths, values, and denial evidence.

Law 41 requirement: existing-truth assertion proof must be a sealed state that
backend execution consumes. Execution must not accept unverified assertion
inputs and then re-check them by convention.

### Derived materialization and retained rows

`Vec<serde_json::Value>` rows and retained JSON row decoding are legacy when
they feed core Query facts. Derived views need native row/value artifacts
first; JSON projection may be terminal presentation only.

Law 41 requirement: materialization must produce a retained native row proof
type, and projection consumption must require that proof rather than accepting
raw backend rows or terminal JSON projections.

### Projection consumption

Fact extraction cannot require row-like JSON translation as an internal bridge.
Consumed facts and grouping values must use native carriers and canonical
digest preparation.

Law 41 requirement: consumed facts must be sealed outputs of native extraction,
not public structs or helper-built rows that tests or downstream code can
synthesize from weaker materialization evidence.

### Effects and workflow capabilities

Effect delivery and workflow authoring cannot preserve JSON desired-aspect
state as mutation meaning. Runtime effects and workflow semantics must digest
native mutation intent.

Law 41 requirement: workflow and effect APIs must consume the correct mutation
proof state for their phase. A workflow declaration cannot promote authored
JSON or authoring input directly into delivered effect authority.

### Memory workspace

The in-memory workspace cannot teach Query's core model as a hybrid of native
aspect maps plus JSON external rows. External rows must be removed, terminal,
or explicitly allowlisted as non-authoritative I/O.

## Target directory skeleton

Names may refine during implementation, but the authority map must remain
visible. New files should land only when they replace a real authority seam,
not as decorative topology.

```text
crates/worth-query/src/
  runtime/
    mutation/
      native_intent/
        target.rs
        desired_aspect.rs
        asserted_aspect.rs
        proof_states.rs
        metadata.rs
        lowering.rs
        validation.rs
      existing_truth/
        fact.rs
        probe.rs
        assertion.rs
    surface/
      mutation/
        command.rs
      receipt/
        native_delta.rs
        evidence.rs
        executed_receipt.rs
    backend/
      existing_truth_contract.rs
      native_execution.rs
    materialization/
      native_row.rs
      proof_states.rs
      retained_row.rs
      derived_view.rs
      external_projection.rs
    effect/
      native_delivery.rs
      routing.rs
      declaration.rs
  projection_consumption/
    native_rows/
      fact.rs
      proof_states.rs
      extraction.rs
      grouping.rs
      digest.rs
    certification/
      oracle_terms.rs
  memory_workspace/
    native_entity_row.rs
    native_aspect_lookup.rs
    external_projection/
      json.rs
  external_io/
    json_ingress/
    terminal_json_projection/
```

Directory rule: files named `json`, `external`, `ingress`, or `projection`
must live under an external I/O branch or a clearly terminal projection module.
Runtime mutation, backend, materialization, projection consumption, effect, and
certification authority modules must not import those files.

## Migration zone register

This register intentionally uses subfolders and representative files. Top-level folders such as `runtime/` and `domain_capabilities/` are too broad to be actionable work units.

### Zone 1: `runtime/surface/mutation/`

Representative files:

- `crates/worth-query/src/runtime/surface/mutation/command.rs`
- `crates/worth-query/src/runtime/surface/mutation/write_receipt/`

Current state:

- `WORTHQueryWriteCommand::UpdateAspect` still carries `aspect_path: String` and `value: serde_json::Value`;
- delete variants still carry `touched_aspect_paths: Vec<String>`;
- receipt helpers reflect the same legacy command shape.

Required end state:

- expert write commands carry native mutation targets and foundational values or patches;
- string aspect paths exist only as caller-facing authoring sugar before command construction;
- write receipts expose native aspect operation evidence.

Migration class: first production blocker.

### Zone 2: `runtime/mutation/`

Representative files:

- `crates/worth-query/src/runtime/mutation/aspect.rs`
- `crates/worth-query/src/runtime/mutation/lowering.rs`
- `crates/worth-query/src/runtime/mutation/probe.rs`
- `crates/worth-query/src/runtime/mutation/metadata.rs`
- `crates/worth-query/src/runtime/mutation/assertion.rs`

Current state:

- `WORTHQueryAspectValue` is still a string-path plus JSON carrier;
- mutation lowering and assertion diagnostics serialize JSON values for evidence;
- existing-truth probes still reason in `(String, Value)` field pairs;
- metadata uses JSON as an open-ended bag.

Required end state:

- introduce native desired-aspect and asserted-aspect carriers;
- lower into foundational patch/state vocabulary once;
- represent probe results and assertion evidence in native aspect facts;
- either remove JSON metadata or replace it with typed metadata entries.

Migration class: first production blocker, paired with Zone 1.

### Zone 3: `runtime/backend/`

Representative files:

- `crates/worth-query/src/runtime/backend/contracts.rs`
- `crates/worth-query/src/runtime/backend/bridge_backed.rs`

Current state:

- backend contracts still return `Vec<(String, serde_json::Value)>` for existing-truth probes;
- bridge-backed execution consumes the legacy runtime mutation carrier.

Required end state:

- backend contracts return native existing-truth facts;
- bridge-backed execution accepts native mutation carriers directly;
- no backend trait makes JSON part of the production authority contract.

Migration class: production contract rewrite.

### Zone 4: `runtime/computed/` plus derived materialization root files

Representative files:

- `crates/worth-query/src/runtime/computed/surface.rs`
- `crates/worth-query/src/runtime/computed/routing.rs`
- `crates/worth-query/src/runtime/workspace_queries.rs`
- `crates/worth-query/src/runtime/runtime_reads_programs.rs`
- `crates/worth-query/src/runtime/retained_rows.rs`
- `crates/worth-query/src/runtime/surface/derived_materialization_result.rs`

Current state:

- derived materialization defaults to `serde_json::Value`;
- computed rows and patch payloads are JSON rows;
- retained row decoding still assumes JSON materialization.

Required end state:

- derived views retain native row/value artifacts;
- materialization result types no longer expose JSON rows as the default production shape;
- retained row decoding operates on typed/native rows rather than JSON.

Migration class: second production blocker.

### Zone 5: `runtime/effect/`

Representative files:

- `crates/worth-query/src/runtime/effect/delivery.rs`
- `crates/worth-query/src/runtime/effect/routing.rs`
- `crates/worth-query/src/runtime/effect/declaration.rs`

Current state:

- effect delivery payloads and handles default to `serde_json::Value`;
- routing outcomes synthesize JSON payloads.

Required end state:

- effect delivery uses typed/native effect outputs;
- mutation effects carry native mutation intent, not JSON desired-aspect state;
- any JSON-like delivery value must be test-only or allowlisted with a hard external reason.

Migration class: production delivery rewrite after mutation carriers are fixed.

### Zone 6: `runtime/surface/` retained scalar and live surfaces

Representative files:

- `crates/worth-query/src/runtime/surface/retained_scalar_facts.rs`
- `crates/worth-query/src/runtime/surface/retained_scalar_alignment.rs`
- `crates/worth-query/src/runtime/surface/live.rs`
- `crates/worth-query/src/runtime/surface/derived_artifact_binding.rs`
- `crates/worth-query/src/runtime/surface/read_receipt_support.rs`

Current state:

- retained scalar facts and alignment still store JSON values;
- live view generic defaults still use `serde_json::Value`;
- read receipt support serializes external rows.

Required end state:

- retained facts and alignment use native scalar/aspect facts;
- live defaults stop teaching JSON as the normal row type;
- read receipt support digests native rows or explicitly typed external artifacts.

Migration class: production surface cleanup after Zones 4 and 5.

### Zone 7: `projection_consumption/consumed/`

Representative files:

- `crates/worth-query/src/projection_consumption/consumed/facts.rs`
- `crates/worth-query/src/projection_consumption/consumed/set.rs`

Current state:

- consumed fact values, member identities, and grouping values are `serde_json::Value`;
- fact digests canonicalize JSON.

Required end state:

- consumed facts use native value carriers;
- digesting is based on native canonical value encodings;
- JSON is absent from production consumed fact storage.

Migration class: read-side production blocker.

### Zone 8: `projection_consumption/extraction/`

Representative files:

- `crates/worth-query/src/projection_consumption/extraction/row_materialization/row_like.rs`
- `crates/worth-query/src/projection_consumption/extraction/grouped.rs`
- `crates/worth-query/src/projection_consumption/extraction/query_context.rs`

Current state:

- relational, bridge, Query read, live binding, retained binding, grouped, and
  query-context extraction now construct consumed field/grouping facts from
  native `AspectValue`;
- the former native-to-consumption JSON shim has been deleted;
- materialization rows and external projection fallback surfaces still need
  their own native row proof state.

Required end state:

- extraction accepts native projected-field rows;
- grouped/query-context extraction uses native identity and grouping values.

Migration class: read-side production blocker, paired with Zone 7.

### Zone 9: `projection_consumption/certification/` and `projection_consumption/tests/`

Representative files:

- `crates/worth-query/src/projection_consumption/certification/oracle_comparison_terms.rs`
- `crates/worth-query/src/projection_consumption/tests/phase_four_support.rs`
- `crates/worth-query/src/projection_consumption/tests/phase_four_remaining_sources.rs`
- `crates/worth-query/src/projection_consumption/tests/phase_four_query_context.rs`

Current state:

- certification oracle comparison still canonicalizes JSON facts;
- tests use JSON fixtures for rows and expected values.

Required end state:

- certification compares native canonical terms;
- any remaining JSON fixtures are marked as legacy debt and isolated from native proof tests.

Migration class: certification/test cleanup after Zones 7 and 8.

### Zone 10: `memory_workspace/`

Representative files:

- `crates/worth-query/src/memory_workspace/mod.rs`
- `crates/worth-query/src/memory_workspace/workspace.rs`
- `crates/worth-query/src/memory_workspace/external_projection.rs`
- `crates/worth-query/src/memory_workspace/tests.rs`

Current state:

- `WORTHQueryEntityRow` mixes native aspect maps with JSON external projections;
- `external_projection.rs` exists only to build JSON objects;
- tests author aspect values through `json!`.

Required end state:

- remove production external projection rows or move them behind an approved external-contract allowlist;
- expose native entity rows and native aspect lookup as the primary workspace representation;
- keep JSON tests only as marked legacy coverage while migration volume is being paid down.

Migration class: small but important production cleanup.

### Zone 11: `aspect_field_authoring/`

Representative files:

- `crates/worth-query/src/aspect_field_authoring/external_json_ingress.rs`
- `crates/worth-query/src/aspect_field_authoring/external_json_projection.rs`
- `crates/worth-query/src/aspect_field_authoring/keys.rs`
- `crates/worth-query/src/aspect_field_authoring/declarations.rs`

Current state:

- the key/declaration helpers are useful native authoring support;
- the external JSON ingress/projection files are legacy shim candidates.

Required end state:

- preserve native key and declaration helpers;
- delete, move, or explicitly allowlist the external JSON files;
- prevent other zones from depending on these files as a general migration escape hatch.

Migration class: exception audit with deletion as the default answer.

### Zone 12: `domain_capabilities/authoring/` and `domain_capabilities/payloads/`

Representative files:

- `crates/worth-query/src/domain_capabilities/authoring/workflow.rs`
- `crates/worth-query/src/domain_capabilities/payloads/workflow_semantics.rs`

Current state:

- workflow authoring now accepts native `AspectFieldPatch` carriers;
- workflow semantics digest canonical patch bytes rather than external JSON
  field objects;
- effect delivery still has remaining typed/native payload work.

Required end state:

- workflow authoring accepts native desired-aspect carriers;
- workflow semantics digest native mutation intent;
- contribution payload vocabulary remains only where it means proof or capability payload, not JSON truth.

Migration class: targeted production workflow cleanup.

### Zone 13: `effect_lifecycle/` tests and certification

Representative files:

- `crates/worth-query/src/effect_lifecycle/tests/support.rs`
- `crates/worth-query/src/effect_lifecycle/tests/execution/`
- `crates/worth-query/src/effect_lifecycle/tests/batch/`
- `crates/worth-query/src/effect_lifecycle/certification/`

Current state:

- most JSON here is test/certification fixture authoring;
- those fixtures mirror the legacy runtime/effect mutation substrate.

Required end state:

- update primary certification fixtures to native mutation intent;
- mark any retained JSON tests as legacy migration debt;
- avoid paying this zone down before Zones 1, 2, and 5 give it native APIs to use.

Migration class: test and certification follow-up.

### Zone 14: `intent_admission/certification/` and `lower_runtime_routing/certification/`

Representative files:

- `crates/worth-query/src/intent_admission/certification/fixtures/runtime.rs`
- `crates/worth-query/src/intent_admission/certification/fixtures/effect.rs`
- `crates/worth-query/src/lower_runtime_routing/certification/surface/fixtures/`

Current state:

- certification fixtures still construct `WORTHQueryWriteCommand::UpdateAspect` and `serde_json::Value` rows;
- runtime fixture probes still return string/JSON field pairs.

Required end state:

- certification fixtures use the native write and probe carriers;
- legacy JSON fixture cases are explicitly marked and isolated.

Migration class: certification follow-up after Zones 1, 2, 3, and 5.

### Zone 15: broad test suites under `runtime/tests`, `domain_capabilities/tests`, and related closeout fixtures

Current state:

- this is the largest JSON volume by count;
- much of it exists because production APIs still force or reward JSON-shaped fixtures.

Required end state:

- only update tests opportunistically until production APIs are replaced;
- once a production zone is migrated, update the tests in that zone to native fixtures;
- keep a known debt list for tolerated JSON tests so the exception does not silently become policy.

Migration class: final cleanup and debt tracking.

## Migration phases

### Phase 1. Lock the target ontology and boundary rules

Define the crate-wide rule set before editing code:

- list the exact foundational carrier families Query will treat as authoritative for mutation, retained state, row materialization, and projection facts;
- define the tiny production `serde_json::Value` allowlist, if any, including owner, reason, and removal condition;
- document whether string aspect paths remain only ergonomic caller inputs or remain part of any authoritative internal representation.
- define the Law 41 proof-state ladder for mutation, read/materialization,
  projection consumption, effect delivery, and certification.

Acceptance evidence:

- this spec names every production JSON allowlist entry that survives Phase 1;
- this spec names the proof state consumed and produced by the first
  implementation batch;
- Phase 1 chooses mechanical enforcement probes for the first slice before
  naming residue scans;
- residue scan output for `serde_json::Value`, `json!`, `aspect_path`,
  `touched_aspect_paths`, legacy external desired-field arguments, and
  `WORTHQueryAspectValue` is triaged by zone as discovery evidence only;
- the first implementation batch target is chosen from a production blocker,
  not a broad terminology cleanup.

### Phase 2. Rewrite runtime mutation authority carriers

Focus zones:

- Zone 1: `runtime/surface/mutation/`
- Zone 2: `runtime/mutation/`
- Zone 3: `runtime/backend/`

Deliverables:

- replace `UpdateAspect { aspect_path, value }` style authority primitives with foundational-native mutation carriers;
- remove `WORTHQueryAspectValue` from the core mutation path or collapse it into a native desired-aspect carrier;
- make runtime batch writes, backend contracts, and existing-truth verification consume the new native carriers directly.
- introduce sealed mutation proof states so authored input, validated desired
  aspect, backend-admissible request, authoritative patch/state request, and
  executed receipt cannot be confused.

Acceptance evidence:

- production mutation commands no longer expose path-plus-JSON authority;
- backend execution APIs consume a sealed admissible mutation proof state, not
  authored input or a catch-all command enum;
- existing-truth probes return native facts rather than `(String, Value)` pairs;
- mutation receipts expose native aspect operation evidence;
- compile-fail tests prove old command literals, path-plus-JSON helpers, and
  backend probe tuples cannot be used as authority;
- compile-fail tests prove weaker proof states cannot satisfy admitted or
  executed mutation APIs;
- the compiler, not a scan, proves path-plus-JSON command construction cannot
  reach backend execution.

### Phase 3. Rewrite read, materialization, and projection consumption

Focus zones:

- Zone 4: `runtime/computed/` plus derived materialization root files
- Zone 6: `runtime/surface/` retained scalar and live surfaces
- Zone 7: `projection_consumption/consumed/`
- Zone 8: `projection_consumption/extraction/`

Deliverables:

- replace JSON row materialization as the default internal representation;
- introduce native row/value abstractions for derived views, retained rows, and consumed fact extraction;
- ensure bridge-backed and relational-backed read paths converge on native row contracts before any optional external projection.
- introduce sealed read/materialization proof states from validated projection
  request through retained native row and consumed fact set.

Acceptance evidence:

- core derived materialization no longer requires `Vec<serde_json::Value>`;
- projection consumption APIs accept retained native row proof, not raw backend
  rows or terminal JSON;
- consumed fact storage and digesting use native canonical carriers;
- the former native-to-consumption JSON shim stays deleted or, if reintroduced,
  is terminal-only with denial tests proving it cannot feed authority;
- raw backend rows and terminal JSON projections fail to type-check against
  consumed fact extraction.

### Phase 4. Delete JSON shims or prove the tiny allowlist

Focus zones:

- Zone 10: `memory_workspace/`
- Zone 11: `aspect_field_authoring/`
- any production facade-level compatibility helper that still mentions JSON

Deliverables:

- delete production JSON ingress/projection shims that exist only for legacy convenience;
- produce an explicit allowlist for any production JSON that survives, including owner, reason, and removal condition;
- remove duplicate lowering/projection utilities from runtime and support layers;
- leave test JSON triage to the later test phases unless a test directly blocks production shim deletion.

Acceptance evidence:

- every surviving production JSON module is under explicit external ingress or
  terminal projection topology;
- native authority modules have no imports from JSON ingress/projection
  modules, enforced by visibility and facade/export tests rather than scan-only
  convention;
- deleting terminal JSON projection code would not change mutation,
  materialization, effect, certification, or replay meaning.

### Phase 5. Rewrite effect and workflow mutation substrates

Focus zones:

- Zone 5: `runtime/effect/`
- Zone 12: `domain_capabilities/authoring/` and `domain_capabilities/payloads/`

Deliverables:

- replace effect delivery JSON payloads with typed/native outputs;
- replace legacy workflow external JSON desired-field arguments with native
  desired-aspect carriers;
- keep benign contribution payload vocabulary where it remains truthful;
- confirm that declarations, admissions, and capability traces digest native substrates instead of legacy JSON intermediates.
- make workflow/effect transitions consume and produce distinct proof states
  so authored input cannot skip mutation admission.

Acceptance evidence:

- effect delivery payloads are typed/native before terminal projection;
- workflow declarations cannot be authored from legacy external JSON
  desired-field arguments;
- workflow semantics digests are derived from native mutation intent.
- compile-fail tests prove authored workflow/effect input cannot satisfy
  delivered effect authority without the intermediate proof state.

### Phase 6. Rewrite certification fixtures by migrated zone

Focus zones:

- Zone 9: `projection_consumption/certification/` and `projection_consumption/tests/`
- Zone 13: `effect_lifecycle/` tests and certification
- Zone 14: `intent_admission/certification/` and `lower_runtime_routing/certification/`

Deliverables:

- align test fixtures with the new authoritative carriers;
- preserve only explicitly marked legacy compatibility tests for JSON, with migration debt called out;
- prove certification, replay, and parity surfaces no longer depend on JSON carriers.

Acceptance evidence:

- migrated-zone certification fixtures construct native carriers directly;
- any remaining JSON test fixture has a debt marker naming the production API
  still being replaced;
- hostile tests prove JSON rows, dotted aspect paths, and projection objects
  cannot become authority.

### Phase 7. Retire broad test debt and close the docs

Focus zones:

- Zone 15: broad test suites under `runtime/tests`, `domain_capabilities/tests`, and related closeout fixtures
- `plans/worth-query/`

Deliverables:

- update tests opportunistically by migrated production zone;
- maintain a visible list for any tolerated JSON tests that remain due to crate size;
- produce closeout docs that distinguish ergonomic facade wins from true substrate completion.

Acceptance evidence:

- no unmarked JSON authority tests remain;
- production residue scan has only approved external ingress/export rows;
- closeout states which public ergonomics are authoring sugar and which native
  carriers own authority after lowering.

## Forbidden hybrids

These are not acceptable intermediate end states:

- native `AspectValue` created by a production helper that accepts arbitrary
  `serde_json::Value` after command construction;
- one broad native command enum that represents authored, validated, admitted,
  and executed states without distinct proof wrappers;
- public constructors or struct literals for validated, admitted, retained, or
  certified proof states;
- native field paths paired with legacy dotted strings as equal authority;
- mutation commands that carry native masks but still verify existing truth via
  `(String, Value)` probes;
- materialization rows that retain native carriers but compute consumed facts
  from JSON mirrors;
- effect delivery that stores JSON payloads as the semantic effect and native
  carriers only as diagnostics;
- certification bundles that keep JSON oracle terms as the retained truth;
- certification APIs that accept weaker proof states and then trust runtime
  checks to confirm the missing phase;
- public facade constructors that silently accept both native carriers and
  path-plus-JSON for the same authority field;
- external JSON ingress/projection modules imported by runtime mutation,
  backend, materialization, projection-consumption, effect, replay, or
  certification authority.

Temporary test adapters may use JSON only when the module path and test name
make the external I/O or legacy-debt boundary explicit. They may not be general
fixture conveniences for migrated code.

## Must preserve

- Query remains the pleasant day-to-day authoring API and does not force ordinary callers to hand-assemble raw foundational patch machinery.
- Public facade ergonomics added during aspect API finalization remain available unless a specific surface is proven dishonest or unsafe.
- Query does not steal bridge or relational authority; it lowers into their authoritative contracts cleanly.
- Preview, live delivery, inspection, and certification keep their current capability breadth even as their substrates change.

## Must remove

- JSON as an internal mutation authority carrier.
- JSON as the default internal row/materialization carrier for derived and consumed truth.
- production JSON shims whose only purpose is easing migration from legacy payload-shaped APIs.
- string aspect paths as the sole authoritative representation of aspect identity inside mutation/runtime internals.
- test and doc language that implies the facade freeze already completed the foundational-native migration.

## Acceptance evidence

- no runtime mutation authority type uses `serde_json::Value` as its authoritative desired-aspect carrier;
- mutation, read/materialization, projection consumption, effect, and
  certification boundaries use sealed proof states whose constructors are not
  publicly WORTHable;
- no admitted, retained, executed, or certified API accepts an earlier proof
  state and relies on a runtime check to discover the missing transition;
- no read/materialization substrate requires projecting native aspect values into JSON before core fact extraction or derived-view retention;
- production `serde_json::Value` usage in Query is zero except for an approved allowlist with owner, reason, and removal condition;
- tolerated test JSON is explicitly marked as legacy compatibility or migration debt;
- certification, replay, parity, and adversarial tests prove the new substrate instead of only the facade;
- a closeout doc can state, truthfully, that Query is foundational-native internally while still offering ergonomic aspect authoring externally.

## Batch plan template

Every implementation turn against this spec must start by reading the required
docs and then writing a concrete batch plan using this shape:

```text
Batch target:
- Native authority seam being replaced:
- Law 41 proof state consumed at the batch entry:
- Law 41 proof state produced at the batch exit:
- Sealed constructors/private fields/read-only accessors introduced:
- Compile-fail/public-boundary tests proving invalid construction or transition:
- Visibility/module fences introduced:
- Source-derived audits or facade export tests introduced:
- How far back this batch must move to delete upstream legacy:
- Files expected to delete or rewrite:
- New directory skeleton for this slice:
- Foundational carriers introduced:
- Legacy carriers removed:
- Public/test/harness JSON or dotted-string shims deleted:
- Weaker-proof promotion paths made unrepresentable:
- Failure/diagnostic/certification paths rewritten:
- Production JSON allowlist changes:
- Residue scan terms this batch will use only as diagnostic follow-up:
- Certification or compile checks to run at end:
- Tests intentionally skipped because the change is trivial/docs-only:
```

Do not split a subsystem unless the split removes or quarantines one of the
forbidden authority carriers. Do not declare a batch complete if any touched
production path still exposes a path-plus-JSON fallback or foreign-format
alias. If deleting that alias breaks callers, update the callers in the same
batch.

Do not declare a Law 41 batch complete if a weaker proof state can still be
passed to a later-stage API, if a proof state has public fields or public broad
constructors, or if downstream code must re-check a property already guaranteed
by the proof state it receives.

Do not declare a batch complete by saying a scan is clean. A batch is complete
only when invalid construction, invalid transition order, and lower-authority
promotion are mechanically blocked; scans are supporting evidence after that
bar is met.

## Verification policy

- For docs-only edits, run a lightweight diff or formatting sanity check.
- During production Query source rewrites, expect and tolerate a red compiler
  for most of the batch. Hundreds of `cargo check` errors are acceptable while
  the proof ladder is being moved, because fixing test fallout before the
  authority slice is coherent is slower and usually churns the wrong surface.
- Use `cargo check -p worth-query` or narrower `cargo check` commands as the
  primary navigation loop. Treat compiler errors as a work queue for the active
  proof slice, not as a mandate to make the whole crate green after each local
  edit.
- Do not run runtime tests, trybuild tests, certification suites, or broad
  workspace tests until the planned proof slice has landed and the relevant
  source surface is close to compiling.
- When the batch reaches a coherent source shape, run the smallest focused
  verification that proves the moved authority boundary. Use a 10 minute
  timeout per command.
- When a source change touches bridge-backed execution or relational authority
  boundaries, also run relevant focused `worth-runtime-bridge` or
  `worth-relational` checks only after the Query slice is coherent enough that
  cross-crate errors are meaningful.
- Do not run broad workspace tests after trivial edits or mid-refactor red
  states.

### Test-cost discipline

The runtime bridge refactor showed that tests can consume most of the wall
time if they are run before the type migration has settled. Query should avoid
that trap.

Default loop for implementation batches:

1. edit the vertical proof slice;
2. run `cargo check -p worth-query` only when compiler feedback will guide the
   next structural move;
3. keep moving through expected red states instead of repairing every test
   fixture immediately;
4. once production code for the slice has the intended proof-state topology,
   update only the coupled tests/fixtures needed to prove that topology;
5. run the narrowest focused test or compile-fail target that demonstrates the
   mechanical boundary;
6. defer broad test cleanup to the phase that owns that test zone.

Do not spend a batch making legacy JSON/path-string tests green unless those
tests are directly coupled to the proof boundary being replaced. A failing
legacy test is often useful evidence that the old model has been broken; it is
not automatically a blocker until the owning production zone is ready for
closeout.

## Sequencing notes

- Do not start with global terminology churn. Start with runtime mutation and read/materialization truth.
- Delete compatibility helpers once core consumers are moved off them; do not let them become permanent legacy shims.
- Expect `runtime/` and `projection_consumption/` to dominate the engineering time.
- Treat `application/` as mostly preserved unless a concrete seam proves otherwise.
- Treat `domain_capabilities/` as an audit problem, not an automatic rename problem.
- Finish with tests and closeout docs, not before them.

The highest-value first implementation slice is Phase 2, Zone 1 plus the
minimum Zone 2/3 carrier surface needed to remove path-plus-JSON mutation
authority. That slice should target `WORTHQueryWriteCommand`,
`WORTHQueryAspectValue`, existing-truth probe facts, and the directly coupled
write receipt/helper tests. Read/materialization JSON should not be attacked
first unless the mutation rewrite discovers an unavoidable upstream blocker.
The first slice should explicitly introduce the initial mutation proof ladder
rather than landing one all-purpose native command replacement.

## Implementation checkpoints

These checkpoints record landed enforcement moves. They are not phase closeout
claims.

### Phase 2 mutation authority slice

- `WORTHQueryAspectValue` now retains parsed native desired-aspect state and
  exposes JSON only through terminal projection helpers.
- `WORTHQueryAspectTouch` retains parsed native delete/touch targets instead
  of raw touched aspect path strings.
- `WORTHQueryAspectValue::new_clear` and
  `WORTHQueryAspectMutationBuilder::clear` now require
  `WORTHQueryAspectTouch`, so clear-authority cannot be constructed from raw
  aspect path strings inside the proof-bearing mutation builder.
- `WORTHQuerySymbolicAspectReference::same_batch_entity_identity`,
  `WORTHQueryAspectMutationBuilder::symbolic_entity_identity`, and graph
  relation symbolic identity authoring now require `WORTHQueryAspectTouch`, so
  symbolic reference authority cannot be promoted from raw aspect path strings.
- `WORTHQueryAspectValue::new_set_native` and crate-local symbolic
  evidence-identity set construction now require `WORTHQueryAspectTouch`, so
  native/evidence set-aspect proof construction reuses admitted targets instead
  of re-parsing terminal path text.
- `WORTHQueryAspectValue::new`, `WORTHQueryAspectValue::new_set`,
  `WORTHQueryAspectMutationBuilder::aspect`, and graph relation set-aspect
  authoring now require `WORTHQueryAspectTouch`, so proof-bearing set-aspect
  construction cannot be satisfied by raw dotted path strings.
- Set-aspect construction now requires foundational `AspectValue` instead of
  generic `Serialize` values. Program write-template binding projects retained
  `WORTHQueryProgramValue` into native scalar values, and the old desired-aspect
  compatibility JSON lowering path has been removed.
- Graph touch descriptor rows, descriptor constructors, obligation selectors,
  and lookup-key derivation now retain `WORTHQueryAspectMutationOperation` and
  `WORTHQueryAspectTouch` values for declared/touched aspect evidence instead
  of accepting or parsing `set:path` and touched-path strings.
- `WORTHQueryDeleteMutationBuilder::touch` and `touches` now require
  `WORTHQueryAspectTouch`, so delete touch authority cannot be constructed
  from raw aspect path strings inside the proof-bearing builder.
- `WORTHQueryWriteCommand::UpdateAspect` no longer carries
  `aspect_path: String` plus `serde_json::Value`; it carries the native
  desired-aspect wrapper.
- backend `write` and `write_batch` now require
  `WORTHQueryBackendAdmissibleMutation`, so a raw authored
  `WORTHQueryWriteCommand` cannot satisfy backend execution directly.
- existing-truth probe requests now retain `WORTHQueryAspectTouch` values and
  public request construction requires touch carriers; workspace ergonomic
  probe helpers lower authoring path text before constructing the request.
- existing-truth probes now return `WORTHQueryExistingTruthProbeField` with a
  foundational `AspectValue` and parsed target; `(String, Value)` probe
  responses are no longer a production backend contract.

### Phase 3 retained materialization slice

- retained scalar fact extraction now admits the single JSON materialization
  row through a private retained-scalar row proof state and stores
  `AspectValue` in retained scalar facts and scalar alignment facts.
- derived materialization result public row exposure is now named
  `terminal_json_rows_projection`; the raw `rows()` accessor is runtime-local.
- computed derived materialization public mutation now uses retained row
  carriers through `push_retained_row`, `replace_retained_rows`, and
  `retain_retained_rows`; terminal JSON row mutation remains test-only
  compatibility.
- retained upstream inputs now accept computed rows as
  `WORTHQueryRetainedMaterializedRow` carriers at construction; terminal JSON
  row admission is cfg-test compatibility only.
- retained upstream computed rows are exposed publicly only as
  `terminal_json_computed_rows_projection`; runtime-local access now uses
  `retained_computed_rows`, and typed terminal decode helpers project JSON
  explicitly from retained rows.
- `projection_consumption` retained-binding extraction uses the named terminal
  projection until the next native retained-row proof carrier replaces that
  compatibility boundary.

### Phase 5 workflow mutation substrate slice

- workflow mutation lowering now carries `AspectFieldPatch` directly for
  intent reconciliation instead of accepting an external JSON field object.
- domain capability workflow authoring now accepts native field patches at the
  authoring boundary and passes those patches through lowering without
  re-materializing JSON.
- workflow semantics identity now hashes the canonical patch bytes, so the
  semantic digest follows the native mutation substrate.
- primary effect lifecycle and lower-runtime-routing certification fixtures use
  native string field patch helpers for name mutation scenarios.
- source enforcement audit: the old workflow external desired-field argument
  name has no remaining source matches outside stale docs at the time this
  checkpoint was written.

### Phase 5 runtime effect delivery payload slice

- `WORTHQueryEffectDelivery` now stores `WORTHQueryEffectPayload`, a native
  payload carrier with private fields, instead of retaining
  `serde_json::Value` as delivery meaning.
- effect routing constructs native `WORTHQueryEffectPayload` values for
  always and expression outcomes instead of using `json!` objects.
- pending-intent handoff hashing and runtime intent declaration still bridge
  to legacy JSON input through the explicit
  `terminal_json_payload_projection` boundary.
- `WORTHQueryEffectPayload` is exported through the runtime facade so public
  delivery payload access has a named native type.

### Phase 3 consumed field fact slice

- `ConsumedFieldValueFact` now stores foundational `AspectValue` rather than
  `serde_json::Value`, so display-field and derived-scalar consumed facts
  cannot be constructed from raw JSON.
- relational row-set extraction passes projected native aspect values directly
  into consumed field facts.
- bridge row-set extraction admits only scalar validated aspect payloads into
  consumed field facts; struct payloads are rejected instead of being projected
  into JSON authority.
- query read, live binding, retained binding, and query-context extraction
  route remaining external row values through a named extraction-local scalar
  admission boundary before constructing consumed field facts.
- consumed field fact digests use native scalar text rather than JSON canonical
  text.

### Phase 3 grouped projection fact slice

- `ConsumedMembershipFact` and grouped `ConsumedRelationEndpointFact` now store
  native `AspectValue` for member identity and grouping value instead of
  `serde_json::Value`.
- relational and bridge grouped extraction pass native grouped member values
  directly into consumed facts instead of projecting them through JSON.
- consumed fact-set identity and grouped certification oracle comparison now
  digest native grouping values.
- the former native-to-consumption JSON projection shim was deleted and removed
  from identity-boundary inventory sources.

### Phase 3 graph read admitted aspect key slice

- admitted graph-read projection, predicate, ordering, boolean predicate, and
  predicate selectivity rows now retain foundational `AspectKey` values instead
  of raw aspect strings.
- graph-read predicate and ordering access authorities now retain `AspectKey`
  values and project aspect text only for existing reporting/digest accessors.
- schema-reference admission now lowers authoring aspect text into
  `AspectKey` at the admitted-reference boundary and passes that native key
  through boolean admission, selectivity normalization, and access-authority
  construction.
- the unused graph touch selector string-authoring aspect-path adapter was
  removed; selector aspect matching now requires an admitted
  `WORTHQueryAspectTouch`.

### Phase 4 memory workspace mutation touch slice

- `WORTHQueryMutationDelta` now stores admitted `WORTHQueryAspectTouch` values
  instead of retaining raw touched aspect path strings.
- `WORTHQueryLivePatch` now has the same native touch carrier shape and exposes
  aspect paths only through terminal projection helpers.
- memory workspace insert/update receipts preserve parsed touch proof from
  `WORTHQueryAspectValue` through `WORTHQueryAspectTouch::from_parsed_target`
  instead of reconstructing touch authority from path text.
- computed routing, effect routing, live subscription delivery, preview
  relevance, batch aggregation, inspection, and certification reporting now
  consume native touches or explicitly call terminal path projections at the
  remaining string-based reporting/matching boundaries.
- production direct-field reach-through for `delta.aspect_paths` was removed;
  the remaining non-test string path carriers are now visible as separate
  follow-up zones rather than hidden inside the memory workspace receipt model.

### Phase 3 retained binding scalar access slice

- retained binding projection-consumption extraction no longer walks
  `terminal_json_rows_projection` or calls JSON scalar admission directly.
- `WORTHQueryDerivedMaterializationResult` now exposes crate-local retained
  scalar accessors (`retained_row_count`, `retained_scalar_value_at`) that
  return native `AspectValue` or missing-field evidence.
- retained binding extraction now consumes those native scalar accessors while
  preserving its existing missing-field and invalid-shape error boundaries.
- the remaining JSON materialization row storage is localized in the runtime
  materialization surface; projection consumption no longer treats retained
  JSON rows as its source of authority.

### Phase 2 mutation operation and denial evidence slice

- `WORTHQueryAspectMutationOperation` now stores `WORTHQueryAspectTouch`
  instead of retaining an aspect path string; ordering, hashing, and reporting
  derive from the terminal projection only at the descriptor boundary.
- `WORTHQuerySymbolicAspectReference`,
  `WORTHQuerySymbolicAspectResolutionEvidence`, and graph-composition
  resolution entries now retain admitted touches for symbolic aspect targets.
- existing-truth assertion/probe denials now store a distinct denied-path proof
  state: admitted touch when the path parses, rejected terminal text only when
  preserving invalid denied input evidence.
- existing-truth assertion denials now store and digest optional expected/found
  native value digest evidence instead of expected/found external JSON strings;
  the old external JSON accessor names are mechanically rejected by the
  aspect-native compile-fail suite.
- existing-truth probe fields now retain only the native foundational
  `AspectValue` plus native value digest evidence; terminal JSON is computed
  as an explicitly named projection instead of cached under `external` naming,
  and the old `external_value_json` accessor is mechanically rejected by the
  aspect-native compile-fail suite.
- command-declared mutation evidence lowering now feeds native aspect digest
  material into evidence identity rows instead of terminal JSON projection
  strings.
- production constructors that create mutation operations now consume existing
  parsed targets or admitted touches rather than rebuilding operation authority
  from raw strings.

### Phase 2 verified assumption touch slice

- `WORTHQueryVerifiedAssumptionSet` now stores asserted aspects as
  `WORTHQueryAspectTouch` values instead of raw asserted aspect path strings.
- `WORTHQueryVerificationReadSetBreadth` computes distinct asserted aspect
  counts from admitted touches and projects path text only for existing counter
  labels/reporting.
- graph-composition assumption summaries aggregate asserted touches first, then
  create terminal path projections only for digest/reporting evidence.
- existing-truth assertion verification now preserves the parsed targets from
  `WORTHQueryAspectValue` when building verified assumption evidence.

### Phase 5 effect delivery touch slice

- `WORTHQueryEffectTrigger` and `WORTHQueryEffectExpression` now retain
  `WORTHQueryAspectTouch` declarations for trigger, input, and output aspects
  instead of storing raw aspect path strings.
- public effect trigger and expression constructors, plus the workspace effect
  builder trigger/condition methods, now require `WORTHQueryAspectTouch`
  values so weaker authoring strings cannot satisfy effect declaration
  authority.
- `WORTHQueryEffectDelivery` now stores changed trigger aspects as
  `WORTHQueryAspectTouch` instead of raw aspect path strings.
- `WORTHQueryEffectPayload` stores native changed-aspect touches and exposes
  path text only through terminal projection helpers; expression payload
  input/output declarations are also retained as touches.
- effect declaration admission now validates trigger/input/output touch
  declarations before routing can construct deliveries.
- effect routing builds trigger changes as native touches for live and computed
  sources, using string projections only for matching against changed terminal
  paths and reporting/digest output.
- effect routing aspect helpers were split into a small module so the migrated
  routing file remains under the workspace Rust line cap.

### Phase 3 computed patch touch slice

- `WORTHQueryDerivedPatch` now stores produced/changed aspects as
  `WORTHQueryAspectTouch` instead of raw aspect path strings.
- public `WORTHQueryDerivedPatch::incremental` and
  `whole_refresh_materialized` constructors now require
  `WORTHQueryAspectTouch` values, so maintainers cannot construct patch
  authority from raw aspect path strings.
- derived patch path APIs now expose terminal projections while internal
  consumers can read native patch touches directly.
- computed derived view admission now validates dependency and produced aspect
  declarations as admissible touches before patches are constructed.
- effect routing consumes derived patch touches directly for computed triggers,
  removing the former readmit-from-string hop.

### Phase 3 retained refresh context touch slice

- `WORTHQueryRetainedRefreshContext` now stores touched aspects as
  `WORTHQueryAspectTouch` instead of raw touched aspect path strings.
- computed routing passes mutation delta touches into retained refresh context
  construction directly instead of terminal path projections.
- retained refresh public path access is now a terminal projection over native
  touches.

### Phase 4 preview execution evidence touch slice

- `WORTHQueryPreviewExecutionEvidence` now stores preview-routed aspect changes
  as `WORTHQueryAspectTouch` values instead of raw aspect path strings.
- preview execution evidence construction is split into aspect-touch evidence
  and intent-strategy evidence, so a pending intent strategy label can no
  longer masquerade as an aspect path.
- preview live/computed/effect routing preserves native touches for execution
  evidence and uses terminal path projections only for declaration matching and
  test/reporting assertions.
- directly coupled preview tests now assert the intent strategy subject through
  `intent_strategy_name` and use terminal aspect path projections when they
  inspect text.

### Phase 4 batch receipt inspection touch slice

- `WORTHQueryBatchWriteReceipt` now stores batch touched aspects as
  `WORTHQueryAspectTouch` instead of raw touched aspect path strings.
- batch receipt aggregate derivation deduplicates admitted touches from write
  receipt deltas, so callers cannot WORTH batch touched-aspect evidence by
  passing arbitrary strings.
- unified batch and component inspection retain native touches and derive
  terminal path projections only for digest/reporting APIs.
- authoritative batch writes pass native touches from the combined mutation
  receipt into batch receipt construction rather than projecting strings and
  reusing them as authority.

### Phase 4 graph touch descriptor slice

- graph touch descriptor rows now store touched aspects as
  `WORTHQueryAspectTouch` values instead of raw touched aspect path strings.
- graph read touch shapes now require `WORTHQueryAspectTouch` values at
  construction; read obligation dispatch performs the explicit admission from
  read-family field paths before descriptor evidence is created.
- command-derived touch rows, descriptor inventory, and obligation lookup-key
  derivation retain admitted touches and project path text only for matching
  keys, counts, and digests.
- declared mutation graph touch descriptors and graph obligation aspect
  selectors now admit touched aspect paths through `WORTHQueryAspectTouch`
  before constructing descriptor or selector evidence.
- graph descriptor and selector denials now include an invalid-aspect-path
  state, so invalid touched-aspect input is rejected at the graph boundary
  instead of being retained as authority.

### Phase 3 computed declaration touch slice

- `WORTHQueryDerivedView` now stores dependency and produced aspects as
  `WORTHQueryAspectTouch` values instead of raw string vectors.
- `WORTHQueryDerivedView::new` and `WORTHQueryDerivedView::produces` now
  require `WORTHQueryAspectTouch` values, so callers cannot construct a
  proof-bearing derived declaration from raw aspect path strings.
- workspace computed-builder reads/produces methods now carry admitted touches
  into the derived declaration; path text remains only in earlier ergonomic
  authoring helpers or terminal projections.
- computed declaration admission no longer revalidates projected strings; it
  receives a proof-bearing declaration whose aspect fields were admitted at
  construction.
- computed inspection evidence now retains dependency/produced touches and
  derives terminal path projections only for digest/reporting accessors.

### Phase 2 program command template touch slice

- `WORTHQueryAspectValueTemplate` now stores its aspect identity as
  `WORTHQueryAspectTouch` instead of retaining a raw aspect path string.
- `WORTHQueryWriteCommandTemplate::UpdateAspect` now stores an admitted
  `WORTHQueryAspectTouch`; binding projects path text only when constructing
  the already-native `WORTHQueryAspectValue`.
- `WORTHQueryAspectValueTemplate::new` and
  `WORTHQueryTypedPort::with_required_aspect` now require
  `WORTHQueryAspectTouch`, so program command template and typed port
  authority cannot be constructed from raw aspect path strings.

### Phase 3 memory workspace native row projection slice

- memory workspace aspect-projection rows no longer retain a JSON object beside
  their native aspect map.
- `WORTHQueryEntity` now stores external projection path values as native
  `AspectValue` and derives JSON only through terminal row projection helpers.
- `WORTHQueryEntity` terminal JSON row projection helpers are now cfg-test
  compatibility only, and the aspect-native compile-fail suite proves public
  callers cannot recover row truth through those projection methods.
- the supporting memory-workspace external JSON projection modules are also
  test-only after native row projection became the production substrate.
- live-binding and row-like projection-consumption extraction now read native
  external-path values from memory workspace rows before falling back to
  explicitly external JSON rows.
- the old generic JSON row path walker in projection consumption was deleted,
  reducing the chance that memory workspace rows silently become JSON
  authority again.
- read-composition and read receipt digest callers now name the terminal JSON
  row projection boundary when they need exported row shape.

### Phase 3 retained materialized row carrier slice

- `WORTHQueryRetainedMaterializedRow` is now the retained proof carrier for
  derived materialization rows; its fields are private and construction admits
  terminal JSON only through an explicit scalar-leaf ingress.
- `WORTHQueryDerivedMaterializationResult` and
  `WORTHQueryDerivedViewMaterialization` now store retained materialized rows
  instead of `Vec<serde_json::Value>`.
- retained scalar facts read `AspectValue` directly from retained
  materialized rows rather than walking nested JSON rows.
- retained scalar field facts no longer expose a production terminal JSON
  projection accessor; callers must use the native `AspectValue` accessor, and
  the removed terminal projection method is guarded by the aspect-native
  compile-fail suite.
- retained scalar fact-set and retained-row digests use variant-aware native
  `AspectValue` digest text instead of serializing retained scalar values
  through terminal JSON.
- retained upstream computed inputs store retained materialized rows and use a
  runtime-private native constructor when passing computed rows between
  derived views.
- published artifact and inspection callers that still need row JSON now call
  terminal projection helpers after materialization authority has already been
  retained natively.

### Phase 3 derived patch payload fence slice

- `WORTHQueryDerivedPatch` no longer stores raw `serde_json::Value` as its
  payload field.
- derived patch constructors now require `WORTHQueryDerivedPatchPayload`, so
  callers must supply an explicit native payload proof instead of passing raw
  JSON into the stronger patch proof state.
- `WORTHQueryDerivedPatchPayload` now stores private native payload variants:
  empty payload, foundational scalar payload, or retained materialized row
  payloads.
- public payload construction is native-only through `empty`,
  `from_scalar_value`, `from_retained_row`, and `from_retained_rows`; the
  public `from_terminal_json` constructor was removed so arbitrary JSON cannot
  satisfy derived-patch payload authority.
- the old `payload() -> &Value` accessor was replaced by
  `terminal_json_payload_projection`, making remaining JSON exposure a named
  projection boundary.
- production computed routing emits the native empty payload proof for fallback
  patches.
- runtime API stabilization and aspect API certification maintainers now carry
  retained row payload proofs instead of rebuilding patch payloads from JSON.
- directly coupled computed assertions that inspect payload JSON now name the
  terminal projection accessor; broader cfg-test maintainer constructor
  migration remains a focused test-support slice.

### Phase 3 retained materialization execution slice

- derived materialization execution now constructs
  `WORTHQueryDerivedMaterializationResult` from retained materialized rows
  instead of projecting runtime materialization through `Vec<Value>`.
- shared-read published artifact generation also binds retained
  materialized rows directly before deriving terminal JSON for artifact
  consumers.
- `WORTHQueryDerivedMaterializationResult` production construction no longer
  accepts terminal JSON rows; terminal JSON admission is gated to test-only
  helpers.
- runtime/workspace convenience APIs now expose explicit
  `terminal_json_read_derived` and `terminal_json_materialize` names for
  terminal export.
- `read_derived`, `read_derived_result`, `materialize`, and
  `materialize_result` now return the retained materialization result so the
  default vocabulary no longer teaches callers to consume `Vec<Value>` rows.
- retained materialization results expose `row_count()` for count-only callers
  that should not force terminal JSON row projection.
- terminal row decoding now receives retained materialized row carriers and
  projects JSON only inside the explicitly terminal decode boundary.
- computed materialization inspection identity now receives retained rows and
  performs terminal JSON projection only while constructing reporting digest
  text.

### Phase 3 retained materialization publication slice

- `WORTHQueryRetainedMaterializedRow` now has a native
  `from_scalar_values` constructor over foundational `AspectValue` maps, with
  private storage preserved.
- `WORTHQueryDerivedViewMaterialization` exposes native row publication
  methods: `replace_retained_rows`, `push_retained_row`, and
  `retain_retained_rows`.
- terminal JSON materialization mutators are no longer public API; they remain
  cfg-test compatibility helpers only.
- runtime API stabilization and aspect API certification maintainers now
  publish retained rows through native scalar maps instead of calling terminal
  JSON row mutators.
- retained row decode helpers now use retained-row names:
  `decode_single_retained_row`, `decode_retained_row_pair`, and
  `decode_retained_row_triple`; the old terminal JSON names are cfg-test
  compatibility shims only.
- projection-consumption forbidden-fallback audits now look for the retained
  decode seam names, so ordinary paths stay mechanically checked without
  relying on generic grep claims.

### Phase 2 mutation metadata retained evidence slice

- `WORTHQueryMutationMetadata` no longer stores metadata entries as
  `serde_json::Value`.
- metadata values are retained as `WORTHQueryMutationMetadataValue`, a private
  encoded evidence carrier with a named `terminal_json_text` projection for
  digest/reporting boundaries.
- write-receipt digest construction, causal observation identities, and bridge
  mutation-authority identities now consume metadata terminal text from the
  retained carrier instead of re-serializing JSON values.
- the old causal `stable_json_value` helper was deleted, removing one more
  generic JSON evidence path from production mutation inspection.

### Phase 2 backend-admissible mutation family-shape slice

- `WORTHQueryBackendAdmissibleMutation` no longer retains a whole
  `WORTHQueryWriteCommand` behind the admitted proof wrapper.
- The admitted mutation proof now stores a private family-specific shape:
  insert, direct update, existing-truth update, assertion, symbolic update,
  direct delete, existing-truth delete, or symbolic delete.
- The old command reconstruction/read path is gone from the proof state:
  public callers cannot call `into_command()`, cannot call `command()`, cannot
  call the crate-local admission constructor, and cannot WORTH the private
  shape by struct literal.
- Write-authority receipt construction and bridge-backed drift checking derive
  subject identity from the admitted mutation proof instead of recovering a
  command from it.
- Consumer-kit, stateful bridge, public bridge, runtime transcript, signal,
  and intent-admission write-authority fixtures now consume read-only
  admitted-mutation accessors for family, target, aspect values, symbolic
  references, touched aspects, asserted aspects, metadata, naming, and
  continuity.
- The private shape conversion lives in a sibling module so both the proof
  wrapper and the shape module remain under the workspace Rust line cap.
- Write-authority execution receipt routing was split into its own adapter
  module so the lower-runtime backend adapter also remains under the workspace
  Rust line cap.
- Verification for this slice: `cargo fmt -p worth-query`,
  `cargo check -p worth-query --tests`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused write-authority drift and
  signal-invalidation receipt tests, `public_bridge_runtime_bootstrap`,
  `public_submission_lane_replacements`, and `intent_admission --lib`.

### Phase 2 bridge writeback native effect-intent slice

- Bridge writeback authority no longer lowers admitted Query mutations into a
  synthetic `WORTH.query.writeback` scalar marker.
- Added a crate-private `WORTHQueryBridgeWritebackEffectIntent` proof that
  accepts only `WORTHQueryBackendAdmissibleMutation` and lowers it into a
  foundational `AuthoritativeRecordAspectPatch` before bridge provenance is
  minted.
- Whole-aspect set/clear and single-field set/clear operations now produce
  bridge `BridgeWritebackEffectIntent::from_authoritative_patch(...)`
  evidence. Nested field paths remain an explicit unsupported denial until the
  field-patch substrate has a real nested-path contract.
- Bridge provenance now binds its canonical effect-intent basis to the actual
  admitted aspect patch value, with entries such as `title.field.value.set`
  and `exact-text:Done`, instead of an identity-only Query marker.
- Mechanical enforcement: the proof carrier is crate-private, has private
  fields, is only constructed from an admitted mutation transition, and has an
  aspect-native compile-fail fixture proving it is not public facade API.
- Verification for this slice: `cargo fmt -p worth-query`,
  `cargo check -p worth-query --tests`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused bridge writeback effect
  intent provenance test, focused write-authority drift and signal-invalidation
  receipt tests, `public_bridge_runtime_bootstrap`, and
  `public_submission_lane_replacements`.

### Phase 2 program value carrier slice

- program operation inputs, outputs, and value-expression literals now retain
  `WORTHQueryProgramValue` instead of storing raw `serde_json::Value`.
- `WORTHQueryProgramValue` now retains a private native value tree rather than
  a named `serde_json::Value` wrapper.
- value-expression evaluation assembles object and array results from retained
  program values instead of projecting child values into JSON and readmitting
  them.
- public `WORTHQueryValueExpr::literal`, `WORTHQueryOperationInput::new`, and
  `WORTHQueryOperationOutput::new` require `WORTHQueryProgramValue`, so public
  program authoring cannot promote raw JSON into the retained program-value
  proof state.
- runtime live-read program outputs now build `WORTHQueryProgramValue` rows
  directly from native `CanonicalFieldPath -> AspectValue` entity projections;
  the former terminal JSON row projection/readmission path was removed.
- program value, operation input, and operation output terminal JSON
  projection accessors are now cfg-test compatibility only; public callers must
  inspect retained program values through native accessors instead of using
  program IO as a JSON escape hatch.
- `WORTHQueryProgramTrace` records bound input names from the retained program
  value map rather than from a raw JSON input map.
- `WORTHQueryPortType::TerminalJson` remains a terminal program-port compatibility
  shape; it must not be treated as mutation, materialization, or projection
  authority in later slices.
- directly coupled cfg-test program fixtures now construct
  `WORTHQueryProgramValue` for operation inputs and value-expression literals
  instead of teaching raw JSON program authoring.
- `tests/aspect_native_query_compile_fail.rs` now mechanically proves that raw
  JSON cannot satisfy program operation input, raw JSON cannot satisfy derived
  patch retained-row payload authority, and private retained program/effect
  payload fields cannot be WORTHd through public struct literals.
- the aspect-native compile-fail suite also proves public callers cannot call
  the removed program terminal JSON projection methods.

### Phase 2 intent input carrier slice

- `WORTHQueryIntentDeclaration` now retains `WORTHQueryIntentInput` instead
  of storing raw `serde_json::Value` directly.
- `WORTHQueryIntentInput` now retains a private native input tree rather than
  a named `serde_json::Value` wrapper.
- `strategy_commit` now requires the retained input carrier, so public callers
  cannot pass raw JSON into the runtime-intent declaration proof state.
- runtime effect-triggered intent construction builds the input carrier from
  native effect payload fields instead of lowering a terminal JSON payload
  projection.
- `input()` returns the retained input carrier; terminal exposure must use
  explicit terminal/reporting projection APIs only when a terminal boundary is
  being crossed.
- string-field reads needed by authority adapters use native
  `input_string_field` access instead of projecting the whole input to JSON.
- intent input digests serialize from the named terminal projection boundary,
  not from a public raw JSON authority field.
- directly coupled production/certification fixtures now build object-shaped
  `WORTHQueryIntentInput` values instead of `json!` payloads.

### Phase 4 external JSON ingress quarantine slice

- `aspect_field_authoring/external_json_ingress.rs` is now a cfg-test module,
  matching the already test-only external JSON projection helper.
- production lower-runtime, workflow, effect-authority, and relational merge
  certification fixtures now construct `AspectFieldPatch` values through
  `single_native_string_aspect_field_patch` instead of lowering `json!(...)`
  through the external compatibility ingress.
- `cargo check -p worth-query` mechanically proves production code no longer
  depends on the external JSON ingress export; the focused aspect field tests
  keep the compatibility ingress covered only as test debt.
- No production JSON allowlist entry remains approved for aspect field
  authoring after this slice.

### Phase 4 mutation/probe terminal JSON accessor fence

- `WORTHQueryAspectValue` and `WORTHQueryDesiredAspectValue` terminal JSON
  projection accessors are now cfg-test compatibility only; production and
  public callers retain `AspectValue` plus native digest material instead.
- `WORTHQueryExistingTruthProbeField::terminal_json_projection_string` is now
  cfg-test compatibility only; public probe assertions must use
  `foundational_value` or `value_digest`.
- The public bridge runtime bootstrap certification now asserts probed
  foundational values directly instead of reading terminal JSON strings.
- The aspect-native compile-fail suite now mechanically proves public callers
  cannot call aspect-value or probe-field terminal JSON projection accessors.

### Certification fixture native handle marker slice

- Intent-admission effect certification fixtures now declare live views and
  effects with `WORTHQueryNativeRow` handle markers instead of
  `serde_json::Value`.
- The legacy/canonical delegation parity fixture now uses the same native row
  marker for both delegated and canonical effect paths.
- Runtime API stabilization transcript fixtures and aspect API finalization
  certification rows now use `WORTHQueryNativeRow` for live, computed, and
  effect handles instead of `serde_json::Value`.
- Consumer-kit in-memory backend behavior tests now use `WORTHQueryNativeRow`
  live-view markers instead of `serde_json::Value`, so even src-hosted test
  surfaces demonstrate the native read handle path.
- `cargo check -p worth-query` plus the handle-marker scan mechanically prove
  the certification fixture layer no longer depends on JSON as the
  live/computed/effect handle type.
- Dead cfg-test terminal JSON mutators on
  `WORTHQueryDerivedViewMaterialization` were removed; retained-row native
  mutation helpers are the remaining materialization mutation path.
- Dead cfg-test `terminal_json_decode_*` aliases on retained derived
  materialization results, bundles, and artifact bindings were removed; tests
  and shared-read artifact code now call `decode_single_retained_row`,
  `decode_retained_row_pair`, and `decode_retained_row_triple` directly.
- Stale test-only `serde_json::json` imports/helpers were removed from
  domain-capability, effect-lifecycle, and shared-read test support so focused
  test output no longer hides current signal behind old JSON compatibility
  noise.

### Phase 3 retained materialization constructor fence slice

- The remaining cfg-test
  `WORTHQueryDerivedMaterializationResult::new(Vec<serde_json::Value>, ...)`,
  `from_terminal_json_rows`, and JSON-row `test_only` constructors were
  removed.
- Runtime-local retained materialization fixtures now construct
  `WORTHQueryRetainedMaterializedRow` values explicitly from
  `WORTHQueryRetainedFieldPath -> AspectValue` maps before creating a derived
  materialization result.
- Cross-module test fixtures that need retained materialization results use a
  cfg-test `test_only_retained_rows` constructor that accepts retained rows,
  not terminal JSON rows, so test code can no longer promote arbitrary JSON
  into the retained materialization proof carrier.
- Derived artifact binding tests now use `WORTHQueryNativeRow` handle markers
  and decode into typed row structs instead of using `serde_json::Value`
  handles and JSON row assertions.
- Focused verification covered `cargo check -p worth-query`,
  `cargo test -p worth-query retained_scalar --lib`,
  `cargo test -p worth-query retained_live --lib`, and
  `cargo test -p worth-query decode_row --lib`; the deleted constructor scan
  for the old JSON-row materialization names returned no matches.

### Phase 3 retained upstream input ingress deletion slice

- `WORTHQueryRetainedUpstreamInputs::terminal_json_test_inputs` was removed;
  computed upstream test fixtures now seed retained computed rows through
  `from_retained_computed_rows`.
- The retained materialized row compatibility admission function
  `WORTHQueryRetainedMaterializedRow::from_terminal_json_row` was deleted
  after its final caller disappeared.
- The supporting JSON-to-`AspectValue` scalar converter was removed, leaving
  retained-row construction on the explicit
  `WORTHQueryRetainedFieldPath -> AspectValue` path and terminal JSON only as
  named projection/reporting output.
- The aspect-native compile-fail suite continues to prove public callers
  cannot call the removed retained-row terminal JSON ingress.
- `WORTHQueryRetainedUpstreamInputs::terminal_json_computed_rows_projection`
  was also removed after computed refresh tests switched row-count logic to
  `retained_computed_rows`, so upstream maintainers do not need terminal JSON
  projection even for test-only row counting.
- Focused verification covered `cargo check -p worth-query`,
  `cargo test -p worth-query retained_upstreams_decode_single_computed_rows_through_query_runtime_floor --lib`,
  `cargo test -p worth-query upstream --lib`, and
  `cargo test -p worth-query --test aspect_native_query_compile_fail`.

### Runtime test native handle marker slice

- Src-hosted runtime tests now use `WORTHQueryNativeRow` for live, maintained
  derived/computed, and effect handle markers instead of `serde_json::Value`
  wherever the type parameter is only a handle marker.
- The migration covered runtime computed, live, effect, intent-admission,
  preview, live-state, mutation, graph-composition, shared-read, assembly, and
  support test modules, plus shared-read pinning hostile matrix helpers.
- Actual JSON fixture/support values remain only where tests still exercise an
  external row, terminal projection, or explicit JSON document boundary; they
  no longer leak into the generic handle marker surface.
- The handle-marker residue scans for `WORTHQueryLiveView<Value>`,
  `WORTHQueryDerivedViewHandle<Value>`, `WORTHQueryEffectHandle<Value>`,
  `declare_live_view::<Value>`, `computed::<Value>`, `effect::<Value>`, and
  maintained-derived and equivalent `serde_json::Value` handle spellings now
  return no matches across `crates/worth-query/src` and
  `crates/worth-query/tests`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib`, and `cargo test -p
  worth-query --test aspect_native_query_compile_fail`.

### Terminal JSON projection compile-fail extension

- The aspect-native compile-fail suite now proves public callers cannot call
  terminal JSON projection accessors on `WORTHQueryDerivedPatchPayload`,
  `WORTHQueryDerivedPatch`, `WORTHQueryEffectPayload`, or
  `WORTHQueryEffectDelivery`.
- These fixtures complement the existing retained-row, aspect-value,
  program-value, entity-row, and probe-field projection fences, so terminal
  JSON reporting helpers remain test-local or internal instead of public
  authority vocabulary.
- Verification covered `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail` to create the new expected diagnostics,
  followed by normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail` and `cargo check -p worth-query`.

### Mutation delta native touch constructor fence slice

- `WORTHQueryMutationDelta::from_touched_aspects` is now the public mutation
  delta constructor; the former string-path `WORTHQueryMutationDelta::new`
  constructor was removed entirely, including its cfg-test compatibility path.
- Runtime write receipts, computed patch conversion, certification fixtures,
  transcript authorities, public bridge runtime support, and src-hosted runtime
  test authorities now build deltas from retained `WORTHQueryAspectTouch`
  values instead of round-tripping through dotted path strings.
- `WORTHQueryWriteCommand::declared_aspect_touches()` exposes the retained
  declared touch state needed by receipt construction, while
  `declared_aspect_paths()` remains a terminal reporting projection.
- The aspect-native compile-fail suite now proves public callers cannot use
  the removed string constructor and are directed toward the native
  `from_touched_aspects` constructor instead.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib`, `TRYBUILD=overwrite cargo
  test -p worth-query --test aspect_native_query_compile_fail`, normal `cargo
  test -p worth-query --test aspect_native_query_compile_fail`, and a residue
  scan showing the only `WORTHQueryMutationDelta::new` reference is the
  compile-fail fixture.

### Mutation delta terminal projection naming slice

- The neutral `WORTHQueryMutationDelta::aspect_paths()` alias was removed so
  mutation deltas expose native authority through `touched_aspects()` and expose
  dotted strings only through the explicitly terminal
  `terminal_aspect_paths_projection()` accessor.
- Production digest/reporting callers and src-hosted tests were migrated to the
  terminal projection name wherever they assert or serialize path text.
- The aspect-native compile-fail suite now proves public callers cannot use the
  removed mutation-delta path alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a narrow residue scan showing the only
  mutation-delta `aspect_paths()` call is the compile-fail fixture.

### Write command declared touch alias fence slice

- The neutral `WORTHQueryWriteCommand::declared_aspect_paths()` method and its
  lowering helper were removed so declared mutation authority flows through
  `declared_aspect_touches()` and `declared_aspect_operations()`.
- Runtime-backend subject digest construction now counts native declared
  touches instead of projecting command state to path strings.
- The aspect-native compile-fail suite now proves public callers cannot ask a
  write command for declared path strings and receive the compiler hint toward
  `declared_aspect_touches()`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a narrow residue scan showing the only
  `declared_aspect_paths()` method call is the compile-fail fixture.

### Live patch terminal projection naming slice

- The neutral `WORTHQueryLivePatch::aspect_paths()` alias was removed so live
  patches expose native authority through `touched_aspects()` and expose dotted
  strings only through the explicitly terminal
  `terminal_aspect_paths_projection()` accessor.
- The aspect-native compile-fail suite now proves public callers cannot recover
  live patch path strings through the removed neutral alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a narrow residue scan showing the only
  live-patch `aspect_paths()` call is the compile-fail fixture.

### Derived patch terminal projection naming slice

- The neutral `WORTHQueryDerivedPatch::aspect_paths()` alias was removed so
  derived patches expose native authority through `aspect_touches()` and expose
  dotted strings only through the explicitly terminal
  `terminal_aspect_paths_projection()` accessor.
- The aspect-native compile-fail suite now proves public callers cannot recover
  derived patch path strings through the removed neutral alias, and the compiler
  suggests the native `aspect_touches()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a narrow residue scan showing the only
  derived-patch `aspect_paths()` call is the compile-fail fixture.

### Effect delivery terminal projection naming slice

- The neutral `WORTHQueryEffectDelivery::aspect_paths()` alias was removed so
  effect deliveries expose native authority through `aspect_touches()` and
  expose dotted strings only through the explicitly terminal
  `terminal_aspect_paths_projection()` accessor.
- Runtime effect inspection identity and intent-admission handoff binding
  digests now use the terminal projection name only at digest/reporting
  boundaries.
- The aspect-native compile-fail suite now proves public callers cannot recover
  effect delivery path strings through the removed neutral alias, and the
  compiler suggests the native `aspect_touches()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a narrow residue scan showing the only
  effect-delivery `aspect_paths()` call is the compile-fail fixture.

### Existing-truth probe request terminal projection naming slice

- The neutral `WORTHQueryExistingTruthProbeRequest::aspect_paths()` alias was
  removed so probe requests expose native authority through `aspect_touches()`
  and expose dotted strings only through
  `terminal_aspect_paths_projection()`.
- The legacy workspace-routing certification comparison now uses the explicit
  terminal projection name only at the legacy path-based execution boundary.
- The aspect-native compile-fail suite now proves public callers cannot recover
  probe request path strings through the removed neutral alias, and the compiler
  suggests the native `aspect_touches()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a narrow residue scan showing the only
  probe-request `aspect_paths()` call is the compile-fail fixture.

### Preview execution evidence terminal projection naming slice

- The neutral `WORTHQueryPreviewExecutionEvidence::aspect_paths()` alias was
  removed so preview execution evidence exposes native authority through
  `aspect_touches()` and exposes dotted strings only through
  `terminal_aspect_paths_projection()`.
- The aspect-native compile-fail suite now proves public callers cannot recover
  preview execution path strings through the removed neutral alias, and the
  compiler suggests the native `aspect_touches()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a broad neutral-alias scan confirming
  the removed `aspect_paths()` public aliases only remain in compile-fail
  fixtures.

### Read touch row native emptiness slice

- Internal read-obligation dispatch now checks
  `WORTHQueryGraphTouchReadRow::aspect_touches().is_empty()` when deciding
  whether a read observes aspect paths, instead of projecting the read touch
  row to dotted strings first.
- The neutral `WORTHQueryGraphTouchReadRow::aspect_paths()` alias was removed;
  dotted strings remain available only through
  `terminal_aspect_paths_projection()` for explicit reporting/projection uses.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`, and a targeted scan
  proving the read-obligation dispatch no longer calls the removed alias.

### Write command touched path alias fence slice

- The neutral `WORTHQueryWriteCommand::touched_aspect_paths()` method was
  removed so touched mutation authority flows through `touched_aspects()`.
- Dotted strings remain available only through the explicitly terminal
  `terminal_touched_aspect_paths_projection()` accessor.
- The aspect-native compile-fail suite now proves public callers cannot recover
  touched path strings through the removed neutral alias, and the compiler
  suggests the native `touched_aspects()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a focused residue scan showing the
  only `WORTHQueryWriteCommand::touched_aspect_paths()` call is the compile-fail
  fixture.

### Retained refresh context touched path alias fence slice

- The neutral `WORTHQueryRetainedRefreshContext::touched_aspect_paths()` method
  was removed so retained refresh consumers use `touched_aspects()` for native
  authority and the explicit
  `terminal_touched_aspect_paths_projection()` accessor for reporting text.
- The only src-hosted computed test caller was migrated to the terminal
  projection name because that assertion intentionally formats dotted paths into
  a diagnostic row value.
- The aspect-native compile-fail suite now proves public callers cannot recover
  retained refresh path strings through the removed neutral alias, and the
  compiler suggests the native `touched_aspects()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a focused residue scan showing the
  only retained-refresh `touched_aspect_paths()` call is the compile-fail
  fixture.

### Verified assumption asserted path alias fence slice

- The neutral `WORTHQueryVerifiedAssumptionSet::asserted_aspect_paths()` method
  was removed so verified assumption consumers use `asserted_aspects()` for
  native authority and the explicit
  `terminal_asserted_aspect_paths_projection()` accessor for reporting text.
- The two bridge-backed verification execution assertions were migrated to the
  terminal projection name because they intentionally compare dotted path text.
- The aspect-native compile-fail suite now proves public callers cannot recover
  asserted path strings through the removed neutral alias, and the compiler
  suggests the native `asserted_aspects()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a focused residue scan showing the
  only `asserted_aspect_paths()` call is the compile-fail fixture.

### Batch write receipt touched path alias fence slice

- The neutral `WORTHQueryBatchWriteReceipt::touched_aspect_paths()` method was
  removed so batch receipt consumers use `touched_aspects()` for native
  authority and the explicit `terminal_touched_aspect_paths_projection()`
  accessor for reporting text.
- Batch receipt assertions and aspect API certification rows were migrated to
  the terminal projection name because those paths intentionally feed dotted
  path reporting/digest evidence.
- The aspect-native compile-fail suite now proves public callers cannot recover
  batch receipt path strings through the removed neutral alias, and the
  compiler suggests the native `touched_aspects()` accessor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a focused residue scan showing receipt
  call sites no longer use the removed alias.

### Batch write inspection touched path alias fence slice

- The neutral
  `WORTHQueryBatchWriteReceiptInspection::touched_aspect_paths()` and
  `WORTHQueryBatchWriteComponentInspection::touched_aspect_paths()` methods
  were removed so inspection consumers use `touched_aspects()` for native
  authority and explicit `terminal_touched_aspect_paths_projection()` accessors
  for reporting text.
- The batch and graph-composition assertions that intentionally compare dotted
  path text were migrated to terminal projection names.
- The aspect-native compile-fail suite now proves public callers cannot recover
  batch/component inspection path strings through the removed neutral aliases,
  and the compiler suggests the native `touched_aspects()` accessors.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a focused residue scan showing the
  public aliases only remain in compile-fail fixtures.

### Graph touch descriptor row internal path alias cleanup

- The internal neutral
  `WORTHQueryGraphTouchDescriptorRow::touched_aspect_paths()` alias was removed
  so graph touch descriptor row consumers use `touched_aspects()` for native
  authority or the explicit `terminal_touched_aspect_paths_projection()` helper
  for reporting text.
- No public compile-fail fixture was added because the row alias is crate-local
  implementation vocabulary; ordinary compilation enforces any remaining
  internal use.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, and a focused residue scan confirming the removed neutral
  `touched_aspect_paths()` / `asserted_aspect_paths()` aliases only remain in
  aspect-native compile-fail fixtures.

### Effect payload neutral aspect alias fence slice

- The neutral `WORTHQueryEffectPayload::input_aspects()`,
  `WORTHQueryEffectPayload::output_aspects()`, and
  `WORTHQueryEffectPayload::changed_aspects()` string accessors were removed so
  effect payload consumers use `*_aspect_touches()` for native authority and
  explicit `terminal_*_aspects_projection()` accessors for reporting text.
- `WORTHQueryEffectPayload` now mirrors the effect declaration vocabulary:
  input/output aspect strings are terminal projections, while changed aspects
  retain the existing terminal changed-aspects projection.
- The aspect-native compile-fail suite now proves public callers cannot recover
  effect payload path strings through the removed neutral aliases, and the
  compiler suggests the native touch accessors.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a focused residue scan showing the
  removed neutral effect payload aliases only remain in the compile-fail
  fixture.

### Derived view neutral aspect alias fence slice

- The neutral `WORTHQueryDerivedView::dependency_aspects()` and
  `WORTHQueryDerivedView::produced_aspects()` string accessors were removed so
  derived declarations expose native authority through
  `dependency_aspect_touches()` / `produced_aspect_touches()`.
- Dotted strings remain available only through explicit
  `terminal_dependency_aspects_projection()` and
  `terminal_produced_aspects_projection()` reporting helpers.
- Preview routing and derived maintainer test helpers now keep
  `WORTHQueryAspectTouch` values alive instead of projecting strings and
  re-admitting them.
- The neutral `WORTHQueryComputedInspectionEvidence::dependency_aspects()` and
  `WORTHQueryComputedInspectionEvidence::produced_aspects()` inspection aliases
  were removed with the same native-touch / terminal-projection split.
- The aspect-native compile-fail suite now proves public callers cannot recover
  derived declaration or computed inspection path strings through the removed
  neutral aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a focused residue scan showing the
  removed neutral aliases only remain in compile-fail fixtures.

### Effect inspection trigger aspect alias fence slice

- `WORTHQueryEffectInspectionEvidence` now stores trigger aspects as
  `WORTHQueryAspectTouch` values instead of terminal strings.
- The neutral `WORTHQueryEffectInspectionEvidence::trigger_aspects()` accessor
  was removed so inspection consumers use `trigger_aspect_touches()` for native
  authority and `terminal_trigger_aspects_projection()` for reporting text.
- Preview relevance matching now makes its terminal string comparison explicit
  at the matching boundary instead of receiving string authority from the
  inspection evidence.
- The aspect-native compile-fail suite now proves public callers cannot recover
  trigger aspect path strings through the removed neutral alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and focused inspection-residue scans.

### Effect inspection condition aspect alias fence slice

- `WORTHQueryEffectInspectionEvidence` now stores condition input/output
  aspects as `WORTHQueryAspectTouch` values instead of terminal strings.
- The neutral `condition_inputs()` and `condition_outputs()` accessors were
  removed so inspection consumers use `condition_input_touches()` /
  `condition_output_touches()` for native authority and
  `terminal_condition_*_projection()` for reporting text.
- Effect inspection digests still encode the same terminal path text, but the
  projection now happens inside the digest/reporting boundary instead of being
  retained as inspection authority.
- The aspect-native compile-fail suite now proves public callers cannot recover
  condition aspect path strings through the removed neutral aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and focused condition-residue scans.

### Diagnostic aspect path alias fence slice

- Denial, probe, and symbolic-resolution surfaces no longer expose dotted
  aspect paths through neutral `*_aspect_path()` aliases:
  `WORTHQueryExistingTruthAssertionDenial::asserted_aspect_path()`,
  `WORTHQueryExistingTruthProbeDenial::probed_aspect_path()`,
  `WORTHQueryExistingTruthProbeField::aspect_path()`,
  `WORTHQuerySymbolicAspectResolutionEvidence::aspect_path()`, and
  `WORTHQueryGraphCompositionResolutionEntry::aspect_path()` were removed.
- Dotted strings remain available only through explicit terminal projection
  names such as `terminal_asserted_aspect_path_projection()`,
  `terminal_probed_aspect_path_projection()`, and
  `terminal_aspect_path_projection()`.
- Batch write digest helpers, graph composition evidence, graph-composition
  support assertions, and existing-truth denial assertions now call the terminal
  projection names at reporting/digest boundaries.
- The native/proof side remains intact: symbolic aspect resolution and graph
  composition resolution entries still expose `WORTHQueryAspectTouch` through
  `aspect_touch()` where callers need authority rather than display text.
- The aspect-native compile-fail suite now proves public callers cannot recover
  these diagnostic strings through the removed neutral aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and focused residue scans showing the old
  aliases only remain in compile-fail fixtures or native touch primitives.

### Graph read proof-row key alias fence slice

- Graph-read admitted proof rows and field-authority rows no longer expose
  foundational aspect/field keys through neutral `aspect()` / `field()`
  aliases.
- `WORTHQueryGraphReadPredicateFieldAuthority`,
  `WORTHQueryGraphReadOrderingFieldAuthority`,
  `WORTHQueryAdmittedGraphReadProjectionField`,
  `WORTHQueryAdmittedGraphReadPredicateField`,
  `WORTHQueryAdmittedGraphReadOrderingField`,
  `WORTHQueryAdmittedBooleanPredicateLeaf`, and
  `WORTHQueryBooleanPredicateSelectivityRow` now reserve authority access for
  `native_aspect_key()` / `native_field_key()` and expose strings only through
  explicit `terminal_aspect_key_projection()` /
  `terminal_field_key_projection()` names.
- Graph-read schema and boolean admission internals were adjusted to use native
  keys when indexing admitted proof rows; authoring request fields still keep
  their ergonomic `aspect()` / `field()` input vocabulary because they are not
  admitted proof rows.
- The aspect-native compile-fail suite now proves public callers cannot recover
  graph-read proof-row strings through the removed neutral key aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and focused scans showing graph-read proof
  rows no longer define neutral `aspect()` / `field()` accessors.

### Retained scalar field key alias fence slice

- Retained scalar fact and alignment fact surfaces no longer expose retained
  field paths through neutral `field_key()`, `left_field_key()`, or
  `right_field_key()` aliases.
- `WORTHQueryRetainedScalarFieldFact` keeps native authority available through
  `field_path()` and exposes dotted reporting text only through
  `terminal_field_key_projection()`.
- `WORTHQueryRetainedScalarAlignmentFact` keeps native left/right retained
  field paths available through `left_field_path()` / `right_field_path()` and
  exposes dotted reporting text only through explicit terminal projection
  accessors.
- Retained scalar fact-set and alignment digests now call the terminal
  projection names at digest/reporting boundaries.
- The aspect-native compile-fail suite now proves public callers cannot recover
  retained scalar field strings through the removed neutral aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and focused scans showing the removed
  aliases only remain in compile-fail fixtures.

### Aspect value path alias fence slice

- `WORTHQueryAspectValue` no longer exposes its parsed mutation target through
  the neutral `aspect_path()` alias.
- Native authority is available through `aspect_touch()`, while dotted path
  reporting remains available only through
  `terminal_aspect_path_projection()`.
- Memory-workspace lowering, mutation digest rows, existing-truth test
  adapters, and public bridge support now call the terminal projection name
  only at legacy/external-row or digest/reporting boundaries.
- The aspect-native compile-fail suite now proves public callers cannot recover
  a dotted aspect path from `WORTHQueryAspectValue` through the removed neutral
  alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo test -p worth-query --lib --no-run`,
  `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and focused scans showing the removed
  alias only remains in the compile-fail fixture.

### Aspect mutation carrier path alias fence slice

- `WORTHQueryAspectMutationOperation` no longer exposes its native touch
  through the neutral `aspect_path()` alias.
- `WORTHQuerySymbolicAspectReference` now exposes `aspect_touch()` publicly for
  native authority and reserves dotted path reporting for
  `terminal_aspect_path_projection()`.
- Mutation seed identity, bridge writeback identity, write-receipt digest,
  batch digest, graph touch descriptor, and graph-obligation selector code now
  call terminal projection names only at digest/reporting/key projection
  boundaries.
- Graph obligation selector matching compares required touches through native
  `WORTHQueryAspectTouch` authority instead of re-checking declared operation
  coverage through dotted strings.
- The aspect-native compile-fail suite now proves public callers cannot recover
  a dotted aspect path from `WORTHQueryAspectMutationOperation` or
  `WORTHQuerySymbolicAspectReference` through the removed neutral aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, `cargo test -p worth-query --lib
  --no-run`, and focused scans showing mutation operation / symbolic aspect
  reference `aspect_path()` call sites are gone.

### Entity aspect-key accessor native fence slice

- `WORTHQueryEntity::aspect_value` now requires a foundational `AspectKey`
  instead of accepting a raw string lookup.
- `WORTHQueryEntity::aspect_values` now iterates native `AspectKey` values
  instead of projecting each key to `&str` at the public row boundary.
- Projection-consumption extraction explicitly performs terminal projection at
  its field-path lowering boundary, while the memory workspace and entity row
  keep native aspect authority internally.
- The aspect-native compile-fail suite now proves public callers cannot use the
  old string lookup or treat `aspect_values()` as a `(&str, &AspectValue)`
  iterator.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, `cargo test -p worth-query --lib
  --no-run`, and focused scans of entity aspect accessor call sites.

### Entity external field-path accessor native fence slice

- `WORTHQueryEntity::external_scalar_value` now requires a foundational
  `CanonicalFieldPath` instead of accepting dotted terminal field strings.
- The public `external_aspect_value(&str)` alias was removed so retained entity
  rows no longer teach callers that external projection fields are recovered
  through ad hoc dotted string lookup.
- Read-composition materialization, projection-consumption extraction, memory
  workspace tests, and consumer-kit backend behavior tests now construct native
  field paths before accessing retained external projection values.
- The aspect-native compile-fail suite now proves public callers cannot use the
  old dotted-string external scalar lookup or the removed external aspect alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and `cargo test -p worth-query --lib
  --no-run`.

### Grouped artifact grouping-aspect alias fence slice

- Grouped planning and grouped live proof artifacts no longer expose their
  foundational grouping `AspectKey` through the neutral `grouping_aspect()`
  alias.
- `GroupedBaselineMaterializationContract`,
  `GroupedViewPlanningArtifact`, `GroupedLaneIdentity`,
  `GroupedViewResultArtifact`, `GroupedDesiredStateArtifact`, and
  `GroupedExecutionLaneValue` now reserve authority access for
  `native_grouping_aspect_key()` and expose strings only through
  `terminal_grouping_aspect_projection()`.
- Grouped baseline, grouped execution, view-shape tests, and milestone-eight
  certification harness code now call terminal projection names only where
  producing reporting text or bridge projection contract arguments.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed grouped planning/live `grouping_aspect()` aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and `cargo test -p worth-query --lib
  --no-run`.

### Projection consumption fact alias fence slice

- `ProjectionFactRequest` and `BoundProjectionFactFamily` no longer expose
  field-backed fact authority through the neutral `field_key()` alias.
- `ConsumedFieldValueFact` no longer exposes its `ProjectionFactFieldPath`
  through `field_key()`.
- `ConsumedMembershipFact` and grouped `ConsumedRelationEndpointFact` no longer
  expose grouping authority through the neutral `grouping_aspect()` alias.
- Projection-consumption digest, eligibility, extraction, certification, and
  tests now use `field_path()` plus explicit `terminal_projection()` only at
  digest/reporting/string compatibility boundaries.
- `ProjectionFactFieldPath` now exposes its validated `CanonicalFieldPath`,
  allowing live/read entity scalar extraction to call the typed
  `WORTHQueryEntity::external_scalar_value` path directly instead of reparsing
  dotted text.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed projection-consumption `field_key()` and `grouping_aspect()`
  aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, and `cargo test -p worth-query
  --lib --no-run`.

### Grouped binding proof alias fence slice

- `QueryResultBindingProof` now stores its source binding as an
  `AspectFieldKey` instead of separate neutral source-aspect/source-field
  strings.
- The grouped binding proof no longer exposes `source_aspect()`,
  `source_field()`, or `field_key()` aliases.
- Policy-influence derivation consumes the typed `source_field_key()` directly,
  while grouped plan digesting, grouped execution mismatch text, and tests use
  the explicit `terminal_binding_aspect_projection()` reporting boundary.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed grouped binding string aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, and `cargo test -p worth-query
  --lib --no-run`.

### View-shape focus and delivery aspect alias fence slice

- `ViewShapeDescriptor`, `ViewShapeDeliveryMetadata`, and
  `FocusedInspectorAspectPatchArtifact` now store focused inspector aspects as
  foundational `AspectKey` values instead of `String`.
- Descriptor and delivery grouped aspect text now flows through explicit
  terminal projection accessors instead of the neutral `grouping_aspect()`
  alias.
- The removed public aliases are `ViewShapeDescriptor::focused_aspect()`,
  `ViewShapeDescriptor::grouping_aspect()`,
  `ViewShapeDeliveryMetadata::focus_aspect()`,
  `ViewShapeDeliveryMetadata::grouping_aspect()`, and
  `FocusedInspectorAspectPatchArtifact::focus_aspect()`.
- Admission, planning, live execution, plan digesting, and view-shape tests now
  use native focus/grouping keys internally and terminal projection names only
  for digest/reporting strings.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed view-shape aspect aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, and `cargo test -p worth-query
  --lib --no-run`.

### Validation proof-entry key alias fence slice

- `ValidatedProjectionEntry`, `ValidatedResultShapeBinding`,
  `ValidatedPredicateEntry`, and `ValidatedOrderingEntry` now store
  foundational `AspectKey` / `FieldKey` values instead of authoring
  `AspectName` / `FieldName` pairs.
- The validated proof entries no longer expose neutral `aspect()`, `field()`,
  `source_aspect()`, or `source_field()` aliases.
- Collection planning, live relevance contracts, read scope classification, and
  grouped view planning now use native validation keys internally. The
  validation certification harness uses native source keys for binding checks.
  Terminal projection names appear only when adapting to legacy string-shaped
  constructors or digest/reporting text.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed validated-entry string aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, and `cargo test -p worth-query
  --lib --no-run`.

### Collection ordering key-path alias fence slice

- `OrderingKeyPath` now stores foundational `AspectKey` / `FieldKey` values
  instead of separate neutral ordering aspect/field strings.
- The exported collection ordering proof no longer exposes neutral
  `aspect()` or `field()` aliases.
- Collection ordering digest generation uses explicit terminal projection
  accessors. Validated ordering entries still lower ergonomically, but they are
  converted into native keys at the collection planning boundary.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed collection ordering key-path aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, and `cargo test -p worth-query
  --lib --no-run`.

### Live query field-key alias fence slice

- `QueryFieldKey` now stores foundational `AspectKey` / `FieldKey` values
  instead of separate neutral live query aspect/field strings.
- The exported live relevance and patch proof key no longer exposes neutral
  `aspect()` or `field()` aliases.
- Live relevance matching still accepts bridge field deltas at the terminal
  ingress boundary, but live proof artifacts, detail patch digesting, and
  view-shape focus filtering now consume native query field keys internally.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed live query field-key aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, and `cargo test -p worth-query
  --lib --no-run`.

### Bridge field delta key native fence slice

- `BridgeFieldDelta` still accepts terminal aspect/field text at bridge ingress,
  but now immediately retains that field identity as a `QueryFieldKey`.
- Public bridge delta consumers use `field_key()` to inspect the native field
  carrier. The old `aspect()`/`field()` string aliases were removed entirely
  instead of being kept as crate-local compatibility.
- Live relevance matching and patch extraction now compare/clone the retained
  `QueryFieldKey`, so bridge deltas no longer reconstruct live patch field
  identity from raw aspect/field strings after construction.
- The aspect-native compile-fail suite now proves facade callers cannot use the
  removed `BridgeFieldDelta::aspect()` or `BridgeFieldDelta::field()` aliases,
  and the compiler suggests `field_key()` for the field case.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query live --lib`, focused `cargo test -p
  worth-query view_shape_live --lib`, line-count checks noting `live/mod.rs`
  is already allowlisted, and focused scans confirming `BridgeFieldDelta`
  exposes `field_key()` rather than public string aliases.

### Authorized projection field-bag native fence slice

- `AuthorizedProjectionFieldPath` now stores foundational `AspectKey` /
  `FieldKey` values plus an explicit terminal projection for reporting and
  bridge-shaped ingress boundaries.
- Authorized projection visible fields, masked fields, non-disclosing fields,
  policy-aware delivery fields, optimizer visible fields, and projection
  consumption binding visible fields now retain
  `AuthorizedProjectionFieldPath` instead of `Vec<String>` field bags.
- The neutral public string-bag accessors were removed:
  `visible_fields()`, `masked_fields()`, `non_disclosing_fields()`,
  `delivered_fields()`, and `authorized_visible_fields()`. Native access now
  flows through `*_field_paths()` names.
- Live relevance and placeholder delivery requests still accept terminal
  strings where bridge/runtime events arrive that way, but comparisons happen
  against typed authorized field paths and terminal projection is explicit at
  the boundary.
- Certification fixtures, policy-plan density checks, projection-consumption
  fixtures, and public bridge/entity tests were migrated to construct typed
  field paths before satisfying proof-bearing APIs.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed authorized projection string-bag accessors.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, `cargo test -p
  worth-query --lib --no-run`, and focused scans showing the removed
  authorized projection field-bag aliases only remain in the compile-fail
  fixture.

### Runtime derived materialization result-boundary alias fence slice

- `WORTHQueryRuntime::read_derived(...)` was removed so runtime callers cannot
  bypass the explicit `Result`-bearing derived materialization boundary.
- Runtime callers now use `read_derived_result(...)` and must either propagate
  `WORTHQueryRuntimeError` or make the expectation explicit at test/assertion
  sites.
- The retained materialization carrier remains unchanged:
  `read_derived_result(...)` returns `WORTHQueryDerivedMaterializationResult`,
  whose rows are retained native materialization proof rows rather than
  terminal JSON rows.
- Runtime computed/effect/intent/preview tests were migrated to the explicit
  result API while preserving the same retained-row assertions.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed unchecked runtime derived materialization alias, and the compiler
  points them to `read_derived_result(...)`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, `cargo test -p
  worth-query --lib --no-run`, and focused scans showing the removed
  `read_derived(...)` alias only remains in the compile-fail fixture.

### Computed patch observation handle fence slice

- `WORTHQueryRuntime::drain_derived_patches(...)` and
  `WORTHQueryWorkspace::observe_computed(...)` now require
  `&WORTHQueryDerivedViewHandle<_>` instead of raw derived view-name text.
- The raw name drain remains crate-local as
  `drain_derived_patches_by_name(...)` only for internal runtime routing code
  that already owns declared view names.
- Computed, preview, and workspace support tests were migrated to carry the
  derived handle through patch observation instead of projecting `handle.name()`
  and reusing the string as authority.
- One declaration-only computed test now creates a runtime-scoped derived handle
  after declaration, making the proof handoff explicit even where the test uses
  the lower-level declaration API.
- The aspect-native compile-fail suite now proves public callers cannot drain
  or observe computed patches by arbitrary raw view-name strings.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite cargo
  test -p worth-query --test aspect_native_query_compile_fail`, normal `cargo
  test -p worth-query --test aspect_native_query_compile_fail`, focused `cargo
  test -p worth-query computed --lib`, line-count checks, and focused scans
  proving raw-name computed patch observation only remains in the compile-fail
  fixture.

### Workspace materialization result-boundary alias fence slice

- `WORTHQueryWorkspace::materialize(...)` was removed so workspace callers
  cannot bypass the explicit derived materialization result boundary.
- Workspace callers now use `materialize_result(...)` when they need the
  convenience retained materialization result, or `materialize_intent(...)`
  when they need the full intent admission/review/admit/execute ladder.
- The retained materialization carrier remains unchanged:
  `materialize_result(...)` returns `WORTHQueryDerivedMaterializationResult`
  with retained native materialization rows rather than terminal JSON rows.
- Runtime tests, aspect API certification rows, intent-admission surface
  inventories, intent-admission doc examples, runtime API closeout wording, and
  current user-facing docs were migrated away from the deleted shortcut.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed unchecked workspace materialization alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, `cargo test -p
  worth-query --lib --no-run`, and focused scans showing the removed
  `workspace.materialize(&...)` alias only remains in the compile-fail fixture.

### Existing-truth probe field lookup native fence slice

- `WORTHQueryExistingTruthProbe::field(&str)` was removed so public and runtime
  callers cannot reparse raw dotted aspect paths when reading verified existing
  truth.
- Probe field lookup now flows through
  `field_for_touch(&WORTHQueryAspectTouch)`, carrying the already-parsed native
  aspect proof state from request construction into result consumption.
- Runtime mutation, intent-admission, mixed-authority, bridge-backed
  verification, and public bridge bootstrap tests were migrated to use typed
  touches for probe field assertions.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed raw-string probe lookup alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused runtime tests
  for existing-truth probe and public bridge bootstrap lookup, and focused
  scans showing the removed `probe.field("...")` alias only remains in the
  compile-fail fixture.

### Live relevance authorized field native fence slice

- `PolicyAwareLiveRelevanceContract` now stores authorized relevance fields as
  `AuthorizedProjectionFieldPath` values instead of a `Vec<String>` string bag.
- `admit_policy_aware_live_plan(...)` still accepts requested relevance fields
  as terminal ingress text, but once those requests are admitted against the
  authorized projection, the retained relevance proof carries typed field paths.
- The neutral public `authorized_fields()` accessor was removed; callers now use
  `authorized_field_paths()` and must make terminal projection explicit when
  they need reporting text.
- Live density/drift certification and policy-live tests were migrated to
  consume the typed field paths for width/proof assertions.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed live relevance string-bag alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query policy_live`, and focused scans showing the removed
  `authorized_fields()` alias only remains in the compile-fail fixture.

### Placeholder masking terminal request naming slice

- `PolicyPlaceholderMaskingRequest` and `PolicyPlaceholderMaskingDenial` no
  longer expose the neutral `requested_placeholder_fields()` string-bag alias.
- Placeholder masking remains a terminal request boundary because callers ask
  for placeholder fields by delivered field text, but production code must now
  use `terminal_requested_placeholder_fields_projection()` so the boundary is
  explicit.
- `PolicyPlaceholderMaskingRequest::new(...)` was removed for the same reason:
  the constructor is now named
  `terminal_requested_placeholder_fields(...)`, making it mechanically visible
  that this is terminal compatibility text rather than native aspect authority.
- Masking checks continue to compare requested terminal text against typed
  masked `AuthorizedProjectionFieldPath` values from the narrowed authorized
  projection.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed placeholder request/denial string-bag aliases or the removed
  neutral request constructor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite cargo
  test -p worth-query --test aspect_native_query_compile_fail`, normal `cargo
  test -p worth-query --test aspect_native_query_compile_fail`, focused `cargo
  test -p worth-query policy_delivery`, line-count checks, and focused scans
  showing the removed `requested_placeholder_fields()` aliases and neutral
  `PolicyPlaceholderMaskingRequest::new(...)` constructor only remain in
  compile-fail fixtures.

### Retained scalar fact-set field lookup native fence slice

- `WORTHQueryRetainedScalarFactSet::field_value(&str)` was removed so public
  callers cannot reparse raw dotted retained field paths when reading retained
  scalar facts.
- Retained scalar fact-set lookups now flow through
  `field_value_at(&WORTHQueryRetainedFieldPath)`, keeping the admitted retained
  field-path proof state as the lookup authority.
- The retained scalar fact-set nested-field test now constructs retained field
  paths once and reads through the typed lookup surface.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed raw-string retained scalar fact-set lookup alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query retained_scalar_fact_set_reads_nested_fields`, file line-count
  checks, and focused scans showing the removed `field_value("...")` alias only
  remains in the compile-fail fixture.

### Artifact bundle target containment native fence slice

- `WORTHQueryDerivedMaterializationBundle::includes_view_name(&str)` and
  `WORTHQueryLiveArtifactBundle::includes_view_name(&str)` were removed so
  public callers cannot test bundle containment through raw view-name strings.
- Derived and live artifact bundle containment now flows through
  `includes_target(...)` with `WORTHQueryDerivedMaterializationTarget` or
  `WORTHQueryLiveArtifactTarget`, preserving the admitted target carrier
  through binding validation.
- Derived and live artifact binding internals now sort and deduplicate target
  carriers first, then derive terminal view-name text only for reporting,
  digests, and error messages.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed raw-string bundle containment aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, line-count checks, and
  focused scans showing the removed `includes_view_name(...)` aliases are gone
  outside the compile-fail fixture.

### Graph touch descriptor aspect lookup native fence slice

- `WORTHQueryGraphTouchDescriptor::touches_aspect_path(&str)` was removed so
  graph touch descriptor consumers cannot query mutation graph evidence through
  raw dotted aspect path text.
- Descriptor containment now flows through
  `touches_aspect(&WORTHQueryAspectTouch)`, comparing retained touched aspects
  and declared aspect operations through the native touch carrier.
- Graph obligation touch selector matching now passes its retained
  `WORTHQueryAspectTouch` directly into descriptor containment instead of
  projecting it to terminal text and reparsing the boundary mentally.
- Runtime and descriptor identity tests now assert descriptor aspect containment
  through typed touches.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed raw-string graph touch descriptor lookup alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query graph_touch_descriptor --lib`, focused `cargo test -p
  worth-query touch_descriptor --lib`, line-count checks, and focused scans
  showing the removed `touches_aspect_path(...)` alias only remains in the
  compile-fail fixture.

### Graph touch descriptor collection lookup native fence slice

- `WORTHQueryGraphTouchDescriptor::touches_collection(&str)` is no longer a
  public descriptor lookup surface.
- Public descriptor collection containment now flows through
  `touches_target_collection(&WORTHQueryMutationTargetCollectionIdentity)`, so
  callers must carry a mutation target collection proof token instead of
  probing descriptor evidence with arbitrary collection text.
- Graph touch descriptor identity and runtime descriptor tests now assert
  collection containment through typed target collection identities.
- The raw `touches_collection(&str)` helper remains crate-private only for
  internal graph obligation selector matching, where selector admission still
  owns the terminal collection text.
- The aspect-native compile-fail suite now proves public callers cannot call
  the removed raw-string descriptor collection lookup.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query touch_descriptor --lib`, focused `cargo test -p worth-query
  graph_touch_descriptor --lib`, line-count checks, and focused scans showing
  raw descriptor collection lookup is crate-private internal use or the
  compile-fail fixture.

### Mutation metadata key lookup native fence slice

- Added `WORTHQueryMutationMetadataKey` as the admitted key token for mutation
  metadata lookup.
- `WORTHQueryMutationMetadata::get(...)` now requires
  `&WORTHQueryMutationMetadataKey` instead of `&str`, so public receipt,
  inspection, and retained refresh consumers cannot re-read mutation metadata
  with arbitrary ambient string keys.
- Metadata authoring remains ergonomic at mutation-builder ingress, but the
  insertion path now lowers through the same metadata key admission token used
  by lookup.
- Runtime metadata inspection and refresh-maintainer tests now construct typed
  metadata keys before reading retained metadata values.
- The aspect-native compile-fail suite now proves public callers cannot call
  mutation metadata lookup with a raw string.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query write_receipt_inspection_retains_authored_mutation_metadata
  --lib`, focused `cargo test -p worth-query delete_with --lib`, focused
  `cargo test -p worth-query
  refresh_fallback_maintainer_receives_retained_mutation_metadata --lib`,
  line-count checks, and focused scans showing runtime metadata string lookup
  only remains in the compile-fail fixture.

### Mutation metadata entries raw-map fence slice

- `WORTHQueryMutationMetadata::entries()` no longer exposes
  `&BTreeMap<String, WORTHQueryMutationMetadataValue>` as a public raw storage
  map.
- Public metadata iteration now yields
  `(WORTHQueryMutationMetadataKey, &WORTHQueryMutationMetadataValue)` so
  receipt, inspection, and backend reporting consumers keep the admitted
  metadata-key proof state instead of recovering arbitrary string keys from the
  storage container.
- Backend mutation diagnostics, causal receipt helpers, and unified
  write-receipt digest construction now consume the typed iterator and project
  key text only at explicit reporting/digest boundaries.
- The aspect-native compile-fail suite now proves external callers cannot bind
  `metadata.entries()` to the old raw `BTreeMap<String, ...>` shape.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query
  write_receipt_inspection_retains_authored_mutation_metadata --lib`, focused
  `cargo test -p worth-query delete_with --lib`, focused `cargo test -p
  worth-query
  refresh_fallback_maintainer_receives_retained_mutation_metadata --lib`,
  line-count checks, and focused raw-map scans showing the old metadata entries
  shape only remains in the compile-fail fixture.

### Declarative projection field key native fence slice

- `DeclarativeProjectionField` now retains its source field as an
  `AspectFieldKey` instead of separate aspect/field strings.
- The public native route is `source_field_key()`, preserving the admitted
  aspect+field proof carrier for facade consumers and downstream declarative
  live/read-composition planning.
- The old neutral `aspect()` and `field()` aliases are no longer public; they
  remain crate-local terminal projections only for internal lowering and digest
  compatibility while the rest of the declarative live cluster is migrated.
- The aspect-native compile-fail suite now proves external callers cannot use
  the old raw string accessors on `DeclarativeProjectionField`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query declarative --lib`, focused `cargo test
  -p worth-query read_composition --lib`, line-count checks noting
  `declarative_live.rs` is already allowlisted, and focused scans confirming
  this slice removed the projection-field public string aliases while other
  declarative field/order/filter aliases remain future work.

### Declarative ordering field key native fence slice

- `DeclarativeOrderingField` now retains its source field as an
  `AspectFieldKey` instead of separate aspect/field strings.
- The public native route is `source_field_key()`, preserving the admitted
  aspect+field proof carrier for facade consumers and read-composition
  planning that orders declarative live results.
- The old neutral `aspect()` and `field()` aliases are no longer public; they
  remain crate-local terminal projections only for internal lowering and
  compatibility while declarative filter/writeback aliases are migrated.
- The aspect-native compile-fail suite now proves external callers cannot use
  the old raw string accessors on `DeclarativeOrderingField`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query declarative --lib`, focused `cargo test
  -p worth-query read_composition --lib`, line-count checks noting
  `declarative_live.rs` is already allowlisted, and focused scans confirming
  this slice removed the ordering-field public string aliases while other
  declarative filter/writeback aliases remain future work.

### Declarative predicate filter key native fence slice

- `DeclarativeEqualityFilter`, `DeclarativeIntegerComparisonFilter`,
  `DeclarativeStringContainsFilter`, `DeclarativeSetMembershipFilter`,
  `DeclarativePresenceFilter`, and the `DeclarativePredicateFilter` enum now
  retain/expose predicate source fields through `AspectFieldKey`.
- The public native route is `source_field_key()`, preserving the admitted
  aspect+field proof carrier for facade consumers and read-composition
  predicate planning.
- The old neutral `aspect()` and `field()` aliases are no longer public on any
  declarative predicate filter; they remain crate-local terminal projections
  only for internal canonical-query lowering and compatibility while branch
  compare/writeback aliases are migrated.
- The aspect-native compile-fail suite now proves external callers cannot use
  the old raw string accessors on concrete declarative predicate filters or the
  predicate enum wrapper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query declarative --lib`, focused `cargo test
  -p worth-query read_composition --lib`, line-count checks noting
  `declarative_live.rs` is already allowlisted, and focused scans confirming
  the remaining public `aspect()`/`field()` aliases in `declarative_live.rs`
  are branch-compare and writeback surfaces, not predicate filters.

### Declarative branch-compare field key native fence slice

- `DeclarativeBranchCompareValue` and `DeclarativeBranchCompareFieldDelta`
  now retain branch-compare fields through `AspectFieldKey` instead of
  separate aspect/field strings.
- Branch comparison joins left/right row values with typed `AspectFieldKey`
  keys, so delta construction no longer splits dotted strings to recover field
  identity.
- The public native route is `source_field_key()`. The old neutral
  `aspect()`/`field()` aliases are gone from compare values and no longer
  public on compare deltas; delta string projection remains crate-local only
  for digest compatibility.
- The aspect-native compile-fail suite now proves external callers cannot use
  the old raw string accessors on branch-compare values or deltas.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query declarative --lib`, line-count checks
  noting `declarative_live.rs` is already allowlisted, and focused scans
  confirming the remaining public `aspect()`/`field()` aliases in
  `declarative_live.rs` are the writeback surface.

### Declarative writeback change field key native fence slice

- `DeclarativeWritebackChange` now retains its source field through
  `AspectFieldKey` instead of separate aspect/field strings.
- The public native route is `source_field_key()`, preserving the admitted
  aspect+field proof carrier for facade consumers and declarative writeback
  artifacts.
- The old neutral `aspect()` and `field()` aliases are no longer public on
  writeback changes; they remain crate-local terminal projections only for
  digest compatibility.
- The aspect-native compile-fail suite now proves external callers cannot use
  the old raw string accessors on `DeclarativeWritebackChange`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query writeback --lib`, focused `cargo test
  -p worth-query declarative --lib`, line-count checks noting
  `declarative_live.rs` is already allowlisted, and focused scans confirming
  no public `aspect()`/`field()` aliases remain in `declarative_live.rs`.

### Authoring selector field key native fence slice

- `AspectFieldSelector`, `OrderingSelector`, `AuthoredResultShapeField`, and
  every concrete `PredicateSelector` carrier already retained `AspectFieldKey`;
  their public facade now exposes that retained proof carrier through
  `source_field_key()` or `target_field_key()`.
- The old neutral `aspect()`/`field()` aliases on projection, ordering, and
  predicate selectors, plus `source_aspect()`/`source_field()` on authored
  result-shape fields, are no longer public. They remain crate-local terminal
  projections only for lowering, digesting, and compatibility adapters.
- This keeps ordinary authoring ergonomic while preventing external code from
  treating post-authoring field authority as raw aspect/field strings.
- The aspect-native compile-fail suite now proves facade callers cannot use the
  old authoring selector or authored result-shape string aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query authoring --lib`, focused `cargo test -p
  worth-query read_composition --lib`, line-count checks for the touched
  authoring files, and focused scans confirming the public authoring
  `aspect()`/`field()` and `source_aspect()`/`source_field()` aliases are gone.

### Artifact bundle by-name lookup visibility fence slice

- `WORTHQueryDerivedMaterializationBundle::materialization_by_name(...)`,
  `WORTHQueryDerivedArtifactBinding::materialization_by_name(...)`,
  `WORTHQueryLiveArtifactBundle::read_by_name(...)`, and
  `WORTHQueryLiveArtifactBinding::read_by_name(...)` are no longer public
  facade shortcuts.
- Public callers must use typed derived handles or live views through
  `materialization(&WORTHQueryDerivedViewHandle<_>)` and
  `read(&WORTHQueryLiveView<_>)`; raw by-name lookup remains crate-private for
  internal projection-consumption iteration over already-bound targets.
- The aspect-native compile-fail suite now proves external callers cannot use
  raw view-name strings to read retained derived/live artifact contents from
  bundles or bindings.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, line-count checks, and
  focused scans showing by-name bundle/binding lookups are either crate-private
  production calls or compile-fail fixtures.

### Retained upstream input raw-name lookup visibility fence slice

- `WORTHQueryRetainedUpstreamInputs::live_rows(&str)` and
  `WORTHQueryRetainedUpstreamInputs::retained_computed_rows(&str)` are no
  longer public maintainer-facing shortcuts.
- Public maintainer access now flows through typed handles with
  `live_rows_for(&WORTHQueryLiveView<_>)` and
  `retained_computed_rows_for(&WORTHQueryDerivedViewHandle<_>)`, so external
  maintainer code cannot satisfy retained upstream lookup with arbitrary raw
  view-name text.
- The raw-name lookups remain crate-private for internal runtime routing and
  tests that iterate already-declared upstream names inside Query.
- The aspect-native compile-fail suite now proves external callers cannot use
  raw view-name strings to recover retained live or computed upstream rows.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query retained_upstream --lib`, focused `cargo test -p worth-query
  refresh_fallback_maintainer --lib`, and focused scans showing raw upstream
  name lookups are crate-private production/test calls or compile-fail
  fixtures.

### Workspace live by-name lookup visibility fence slice

- `WORTHQueryWorkspace::read_live_by_name(...)` was removed so public callers
  cannot execute live reads from arbitrary raw view-name text.
- `WORTHQueryWorkspace::state_live_by_name(...)`,
  `subscription_basis_digest_by_name(...)`, and `inspect_live_by_name(...)`
  are no longer public facade shortcuts.
- Public workspace live access now flows through typed live-view handles:
  `read_live_result(&WORTHQueryLiveView<_>)`,
  `state_live(&WORTHQueryLiveView<_>)`,
  `subscription_basis_digest(&WORTHQueryLiveView<_>)`, and
  `inspect_live(&WORTHQueryLiveView<_>)`.
- The raw-name state, subscription digest, and inspection helpers remain
  crate-private only as internal routing adapters behind typed public methods.
- Runtime obligation and consumer-kit tests were migrated away from raw
  by-name live reads so tests teach the same aspect-native live-view token
  boundary as production callers.
- The aspect-native compile-fail suite now proves external callers cannot use
  raw view-name strings for workspace live read, state, subscription digest, or
  inspection access.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query read_obligation_dispatch --lib`, focused `cargo test -p
  worth-query in_memory_test_runtime_executes_public_insert_and_live_read
  --lib`, line-count checks, and focused scans showing public by-name live
  lookups only remain in the compile-fail fixture.

### Retained scalar by-name extraction visibility fence slice

- `WORTHQueryDerivedArtifactBinding::consume_scalar_fields_by_name(...)` and
  `verify_scalar_alignment_by_name(...)` are no longer public retained-scalar
  shortcuts.
- Public retained scalar extraction and alignment now require typed derived
  view handles through `consume_scalar_fields(&WORTHQueryDerivedViewHandle<_>,
  ...)` and `verify_scalar_alignment(&WORTHQueryDerivedViewHandle<_>,
  &WORTHQueryDerivedViewHandle<_>, ...)`.
- The raw by-name helpers remain crate-private only as internal adapters behind
  typed public methods, preserving existing retained field admission while
  preventing external callers from selecting retained materializations by
  arbitrary raw view-name text.
- Retained scalar fact-set and alignment tests now exercise the typed derived
  handle paths rather than the raw by-name helpers.
- The aspect-native compile-fail suite now proves external callers cannot use
  raw derived view names for retained scalar extraction or retained scalar
  alignment verification.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query retained_scalar_fact_set --lib`, focused `cargo test -p
  worth-query retained_scalar_alignment --lib`, line-count checks, and focused
  scans showing by-name retained scalar helpers only remain crate-private or in
  the compile-fail fixture.

### Aspect touch terminal projection naming fence slice

- `WORTHQueryAspectTouch` no longer exposes the neutral public
  `aspect_path()` alias.
- Public reporting now uses `terminal_aspect_path_projection()`, making the
  final string projection explicit while the touch carrier continues to retain
  native authoring target proof internally.
- Runtime mutation, graph-touch descriptor, digest, preview, and test support
  call sites were mechanically migrated to the terminal projection name only at
  reporting, serialization, or descriptor-boundary points.
- The aspect-native compile-fail suite now proves facade callers cannot use the
  removed `WORTHQueryAspectTouch::aspect_path()` alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query graph_touch_descriptor --lib`, focused `cargo
  test -p worth-query aspect_crud --lib`, line-count checks, and focused scans
  showing `aspect_path()` survives only in compile-fail fixtures.

### Graph touch selector native constructor naming fence slice

- `WORTHQueryGraphTouchSelector::aspect_path(...)` was renamed to
  `aspect_touch(...)` because the constructor takes an admitted
  `WORTHQueryAspectTouch`, not raw dotted aspect-path authority.
- The selector's evidence kind string remains `"aspect-path"` so existing
  graph-obligation selector digest identity does not churn; only the Rust API
  boundary now speaks in native touch terms.
- Graph obligation support-matrix, lookup-selection, registration selector,
  and read-obligation hardening tests were mechanically migrated to the native
  constructor name.
- The aspect-native compile-fail suite now proves facade callers cannot use the
  removed `WORTHQueryGraphTouchSelector::aspect_path(...)` constructor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query selector_matching --lib`, focused `cargo test -p
  worth-query graph_touch_descriptor --lib`, focused `cargo test -p
  worth-query lookup_selection --lib`, line-count checks, and focused scans
  showing the old constructor name survives only in the compile-fail fixture.

### Schema view typed lookup fence slice

- `QuerySchemaView::field(...)`, `has_aspect(...)`, and `relation(...)` now
  require `AspectName`, `FieldName`, and `RelationName` proof carriers instead
  of accepting raw strings at schema lookup time.
- `SchemaFieldView` no longer exposes neutral `aspect()` / `field()` string
  accessors, and `SchemaRelationView` no longer exposes neutral `relation()`;
  callers must use native `*_name()` carriers or explicit
  `terminal_*_projection()` helpers for reporting text.
- Validation, graph-read schema-reference admission, declarative traversal
  validation, typed-harness tests, and workspace declaration tests now preserve
  typed names into schema lookup instead of reparsing raw schema coordinates.
- The aspect-native compile-fail suite now proves facade callers cannot use raw
  schema-view string lookup aliases or the removed schema field/relation
  neutral string accessors.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query validation --lib`, focused `cargo test -p
  worth-query schema_view --lib`, line-count checks, and focused scans showing
  raw schema lookup and neutral schema relation access survive only in the
  compile-fail fixture.

### Traversal selector relation naming fence slice

- `TraversalSelector` no longer exposes the neutral public `relation()` string
  alias.
- Public and internal callers now choose between `relation_name()` when they
  need the retained `RelationName` proof carrier and
  `terminal_relation_projection()` when they are intentionally producing
  reporting, digest, materialization, or schema-reference boundary text.
- Canonicalization, composition expansion, template instantiation,
  declarative-live lowering, read-composition materialization,
  schema-reference admission, and traversal workspace/test support call sites
  were mechanically migrated away from the old neutral alias.
- The aspect-native compile-fail suite now proves facade callers cannot use the
  removed `TraversalSelector::relation()` alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query traversal --lib`, focused `cargo test -p
  worth-query read_composition --lib`, and focused scans showing the old
  traversal relation alias survives only in the compile-fail fixture.

### Validated traversal relation proof reuse slice

- `ValidatedTraversalEntry` no longer exposes the neutral public `relation()`
  string alias.
- Validation output now offers `relation_name()` for the retained
  `RelationName` proof carrier and `terminal_relation_projection()` for
  collection/reporting text.
- Runtime read relationship-proof descriptor admission now consumes the
  validated `RelationName` directly instead of reparsing terminal relation
  text, so validation proof is reused across the relationship-proof boundary.
- Collection planning edge classification now uses the explicit terminal
  projection name where it intentionally needs text.
- The aspect-native compile-fail suite now proves facade callers cannot recover
  neutral relation text from `ValidatedTraversalEntry::relation()`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query validation --lib`, focused `cargo test -p
  worth-query read_composition --lib`, line-count checks, and focused scans
  showing the old validated traversal relation alias survives only in the
  compile-fail fixture.

### Graph-read domain operation relation proof slice

- `WORTHQueryAdmittedGraphReadDomainOperationReference` now stores a
  `RelationName` proof carrier instead of raw relation-name `String` text.
- `WORTHQueryGraphReadOperationRegistration` and admitted domain registered
  operations retain `Vec<RelationName>` for accepted relation sets.
- Public graph-read domain operation APIs now return `&RelationName` or
  `&[RelationName]` for admitted relation authority; terminal string
  projection is explicit through `terminal_relation_projection()` or local
  `RelationName::as_str()` use at digest, denial, and comparison boundaries.
- Operation registration from declarations, registry matching, operation
  mapping, and the phase-three operation-resolution integration test were
  migrated to native accepted relation carriers.
- The aspect-native compile-fail suite now proves facade callers cannot bind
  admitted graph-read domain operation relation authority as `&str` or
  `&[String]`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query --test
  graph_read_access_phase_three_operation_resolution`, line-count checks, and
  focused scans.

### Graph-read admitted relation row proof slice

- `WORTHQueryAdmittedGraphReadRelation` now stores a `RelationName` proof
  carrier instead of raw relation-name `String` text.
- Schema-reference admission now preserves the typed traversal relation into
  the admitted graph-read relation row rather than projecting terminal text
  and reusing it as authority.
- Operation mapping now compares admitted domain-operation relation names and
  admitted schema-reference relation names as `RelationName` carriers.
- Access requirement derivation and graph-read phase-one tests use
  `terminal_relation_projection()` only where they intentionally emit row,
  authority, or assertion text.
- The aspect-native compile-fail suite now proves facade callers cannot use
  the removed `WORTHQueryAdmittedGraphReadRelation::relation()` string alias.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query --tests`, `TRYBUILD=overwrite cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query --test
  graph_read_access_phase_one`, line-count checks, and focused scans.

### Aspect value constructor and JSON-number validation slice

- `WORTHQueryAspectValue::new_set(...)` and `new_clear(...)` are now
  crate-local, and the neutral `new(...)` convenience constructor is test-only.
  Public callers can no longer mint parsed desired-aspect carriers directly
  from weaker touch-plus-value pieces; they must enter through the aspect
  mutation builder or another owning Query authoring boundary.
- The aspect-native compile-fail suite now proves facade callers cannot call the
  direct `WORTHQueryAspectValue::new_set(...)` constructor while the older
  alias-removal fixtures still build aspect values through public builder
  admission before asserting removed path/JSON projections.
- A mechanical `serde_json` dependency-demotion probe moved `serde_json` out of
  normal worth-query dependencies during `cargo check -p worth-query`. That red
  state exposed two in-scope native-value validations and two remaining
  consumer-kit support document contracts.
- `WORTHQueryProgramValue::decimal_text(...)` and
  `WORTHQueryIntentInput::decimal_text(...)` now validate JSON-number text with
  local native grammar checks instead of depending on `serde_json::Number` as a
  production value authority.
- `serde_json` was restored as a normal dependency because the remaining red
  probe failures were support snapshot/pinning document serialization
  contracts, not aspect authority. Those contracts are candidates for a
  separate external-document boundary if the production JSON allowlist is later
  tightened beyond authority-bearing runtime code.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, and the
  temporary dependency-demotion red probe narrowing production `serde_json`
  requirements to the consumer-kit document contracts.

### Retained materialized row constructor boundary slice

- `WORTHQueryRetainedMaterializedRow::from_scalar_values(...)` is now
  crate-local instead of a public facade constructor, so external callers
  cannot directly mint a retained materialization proof row from an arbitrary
  map of retained field paths and values.
- Public maintainer/test authoring now uses
  `WORTHQueryRetainedMaterializedRowBuilder`, which admits native
  `WORTHQueryRetainedFieldPath` plus foundational `AspectValue` pairs and then
  consumes the builder through `try_build()`.
- A deliberate `cargo check -p worth-query --tests` red probe showed the only
  cross-crate consumer was public-bridge hostile certification support. That
  support now uses the builder, preserving the legitimate native maintainer
  authoring route without reopening direct proof construction.
- The aspect-native compile-fail suite now proves facade callers cannot call
  the direct retained-row constructor. The existing terminal-JSON retained-row
  ingress fixture continues to prove JSON row admission is not public.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite
  cargo test -p worth-query --test aspect_native_query_compile_fail`, and
  normal `cargo test -p worth-query --test aspect_native_query_compile_fail`.

### Projection consumption field request authority slice

- `ProjectionFactFieldPath::from_authoring_path(...)` was removed; callers can
  no longer use the neutral authoring-name constructor to mint a
  projection-consumption field request proof.
- Public request construction now has two explicit routes:
  `ProjectionFactFieldPath::from_canonical_field_path(...)` for callers that
  already hold a foundational `CanonicalFieldPath`, and
  `ProjectionFactFieldPath::from_terminal_projection(...)` for the deliberate
  terminal ingress boundary.
- `ProjectMaterializedFacts` and
  `ProjectionConsumptionDeclarationBuilder` now expose
  `display_field_path(...)` / `derived_scalar_field_path(...)` for native
  request proof and `terminal_display_field(...)` /
  `terminal_derived_scalar_field(...)` for terminal text ingress. The old
  neutral `display_field(...)` and `derived_scalar_field(...)` helpers were
  removed.
- The deliberate red probe first broke production `cargo check -p worth-query`
  on seven certification/report fixture call sites. After migrating those to
  terminal request names, production went green; `cargo check -p worth-query
  --tests` then exposed the broader test/support/public-bridge consumers, which
  were mechanically migrated to the explicit terminal names.
- The aspect-native compile-fail suite now proves facade callers cannot call
  the removed neutral field request helpers or the removed neutral field path
  constructor. The older projection-consumption field-key alias fixture
  now uses `from_terminal_projection(...)` so it continues to test the intended
  alias boundary rather than failing early on field-path construction.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite cargo
  test -p worth-query --test aspect_native_query_compile_fail`, normal `cargo
  test -p worth-query --test aspect_native_query_compile_fail`, focused `cargo
  test -p worth-query projection_consumption --lib`, line-count checks for the
  touched projection-consumption files, and focused scans proving the old
  neutral request helper names no longer appear in source/tests.

### View-shape focused inspector constructor authority slice

- `ViewShapeDescriptor::inspector_detail_focused(...)` and
  `identity_aware_inspector_detail_focused(...)` now require a foundational
  `AspectKey`, so the neutral constructor names can no longer promote raw
  aspect text into focused-inspector view-shape proof.
- `DeclarativeLiveViewShape::inspector_focused(...)` and
  `identity_aware_inspector_focused(...)` now retain `AspectKey` as the
  declarative live proof carrier, and descriptor lowering consumes that native
  key directly.
- Existing string-based authoring/test sites now use the explicit terminal
  ingress helpers:
  `terminal_inspector_detail_focused(...)`,
  `terminal_identity_aware_inspector_detail_focused(...)`,
  `terminal_inspector_focused(...)`, and
  `terminal_identity_aware_inspector_focused(...)`.
- A deliberate production red probe first exposed declarative live lowering and
  live subscription relevance matching. The former was moved to native
  `AspectKey` flow; the latter now makes its terminal string comparison
  explicit with `focused_aspect.as_str()`.
- `cargo check -p worth-query --tests` then exposed saved-query, view-shape,
  view-shape-live, and certification fixture consumers that still used raw
  strings under the neutral constructor names. Those call sites were migrated
  mechanically to terminal constructor names.
- The aspect-native compile-fail suite now proves facade callers cannot pass
  raw strings to the neutral focused-inspector descriptor or declarative live
  shape constructors.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite cargo
  test -p worth-query --test aspect_native_query_compile_fail`, normal `cargo
  test -p worth-query --test aspect_native_query_compile_fail`, focused `cargo
  test -p worth-query view_shape --lib`, line-count checks, and focused scans
  proving raw-string neutral focused-inspector constructor calls remain only in
  the compile-fail fixture.

### Artifact target raw-name constructor fence slice

- `WORTHQueryDerivedMaterializationTarget::new(...)` and
  `WORTHQueryLiveArtifactTarget::new(...)` are no longer public facade
  constructors. Runtime code may still construct these target proof carriers
  from already-owned view names, but external callers cannot mint artifact
  target authority from arbitrary raw text.
- Public construction now flows through the typed routes:
  `From<&WORTHQueryDerivedViewHandle<_>>` for derived materialization targets
  and `From<&WORTHQueryLiveView<_>>` for live artifact targets.
- Artificial retained-live projection fixtures that assemble bundles without a
  runtime declaration use explicit `cfg(test)` `test_only(...)` constructors,
  keeping the fixture escape hatch named and crate-local.
- The deliberate red probe kept production `cargo check -p worth-query` green
  while `cargo check -p worth-query --tests` exposed the retained-live support
  fixtures that still fabricated targets from raw strings.
- The aspect-native compile-fail suite now proves facade callers cannot call
  the removed raw-name constructors.
- This slice is adjacent enforcement rather than a core aspect-value
  conversion: it strengthens the same Law 41 proof-carrier discipline around
  artifact targets, but it does not substitute for deeper JSON/aspect authority
  removal.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query retained_live --lib`, line-count checks, and focused scans
  showing public raw target constructors only in the compile-fail fixture.

### Aspect value and probe field JSON assertion removal slice

- `WORTHQueryAspectValue` no longer exposes test-visible
  `terminal_json_projection()` or `terminal_json_projection_string()` helpers,
  and the underlying desired-aspect proof state no longer retains a test JSON
  projection helper.
- `WORTHQueryExistingTruthProbeField` no longer exposes
  `terminal_json_projection_string()`. Probe fields retain the native
  foundational `AspectValue` plus native value digest evidence; tests now
  assert through `foundational_value()` instead of comparing JSON strings.
- Stateful bridge and existing-truth test adapters still store fake external
  rows as JSON at their explicit external-row boundary, but assertion
  verification immediately admits those values back into `AspectValue` before
  comparison. JSON rows can no longer satisfy verified-existing assertions by
  direct equality against an aspect-value projection.
- The deliberate red probe first kept production `cargo check -p worth-query`
  green, then `cargo check -p worth-query --tests` exposed the bridge support
  and existing-truth assertion consumers that still depended on aspect-value
  JSON projections.
- The aspect-native compile-fail suite already proves facade callers cannot
  call the removed JSON projection methods; this slice extends that mechanical
  fence into crate test builds so Query's own harness stops teaching JSON as
  the assertion vocabulary.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query probe_existing --lib`, focused `cargo test -p worth-query
  bridge_backed_verification_execution --lib`, focused `cargo test -p
  worth-query verified_existing --lib`, focused `cargo test -p worth-query
  mixed_authority --lib`, line-count checks, and focused scans showing the
  removed aspect/probe JSON projection names only survive in compile-fail
  fixtures or unrelated terminal projection boundaries.

### Program output JSON assertion removal slice

- `WORTHQueryProgramValue`, `WORTHQueryOperationInput`, and
  `WORTHQueryOperationOutput` no longer expose crate-test-only terminal JSON
  projection helpers. Program execution tests can no longer inspect operation
  values by projecting the whole value tree into `serde_json::Value`.
- Program values now expose small native read-only inspection routes:
  `array_len()`, `field_path_value(&CanonicalFieldPath)`, and
  `field_path_string_value(&CanonicalFieldPath)`, plus
  `array_field_path_string_value(index, &CanonicalFieldPath)` for row-shaped
  outputs. These preserve the private program-value tree while letting tests
  assert output through foundational field paths and scalar access instead of
  JSON indexing.
- The deliberate red probe with `cargo check -p worth-query --tests` exposed
  exactly the program and preview operation assertions that still used JSON
  projection. Those assertions now use a `CanonicalFieldPath` for
  `title.value` and `array_len()` for preview output cardinality.
- The aspect-native compile-fail suite already proves facade callers cannot
  call the removed program JSON projection helpers; this slice extends that
  fence into crate tests so Query's own runtime harness stops teaching JSON as
  the program-output assertion vocabulary.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query compiled_typed_program --lib`, focused `cargo test -p
  worth-query sandboxed_preview_run_operation --lib`, line-count allowlist
  confirmation for `program.rs`, and focused scans showing the removed program
  JSON projection names survive only in the compile-fail fixture.

### Retained row JSON decode helper removal slice

- Removed the test-only retained-row JSON round-trip decoder module and the
  crate-test JSON projection helpers on retained materialized rows, retained
  scalar facts, derived materialization results, derived patch payloads, and
  runtime/workspace derived materialization shortcuts.
- Added native fail-closed retained-row accessors:
  `WORTHQueryDerivedMaterializationResult::single_retained_row()` and
  `WORTHQueryRetainedUpstreamInputs::single_retained_computed_row(...)`.
  These preserve the previous cardinality checks but return retained row proof
  carriers instead of deserializing through `serde_json::Value`.
- Removed retained artifact binding pair/triple decode helpers. Tests now
  obtain the retained materialization through the binding/bundle, require a
  single retained row, and inspect native `AspectValue` at
  `WORTHQueryRetainedFieldPath`.
- The deliberate red probe with `cargo check -p worth-query --tests` exposed
  the retained-upstream, derived materialization intent, bundle, binding, and
  post-write artifact tests that still used JSON decode/projection helpers.
  Those assertions now use retained row carriers and native scalar lookup.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query retained_upstreams --lib`, focused `cargo test -p worth-query
  derived_materialization_bundle --lib`, focused `cargo test -p worth-query
  row_pair_uses_bound --lib`, focused `cargo test -p worth-query
  row_triple_uses_bound --lib`, focused `cargo test -p worth-query
  materialize_intent_common_path_helper --lib`, focused `cargo test -p
  worth-query batch_write_retained_artifact --lib`, line-count checks, and
  focused scans showing the removed retained-row JSON decode/projection names
  survive only in compile-fail fixtures for public facade boundaries.

### Effect and entity row JSON assertion removal slice

- `WORTHQueryEffectPayload` and `WORTHQueryEffectDelivery` no longer expose
  crate-test-only `terminal_json_payload_projection()` helpers. Runtime effect
  delivery remains native through `WORTHQueryEffectPayload`,
  `WORTHQueryAspectTouch`, and explicit terminal aspect projections for
  reporting.
- `WORTHQueryEntity` no longer exposes crate-test-only
  `terminal_json_row_projection()` or `into_terminal_json_row_projection()`.
  The test-only aspect-field external JSON projection and memory-workspace
  external projection modules were deleted, so row assertions cannot round-trip
  entity truth through object-shaped JSON.
- Runtime tests now inspect entity rows through native helpers over
  `CanonicalFieldPath` and `AspectValue`: scalar lookup, string extraction, and
  first-segment field-prefix detection for row-family assertions. The
  basis-context red probe caught a false `read.value` assumption and was fixed
  to assert the real native field-path family instead of recreating JSON object
  semantics.
- The deliberate `cargo check -p worth-query --tests` red probe exposed the
  coupled graph-composition, preview, symbolic-reference, and basis-context
  assertions that still depended on entity JSON row projection. Those
  assertions now use native scalar/path helpers.
- The aspect-native compile-fail suite proves facade callers cannot call the
  removed effect delivery JSON payload projection or entity terminal JSON row
  projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query aspect_crud --lib`, focused `cargo test -p worth-query
  graph_composition --lib`, focused `cargo test -p worth-query
  sandboxed_preview_run_operation --lib`, focused `cargo test -p worth-query
  preview_promotion --lib`, focused `cargo test -p worth-query basis_context
  --lib`, focused `cargo test -p worth-query symbolic_reference --lib`, and
  focused scans showing the removed entity/effect JSON projection names survive
  only in aspect-native compile-fail fixtures.

### Runtime insert command test-helper JSON ingress removal slice

- Deleted the last test-only `aspect_field_authoring::external_json_ingress`
  module. Its foundational compatibility tests were useful during migration,
  but no current production or test caller needed a Query-local JSON-to-field
  patch ingress helper.
- `runtime::tests::support::insert_command(...)` no longer accepts arbitrary
  serializable values and no longer lowers through `serde_json::to_value(...)`
  plus a local JSON-to-`AspectValue` converter. Test-authored insert commands
  now provide native `AspectValue` inputs directly.
- The deliberate `cargo check -p worth-query --tests` red probe produced 74
  compile errors across computed, effect, live, preview, stop-class, assembly,
  intent, and session-label tests where `json!("...")` was still used as the
  command-authoring value. Those call sites were migrated to
  `test_string_aspect_value(...)` inside `insert_command(...)` blocks, leaving
  unrelated JSON document/export assertions untouched.
- This slice is test-substrate enforcement rather than a public facade
  boundary: it prevents Query's own runtime harness from teaching JSON as the
  normal aspect-mutation authoring value while preserving explicit external
  bridge row JSON fixtures for later slices.
- Because the mechanical rewrite touched an over-cap effect delivery test file,
  suppression/failure coverage was split into `effect/suppression.rs` so the
  touched effect test files stay under the workspace Rust line cap.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test
  -p worth-query computed --lib`, focused `cargo test -p worth-query effect
  --lib`, focused `cargo test -p worth-query effect_delivery --lib`, focused
  `cargo test -p worth-query preview_promotion --lib`, line-count checks, and
  focused scans showing no `insert_command(...)` block still passes `json!(...)`.

### Bridge test-support external row native storage slice

- The crate-internal stateful bridge runtime support no longer stores
  authoritative rows as nested `serde_json::Value` objects. Its
  `rows_by_collection` map now retains `BTreeMap<CanonicalFieldPath,
  AspectValue>` rows directly.
- Stateful bridge writes now apply admitted `WORTHQueryAspectValue` values into
  native external field paths. Existing-truth assertion verification,
  existing-truth probes, live materialization, and grouped-baseline lookup read
  those native values directly instead of walking JSON and re-admitting scalar
  leaves.
- The public bridge runtime support used by integration tests now follows the
  same native row shape. Public bridge live reads clone native field-path rows
  into `WORTHQueryEntity`, and public existing-truth seed/probe/assertion state
  stores `AspectValue` rather than JSON values.
- The deliberate public red probe changed
  `seed_backend_authoritative_truth(...)` to require `AspectValue`; this broke
  the remaining six `json!(...)` authoritative-truth seed sites in public
  bridge graph/bootstrap tests, which were migrated to native `text(...)`
  helpers.
- This slice intentionally preserves separate consumer-kit support snapshot
  JSON document tests and trybuild fixtures that are proving external document
  or removed-API behavior; it removes JSON as bridge test-support authority,
  not all JSON syntax in the repository.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test
  -p worth-query bridge_backed_verification_execution --lib`, focused `cargo
  test -p worth-query probe_existing --lib`, focused `cargo test -p
  worth-query live --lib`, focused `cargo test -p worth-query --test
  public_bridge_runtime_bootstrap`, focused `cargo test -p worth-query --test
  graph_composition_public_bridge`, focused `cargo test -p worth-query --test
  graph_composition_public_bridge_existing`, line-count checks, and focused
  scans proving stateful/public bridge runtime support no longer mentions
  `serde_json`, bare `Value`, `json!`, `aspect_value_from_json`, or external row
  projection decoders.

### Existing-truth verification adapter native seed storage slice

- The legacy `TestExistingTruthVerificationAdapter` no longer stores
  authoritative test truth as `serde_json::Value` and no longer converts JSON
  leaves back into `AspectValue` during assertion/probe verification.
- The adapter seed API now requires `AspectValue` directly, and mismatch
  diagnostics digest the native value rather than a re-admitted JSON scalar.
  Existing-truth probe fields are produced from cloned native values, preserving
  the Law 41 proof-state carrier from seed through verification and probe
  response.
- The deliberate `cargo check -p worth-query --tests` red probe exposed 16
  JSON-backed `.with_value(...)` seeds in bridge-backed verification and
  graph-composition existing-target tests. Those seeds now use
  `test_string_aspect_value(...)`, so the tests cannot accidentally treat JSON
  as the authority substrate.
- This slice closes the last obvious JSON root in
  `runtime/tests/support/adapters/existing_truth_verification.rs`. Remaining
  JSON in consumer-kit snapshot/pinning tests is external serialized-document
  corruption coverage, and remaining computed-test JSON helpers are a separate
  assertion/export root that should be handled explicitly rather than blurred
  into bridge authority cleanup.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query bridge_backed_verification_execution --lib`, focused `cargo test
  -p worth-query graph_composition_edge_split --lib`, focused `cargo test -p
  worth-query graph_composition_verified_existing --lib`, broad filtered
  `cargo test -p worth-query graph_composition --lib`, line-count checks, and
  focused scans proving the adapter and migrated mutation tests no longer use
  `serde_json::Value`, `native_digest_from_json_value`,
  `aspect_value_from_test_json`, or `.with_value(... json!(...))`.

### Computed retained-row JSON assertion helper removal slice

- `runtime/tests/computed.rs` no longer has `read_derived_json(...)`,
  `retained_value_json_rows(...)`, or the `test_retained_scalar_json(...)`
  converter that projected retained materialized rows into `serde_json::Value`
  for assertions.
- Computed runtime tests now assert retained materialization and derived patch
  payload values as native `AspectValue` vectors through
  `read_derived_value_aspects(...)` and `retained_value_aspects(...)`. Expected
  rows use `test_string_aspect_value(...)`, keeping the retained-row assertion
  vocabulary aligned with the `WORTHQueryRetainedMaterializedRow` +
  `WORTHQueryRetainedFieldPath` + `AspectValue` proof carriers.
- The deliberate red probe with `cargo check -p worth-query --tests` exposed
  13 active computed assertion sites still calling the removed JSON helpers.
  Those were migrated to native retained value assertions. A disabled
  `#[cfg(any())]` computed metadata example was also updated so it no longer
  contains stale `Value::String` patch payload examples.
- The shared runtime test prelude no longer re-exports
  `serde_json::Value`; this prevents ordinary runtime tests from importing a
  JSON value carrier by default while preserving explicit `json!` use for
  remaining external document/report assertions.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test
  -p worth-query computed --lib`, line-count allowlist confirmation for the
  pre-existing over-cap `runtime/tests/computed.rs`, and focused scans proving
  the computed root no longer contains `read_derived_json`,
  `retained_value_json_rows`, `test_retained_scalar_json`,
  `serde_json::Value`, bare `Value`, or `Value::String`.

### Runtime support-profile JSON receipt summary removal slice

- `runtime/tests/assembly/support_profile/facade_phase_nine.rs` no longer
  compares submission-lane and workspace-convenience write receipts by lowering
  them into a `serde_json::Value` document with `json!(...)`.
- The parity assertion now uses typed `MutationReceiptSummary` and
  `MutationDeltaSummary` test structs. Scalar reporting fields remain explicit
  strings where the receipt API is already terminal/reporting-facing, while the
  delta touch shape is retained as `Vec<WORTHQueryAspectTouch>` instead of
  projected aspect-path strings.
- The shared runtime test prelude no longer exports `serde_json::json`, so
  runtime tests do not receive either `serde_json::Value` or `json!` by
  default. Any remaining JSON use must import JSON explicitly at an external
  document/report boundary or in compile-fail fixtures.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test
  -p worth-query phase_nine --lib`, line-count checks, and focused scans
  proving the support-profile receipt summary root no longer contains
  `serde_json::Value`, `json!`, bare `Value`, `serde_json::json`, or terminal
  aspect-path projection for the compared delta touch shape.

### Consumer-kit terminal document JSON boundary naming slice

- The remaining consumer-kit support snapshot and support pinning JSON mutation
  tests are serialized document boundary tests, not Query authority carriers.
  They intentionally tamper with exported support documents to prove typed
  load/validation denials.
- Those tests now mark that boundary explicitly with terminal document aliases:
  `TerminalSupportSnapshotDocumentJson` and
  `TerminalSupportPinContractDocumentJson`. Mutation helpers are named
  `terminal_support_snapshot_json_*`,
  `terminal_support_pin_contract_document_json(...)`, and
  `terminal_support_pin_contract_json(...)`.
- Raw `serde_json::Value::String` / `Number` construction was moved behind
  terminal document helpers such as `terminal_document_string(...)`,
  `terminal_document_number(...)`, and `terminal_pin_document_string(...)`.
  This keeps JSON visible as external document tampering rather than ordinary
  runtime/test assertion vocabulary.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test
  -p worth-query support_snapshot --lib`, focused `cargo test -p worth-query
  support_pinning --lib`, line-count checks, and focused scans showing
  consumer-kit support snapshot/pinning JSON references now survive only as
  terminal document aliases/helpers. The remaining broader JSON hits are
  deliberate aspect-native/UI compile-fail fixtures.

### Mutation delta terminal aspect-path projection removal slice

- `WORTHQueryMutationDelta` no longer exposes
  `terminal_aspect_paths_projection()`. Public receipt consumers must keep
  delta evidence as native `WORTHQueryAspectTouch` values through
  `touched_aspects()` instead of recovering dotted path strings from the
  executed mutation delta.
- The deliberate red probe with `cargo check -p worth-query --tests` exposed
  the real consumers: live subscription relevance matching, write receipt
  identity digesting, consumer-kit equivalence reporting, memory-workspace
  tests, runtime program/computed maintainers, live/mutation receipt
  assertions, and aspect API finalization certification rows.
- Production/reporting call sites that truly need terminal text now project
  locally from `&[WORTHQueryAspectTouch]`: equivalence reports, write-receipt
  committed-truth identity construction, live subscription relevance matching,
  and certification touched-aspect digests. The projection is explicit and
  local to those terminal/report boundaries rather than exported by the
  executed delta proof carrier.
- Runtime tests and test maintainers now compare or propagate native touches
  directly. Fallback computed/program maintainers use
  `delta.touched_aspects().to_vec()` instead of projecting to strings and
  re-admitting with `WORTHQueryAspectTouch::new(...)`; receipt assertions use
  `test_aspect_touches(...)` slices.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed `WORTHQueryMutationDelta::terminal_aspect_paths_projection()`
  method.
- While touching oversized tests, split live redeclaration/grouped-inspection
  and preview insert coverage into sibling modules so touched non-allowlisted
  Rust test files remain under the workspace line cap.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  memory_workspace --lib`, focused `runtime_declares_live_view...`,
  `aspect_crud`, `delete_with`, verified update/delete receipt filters,
  `computed`, the moved preview/redeclaration/grouped-inspection filters,
  line-count checks, and a focused scan proving the removed mutation-delta
  terminal projection survives only in the compile-fail fixture.

### Derived patch terminal aspect-path projection removal slice

- `WORTHQueryDerivedPatch` no longer exposes
  `terminal_aspect_paths_projection()`. Public computed-patch consumers must
  keep changed/produced aspect evidence as native `WORTHQueryAspectTouch`
  values through `aspect_touches()`.
- The deliberate root break left production `cargo check -p worth-query`
  green, proving core computed execution was already retaining native touches.
  `cargo check -p worth-query --tests` then exposed the remaining five
  computed assertion sites that still treated derived patches as terminal
  dotted-path evidence.
- Computed tests now compare derived patch evidence against
  `test_aspect_touches(...)` slices. Computed inspection identity construction
  still emits terminal aspect text for digest material, but that projection is
  local to `runtime/computed/surface.rs` and derives from
  `patch.aspect_touches()`.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed `WORTHQueryDerivedPatch::terminal_aspect_paths_projection()`
  method. The existing derived-patch terminal-JSON fixture stderr was refreshed
  because the compiler no longer suggests the removed terminal aspect-path
  projection as a nearby method.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  computed --lib`, line-count allowlist confirmation for the pre-existing
  over-cap computed files, and a focused scan proving the removed derived-patch
  terminal projection survives only in the compile-fail fixture.

### Effect delivery terminal aspect-path projection removal slice

- `WORTHQueryEffectDelivery` no longer exposes
  `terminal_aspect_paths_projection()`. Public effect-delivery consumers must
  retain native trigger/change evidence through `aspect_touches()`.
- The deliberate production red probe exposed the effect-triggered intent
  handoff binding digest as the remaining production consumer. That boundary
  now projects terminal aspect path text locally from
  `pending_delivery.aspect_touches()` for digest material instead of calling a
  public delivery escape hatch.
- Effect inspection identity construction likewise projects terminal path text
  locally from `delivery.aspect_touches()`, while effect delivery tests compare
  native `WORTHQueryAspectTouch` slices with `test_aspect_touches(...)`.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed `WORTHQueryEffectDelivery::terminal_aspect_paths_projection()`
  method. The existing effect terminal-JSON fixture stderr was refreshed
  because the compiler no longer suggests the removed terminal aspect-path
  projection.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  effect_delivery --lib`, line-count checks, and a focused scan proving the
  removed effect-delivery terminal projection survives only in the compile-fail
  fixture.

### Preview execution terminal aspect-path projection removal slice

- `WORTHQueryPreviewExecutionEvidence` no longer exposes
  `terminal_aspect_paths_projection()`. Public preview-execution consumers must
  retain native routed-change evidence through `aspect_touches()` instead of
  recovering dotted aspect-path strings from the preview evidence carrier.
- The deliberate root break left production `cargo check -p worth-query`
  green, proving preview execution routing already retained native touches.
  `cargo check -p worth-query --tests` then exposed the three assertion sites
  in intent and preview execution-binding tests that were still comparing
  terminal path arrays.
- Those tests now compare native `WORTHQueryAspectTouch` slices with
  `test_aspect_touches(...)`, preserving the Law 41 proof-state carrier across
  preview, intent, and execution evidence assertions.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed
  `WORTHQueryPreviewExecutionEvidence::terminal_aspect_paths_projection()`
  method.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test
  -p worth-query
  preview_write_routes_bound_live_computed_and_redirected_effect_without_authoritative_residue
  --lib`, focused `cargo test -p worth-query
  composed_runtime_surface_proves_facade_handles_stay_proof_bearing_across_preview_and_intents
  --lib`, line-count checks, and a focused scan proving the removed
  preview-execution terminal projection survives only in the compile-fail
  fixture.

### Existing-truth probe request terminal aspect-path projection removal slice

- `WORTHQueryExistingTruthProbeRequest` no longer exposes
  `terminal_aspect_paths_projection()`. Public existing-truth probe consumers
  must retain requested aspects as native `WORTHQueryAspectTouch` values
  through `aspect_touches()`.
- The deliberate production red probe exposed one certification fixture
  consumer: the workspace legacy parity lane that still needs terminal string
  ingress for the older `workspace.probe_existing(...)` API. That projection is
  now local to the certification fixture and derives from
  `workspace_legacy_request.aspect_touches()`.
- Existing-truth probe request construction, backend routing, runtime routing,
  and test adapters already consume native touches; the removed method was only
  a public evidence escape hatch.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed
  `WORTHQueryExistingTruthProbeRequest::terminal_aspect_paths_projection()`
  method.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  routing --lib`, line-count checks, and a focused scan proving the removed
  probe-request terminal projection survives only in the compile-fail fixture.

### Live patch terminal aspect-path projection removal slice

- `WORTHQueryLivePatch` no longer exposes
  `terminal_aspect_paths_projection()`. Public live patch consumers must retain
  native touched-aspect evidence through `touched_aspects()`.
- Production `cargo check -p worth-query` stayed green after the break, showing
  current live routing and inspection code already consumes native touches or
  performs terminal projection locally from mutation deltas at report/delivery
  boundaries.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed `WORTHQueryLivePatch::terminal_aspect_paths_projection()`
  method.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query live
  --lib`, line-count checks, and a focused scan proving the removed live patch
  terminal projection survives only in the compile-fail fixture.

### Graph read touch shape terminal projection deletion slice

- Deleted the unused
  `WORTHQueryGraphReadTouchShape::terminal_aspect_paths_projection()` helper.
  Graph read touch shape evidence already exposes native `WORTHQueryAspectTouch`
  values through `aspect_touches()`, and descriptor row derivation consumes
  those native touches directly.
- This was an internal dead-helper cleanup rather than a facade boundary, so no
  new compile-fail fixture was needed; the relevant mechanical evidence is that
  no `pub` or `pub(in ...)` `terminal_aspect_paths_projection(...)` method
  remains under `crates/worth-query/src`, apart from local `pub(super)` terminal
  projection helpers scoped to effect/preview boundary modules.
- Verification covered `cargo check -p worth-query`, `cargo check -p
  worth-query --tests`, line-count checks, and focused scans for remaining
  public terminal aspect-path projection methods.

### Mutation evidence touched/asserted terminal projection removal slice

- Removed public terminal touched/asserted aspect-path projection helpers from
  write commands, batch write receipts, retained refresh contexts, graph touch
  descriptor rows, unified batch write inspections, unified component
  inspections, and verified assumption sets. These evidence/proof carriers now
  expose native `WORTHQueryAspectTouch` values through `touched_aspects()`,
  `declared_aspects()`, `declared_aspect_operations()`, or
  `asserted_aspects()` rather than teaching callers to recover dotted path
  strings from mutation evidence.
- The deliberate production red probe with `cargo check -p worth-query`
  exposed batch write digest construction as the remaining production consumer.
  Digest/reporting boundaries now project terminal text locally from native
  touches instead of calling public receipt or inspection escape hatches.
- The deliberate test red probe with `cargo check -p worth-query --tests`
  exposed nine real assertion/certification consumers across computed retained
  refreshes, batch mutation receipts, bridge-backed verification execution,
  graph composition inspection, and aspect API finalization certification rows.
  Those assertions now compare native touch/assertion slices with
  `test_aspect_touches(...)` where possible, leaving terminal text projection
  only inside local digest/report helpers.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed projection helpers on write commands, batch write receipts, batch
  write inspections, batch write component inspections, retained refresh
  contexts, graph touch descriptor rows, or verified assumption sets.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused batch,
  preview-batch, graph-composition, bridge-backed verification, and computed
  refresh tests, line-count allowlist confirmation for touched over-cap files,
  and a focused scan proving no public or crate-public touched/asserted
  terminal projection methods remain under `crates/worth-query/src`.

### Computed/effect/program terminal aspect-list projection removal slice

- Removed public terminal dependency/produced aspect-list projections from
  `WORTHQueryComputedInspectionEvidence` and `WORTHQueryDerivedView`, public
  terminal input/output/changed projections from `WORTHQueryEffectPayload` and
  `WORTHQueryEffectExpression`, public trigger aspect projection from
  `WORTHQueryEffectTrigger`, and the dead graph touch descriptor row terminal
  declared-operation projection. These surfaces already expose native
  `WORTHQueryAspectTouch` or `WORTHQueryAspectMutationOperation` accessors;
  callers must keep those proof carriers instead of recovering string arrays.
- The deliberate production red probe exposed effect trigger inspection identity
  construction as the remaining production consumer. That terminal digest
  material is now localized to the effect inspection reporting boundary over
  `effect.declaration.trigger().aspect_touches()`.
- The deliberate test red probe exposed five computed/intent assertions that
  still compared terminal dependency/produced aspect arrays. Those assertions
  now compare native `WORTHQueryAspectTouch` slices with
  `test_aspect_touches(...)`.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed computed inspection, derived view, effect trigger, effect expression,
  effect payload, or graph descriptor row terminal projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused computed inspection tests,
  focused composed intent/effect delivery tests, line-count allowlist checks,
  and focused scans proving the removed method names survive only in
  compile-fail fixtures.

### Effect inspection terminal aspect-list projection removal slice

- Removed public terminal trigger, condition-input, and condition-output
  aspect-list projections from `WORTHQueryEffectInspectionEvidence`. Effect
  inspection evidence now exposes native `WORTHQueryAspectTouch` slices through
  `trigger_aspect_touches()`, `condition_input_touches()`, and
  `condition_output_touches()`.
- The deliberate production red probe exposed preview effect relevance routing
  as the remaining production consumer. That routing now compares against
  native trigger touches and performs terminal string comparison locally only
  because the affected preview routing map is still keyed by terminal paths.
- The deliberate test red probe exposed two effect delivery assertions that
  still compared terminal condition input/output arrays. Those now compare
  native touch slices with `test_aspect_touches(...)`.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed effect inspection terminal trigger or condition projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused effect delivery and preview
  execution routing tests, line-count checks, and a focused scan proving the
  removed method names survive only in compile-fail fixtures.

### View-shape grouped/focused terminal aspect projection removal slice

- Removed public terminal focus/grouping/binding aspect projection helpers from
  view-shape descriptors, delivery metadata, grouped planning artifacts,
  grouped binding proofs, grouped live state artifacts, grouped execution lane
  values, and focused inspector patch artifacts. These surfaces already expose
  native `AspectKey` accessors, so callers now keep aspect keys instead of
  recovering terminal strings from view-shape proof carriers.
- The deliberate production red probe exposed admission digest construction,
  plan digest construction, and grouped execution mismatch diagnostics as the
  remaining production consumers. Those boundaries now call `.as_str()` locally
  on native `AspectKey` values only while constructing digest/error text.
- The deliberate test red probe exposed grouped view-shape assertions,
  grouped live baseline assertions, and milestone-eight grouped certification
  fixture selection. Assertions now compare native `AspectKey` values; fixture
  selection still converts to terminal text locally where it chooses a legacy
  grouped projection field name.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed descriptor, delivery, grouped planning, grouped binding, grouped live,
  or focused inspector patch terminal projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused grouped admission, grouped
  baseline, and grouped delta tests, line-count allowlist checks, and a focused
  scan proving the removed method names survive only in compile-fail fixtures.

### Graph-read native row terminal key projection removal slice

- Removed public `terminal_aspect_key_projection()` and
  `terminal_field_key_projection()` helpers from native graph-read selectivity
  rows, predicate/ordering field authorities, admitted boolean predicate
  leaves, and admitted projection/predicate/ordering schema-reference rows.
  These rows already expose native `AspectKey` and `FieldKey` accessors, so
  consumers now keep typed keys instead of recovering terminal strings.
- Production `cargo check -p worth-query` stayed green after the break,
  confirming graph-read derivation already flows through native aspect and
  field key carriers.
- The deliberate test red probe exposed eight phase-four authority assertions
  still comparing terminal strings. Those assertions now compare native
  `AspectKey` and `FieldKey` values directly.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed graph-read terminal key projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused graph-read phase-four authority
  and support tests, line-count checks, and a focused scan proving the removed
  method names survive only in compile-fail fixtures.

### Retained scalar field terminal projection removal slice

- Removed public terminal field-key projection helpers from retained scalar
  field facts and retained scalar alignment facts. These facts already expose
  native `WORTHQueryRetainedFieldPath` carriers, so facade callers must keep
  the retained field path proof instead of recovering field-key strings.
- Production and test `cargo check` stayed green after the break, confirming
  scalar fact/alignment digesting was the only active consumer.
- Retained scalar fact-set and alignment digests now project terminal field
  text locally from `field_path()` only at the digest boundary.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed retained scalar terminal field-key projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused retained scalar fact/alignment
  tests, line-count checks, and a focused scan proving the removed method names
  survive only in compile-fail fixtures.

### Symbolic graph resolution terminal projection removal slice

- Removed public `terminal_aspect_path_projection()` helpers from
  `WORTHQuerySymbolicAspectResolutionEvidence` and
  `WORTHQueryGraphCompositionResolutionEntry`. Both surfaces already expose
  native `WORTHQueryAspectTouch` carriers, so facade callers must retain the
  resolved touch proof instead of recovering dotted path text.
- The deliberate production red probe exposed batch-write inspection digest,
  graph-composition evidence digest, and mutation-evidence digest builders as
  the remaining production consumers. Those boundaries now project terminal
  path text locally from `aspect_touch()` only while building evidence
  identities.
- The deliberate test red probe exposed graph-composition assertions still
  comparing terminal path strings. Those assertions now compare native
  `WORTHQueryAspectTouch` values.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed symbolic resolution and graph-composition resolution terminal path
  projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, broad filtered graph-composition tests,
  line-count checks, and a focused scan proving the removed method names
  survive only in compile-fail fixtures.

### Collection/live key terminal projection removal slice

- Removed public terminal aspect/field projection helpers from
  `OrderingKeyPath` and `QueryFieldKey`. Both carriers already expose native
  `AspectKey` and `FieldKey` accessors, so facade callers must keep typed key
  proof instead of recovering terminal strings.
- Production and test `cargo check` stayed green after the break, confirming
  collection and live digests were the only active consumers of the removed
  helpers.
- Collection ordering digesting, detail live patch digesting, and view-shape
  live focus rejection reporting now project terminal text locally from native
  keys only at digest/error boundaries.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed collection/live terminal key projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused detail live and ordered collection
  live tests, line-count checks, and a focused scan showing the removed method
  names survive only in compile-fail fixtures.

### Aspect mutation operation terminal projection removal slice

- Removed public `terminal_aspect_path_projection()` from
  `WORTHQueryAspectMutationOperation`. Mutation operations already expose their
  native `WORTHQueryAspectTouch`, so facade callers must keep the operation
  proof and project only from the touch when they intentionally need terminal
  reporting text.
- The deliberate production red probe exposed intent-admission mutation seed
  digesting, runtime mutation authority digesting, write/batch inspection
  digesting, graph-obligation lookup key derivation, graph-touch selector
  identity construction, and descriptor inventory reporting as the remaining
  production consumers. Those boundaries now project terminal text locally
  through `operation.aspect_touch()`.
- The deliberate test red probe exposed mutation aspect CRUD, batch, and delete
  assertions still formatting operation terminal strings directly. Those
  assertions now project through the operation's native touch at the assertion
  formatting boundary.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed mutation-operation terminal aspect-path projection helper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused mutation aspect CRUD/batch/delete
  tests, line-count checks, and a focused scan proving
  `operation.terminal_aspect_path_projection()` survives only in the
  compile-fail fixture.

### Aspect value terminal projection removal slice

- Removed public `terminal_aspect_path_projection()` from
  `WORTHQueryAspectValue`. Aspect values already expose native
  `WORTHQueryAspectTouch` through `aspect_touch()`, so callers must keep the
  typed desired-aspect proof and project terminal text only from the touch at
  declared reporting boundaries.
- The deliberate production red probe exposed memory-workspace field patch
  validation, aspect mutation builder duplicate diagnostics, existing-truth
  assertion evidence, and mutation lowering digest rows as the remaining
  production consumers. Those boundaries now project terminal text locally
  through `aspect.aspect_touch()`.
- The deliberate test red probe exposed internal and public bridge existing
  truth verification adapters plus bridge external-row support that still used
  aspect values as terminal path carriers. Those adapters now derive local
  terminal row keys from the native touch retained by each aspect value.
- Removed the now-dead parsed desired-aspect authoring-path helper after the
  public aspect-value shortcut was deleted.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed aspect-value terminal aspect-path projection helper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused existing-truth assertion and
  bridge-backed verification tests, public bridge compile sweep, line-count
  checks, and a focused scan proving `aspect.terminal_aspect_path_projection()`
  survives only in the compile-fail fixture for `WORTHQueryAspectValue`.

### Symbolic aspect reference terminal projection removal slice

- Removed public `terminal_aspect_path_projection()` from
  `WORTHQuerySymbolicAspectReference`. Symbolic aspect references already
  expose their native `WORTHQueryAspectTouch`, so callers must retain the
  symbolic reference proof and project terminal text only through
  `aspect_touch()` at evidence/reporting boundaries.
- The deliberate production red probe exposed intent-admission mutation seed
  identity construction and mutation lowering symbolic aspect reference digest
  rows as the only production consumers. Both now project terminal text locally
  through `reference.aspect_touch()`.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed symbolic-aspect-reference terminal aspect-path projection helper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused symbolic-reference runtime tests,
  and a focused scan proving `reference.terminal_aspect_path_projection()`
  survives only in the compile-fail fixture.

### Existing-truth probe field terminal projection removal slice

- Added native `aspect_touch()` access to `WORTHQueryExistingTruthProbeField`
  and removed its public `terminal_aspect_path_projection()` helper. Probe
  fields now keep their parsed target proof as authority and can only project
  terminal text through the native touch at local reporting boundaries.
- Production stayed green after the helper removal; the only implementation
  consumer was the existing-truth probe digest row, which now projects terminal
  text locally through `field.aspect_touch()`.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed probe-field terminal aspect-path projection helper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused existing-truth probe and verified
  existing tests, line-count checks, and a focused scan proving
  `field.terminal_aspect_path_projection()` survives only in the compile-fail
  fixture.

### Existing-truth denial terminal projection removal slice

- Removed public terminal aspect-path projection helpers from
  `WORTHQueryExistingTruthAssertionDenial` and
  `WORTHQueryExistingTruthProbeDenial`. Denials still retain typed internal
  denied-path evidence for digest construction, but facade consumers now get
  native `asserted_aspect_touch()` and `probed_aspect_touch()` access when the
  denied path admitted as an aspect touch.
- The deliberate `cargo check -p worth-query --tests` red exposed seven
  denial assertions still expecting terminal strings across verify-existing,
  verified-update, verified-delete, and probe-existing tests. Those assertions
  now compare `WORTHQueryAspectTouch` values.
- Added two aspect-native trybuild fixtures proving facade callers cannot call
  the removed assertion/probe denial terminal projection helpers. Existing
  denial alias fixtures were refreshed so compiler suggestions now point to the
  native touch accessors.
- Split the over-line-cap `verify_existing.rs` test module after touching it:
  denial-focused tests remain in `verify_existing.rs`, while batch, preview,
  and relation receipt coverage lives in `verify_existing_receipts.rs`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused verify-existing/probe-existing
  and verified update/delete tests, line-count checks, and a focused scan
  proving the removed terminal denial helpers survive only in compile-fail
  fixtures.

### Validated entry terminal key projection removal slice

- Removed public terminal aspect/field key projection helpers from
  `ValidatedProjectionEntry`, `ValidatedPredicateEntry`,
  `ValidatedOrderingEntry`, and `ValidatedResultShapeBinding`. These validated
  entries already expose native `AspectKey` and `FieldKey` accessors, so
  callers must retain the typed validation proof and project text only at local
  digest, ordering-key, or binding-proof boundaries.
- The deliberate production red probe exposed collection ordering key
  construction, live relevance field classification, and grouped view-shape
  binding proof construction as the remaining production consumers. Those
  boundaries now project locally through the native keys.
- Added four aspect-native trybuild fixtures proving facade callers cannot call
  the removed validated projection, predicate, ordering, or result-shape
  terminal key projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused live, collection, and grouped
  runtime tests, line-count checks, and a focused scan proving the removed
  validated-entry terminal key helpers survive only in compile-fail fixtures.

### Graph-read admission error terminal key projection removal slice

- Replaced terminal aspect/field storage on
  `WORTHQueryBooleanExpressionAdmissionError` and
  `WORTHQueryGraphReadSchemaReferenceAdmissionError` with native `AspectKey`
  and `FieldKey` storage.
- Removed public terminal aspect/field projection helpers from both exported
  graph-read admission error surfaces and exposed native key accessors instead.
  Error reporting may still project locally, but facade callers cannot recover
  the failed graph-read field as arbitrary terminal strings from the error
  object.
- Added two aspect-native trybuild fixtures proving facade callers cannot call
  the removed graph-read boolean-expression or schema-reference admission error
  terminal key projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused graph-read phase-one and
  phase-four requirement tests, line-count checks, and a focused scan proving
  the removed graph-read admission error terminal key helpers survive only in
  compile-fail fixtures.

### Schema field view terminal key projection removal slice

- Removed public terminal aspect/field projection helpers from
  `SchemaFieldView`. Schema fields already expose typed `AspectName` and
  `FieldName` carriers, so callers must retain the schema field proof instead
  of recovering terminal key strings from the view object.
- `QuerySchemaView` digest construction now projects locally through
  `aspect_name().as_str()` and `field_name().as_str()` only at the schema view
  identity boundary.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed schema field terminal key projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused schema view tests, line-count
  checks, and a focused scan proving the removed schema field terminal key
  helpers survive only in compile-fail fixtures.

### Projection fact field path terminal helper removal slice

- Removed public `ProjectionFactFieldPath::from_terminal_projection(...)` and
  `ProjectionFactFieldPath::terminal_projection()`. Public callers now build
  projection fact field paths from `CanonicalFieldPath`, keeping the
  foundational path carrier as the authority surface.
- Kept terminal field-path parsing and projection behind crate-local
  `from_terminal_ingress(...)` and `terminal_projection_for_boundary()` names
  for legacy authoring sugar, digest construction, diagnostics, and extraction
  errors. The renamed helpers deliberately exposed production consumers through
  `cargo check` red before they were migrated.
- Projection-consumption tests that previously asserted terminal field text
  now compare `CanonicalFieldPath` values, preserving the native proof carrier
  through bound fact families and consumed field facts.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed projection fact field-path terminal constructor or projection
  helper. Refreshed adjacent projection-consumption stderr after setup code
  stopped using the removed terminal constructor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  projection_consumption --lib`, line-count checks, and focused scans proving
  the removed `ProjectionFactFieldPath` terminal helpers survive only in the
  new compile-fail fixture.
- Adjacent bundled follow-up: `AuthorizedProjectionFieldPath` carried the same
  terminal helper shape for policy-delivery and narrowing surfaces, so it was
  migrated in the next field-path authority slice instead of being left as
  cleanup debt.

### Authorized projection field path terminal helper removal slice

- Removed public
  `AuthorizedProjectionFieldPath::from_terminal_projection(...)` and
  `AuthorizedProjectionFieldPath::terminal_projection()`. Public callers now
  construct authorized projection field paths from foundational `AspectKey`
  and `FieldKey` through `from_native_keys(...)`.
- Kept terminal field-path parsing and projection crate-local behind
  `from_terminal_ingress(...)` and `terminal_projection_for_boundary()` for
  policy request ingress, digest/reporting text, certification fixture setup,
  and other explicit terminal boundaries.
- Migrated policy delivery, policy live relevance, policy narrowing, and
  projection-consumption visibility checks to parse requested terminal fields
  once and compare retained authorized field proof carriers or native
  aspect/field keys instead of comparing raw terminal strings.
- Migrated tests that inspected authorized fields to compare native
  aspect/field key pairs. Harness code that must still feed terminal request
  strings into `admit_policy_aware_live_plan(...)` now performs that projection
  in a local terminal-ingress helper.
- Split `authorized_projection/field_path.rs` out of
  `authorized_projection/artifacts.rs`, keeping touched non-allowlisted Rust
  files under the 400-line cap.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed authorized projection field-path terminal constructor or
  projection helper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused authorized projection, policy
  delivery, policy live, policy narrowing, and projection consumption tests,
  line-count checks, and focused scans proving the removed
  `AuthorizedProjectionFieldPath` terminal helpers survive only in the new
  compile-fail fixture.

### Retained field path terminal helper removal slice

- Removed public `WORTHQueryRetainedFieldPath::from_authoring_path(...)` and
  `terminal_projection()`. Public callers now construct retained field paths
  from foundational `CanonicalFieldPath`, keeping retained materialization
  field identity in the native proof carrier.
- Kept terminal retained-field parsing and projection crate-local behind
  `from_terminal_ingress(...)` and `terminal_projection_for_boundary()` for
  current string authoring surfaces, retained-row diagnostics, digest
  construction, and projection-consumption boundary text.
- The deliberate production red exposed retained binding extraction, retained
  scalar fact admission, retained scalar alignment, and retained row digesting
  as the active production consumers. Those consumers now use the explicit
  terminal boundary names or retain native paths directly.
- Public bridge hostile certification setup now constructs retained paths from
  `CanonicalFieldPath`, so public certification fixtures teach the native
  retained-field carrier instead of the removed dotted-string constructor.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed retained field-path terminal constructor or projection helper.
  Refreshed the retained materialized row constructor fixture so it reaches the
  intended private row constructor failure through the native path constructor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused retained scalar tests, retained
  projection-consumption tests, public bridge hostile certification tests,
  public bridge reader-lane honesty tests, line-count checks, and focused
  scans proving the removed retained field-path terminal helpers survive only
  in compile-fail fixtures.

### Policy placeholder terminal field helper removal slice

- Replaced `PolicyPlaceholderMaskingRequest` and
  `PolicyPlaceholderMaskingDenial` storage from terminal `Vec<String>` field
  bags to native `Vec<AuthorizedProjectionFieldPath>` proof carriers.
- Removed public
  `PolicyPlaceholderMaskingRequest::terminal_requested_placeholder_fields(...)`
  and terminal requested-placeholder projection helpers from request and denial
  surfaces. Public callers now construct requests with
  `from_authorized_field_paths(...)` and inspect
  `requested_placeholder_field_paths()`.
- The deliberate test red exposed policy delivery unit coverage and milestone
  nine certification rows as the remaining callers still teaching
  `"secret.salary"` request construction. Those callers now build
  `AuthorizedProjectionFieldPath` values from foundational `AspectKey` and
  `FieldKey`.
- Removed the unused crate-local terminal ingress after production `cargo
  check` proved no production path required it, keeping placeholder masking
  denial proof native instead of preserving a string compatibility lane.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed terminal request constructor or terminal field projection
  helpers. Refreshed adjacent placeholder request alias fixtures so compiler
  suggestions point at the native field-path accessor/constructor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  policy_delivery --lib`, focused `cargo test -p worth-query milestone_nine
  --lib`, line-count checks, and focused scans proving the removed policy
  placeholder terminal helpers survive only in compile-fail fixtures.

### Aspect touch terminal projection removal slice

- Removed public `WORTHQueryAspectTouch::terminal_aspect_path_projection()`.
  Public callers must retain the admitted touch carrier and use
  `native_aspect_key()` / `native_field_path()` when they need to inspect the
  foundational identity.
- Kept terminal aspect-path projection crate-local as
  `terminal_projection_for_boundary()` for digest construction, existing
  backend keying, graph touch descriptors, write receipts, inspection rows, and
  other explicit terminal/reporting boundaries.
- The deliberate production red exposed 80 call sites across mutation
  lowering, intent admission, runtime effects, computed routing, preview,
  inspection, graph touch descriptors, memory workspace, consumer-kit
  equivalence, and public-bridge support. The deliberate test red exposed 95
  total call sites once integration support was included.
- Public bridge runtime support now formats external backend keys locally from
  the native aspect touch accessors instead of calling a public terminal
  projection helper on the proof carrier.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed `WORTHQueryAspectTouch::terminal_aspect_path_projection()`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, line-count checks for the manually
  touched support files, and focused scans proving the removed touch terminal
  projection survives only in compile-fail fixtures.

### Relation terminal projection removal slice

- Removed public terminal relation projection helpers from
  `TraversalSelector`, `SchemaRelationView`,
  `WORTHQueryAdmittedGraphReadDomainOperationReference`, and
  `WORTHQueryAdmittedGraphReadRelation`.
- Each type already retained a `RelationName` proof carrier, so public callers
  now keep relation authority through `relation_name()` and project
  `RelationName::as_str()` only at local assertion/reporting boundaries.
- Kept crate-local `terminal_relation_projection_for_boundary()` helpers for
  digesting, canonicalization reporting, composition diagnostics,
  declarative-live lowering, read materialization, schema-view identity text,
  graph-read access rows, and operation-resolution comparison text.
- The deliberate production red exposed 13 call sites across domain operation
  declaration digesting, canonicalization, composition expansion/template
  instantiation, declarative live, graph-read access derivation, operation
  resolution, read-composition materialization, and schema-view digesting.
  `cargo check --tests` then exposed four assertion/integration call sites
  that were migrated to relation-name carriers or crate-local boundary text.
- Added four aspect-native trybuild fixtures proving facade callers cannot use
  the removed terminal relation projection helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  traversal --lib`, focused `cargo test -p worth-query --test
  graph_read_access_phase_one`, focused `cargo test -p worth-query
  read_composition --lib`, line-count checks, and focused scans proving the
  removed terminal relation projection helpers survive only in compile-fail
  fixtures.

### Projection-consumption terminal field request helper removal slice

- Removed public `ProjectMaterializedFacts::terminal_display_field(...)` and
  `ProjectMaterializedFacts::terminal_derived_scalar_field(...)`. Facade
  callers must now construct `ProjectionFactFieldPath` from a foundational
  `CanonicalFieldPath` and pass it through `display_field_path(...)` or
  `derived_scalar_field_path(...)`.
- Removed the same terminal field helper forwarding methods from
  `ProjectionConsumptionDeclarationBuilder`, so source-locked declaration
  builders cannot recover projection-consumption fact authority from dotted
  field strings either.
- Added a crate-local `projection_fact_field_path_from_segments(...)` fixture
  helper for Query-owned certification and regression code. It constructs
  native field paths from explicit `FieldKey` segments instead of preserving a
  public dotted-string request API.
- The deliberate production red exposed seven certification/domain-capability
  fixture call sites. `cargo check --tests` exposed 76 total test call sites,
  covering domain-capability aftermath/certification, projection-consumption
  phase tests, retained/live paths, intent-admission tests, shared read support,
  and runtime bridge hostile certification support.
- Public bridge reader-lane support and projection-consumption golden UI
  examples now build `ProjectionFactFieldPath` values from
  `CanonicalFieldPath` and `FieldKey`, so public examples teach the native
  carrier instead of terminal request sugar.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed terminal field request helpers.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, normal `cargo test -p worth-query --test
  phase_boundaries_projection_consumption_compile_fail`, focused `cargo test
  -p worth-query projection_consumption --lib`, line-count checks, and focused
  scans proving the removed terminal request helpers survive only in the new
  compile-fail fixture.

### View-shape focused inspector terminal constructor removal slice

- Removed the temporary public terminal focused-inspector constructors from
  `ViewShapeDescriptor` and `DeclarativeLiveViewShape`:
  `terminal_inspector_detail_focused(...)`,
  `terminal_identity_aware_inspector_detail_focused(...)`,
  `terminal_inspector_focused(...)`, and
  `terminal_identity_aware_inspector_focused(...)`.
- Public/facade callers must now enter focused inspector view-shape authority
  through the existing native constructors with a foundational `AspectKey`.
  Terminal string aspect names may only be turned into `AspectKey` at local
  authoring, fixture, assertion, or reporting edges.
- The deliberate compiler break kept `cargo check -p worth-query` green and
  used `cargo check -p worth-query --tests` to expose 24 test/certification
  consumers across saved-query, view-shape, view-shape-live, milestone-eight,
  and milestone-nine-five fixtures. Those consumers now call the native
  constructors with explicit `AspectKey` values.
- Root UI fixtures that still test identity-aware shortcut denial and
  post-admission mutation now construct descriptors through the native
  `AspectKey` constructor, so their failures keep targeting the intended bool
  shortcut/private-field boundaries rather than the removed terminal helper.
- The aspect-native compile-fail fixture now proves facade callers cannot call
  the removed terminal focused-inspector constructor; a focused scan shows the
  removed terminal constructor names survive only in that negative fixture.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, `TRYBUILD=overwrite` and normal `cargo
  test -p worth-query --test phase_boundaries_compile_fail`, focused `cargo
  test -p worth-query view_shape --lib`, focused `cargo test -p worth-query
  view_shape_live --lib`, line-count allowlist checks for the already-over-cap
  touched files, and focused scans.

### Validated traversal terminal relation projection removal slice

- Removed public `ValidatedTraversalEntry::terminal_relation_projection()`.
  Facade callers must retain the validation proof through `relation_name()`
  instead of recovering terminal relation text from the validated traversal row.
- Collection planning now projects `entry.relation_name().as_str()` only at the
  `TraversalEdgeClass` construction boundary where edge classification still
  intentionally needs text.
- `cargo check --tests` stayed green after the deliberate root break because
  collection planning was the only remaining production consumer; the
  compile-fail fixture now makes public recovery impossible.
- Added an aspect-native trybuild fixture proving facade callers cannot call the
  removed validated traversal terminal relation projection helper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  validation --lib`, focused `cargo test -p worth-query collection --lib`,
  line-count checks, and focused scans proving the removed helper survives only
  in intentional aspect-native negative fixtures.

### Aspect touch boundary projection root break slice

- Removed the crate-wide `WORTHQueryAspectTouch::terminal_projection_for_boundary()`
  root and replaced it with an explicitly named
  `aspect_path_text_for_boundary()` text projection for presentation,
  diagnostics, digest, and external-row boundaries.
- Added native `WORTHQueryAspectTouch::matches_or_contains(...)` proof-state
  comparison so effect routing, computed routing, preview routing, retained
  refresh deduplication, program derived-aspect deduplication, and graph
  composition selector/dedup paths no longer compare aspect authority through
  dotted strings.
- The deliberate root break produced 80 production compiler errors across
  consumer-kit reporting, intent admission, memory workspace, program lowering,
  effect routing, computed routing, mutation lowering, graph composition,
  receipts, and inspection. The migrated authority paths now use native touch
  equality, ordering, and containment; remaining text calls are explicit
  boundary projections.
- Added an aspect-native trybuild fixture proving facade callers cannot call the
  removed touch terminal boundary projection helper.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query effect
  --lib`, focused `cargo test -p worth-query computed --lib`, focused `cargo
  test -p worth-query graph_composition --lib`, line-count checks, and a
  focused scan proving the removed touch method name survives only in the
  intentional compile-fail fixture.

### Memory workspace declared aspect native lookup slice

- `WORTHQueryMemoryWorkspace::field_patch_from_aspect_values(...)` no longer
  projects an admitted `WORTHQueryAspectValue` back to terminal text in order to
  rediscover declared aspect authority. It now carries the value's
  `WORTHQueryAspectTouch` into declaration matching and relational field-patch
  construction.
- Declared memory-workspace aspects match requested mutations through native
  `WORTHQueryAspectTouch` equality/containment over foundational aspect keys and
  canonical field paths. A whole-aspect declaration such as `title` can satisfy
  a field touch such as `title.value` without treating dotted strings as the
  authority basis.
- Existing-truth probe missing-aspect denial construction now has a private
  admitted-touch constructor, so a backend response that omits a requested
  aspect preserves the admitted probe touch until denial digest/report
  projection instead of demoting it through string reparsing.
- Added a memory-workspace regression proving that a declaration
  `aspect("title", "title.value")` accepts an inserted native touch
  `title.value` and retains both native aspect value and external scalar lookup
  evidence.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, focused `cargo test -p
  worth-query memory_workspace --lib`, focused `cargo test -p worth-query
  probe_existing --lib`, focused `cargo test -p worth-query verified_existing
  --lib`, line-count checks, and focused scans proving the old
  `declared_aspect_for_path(...)` lookup is gone.
- This slice is a proof-flow hardening slice, not a deep public root break:
  compiler checks stayed green. The next slice should intentionally break a
  broader root, preferably one that still accepts or returns native-looking
  carriers through compatibility `String`/JSON constructors.

### Existing-truth denial native constructor slice

- Changed `WORTHQueryExistingTruthAssertionDenial::new(...)` so an
  aspect-specific denial must receive `Option<WORTHQueryAspectTouch>` instead
  of `Option<String>`. Public/backend adapter callers can still report a
  denial, but cannot name the asserted aspect through a terminal dotted path.
- Changed `WORTHQueryExistingTruthProbeDenial::new(...)` the same way, so
  missing-probe denials preserve requested probe touch evidence instead of
  accepting terminal path text.
- Removed the now-dead rejected-string branch from the shared denied-aspect
  path helper; existing-truth denials now retain admitted touch evidence until
  digest/report projection.
- The deliberate `cargo check -p worth-query --tests` red exposed public bridge
  verification support, runtime verification adapters, stateful bridge support,
  intent-admission certification fixtures, and stop-class fixtures that still
  manufactured assertion/probe denial evidence from strings.
- Migrated those consumers to propagate the admitted aspect touch they were
  already verifying, or to construct a native representative touch in manual
  stop-class fixtures.
- Added aspect-native trybuild fixtures proving public callers cannot call the
  assertion-denial or probe-denial constructors with
  `Some("status.value".to_string())`.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, deliberate red and final green `cargo check -p worth-query
  --tests`, `TRYBUILD=overwrite` and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  probe_existing --lib`, focused `cargo test -p worth-query verify_existing
  --lib`, focused `cargo test -p worth-query stop_class --lib`, focused `cargo
  test -p worth-query --test public_bridge_runtime_bootstrap`, line-count
  checks, and focused scans proving the old terminal denied-aspect conversion
  helper is gone.

### Graph obligation selector native lookup key slice

- Changed graph obligation touch lookup keys so aspect selectors use
  `WORTHQueryAspectTouch` and declared aspect operations use
  `WORTHQueryAspectMutationOperation` instead of terminal dotted-string lookup
  keys.
- Removed public `WORTHQueryGraphTouchSelector::selector_value()` and
  `selector_kind()` access. Internal consumers that need presentation/reporting
  text now call explicit `terminal_selector_*_for_boundary(...)` helpers.
- Added an internal `WORTHQueryGraphTouchSelectorClass` proof-state classifier
  and native value accessors so obligation-index registration no longer routes
  selectors by matching strings like `aspect-path`, reparsing relation ids, or
  reconstructing mutation/read/lifecycle families from terminal text.
- Split selector helper code into small private modules so the native selector
  state machine stays under the workspace line cap.
- The deliberate red exposed the obligation index's missing internal typed
  selector-class export, plus public/test selector-kind call sites. Those
  consumers now use native index keys or explicit terminal-boundary projection.
- Added aspect-native trybuild fixtures proving facade callers cannot call the
  removed selector kind/value aliases.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  graph_obligation --lib`, focused `cargo test -p worth-query
  graph_composition --lib`, line-count checks, and focused scans proving the
  old public aliases survive only in intentional negative fixtures.
- This slice uses Arch Law 41 more strongly than a grep audit: the forbidden
  selector aliases are mechanically uncallable, while graph-obligation lookup
  receives native selector proof state instead of rediscovering authority from
  strings.

### Derived patch scalar payload constructor fence slice

- Removed public `WORTHQueryDerivedPatchPayload::from_scalar_value(...)` and
  the scalar-only payload variant. Derived patch payloads now carry either no
  payload or retained materialized rows, so facade callers cannot publish a
  detached scalar value without row/field authority.
- Added an aspect-native trybuild fixture proving facade callers cannot recover
  the removed scalar payload constructor.
- Verification covered `cargo fmt -p worth-query`, `cargo check -p
  worth-query`, `cargo check -p worth-query --tests`, `TRYBUILD=overwrite`
  and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  computed --lib`, and focused scans proving the removed constructor survives
  only in the intentional negative fixture.
- This was a narrow fence, not the desired broad red root. The next slice should
  target a still-active ingress/projection carrier with live consumers, rather
  than another unused compatibility constructor.

### Projection and retained field-path terminal ingress quarantine slice

- Removed the generic crate-local `ProjectionFactFieldPath::from_terminal_ingress(...)`.
  Projection fact paths now enter production through canonical
  `CanonicalFieldPath` proof carriers, while external row extraction keeps its
  remaining dotted-key decoding inside explicitly named extraction-boundary
  helpers.
- Query read-result row extraction now lowers native entity aspect labels and
  external projection field keys through local boundary helpers instead of using
  a reusable terminal constructor on the projection fact proof type.
- Deliberately removed `WORTHQueryRetainedFieldPath::from_terminal_ingress(...)`
  before migrating callers. The red `cargo check -p worth-query --tests` pass
  exposed retained binding extraction, retained scalar fact admission, retained
  scalar alignment, runtime test support, projection-consumption retained-live
  fixtures, and public certification/transcript row helpers as consumers.
- Retained binding extraction now converts a requested `ProjectionFactFieldPath`
  directly into `WORTHQueryRetainedFieldPath` through its canonical field path,
  preserving proof state instead of projecting to text and reparsing.
- Retained scalar fact/alignment public methods still accepted ergonomic field
  strings at this point, but their parsing was quarantined in boundary-named
  local helpers. A later retained-scalar public API fence slice removed this
  compatibility hole entirely. Test and certification helpers were moved to
  build retained field paths from `CanonicalFieldPath`/`FieldKey` or through
  scoped test helpers rather than a reusable carrier constructor.
- Split retained scalar fact tests into a sibling test module to keep touched
  Rust files under the 400-line workspace cap.
- Added an aspect-native trybuild fixture proving facade callers cannot call
  the removed `ProjectionFactFieldPath::from_terminal_ingress(...)` constructor.
  `WORTHQueryRetainedFieldPath::from_terminal_ingress(...)` was crate-local
  already, so the mechanical proof for that root is the internal compiler-red
  pass plus the absence of the method and call sites.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, final green `cargo check -p
  worth-query`, `TRYBUILD=overwrite` and normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  projection_consumption --lib`, focused `cargo test -p worth-query
  retained_scalar --lib`, line-count checks, and focused scans proving retained
  terminal ingress is gone while projection terminal ingress survives only in
  the intentional negative fixture.

### Authorized projection field-path terminal ingress removal slice

- Removed the reusable `AuthorizedProjectionFieldPath::from_terminal_ingress(...)`
  and `AuthorizedProjectionFieldPath::from_parts(...)` constructors from the
  proof carrier. Authorized projection field paths now cross the carrier
  boundary through native `AspectKey` and `FieldKey` values.
- The deliberate `cargo check -p worth-query --tests` red exposed authorized
  projection derivation, live policy admission relevance matching, projection
  consumption test bindings, projection consumption certification fixtures, and
  domain capability certification fixtures as remaining terminal-string
  consumers.
- Authorized projection derivation now validates canonical result-shape aspect
  and field names locally, then creates the proof field path with
  `from_native_keys(...)`.
- Live policy admission still accepts ergonomic requested relevance field text,
  but parsing is quarantined in
  `authorized_projection_field_from_live_relevance_boundary(...)` instead of a
  reusable constructor on the proof type.
- Projection-consumption and domain-capability fixtures now build authorized
  paths from foundational `AspectKey`/`FieldKey` values through scoped fixture
  helpers.
- Added an aspect-native trybuild fixture proving facade callers cannot mint
  authorized projection paths through terminal ingress or string-parts
  constructors.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, green `cargo check -p
  worth-query`, `TRYBUILD=overwrite` and normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  authorized_projection --lib`, focused `cargo test -p worth-query
  policy_live --lib`, focused `cargo test -p worth-query
  projection_consumption --lib`, line-count checks, and focused scans proving
  the removed constructors survive only in intentional negative fixtures.
- This slice uses Arch Law 41 as mechanical enforcement: the authorized
  projection proof type no longer offers a lower-proof terminal-string
  promotion path, while real external/test boundaries are named where they
  still parse authoring text.

### Live policy relevance native admission slice

- Changed public `admit_policy_aware_live_plan(...)` so requested relevance
  fields are `&[AuthorizedProjectionFieldPath]` instead of `&[String]`. Live
  policy admission now receives authorized projection field proof values
  directly instead of authority-bearing terminal field bags.
- Removed the temporary live-relevance terminal parser helper from
  `policy_live::admission`; masked relevance denial now compares native
  authorized projection field paths.
- The deliberate red `cargo check -p worth-query --tests` exposed policy-live
  unit tests and milestone-nine certification rows that were still handing live
  admission terminal strings.
- Policy-live tests now mint `AuthorizedProjectionFieldPath` values from
  foundational `AspectKey`/`FieldKey` test helpers. Milestone-nine
  certification now clones the narrowed artifact's visible native field paths
  for admitted live plans and uses a native masked-field helper for hostile
  relevance denial.
- Added an aspect-native trybuild fixture proving facade callers cannot pass
  `Vec<String>` relevance fields to `admit_policy_aware_live_plan(...)`.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, `TRYBUILD=overwrite` and normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query policy_live --lib`, focused `cargo test -p
  worth-query milestone_nine_certification --lib`, line-count checks, and
  focused scans proving the removed string-bag signature/helper are gone.
- This is a stronger Arch Law 41 move than the constructor fence alone: live
  admission now requires the proof state produced by authorized projection
  rather than reparsing weaker authoring text at admission time.

### Declarative/program JSON vocabulary native value slice

- Replaced public `DeclarativeWritebackValue` enum variants with a private
  `AspectValue` carrier plus scalar constructors (`string`, `integer`,
  `boolean`) and an `aspect_value()` accessor. Public callers can no longer use
  `DeclarativeWritebackValue::StructuredJson(String)` as an authority-shaped
  escape hatch.
- Declarative writeback digests now derive from native `AspectValue` structure
  rather than a `structured_json:{...}` variant branch.
- Renamed public `WORTHQueryPortType::TerminalJson` to
  `WORTHQueryPortType::ProgramValue`, keeping program port language tied to the
  native program value carrier instead of terminal serialization vocabulary.
- Renamed canonical-number helper/error text away from JSON terminology and
  renamed row-like extraction's internal `extract_json_rows(...)` helper to
  `extract_entity_rows(...)`.
- Split intent input construction and canonical-number parsing into
  `runtime/intent/input.rs` so the touched `runtime/intent/declaration.rs`
  returns under the workspace 400-line cap.
- Added aspect-native trybuild fixtures proving facade callers cannot use the
  removed `StructuredJson` writeback variant or `TerminalJson` port type.
- The first deliberate red exposed remaining `DeclarativeWritebackValue::String`
  construction and the missing native digest helper. The module split then
  produced a re-export red (`WORTHQueryIntentInput` still being exported through
  `declaration`) before the final green pass.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, green `cargo check -p
  worth-query`, `TRYBUILD=overwrite` and normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  declarative --lib`, focused `cargo test -p worth-query program --lib`,
  line-count checks, and focused scans proving the old JSON vocabulary survives
  only in intentional negative fixtures.
- This slice is a public vocabulary/mechanical-fence slice. Its compiler-red
  cone was smaller than the retained field-path root break, but it removes a
  current public JSON-shaped authority vocabulary before deeper native query
  pipeline work begins.

### Memory workspace external projection path native storage slice

- `WORTHQueryAspect` now stores its external projection path as a
  `CanonicalFieldPath` at construction time instead of retaining terminal
  dotted text as the internal authority source.
- Added a crate-local native constructor for `WORTHQueryAspect` so internal
  callers that already have a canonical field path can avoid stringifying and
  reparsing the same proof value.
- Memory workspace schema derivation, field patch lowering, and entity
  projection now consume the native external projection path. Terminal
  projection remains only for declared alias matching and report/digest text.
- `WORTHQueryTestBackendSchema` now stores external projection paths as
  `CanonicalFieldPath`, rejects invalid projection path text at the authoring
  boundary, exposes native paths through `aspects()`, and hands native paths to
  `WORTHQueryAspect` without a second terminal ingress.
- Removed the now-dead reusable terminal projection field-path helper and the
  dead `terminal_field_label(...)` authoring export.
- Added aspect-native trybuild fixtures proving facade callers cannot treat
  `WORTHQueryAspect::external_projection_path()` or
  `WORTHQueryTestBackendSchema::aspects()` projection paths as `&str`.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, `TRYBUILD=overwrite` and normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test
  -p worth-query memory_workspace --lib`, focused `cargo test -p worth-query
  consumer_kit::test_backend --lib`, line-count checks, and focused scans for
  the removed terminal helpers/string fields.
- This is a Law 41 storage-authority hardening slice. It intentionally keeps
  public string authoring at `WORTHQueryAspect::new(...)`, but once accepted,
  downstream memory workspace and consumer-kit schema code cannot promote the
  stored projection authority from terminal text.

### Runtime read/preview native touch construction slice

- Added `WORTHQueryParsedAspectTarget::from_native_parts(...)` and
  `WORTHQueryAspectTouch::from_native_parts(...)` so native callers can build
  touch proof values from `AspectKey` plus `CanonicalFieldPath` without
  formatting an aspect path string and reparsing it.
- Runtime read obligation dispatch now lowers declarative projection,
  predicate, and ordering fields into native `WORTHQueryAspectTouch` values
  directly. The old `format!("{}.{}", aspect, field)` read-touch construction
  helper is gone.
- Preview live routing now builds request projection touches natively before
  comparing them with mutation deltas. Terminal affected-aspect maps from live
  and computed routing remain an explicit next root, not silently solved here.
- Graph touch descriptor inventory now stores and counts declared/touched
  aspect sets as `WORTHQueryAspectTouch` and
  `WORTHQueryAspectMutationOperation` instead of retaining
  `BTreeSet<String>` path/operation projections.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, focused `cargo test -p worth-query
  read_obligation_dispatch --lib`, focused `cargo test -p worth-query preview
  --lib`, focused `cargo test -p worth-query graph_touch_descriptor --lib`,
  line-count checks, and focused scans for removed read/preview string
  construction helpers and descriptor inventory string sets.
- This was another proof-flow slice, not the desired broad red root:
  compilation stayed green. The next slice should deliberately break the
  preview/session affected-aspect maps or another live runtime carrier that
  still moves aspect identity as `Vec<String>`.

### Preview/live affected-aspect native propagation slice

- Preview session routing now keeps `live_affected` and `computed_affected`
  maps as `BTreeMap<String, Vec<WORTHQueryAspectTouch>>` instead of projecting
  native affected touches to `Vec<String>` between live, computed, and effect
  routing.
- Removed the preview-local `terminal_aspect_paths_projection(...)` and
  `preview_aspect_touch(...)` round-trip. Computed and effect preview
  relevance now consume native touch sets directly.
- Live subscription delivery relevance no longer converts
  `delta.touched_aspects()` into terminal aspect paths or uses string
  `starts_with(...)` to decide projected/focused/grouped relevance. It now
  compares request field touches with `matches_or_contains(...)` and compares
  focused/grouping aspects by native `AspectKey`.
- Terminal affected live view ids remain strings because they identify view
  handles, not aspect authority. Computed/effect inspection digest helpers and
  write-receipt reporting still have terminal aspect-list projection helpers
  and should be evaluated as the next possible root.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, focused `cargo test -p worth-query live --lib`,
  focused `cargo test -p worth-query preview --lib`, attempted focused
  `cargo test -p worth-query live_subscription --lib` (0 matching tests), line
  count checks, and focused scans proving the preview/session affected maps and
  live subscription relevance path no longer route aspect identity through
  `Vec<String>`.
- This slice improved Law 41 enforcement inside runtime routing, but again
  stayed compiler-green. The next broad-red candidate should be a public or
  cross-module carrier, not another fully local helper.

### Computed/effect inspection aspect-list native evidence encoding slice

- Computed inspection identity helpers now accept
  `&[WORTHQueryAspectTouch]` for dependency aspects, produced aspects, and
  derived patch aspect touches. The local computed
  `terminal_aspect_paths_projection(...)` helper is gone.
- Effect inspection identity now maps trigger, condition input/output, and
  pending delivery touch slices directly from native `WORTHQueryAspectTouch`
  values into terminal evidence fields only at the digest/report boundary.
- Removed `effect::delivery_helpers::terminal_aspect_paths_projection(...)`.
  Delivery helpers keep the native touch digest sequence helper, so callers no
  longer manufacture an intermediate `Vec<String>` as the semantic aspect-list
  source before evidence encoding.
- This is a narrower Law 41 cleanup than the earlier public constructor and
  admission breaks: it does not add a new public compile-fail fence, but it
  prevents computed/effect inspection from pretending terminal strings are the
  proof-bearing carrier inside the pipeline.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, focused `cargo test -p worth-query computed --lib`,
  focused `cargo test -p worth-query effect --lib`, line-count checks, and
  targeted scans proving the removed computed/effect terminal projection
  helpers and intermediate aspect-path vectors are gone.
- This stayed compiler-green, so it should not count as the requested broad
  root break. The next implementation slice should deliberately break a
  public or cross-module carrier whose signature still accepts terminal
  strings/JSON where a native proof state should flow.

### Affected runtime view target proof carrier slice

- Write, batch, and intent receipts now retain affected live and derived view
  evidence as `WORTHQueryLiveArtifactTarget` and
  `WORTHQueryDerivedMaterializationTarget` values instead of raw
  `Vec<String>` storage.
- Backend/source adapter `affected_live_view_ids(...)` remains a terminal
  external ingress, but runtime routing immediately admits each view handle
  through the runtime-only `WORTHQueryLiveArtifactTarget::from_view_name(...)`
  constructor. Computed affected derived handles are likewise admitted through
  the runtime-only derived materialization target constructor before they reach
  receipts or effect routing.
- Effect routing, receipt aggregation, graph-composition counters, feedback
  counters, and intent-delivery counters now consume typed target carriers.
  Digest/reporting paths use explicit
  `terminal_affected_*_ids_projection()` helpers when terminal view text is
  required.
- Removed the old public raw affected-id accessors from write receipts, batch
  receipts, intent receipts, and unified batch write inspection. Public callers
  now see typed target accessors or intentionally named terminal projection
  helpers.
- Added the aspect-native compile-fail fixture
  `affected_view_ids_terminal_accessors_removed.rs` so the removed public raw
  accessors stay removed mechanically.
- The aspect API finalization certification fixture now uses `delete_with(...)`
  plus explicit aspect touches for the insert/update/delete lane. Bridge
  writeback now correctly rejects a bare delete that carries no admitted aspect
  operation, so the certification row had to prove the native affected aspect
  basis instead of depending on a no-op delete shortcut.
- Verification covered a deliberate red then green `cargo check -p
  worth-query --tests`, `TRYBUILD=overwrite` and normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused live mutation
  receipt routing, batch receipt aggregation, aspect API finalization preview
  rows, all `batch --lib` tests, focused effect delivery routing, focused
  `computed --lib`, and slice-specific line counts. The earlier full
  `live --lib` filter was not rerun after expectation updates; the formerly
  failing live insert receipt test passed directly.
- This is the requested broad red root break: the old raw affected view id
  receipt APIs no longer compile, and downstream runtime/effect/inspection
  consumers have to flow through Law 41-style proof carriers until the final
  terminal reporting boundary.

### Workspace live-view builder field-key native admission slice

- Changed public `WORTHQueryLiveViewBuilder::select(...)` and
  `order_by(...)` so callers must pass `AspectFieldKey` values instead of
  dotted `aspect.field` strings. The builder now stores selected and ordering
  fields as native authoring field-key proof values until declarative request
  construction.
- Exported `AspectFieldKey` through the foundation facade so public callers
  have the native field-key route available at the same boundary where the
  builder is exported.
- Workspace live-view build still emits terminal field text for delivered
  names and schema-basis labels, but the text is derived from the admitted
  field key at that reporting/presentation boundary rather than reparsed as
  request authority.
- The deliberate red `cargo check -p worth-query --tests` exposed 226 builder
  consumers across consumer-kit workspace tests, public bridge tests,
  graph-composition/read-maintenance integration tests, runtime mutation/read
  tests, aspect API certification, and runtime API transcript generation.
  Those callers now construct `AspectFieldKey` explicitly.
- Runtime transcript generation keeps its static spec strings local to a
  transcript helper that mints `AspectFieldKey` before invoking the builder;
  the public builder itself no longer accepts terminal field text.
- Added an aspect-native trybuild fixture proving facade callers cannot pass
  `["identity.id"]` to `select(...)` or `"identity.id"` to `order_by(...)`.
- Verification covered `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, green `cargo check -p worth-query`,
  `TRYBUILD=overwrite` and normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  workspace_declaration --lib`, focused `cargo test -p worth-query --test
  in_memory_test_backend_facade`, slice-specific line-cap checks, and a
  targeted literal-builder scan proving only the intentional negative fixture
  still uses the removed string-shaped builder calls.
- Six pre-existing oversized mutation test files grew slightly from explicit
  field-key construction and were added to the Rust line-cap allowlist with
  scoped removal triggers. This keeps the authority-boundary slice focused
  while making the exemption visible.

### Schema view constructor native-name admission slice

- Changed public `SchemaFieldView::new(...)` so callers must pass admitted
  `AspectName` and `FieldName` values instead of raw aspect/field strings.
- Changed public `SchemaRelationView::new(...)` so callers must pass an
  admitted `RelationName` instead of raw relation text. The existing zero-depth
  rejection remains inside schema relation construction.
- Exported `AspectName` and `FieldName` through the facade beside
  `RelationName`, and updated `worth_query_schema!` to mint the native name
  carriers at macro expansion before constructing schema evidence.
- The deliberate red `cargo check -p worth-query --tests` exposed 129 schema
  constructor consumers across graph-read tests, runtime declaration,
  lower-runtime/intent certification fixtures, view-shape/live support, saved
  query fixtures, and schema macro expansion.
- Workspace live-view declaration now admits builder-derived schema aspect,
  field, and relation names in a scoped schema-admission helper before building
  `QuerySchemaView`; the schema evidence objects no longer parse terminal text
  themselves.
- Split `workspace_declaration_schema.rs` and
  `workspace_live_view_declaration.rs` out of the builder file to keep touched
  Rust files under the workspace line cap.
- Added aspect-native trybuild coverage proving facade callers cannot pass raw
  strings to `SchemaFieldView::new(...)` or `SchemaRelationView::new(...)`.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, green `cargo check -p
  worth-query`, `TRYBUILD=overwrite` and normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  schema_view --lib`, focused `cargo test -p worth-query workspace_declaration
  --lib`, line-count checks, and a structural residue check proving raw schema
  constructor calls survive only in the intentional negative fixture.

### Policy aspect mask field-key native admission slice

- Changed `PolicyAspectMask` so its internal entries are keyed by
  `AspectFieldKey` instead of joined `String` field labels.
- Changed `PolicyAspectMask::with_masked(...)` and
  `with_non_disclosing_use_only(...)` so callers must pass an admitted
  `AspectFieldKey`; raw `(aspect, field)` strings no longer satisfy the mask
  API.
- Removed the crate-local `visibility_for_parts(...)` lookup path. Authorized
  projection derivation now promotes canonical `AspectName`/`FieldName` pairs
  into `AspectFieldKey` proof values before mask lookup, so projection, result
  shape, predicate, ordering, and influence checks use one typed field-key
  authority.
- The deliberate red `cargo check -p worth-query --tests` exposed 32 remaining
  raw mask consumers across authorized projection tests, policy narrowing,
  policy plan/execution seam tests, and milestone-nine certification rows.
  Those callers now construct `AspectFieldKey` explicitly.
- Added aspect-native trybuild coverage proving facade callers cannot pass raw
  strings to policy mask construction, and updated the older mask-privacy
  fixture so it still tests private field mutation rather than failing earlier
  on argument shape.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, green `cargo check -p
  worth-query`, `TRYBUILD=overwrite` and normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  authorized_projection --lib`, focused `cargo test -p worth-query
  policy_narrowing --lib`, line-count checks, and a targeted residue check
  confirming the raw string mask calls survive only in the intentional
  negative fixture.
- This is a direct Arch Law 41 mechanical enforcement slice: policy visibility
  decisions now require the field-key proof state instead of accepting weaker
  text and reconstructing authority inside the policy mask.

### Mutation receipt touched-aspect native evidence encoding slice

- Write receipt committed-truth identity now derives committed delta
  descriptors from `WORTHQueryAspectTouch::native_digest_part()` instead of
  first projecting touched aspects into terminal aspect-path strings.
- Batch write receipt identity now encodes touched aspects from native
  `WORTHQueryAspectTouch` values through a scoped evidence helper. The helper
  converts to text only inside the evidence identity field value, not as a
  retained semantic `Vec<String>`.
- Unified batch-write inspection digest inputs now carry
  `&[WORTHQueryAspectTouch]` for touched aspects instead of
  `&[String]` touched-aspect paths. Component inspection digest construction
  uses the same native touch evidence helper.
- Aspect API finalization certification now computes its
  `touched_aspect_digest` from native touch values instead of
  `terminal_aspect_paths_projection(...)`. The local certification terminal
  projection helper was removed.
- Split `batch_receipt_identity.rs` out of `batch_receipt.rs` after the helper
  migration pushed the receipt file over the workspace 400-line cap.
- Verification covered `cargo fmt -p worth-query`, red and green
  `cargo check -p worth-query --tests`, green `cargo check -p worth-query`,
  focused `cargo test -p worth-query batch_write --lib`, focused `cargo test
  -p worth-query aspect_api_finalization --lib`, line-count checks, and a
  targeted residue scan proving the touched receipt/inspection/certification
  slice no longer contains terminal touched-aspect projection helpers.
- This was a useful internal Law 41 evidence-encoding cleanup, but it stayed
  mostly compiler-green until the certification helper signature changed. It
  should not be treated as a broad root break; the next slice should again
  target a public or cross-module API that still admits weaker terminal aspect
  identity.

### Existing-truth probe shortcut native-touch admission slice

- Changed the crate-local `WORTHQueryWorkspace::probe_existing(...)` shortcut
  so it accepts `WORTHQueryAspectTouch` values directly instead of admitting an
  arbitrary iterator of terminal aspect-path strings and minting proof inside
  the shortcut.
- `WORTHQueryExistingTruthProbeRequest` already retained native touches; its
  request digest now encodes `WORTHQueryAspectTouch::native_digest_part()`
  directly instead of building a temporary `Vec<String>` terminal path
  projection before evidence sealing.
- Intent-admission routing certification no longer reprojects a native probe
  request back to strings just to exercise the workspace convenience path. The
  parity fixture now clones the request's retained aspect touches into the
  shortcut.
- The deliberate red `cargo check -p worth-query --tests` exposed the remaining
  shortcut consumers across intent-admission routing, verified-existing
  execution, bridge-backed verification support/execution, mixed-authority
  mutation tests, delete/update verification tests, and probe-existing runtime
  tests. Those call sites now pass native `test_aspect_touches(...)` proof
  values rather than raw arrays.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, green `cargo check -p
  worth-query`, focused `cargo test -p worth-query probe_existing --lib`,
  focused `cargo test -p worth-query intent_admission --lib`, touched-file
  line-count checks, and targeted scans confirming the probe shortcut no
  longer admits raw aspect-path arrays.
- This is a medium-strength Law 41 enforcement slice: it closes an internal
  cross-module convenience root and removes a terminal evidence projection from
  request identity, while leaving broader public-facade roots for later
  batches.

### Aspect-touch terminal projection runtime-local fence slice

- Narrowed `WORTHQueryAspectTouch::aspect_path_text_for_boundary()` from
  crate-wide visibility to `pub(in crate::runtime)`, so non-runtime modules can
  no longer recover terminal aspect path text from native touch proof carriers.
- The deliberate red `cargo check -p worth-query --tests` exposed seven
  outside-runtime consumers: consumer-kit test-backend equivalence reporting,
  intent-admission runtime certification, mutation eligibility seed identity,
  effect-triggered execution binding hashing, and memory workspace diagnostics.
- Evidence/reporting consumers now encode `WORTHQueryAspectTouch` with
  `native_digest_part()` instead of terminal path projection. Memory workspace
  denial messages also report the native digest material rather than calling
  through the runtime-only terminal boundary.
- The tiny runtime certification probe adapter now matches its fixture aspects
  by native digest material (`identity:id`, `title:value`) instead of terminal
  dotted strings.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, green `cargo check -p
  worth-query`, focused `cargo test -p worth-query memory_workspace --lib`,
  focused `cargo test -p worth-query consumer_kit::test_backend --lib`,
  focused `cargo test -p worth-query intent_admission --lib`, touched-file
  line-count checks, and a targeted scan proving outside-runtime production
  code no longer calls `aspect_path_text_for_boundary()`.
- This is a direct mechanical-enforcement improvement: the terminal projection
  still exists for runtime boundary code that must interoperate with terminal
  rows/digests, but it is no longer a crate-wide escape hatch.

### Aspect-touch terminal projection mutation-boundary fence slice

- Narrowed `WORTHQueryAspectTouch::aspect_path_text_for_boundary()` again from
  runtime-wide visibility to `pub(in crate::runtime::mutation)`, making the
  terminal aspect path projection callable only inside the mutation boundary
  that still lowers to terminal backend/reporting edges.
- The deliberate red `cargo check -p worth-query --tests` exposed 25
  production runtime consumers and 14 test consumers outside mutation:
  computed/effect inspection identities, unified write/batch inspection
  digests, preview execution evidence, graph-composition evidence, verified
  assumption evidence, batch receipt aggregation, runtime intent input
  encoding, and stateful bridge test adapters.
- Evidence, inspection, graph-composition, verified-assumption, and metadata
  consumers now use `WORTHQueryAspectTouch::native_digest_part()` instead of
  terminal dotted path projection.
- Stateful bridge test support stopped using terminal aspect text as its
  authoritative lookup/storage key. Existing-truth verification now keys test
  truth by native digest material, while stateful bridge external rows derive a
  `CanonicalFieldPath` directly from `WORTHQueryAspectTouch` native aspect and
  field-path parts.
- Graph-composition resolution assertion helpers now compare expected fixture
  aspect labels by first admitting them into `WORTHQueryAspectTouch` and then
  comparing native digest material, preserving readable test fixtures without
  preserving terminal strings as the proof value.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, green `cargo check -p
  worth-query`, focused `cargo test -p worth-query computed --lib`, focused
  `cargo test -p worth-query effect --lib`, focused `cargo test -p worth-query
  batch_write --lib`, focused `cargo test -p worth-query graph_composition
  --lib`, touched-file line-count checks, and a targeted scan confirming
  `aspect_path_text_for_boundary()` survives only under `runtime::mutation`.
- This is a stronger mechanical Law 41 fence than a naming cleanup: code outside
  the mutation proof/lowering boundary cannot call the terminal projection at
  all, so downstream phases must consume native touch proof or native digest
  material.

### Graph-obligation selector/index native evidence encoding slice

- Graph-obligation selector identity now encodes `WORTHQueryAspectTouch`
  values with `native_digest_part()` and names the selector kind
  `aspect-touch` instead of deriving selector evidence from terminal
  dotted-path projections.
- Declared aspect-operation selector values, declared mutation collection
  selector values, lookup-key values, and touch descriptor row evidence now
  encode native touch digest material. The old graph-composition helper names
  `touch_paths_projection(...)` and `terminal_declared_aspect_operation(...)`
  are gone.
- Graph obligation lookup key derivation no longer calls its touched-key lane
  `touched_aspect_path`; descriptor counts now expose
  `declared_aspect_touch_count()`, read-obligation verbs use
  `ObservesAspect`, and graph-obligation execution scopes use
  `TouchedAspect`.
- The deliberate red `cargo check -p worth-query --tests` exposed the remaining
  production/test consumers of the old public vocabulary:
  runtime read-obligation dispatch, graph-touch descriptor unit tests, and the
  read-obligation selector hardening test. Those consumers now use the native
  names.
- The graph-touch descriptor compile-fail fixture was updated so it continues
  to prove private struct-literal construction while naming the native private
  field.
- Verification covered `cargo fmt -p worth-query`, deliberate red and final
  green `cargo check -p worth-query --tests`, focused `cargo test -p
  worth-query graph_obligation --lib`, focused `cargo test -p worth-query
  graph_composition --lib`, focused `cargo test -p worth-query
  read_obligation_dispatch --lib`, `cargo test -p worth-query --test
  phase_boundaries_graph_touch_descriptor_compile_fail`, touched-file
  line-count checks, and targeted graph-composition scans confirming the
  selector/index/descriptor lane no longer contains the removed terminal helper
  names or `aspect_path_text_for_boundary()` calls.
- This slice is a medium mechanical Law 41 improvement: the core graph
  obligation matching types already carried native touches, but evidence,
  lookup, counters, and public read/descriptor vocabulary no longer teach
  terminal aspect paths as the semantic graph touch identity.

### Mutation aggregate evidence native-touch hard fence slice

- Mutation declared-aspect aggregate evidence now writes `aspect_touch` fields
  from `WORTHQueryAspectTouch::native_digest_part()` instead of retaining
  terminal `aspect_path` evidence values.
- Symbolic aspect-reference rows, existing-truth assertion denials,
  existing-truth assertion verification rows, existing-truth probe denials,
  probe fields, and probe rows now encode native touch digest material.
- The internal denied-aspect helper was renamed from
  `WORTHQueryDeniedAspectPath` / `denied_aspect_path` to
  `WORTHQueryDeniedAspectTouch` / `denied_aspect_touch`.
- Duplicate aspect diagnostics in mutation builders now report native touch
  digest material, and the stale graph helper name
  `contains_all_aspect_paths(...)` was renamed to
  `contains_all_aspect_touches(...)`.
- Most importantly, deleting the last production consumers exposed that
  `WORTHQueryAspectTouch::aspect_path_text_for_boundary()` and
  `WORTHQueryParsedAspectTarget::authoring_path_projection()` were no longer
  needed. Both methods were removed, and `WORTHQueryParsedAspectTarget` no
  longer retains the original authoring string after admission.
- The aspect-native compile-fail suite was updated for the private-field
  wording drift caused by renaming assertion/probe denial internals; the
  public negative tests still prove the removed path aliases/projections stay
  unavailable.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, focused `cargo test -p worth-query probe_existing
  --lib`, focused `cargo test -p worth-query existing_truth --lib`, focused
  `cargo test -p worth-query mixed_authority --lib`, focused `cargo test -p
  worth-query graph_obligation --lib`, `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, touched-file line-count checks, and
  targeted scans proving `aspect_path_text_for_boundary`,
  `authoring_path_projection`, `denied_aspect_path`,
  `contains_all_aspect_paths`, and mutation `aspect_path` evidence tags are
  gone.
- This is a strong mechanical Law 41 slice: after aspect target admission,
  Query no longer stores or exposes a terminal dotted-path reconstruction
  helper inside `runtime::mutation`; consumers must use native touch proof
  state or native digest material.

### Verified-assumption read-set native-touch evidence slice

- Verification read-set breadth vocabulary now names admitted touches instead of
  terminal paths: `distinct_asserted_aspect_touch_count`,
  `distinct_asserted_aspect_touches`, and `asserted_aspect_touch` evidence
  replace the stale `*_aspect_path*` wording.
- `WORTHQueryVerifiedAssumptionSet` continues to retain
  `WORTHQueryAspectTouch` values for asserted aspects, and its digest evidence
  encodes each touch through `native_digest_part()` rather than by recovering a
  dotted aspect path.
- Graph-composition assumption summaries now collect
  `native_asserted_aspect_touch_digest_parts(...)`; the old
  `terminal_asserted_aspect_paths_projection(...)` helper is gone.
- Runtime graph-composition and bridge-backed verification snapshots now assert
  touch-vocabulary counter rows, so the observable evidence surface no longer
  teaches asserted aspect paths as verification authority.
- Phase-boundary trybuild fixtures were updated to keep proving that
  verification read-set and assumption-summary struct-literal construction is
  private while naming the native-touch private fields.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, focused `cargo test -p worth-query graph_composition
  --lib`, focused `cargo test -p worth-query existing_truth --lib`, focused
  `cargo test -p worth-query bridge_backed_verification_execution --lib`,
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, and
  `cargo test -p worth-query --test phase_boundaries_compile_fail`.
- This was a vocabulary/evidence hardening slice rather than a broad-red root:
  the underlying storage was already native, but the public evidence language
  and graph-composition summary helper now align with Law 41 instead of
  preserving terminal path authority by implication.

### Declaration aspect contract native-key root break

- Broke the next broad root deliberately: `WORTHQueryDeclarationAspectContract`,
  `WORTHQueryDeclarationAspectCoverage`, `WORTHQueryDeclarationAspectPublication`,
  and grouped aspect participation now retain `AspectFieldKey` values instead
  of `Vec<String>` semantic aspect labels.
- Production constructors for those carriers now require typed keys. The old
  `from_slices(&[&str], ...)` helpers are restricted to unit-test builds, so
  non-test production/facade consumers cannot mint declaration aspect authority
  from raw strings.
- Evidence/reporting edges in declaration bridge routing, continuation
  prepared digests, route publication summaries, and grouped participation
  hashing explicitly call `terminal_declaration_aspect_projection(...)` or
  publication terminal projection helpers at the boundary.
- The deliberate `cargo check -p worth-query --tests` red exposed 66
  test/support consumers that still treated declaration aspect carriers as
  string lists. Those consumers now either compare native `AspectFieldKey`
  fixtures through test-only admission helpers or call explicit terminal
  projection helpers when the test is asserting evidence text.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, green plain `cargo check -p worth-query`, focused
  `cargo test -p worth-query declaration_aspect --lib`, focused `cargo test -p
  worth-query declaration_publication --lib`, focused `cargo test -p
  worth-query declaration_route_plan --lib`, focused `cargo test -p
  worth-query declaration_receipt --lib`, focused `cargo test -p worth-query
  declaration_signal_compatibility --lib`, focused `cargo test -p worth-query
  declaration_legality --lib`, focused `cargo test -p worth-query
  declaration_progression --lib`, focused `cargo test -p worth-query
  declaration_entry_orchestration --lib`, focused `cargo test -p worth-query
  declaration_entry_seam --lib`, focused `cargo test -p worth-query
  declaration_relational_routing --lib`, focused `cargo test -p worth-query
  contribution_composed_orchestration --lib`, touched-file line counts, and a
  targeted declaration aspect string-fixture residue scan. The full
  aspect-native compile-fail harness also passed with `cargo test -p
  worth-query --test aspect_native_query_compile_fail`.
- This is a strong mechanical Law 41 root: declaration aspect meaning now
  crosses declaration, route, envelope, authority summary, grouped authoring,
  continuation, and contribution-composed proof records as native field keys.
  String materialization is confined to terminal evidence/reporting and
  test-only fixture projection.

## Query retained-scalar public API native field-path fence

- Closed the retained-scalar compatibility hole left by the terminal ingress
  quarantine slice: `WORTHQueryDerivedArtifactBinding::consume_scalar_fields(...)`
  now requires `WORTHQueryRetainedFieldPath` values instead of raw field-path
  strings.
- `WORTHQueryDerivedArtifactBinding::verify_scalar_alignment(...)` now requires
  pairs of `WORTHQueryRetainedFieldPath` values, so alignment cannot be requested
  with arbitrary dotted text after derived-view handle admission.
- Removed the old public/by-name string fallback path for retained scalar
  extraction and alignment. The surviving implementation path flows through
  retained field-path proof carriers and retained materialized row proof state.
- Deliberate red `cargo check -p worth-query --tests` exposed retained scalar
  test consumers that still passed raw strings or string pairs; those consumers
  were migrated to scoped retained field-path fixtures.
- Added `retained_scalar_public_apis_reject_raw_field_strings.rs` to the
  aspect-native trybuild harness. It proves facade callers cannot pass raw
  field strings to either retained scalar extraction or scalar alignment.
- Verification: `cargo fmt -p worth-query`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query retained_scalar --lib`, focused `cargo test -p
  worth-query projection_consumption --lib`, green `cargo check -p
  worth-query`, touched-file line counts, and the earlier deliberate red
  `cargo check -p worth-query --tests` pass captured in
  `target/aspect-native-retained-scalar-red.log`.

## Query live-read execution installation proof fence

- Removed the crate-local `WORTHQueryRuntime::execute_live_read_by_name(...)`
  execution shortcut. Direct live reads now execute through
  `execute_live_read_for_installation(...)`, consuming
  `WORTHQueryRuntimeLiveSubscriptionInstallation` proof state instead of
  reconstituting read authority from a terminal view-name string.
- `WORTHQueryRuntime::read_live_result(...)` and canonical live-read receipt
  tests now flow through the live view handle's subscription installation.
- Live artifact targets constructed from `&WORTHQueryLiveView<_>` now retain
  the subscription installation. `read_live_artifact_bundle(...)` executes each
  target through that installation proof, so artifact reads no longer use a
  view-name execution shortcut. Synthetic `test_only(...)` targets remain
  name-only for hand-built fixture bundles.
- Retained upstream input collection for derived declarations still starts from
  declared upstream live-view names, but it performs a local subscription-state
  lookup to recover the installation proof before executing the live read.
- Deliberate red `cargo check -p worth-query --tests` exposed the remaining
  execution-by-name consumers in declaration upstream replay, live artifact
  bundle reads, intent-admission read tests, and live receipt posture tests.
- The existing artifact-target compile-fail fixture now proves
  `WORTHQueryLiveArtifactTarget::new("...")` is absent rather than merely
  private, strengthening the raw-name construction fence.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query live_receipt --lib`, focused `cargo test -p
  worth-query live_artifact --lib`, focused `cargo test -p worth-query
  runtime_read_live_delegates_to_canonical_live_read_execution --lib`,
  touched-file line counts, and a residue scan proving the removed
  `execute_live_read_by_name` method is gone from callable source.

## Query memory-workspace aspect declaration native carrier fence

- `WORTHQueryAspect` now stores a native `WORTHQueryAspectTouch` plus
  `CanonicalFieldPath` instead of retaining an aspect label string and parsing
  the external projection path inside its public constructor.
- `WORTHQueryAspect::new(...)` now requires those native carriers directly.
  Public callers cannot declare a memory-workspace aspect with raw terminal
  strings for either aspect identity or external projection path.
- Memory workspace collection setup, projection scope construction, entity
  projection extraction, and declared-aspect matching now consume the stored
  native touch directly instead of repeatedly reparsing the aspect label.
- The consumer-kit test backend remains an authoring boundary: it accepts
  ergonomic schema strings, validates/advises invalid native aspect labels with
  `InvalidAspectLabel`, lowers projection text into `CanonicalFieldPath`, and
  hands native carriers to `WORTHQueryAspect`.
- The memory workspace tests now admit string fixtures through scoped helpers
  before constructing `WORTHQueryAspect`, preserving fixture readability without
  restoring the production raw-string constructor.
- The existing aspect-native trybuild fixture now proves
  `WORTHQueryAspect::new("...", "...")` cannot compile and that
  `external_projection_path()` exposes a `CanonicalFieldPath`, not `&str`.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, green `cargo check -p worth-query`,
  trybuild overwrite plus normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  memory_workspace --lib`, focused `cargo test -p worth-query
  consumer_kit::test_backend --lib`, touched-file line counts, and residue
  scans proving raw `WORTHQueryAspect::new("...", "...")` survives only in the
  intentional negative fixture.

## Query aspect-touch public raw-string constructor removal slice

- Removed the public `WORTHQueryAspectTouch::new(...)` raw dotted-string
  constructor. Public callers now construct touch proof state through native
  `WORTHQueryAspectTouch::aspect(AspectKey)` or
  `WORTHQueryAspectTouch::field_path(AspectKey, CanonicalFieldPath)`.
- Kept terminal authoring-string parsing behind the explicitly named
  crate-local `WORTHQueryAspectTouch::from_authoring_path(...)` boundary for
  legacy fixture/admission code that still starts from ergonomic text. That
  keeps the remaining weaker boundary searchable by intent instead of leaving a
  neutral `new(...)` authority minting surface.
- The deliberate red `cargo check -p worth-query --tests` exposed the expected
  consumers across consumer-kit schema lowering, intent-admission certification
  fixtures, graph composition support, runtime support helpers, public bridge
  tests, and stabilization transcripts. Those consumers now either stay inside
  the named crate-local authoring parser or lower local test fixture strings
  into `AspectKey` and `CanonicalFieldPath` before constructing touch proof.
- Added aspect-native trybuild coverage proving facade callers cannot call
  `WORTHQueryAspectTouch::new("title.value")`, while existing compile-fail
  fixtures now use native touch construction as setup so they continue to test
  their original removed terminal helpers.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, and a
  focused residue scan proving `WORTHQueryAspectTouch::new` survives only in
  the intentional negative fixture. One touched public DX integration inventory
  file remains over the workspace default line cap from pre-existing structure;
  it should be split in a dedicated test-file cleanup rather than bundled into
  this authority-root slice.

## Query declarative live aspect-field constructor native-key fence

- Broke the public declarative live field-authoring root more deeply:
  projection fields, ordering fields, predicate filters, writeback changes, and
  writeback single-aspect intents now require `AspectFieldKey` instead of raw
  aspect/field strings.
- Kept ergonomic text parsing only behind explicitly named crate-local
  `*_from_authoring_parts(...)` helpers for legacy declarative lowering,
  certification fixtures, and tests that still start from authoring text. That
  keeps weaker input visible as a boundary instead of letting `new(...)` mint
  authority-shaped field carriers.
- Added public read-only `source_field_key()` accessors so downstream code can
  keep typed field identity without reconstructing dotted strings from
  accessor pairs.
- The deliberate `cargo check -p worth-query --tests` red exposed 59 real
  consumers across declarative live lowering, runtime workspace declaration,
  read-composition lowering, certification fixtures, runtime schema support,
  and writeback tests. Those consumers were migrated to typed carriers or named
  crate-local authoring boundaries without restoring the public raw-string
  constructors.
- Added aspect-native trybuild coverage proving facade callers cannot pass raw
  strings into `DeclarativeProjectionField`, `DeclarativeOrderingField`,
  predicate filter constructors, or `DeclarativeWritebackChange`; refreshed the
  older alias-removal fixtures so their setup uses native `AspectFieldKey`
  values and continues to test the intended removed helper surface.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, green `cargo check -p worth-query`,
  trybuild overwrite plus normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, line-count checks, and constructor residue
  scans. `declarative_live.rs` remains over 400 lines but is already on the
  workspace Rust line-cap allowlist.

## Query retained-upstream dependency name proof fence

- Closed the retained-upstream proof hole where
  `WORTHQueryRetainedUpstreamInputs::new(...)` publicly minted retained input
  proof from raw view-name maps, and where public/raw lookup methods let callers
  ask for retained live or computed rows with only terminal view-name strings.
- Public retained-upstream access now requires either a typed live/derived view
  handle (`live_rows_for`, `retained_computed_rows_for`,
  `single_retained_computed_row_for`) or a declaration-scoped proof boundary
  (`declared_live_rows`, `declared_retained_computed_rows`,
  `single_declared_retained_computed_row`) that checks the requested name is an
  upstream declared by the `WORTHQueryDerivedView`.
- Removed public `WORTHQueryDerivedView::depends_on_live_name(...)` and
  `depends_on_derived_name(...)`. Remaining raw dependency-name replay is
  explicitly named
  `*_from_workspace_declaration(...)` and crate-local, so workspace declaration
  compatibility cannot masquerade as ordinary public derived-view authoring.
- Deliberate red `cargo check -p worth-query --tests` exposed retained-upstream
  consumers in runtime computed maintainers and runtime program support that
  were still walking upstream rows by raw names. Those consumers now carry the
  derived declaration into access and use declaration-scoped lookup.
- Added aspect-native trybuild coverage proving facade callers cannot call raw
  retained-upstream lookups, cannot publicly construct retained upstream inputs
  from raw maps, and cannot build derived-view dependencies from raw names.
- Added focused runtime coverage proving declaration-scoped single retained
  computed row access succeeds for declared upstreams and fails closed for
  undeclared names.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, green `cargo check -p worth-query`,
  trybuild overwrite plus normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  retained_upstreams_decode_single_computed_rows_through_query_runtime_floor
  --lib`, and retained-upstream/dependency-name residue scans.

## Query declaration terminal aspect projection helper fence

- Removed public terminal aspect-field projection helpers from
  `WORTHQueryDeclarationAspectPublication` and
  `WORTHQueryGroupedAspectParticipationSummary`. Public consumers now keep the
  native `AspectFieldKey` slices through `present()`, `widened()`, `elided()`,
  `masked()`, `present_any()`, `present_all()`, `masked_any()`, and
  `conflicting_any()` instead of recovering terminal strings from declaration
  proof carriers.
- Narrowed `terminal_declaration_aspect_projection(...)` from a crate-wide
  application re-export to an `application`-local helper. Grouped support now
  checks shared posture claims against native key parts, while grouped and
  continuation digest/reporting code own explicitly local
  `*_for_digest` terminal projection helpers.
- Captured the deliberate red compiler break in
  `target/aspect-native-declaration-terminal-projection-red.log`; it exposed
  the broad application re-export plus grouped/continuation consumers that
  still borrowed the generic terminal projector.
- Added `declaration_terminal_aspect_projection_helpers_removed.rs` to the
  aspect-native trybuild harness, proving facade callers cannot call the
  removed declaration publication or grouped participation terminal projection
  methods.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, green `cargo check -p worth-query`,
  trybuild overwrite plus normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  declaration_route_plan --lib`, focused `cargo test -p worth-query grouped
  --lib`, line-count checks, and focused scans showing the removed public
  terminal helpers survive only in the negative fixture.

## Query retained materialization maintainer proof fence

- Removed the free retained-row builder surface entirely and narrowed
  `WORTHQueryDerivedPatchPayload::from_retained_row(s)` plus
  `WORTHQueryDerivedViewMaterialization::{replace_retained_rows,push_retained_row}`
  to runtime-internal authority. Public/facade callers can no longer mint or
  publish retained materialization proof rows directly.
- Added scoped maintainer transitions:
  `replace_retained_scalar_row`, `push_retained_scalar_row`, and
  `WORTHQueryDerivedPatchPayload::from_retained_scalar_values`. External
  maintainers can still express retained native scalar output, but the retained
  row proof is constructed only by the runtime materialization/payload boundary.
- The deliberate red `cargo check -p worth-query --tests` exposed the public
  bridge hostile maintainer plus runtime API stabilization and aspect API
  finalization maintainers that still depended on row-taking materialization
  methods. Those consumers now publish retained scalar values through the
  scoped transition API.
- Added aspect-native trybuild coverage proving facade callers cannot call the
  removed retained-row builder or construct derived patch payloads from retained
  row proof objects. Refreshed the raw-JSON payload fixture so it now also
  records the private retained-row payload constructor.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, green `cargo check -p worth-query`,
  trybuild overwrite plus normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused public bridge runtime bootstrap,
  public bridge reader-lane honesty, hostile certification, runtime API
  stabilization, and aspect API finalization tests. Line-count checks confirmed
  the newly touched small files remain under 400; `runtime/computed/surface.rs`
  remains on the explicit workspace allowlist.

## Query memory-workspace external projection row conversion

- Converted the memory-workspace entity row authority from a sidecar
  `external_projection_values` map to native `CanonicalFieldPath` field
  values. `WORTHQueryAspect::external_projection_path()` remains the declared
  schema/import mapping, but materialized `WORTHQueryEntity` rows no longer
  teach consumers to recover authority from an external projection object.
- Removed the public row APIs that exposed the old model:
  `from_external_projection_values(...)`, `external_projection_values()`,
  `external_scalar_value(...)`, and `external_aspect_value(...)`. Public row
  consumers now use `from_native_field_values(...)` and
  `scalar_value_at(CanonicalFieldPath)`.
- The deliberate red `cargo check -p worth-query --tests` exposed real
  consumers in program materialization, projection-consumption extraction,
  runtime read composition, public bridge adapters, memory-workspace tests, and
  runtime support helpers. Those consumers now flow through native row field
  values instead of external row vocabulary.
- Added aspect-native compile-fail coverage proving facade callers cannot call
  the removed external row constructor, external row map accessor, string scalar
  lookup, or external aspect lookup. The forbidden API names now survive only
  in that negative fixture.
- Verification: `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, green `cargo check -p worth-query`, normal `cargo test
  -p worth-query --test aspect_native_query_compile_fail`, focused
  `memory_workspace --lib`, `runtime_backed_read_bootstrap`,
  `public_bridge_runtime_bootstrap`, `public_submission_lane_replacements`,
  `in_memory_test_backend_facade`, and `projection_consumption --lib`. Line
  counts are under 400 for the newly touched row/workspace/support files;
  `program.rs` remains over 400 only under the existing CI allowlist.

## Query backend-admissible mutation proof fence

- Strengthened `WORTHQueryBackendAdmissibleMutation` from a thin unwrap-capable
  command wrapper into the handoff proof required by backend execution. It
  still exposes `command()` as a read-only diagnostic/execution view, but
  public callers can no longer unwrap it back into a weaker
  `WORTHQueryWriteCommand`.
- Changed `WORTHQueryRuntimeWriteAuthorityAdapter::write`,
  `write_batch`, bridge mutation authority construction, and write-authority
  execution receipt construction to accept `WORTHQueryBackendAdmissibleMutation`
  instead of raw authored commands.
- The deliberate red `cargo check -p worth-query --tests` exposed exactly the
  expected consumers: bridge-backed execution, consumer-kit test backend,
  stateful bridge runtime, public bridge runtime adapter, intent-admission
  certification write authority, runtime API transcript authority, lower
  runtime representative fixtures, and runtime drift tests. Those consumers now
  inspect the admitted command through `mutation.command()` while authority and
  receipt construction stay bound to the admitted proof object.
- Added aspect-native compile-fail coverage proving facade callers cannot call
  `from_admitted_command(...)`, cannot use the removed `into_command()` escape,
  and cannot construct `WORTHQueryBackendAdmissibleMutation` by struct literal.
- Verification: `cargo fmt -p worth-query`, deliberate red then green
  `cargo check -p worth-query --tests`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused write-authority drift and
  signal-invalidation receipt tests, `public_bridge_runtime_bootstrap`,
  `intent_admission --lib`, `intent_admission_public_dx`, and
  `public_submission_lane_replacements`. Touched code/test files for this slice
  are under the 400-line cap.

## Query write-command public variant construction fence

- Marked every `WORTHQueryWriteCommand` struct variant as `#[non_exhaustive]`.
  Runtime internals can still construct and match lower-level command phases,
  but public/facade callers can no longer mint command proof state directly
  with enum variant literals such as `WORTHQueryWriteCommand::Delete { ... }`.
- This preserves `WORTHQueryWriteCommand` as an observable lower-level command
  carrier while forcing external callers back through workspace, builder,
  program, intent, or other authoring/admission APIs that construct the command
  in the intended proof order.
- Split the tail command accessor impl into
  `runtime/surface/mutation/command/accessors.rs` so the touched command module
  stays below the 400-line workspace cap after adding the variant fences.
- Updated the old declared-aspect-path negative fixture so it no longer depends
  on direct public command construction as setup.
- Added `write_command_variant_construction_not_public.rs` to the
  aspect-native compile-fail harness. It proves a facade caller receives
  `E0639: cannot create non-exhaustive variant using struct expression` when
  trying to construct a command variant directly.
- Verification covered `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, trybuild overwrite plus normal `cargo test -p
  worth-query --test aspect_native_query_compile_fail`, focused `cargo test -p
  worth-query mutation::aspect_crud --lib`, focused `cargo test -p
  worth-query runtime_write --lib`, and touched-file line counts. The regular
  cargo-check pass stayed green because remaining direct command construction
  is crate-internal or fixture-local; the external facade break is proven by
  trybuild.
- This is a Law 41 mechanical-enforcement slice: a weaker external caller can
  no longer skip the authoring/admission transition by assembling a lower-level
  command phase object directly.

## Query verified existing-truth assertion runtime minting fence

- Changed the public
  `WORTHQueryRuntimeExistingTruthVerificationAdapter::verify_existing_truth_assertion`
  contract so adapters return only `Result<(), WORTHQueryExistingTruthAssertionDenial>`.
  External verification adapters can now accept or deny native aspect values,
  but they no longer mint `WORTHQueryVerifiedExistingTruthAssertion`.
- Moved the verified assertion transition into
  `WORTHQueryBridgeBackedRuntimeBackend`. After the adapter accepts the binding
  and native aspects, the runtime constructs the verified assertion with its
  own `current_snapshot_identity()`, keeping the proof basis with the runtime
  that owns the snapshot boundary.
- Restricted
  `WORTHQueryVerifiedExistingTruthAssertion::from_snapshot_identity(...)` to
  `pub(in crate::runtime)`. The lower helper remains crate-internal, so public
  facade callers cannot turn a snapshot label plus asserted values into
  verified truth.
- Migrated certification fixtures, runtime test adapters, and the public bridge
  runtime support adapter to validate native aspect values and return `Ok(())`
  instead of fabricating assertion proof state.
- Added
  `verified_existing_truth_assertion_snapshot_constructor_not_public.rs` to the
  aspect-native compile-fail harness. It proves facade callers receive a
  private-function error when trying to call `from_snapshot_identity(...)`.
- Verification covered deliberate red then green `cargo check -p worth-query
  --tests`, `cargo fmt -p worth-query`, normal `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused `cargo test -p worth-query
  verify_existing --lib`, focused `cargo test -p worth-query probe_existing
  --lib`, `cargo test -p worth-query --test public_bridge_runtime_bootstrap`,
  and touched-file line counts.
- This is a Law 41 mechanical-enforcement slice: an adapter may perform the
  verification act, but only the runtime can transition admitted native aspect
  values plus the current snapshot into a verified existing-truth proof.

## Query retained-upstream declared lookup handle fence

- Removed public raw-name declared-upstream lookup from
  `WORTHQueryRetainedUpstreamInputs`. Maintainers can no longer ask for
  declared live or computed upstream rows by passing arbitrary view-name
  strings after derived-view admission.
- Added handle-shaped public lookups:
  `declared_live_rows_for(...)`,
  `declared_retained_computed_rows_for(...)`, and
  `single_declared_retained_computed_row_for(...)`. A maintainer that wants a
  specific upstream must carry the `WORTHQueryLiveView` or
  `WORTHQueryDerivedViewHandle` it declared against.
- Added aggregate declared-row-set iterators for count/snapshot maintainers:
  `declared_live_row_sets(...)` and
  `declared_retained_computed_row_sets(...)`. These enumerate the declaration's
  admitted upstreams without taking a caller-provided raw name.
- The deliberate red `cargo check -p worth-query --tests` exposed the remaining
  raw-name consumers in runtime computed maintainers and program test support.
  Those consumers now either use declaration-owned row-set iterators or capture
  concrete live/derived handles for specific row access.
- Extended `retained_upstream_inputs_raw_name_lookup_not_public.rs` so the
  aspect-native compile-fail harness proves facade callers cannot call the old
  raw declared lookup methods, in addition to the older raw construction and
  non-declared lookup attempts.
- Verification covered deliberate red then green `cargo check -p worth-query
  --tests`, `cargo fmt -p worth-query`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query retained_upstreams --lib`, focused
  `cargo test -p worth-query derived_materialization_bundle --lib`, and scans
  proving the removed raw declared lookup method names survive only in the
  negative fixture. `runtime/computed/surface.rs` and
  `runtime/tests/computed.rs` remain over the normal line cap only under the
  existing explicit CI allowlist.
- This is a Law 41 mechanical-enforcement slice on the read/materialization
  ladder: retained upstream rows can be consumed through admitted declaration
  row sets or previously proven handles, not reconstructed from terminal view
  names at the maintainer boundary.

## Query artifact target-name projection fence

- Removed public neutral `target_view_names()` enumeration from retained
  materialization bundles, live artifact bundles, retained artifact bindings,
  and live artifact bindings. Callers can no longer recover artifact target
  authority by iterating plain view-name strings.
- Added typed target iteration/accessors:
  `WORTHQueryDerivedMaterializationBundle::targets()`,
  `WORTHQueryLiveArtifactBundle::targets()`,
  `WORTHQueryDerivedArtifactBinding::targets()`, and
  `WORTHQueryLiveArtifactBinding::targets()`. Authority-bearing consumers now
  carry `WORTHQueryDerivedMaterializationTarget` or
  `WORTHQueryLiveArtifactTarget` proof carriers.
- Kept explicit terminal projection methods named
  `terminal_target_view_names_projection()` for reporting, digest text, and
  source-reference identity construction. This makes the string boundary
  visible and intentionally terminal instead of presenting it as an authority
  lookup surface.
- Removed retained-upstream public `live_view_names()` and
  `computed_view_names()` enumeration, closing the adjacent raw-name inventory
  path left after declared upstream lookup conversion.
- Converted retained/live projection-consumption extraction to iterate typed
  binding targets and resolve rows/materializations through target-shaped
  binding helpers. Source constructors project target names only at the
  terminal source-reference boundary.
- Removed raw-name binding lookup from `WORTHQueryLiveArtifactBinding` and
  `WORTHQueryDerivedArtifactBinding`. The compiler exposed a deeper retained
  scalar consumer after the derived binding lookup was removed; scalar fact
  extraction and scalar alignment now thread
  `WORTHQueryDerivedMaterializationTarget` internally instead of hopping
  through view-name text.
- Extended the aspect-native compile-fail harness so
  `artifact_bundle_by_name_lookup_not_public.rs` proves facade callers cannot
  call bundle/binding `target_view_names()` or binding raw-name lookup methods,
  and `retained_upstream_inputs_raw_name_lookup_not_public.rs` proves facade
  callers cannot enumerate upstream names.
- Verification covered deliberate red then green `cargo check -p worth-query
  --tests`, `cargo fmt -p worth-query`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  `cargo test -p worth-query derived_materialization_bundle --lib`, focused
  `cargo test -p worth-query post_write_artifact --lib`, focused
  `cargo test -p worth-query retained_scalar --lib`, focused
  `cargo test -p worth-query projection_consumption --lib`, final green
  `cargo check -p worth-query --tests`, and touched-file line counts. The only
  touched files over 400 lines are the already allowlisted
  `runtime/computed/surface.rs` and `runtime/tests/computed.rs`.
- This is a Law 41 mechanical-enforcement slice: artifact target membership
  and consumption flow through typed target proof carriers; strings are only
  terminal reporting/source-reference projections.

## Query command declared-collection identity fence

- Removed public collection string authority from
  `WORTHQueryWriteCommand` and `WORTHQueryBackendAdmissibleMutation`.
  Facade callers can no longer call `declared_collection()` or
  `declared_collection_ref()` on either surface.
- Added `declared_collection_identity()` returning
  `WORTHQueryMutationTargetCollectionIdentity`. Admission seeds, mutation
  handoffs, lower-runtime subject digests, consumer-kit backend admission,
  public bridge adapters, and runtime stabilization transcript authority now
  consume the collection target as a proof carrier instead of reconstructing it
  from text.
- Kept command collection text projection crate-internal and explicitly named
  `terminal_declared_collection_projection()` for receipt rows, preview
  receipts, symbolic maps, and graph touch descriptor reporting. Removed the
  unused backend-admissible terminal projection entirely.
- The deliberate red `cargo check -p worth-query --tests` exposed the expected
  string consumers across consumer-kit backend execution, intent-admission
  seeds/handoffs, lower-runtime subject digesting, runtime write/batch/preview
  receipt construction, graph descriptor validation/rows, public bridge
  adapters, and runtime test authorities.
- Added
  `mutation_declared_collection_string_accessors_removed.rs` to the
  aspect-native compile-fail harness. It proves public callers cannot use the
  old command/backend collection string methods and cannot call terminal
  collection projection as a public escape hatch.
- Verification covered deliberate red then green `cargo check -p worth-query
  --tests`, `cargo fmt -p worth-query`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  identity seed test
  `runtime::tests::identity_boundary::authoritative_mutation_batch_seed_composes_component_evidence_identities`,
  focused handoff test
  `runtime::tests::intent_admission::execution::batch::batch_write_delegates_to_canonical_admission_and_execution_handoff`,
  focused graph descriptor validation test
  `runtime::mutation::graph_composition::touch_descriptor::tests::descriptor_validation::mismatched_program_command_collection_is_denied_with_matching_counts`,
  `cargo test -p worth-query --test public_bridge_runtime_bootstrap`, scans
  for the removed accessors, and touched-file line counts.
- This is a Law 41 mechanical-enforcement slice: command and
  backend-admissible mutation collection state now flows as target identity
  proof between stages; terminal strings are only reporting labels inside the
  crate boundary.

## Query retained scalar target proof fence

- Converted retained scalar fact-set and scalar-alignment proof objects from
  retained view-name strings to `WORTHQueryDerivedMaterializationTarget`
  carriers. `WORTHQueryRetainedScalarFactSet` now exposes `target()`, and
  `WORTHQueryRetainedScalarAlignment` exposes `left_target()` and
  `right_target()`.
- Narrowed `WORTHQueryDerivedMaterializationTarget::view_name()` and
  `WORTHQueryLiveArtifactTarget::view_name()` to crate-internal visibility.
  Public callers that intentionally need label text must use the explicit
  `terminal_view_name_projection()` method on the proof carrier.
- Retained scalar extraction and alignment still project target names inside
  digest/error/report construction, but the stored semantic evidence now
  carries target proof instead of detached view-name strings.
- Added aspect-native compile-fail coverage:
  `artifact_target_view_name_alias_removed.rs` proves facade callers cannot
  call the neutral `view_name()` alias on derived/live artifact targets, and
  `retained_scalar_view_name_aliases_removed.rs` proves retained scalar
  fact/alignment results no longer expose raw view-name aliases.
- Verification covered green `cargo check -p worth-query --tests`,
  `cargo fmt -p worth-query`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query retained_scalar --lib`, final green
  `cargo check -p worth-query --tests`, removed-alias scans, and touched-file
  line counts.
- This is a Law 41 proof-carrier tightening slice: retained scalar proof
  artifacts now preserve the admitted materialization target state, and public
  code cannot treat target proof as an ordinary view-name string without using
  a named terminal projection.

## Query write receipt collection identity fence

- Converted `WORTHQueryWriteReceipt` declared/target collection state from
  `Option<String>` storage and neutral public string accessors to
  `WORTHQueryMutationTargetCollectionIdentity` proof carriers.
  `declared_collection_identity()` and `target_collection_identity()` are now
  the authority-facing receipt accessors.
- Removed the public `declared_collection()` and `target_collection()` aliases
  from write receipts. Callers that intentionally render labels must use the
  explicitly terminal `terminal_declared_collection_projection()` or
  `terminal_target_collection_projection()` methods.
- Routed direct writes, batch write summaries, intent execution receipts,
  preview receipts, graph-composition lineage, inspection digests, and public
  submission tests through receipt collection identities or terminal
  projections according to their role.
- Added aspect-native compile-fail coverage:
  `write_receipt_collection_string_aliases_removed.rs` proves facade callers
  cannot recover collection authority from the removed neutral receipt string
  aliases.
- Verification covered deliberate red then green `cargo check -p worth-query
  --tests`, `cargo fmt -p worth-query`, trybuild overwrite plus normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`,
  focused `cargo test -p worth-query mutation::aspect_crud --lib`, focused
  `cargo test -p worth-query mutation::aspect_preview --lib`,
  `cargo test -p worth-query --test public_submission_lane_replacements`,
  final green `cargo check -p worth-query --tests`, removed-alias scans, and
  touched-file line counts. Focused `cargo test -p worth-query
  mutation::delete --lib` is currently red in
  `workspace_delete_receipt_preserves_family_target_and_inspection_posture`
  before receipt assertions, on an existing delete/effect-admission path:
  `bridge writeback effect intent requires at least one admitted concrete
  aspect operation`.
- This is a Law 41 mechanical-enforcement slice: executed write receipts now
  preserve collection target proof as a typed identity, while string labels are
  explicitly marked as terminal reporting projections.

## Query whole-delete writeback empty-patch proof fence

- Removed the Query-only denial that rejected bridge writeback effect intents
  when the native `AuthoritativeRecordAspectPatch` was empty. Foundational
  already defines `AuthoritativeRecordAspectPatch::empty()` as a valid native
  carrier, and bridge writeback effects already derive their digest and
  canonical basis from that authoritative patch.
- Whole-entity deletes can now flow through bridge writeback authority without
  inventing fake aspect operations or failing before the bridge/foundational
  proof boundary owns the effect intent.
- Added backend receipt coverage:
  `bridge_writeback_effect_intent_accepts_whole_entity_delete_empty_patch`
  proves a `WORTHQueryWriteCommand::Delete` admitted mutation builds bridge
  authority with an authoritative-patch canonical basis containing no set/clear
  operations.
- Verification covered `cargo fmt -p worth-query`, focused
  `cargo test -p worth-query bridge_writeback_effect_intent --lib`, focused
  `cargo test -p worth-query mutation::delete --lib`, final green
  `cargo check -p worth-query --tests`, a scan proving the removed empty-patch
  denial text is gone, and touched-file line counts.
- This is a Law 41 correction: Query no longer rejects a stronger native proof
  carrier because it lacks lower-authority "concrete aspect operation" text.
  Empty authoritative patches remain native bridge/foundational evidence, not
  a reason to fall back to JSON, strings, or invented aspect touches.

## Query existing-truth binding target collection identity fence

- Removed the public neutral `target_collection()` string alias from
  `WORTHQueryExistingTruthTargetBinding`. The binding now exposes
  `target_collection_identity()` for authority flow and
  `terminal_target_collection_projection()` only for explicit reporting,
  adapter-key, and legacy label construction.
- Migrated backend mutation authority, backend-admissible declared-collection
  derivation, runtime authoritative mutation routing, batch write receipt row
  construction, runtime write reporting, probe/assertion denial digests,
  mutation evidence, preview labels, workspace graph error rows, graph
  composition existing-target labels, and public/stateful bridge test adapters
  to either carry `WORTHQueryMutationTargetCollectionIdentity` directly or make
  a named terminal projection at the boundary.
- Split existing-truth denial support into
  `runtime/mutation/binding/existing_truth_denial.rs` so the touched binding
  root stays under the workspace Rust line cap while preserving the public
  re-export surface.
- Added aspect-native compile-fail coverage:
  `existing_truth_binding_target_collection_alias_removed.rs` proves facade
  callers cannot recover collection authority from the removed binding string
  alias.
- Verification covered `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  green `cargo test -p worth-query existing_truth --lib`, removed-alias scans,
  and touched-file line counts.
- This is a Law 41 mechanical-enforcement slice: existing-truth collection
  proof now flows as a typed target collection identity through mutation
  admission, routing, evidence, and receipt construction. Strings remain
  terminal labels only, and the old public string alias is compile-time
  forbidden.

## Query admitted aspect/touch accessor root break

- Deliberately broke broad roots before fixing consumers, to expose real
  downstream reliance through `cargo check -p worth-query --tests` instead of
  grep-only confidence. The staged breaks produced 12 reported errors, then 70,
  then 143 reported errors before Rust stopped, covering backend adapters,
  mutation authority, writeback effects, computed/effect routing, preview,
  inspection, graph-composition touch descriptors, runtime support tests, and
  certification harness rows.
- Removed neutral command/backend-admissible native bag aliases:
  `aspect_values()`, `asserted_aspect_values()`, and `touched_aspects()` on
  `WORTHQueryWriteCommand` and `WORTHQueryBackendAdmissibleMutation`. The
  replacement APIs are `admitted_aspect_values()`,
  `asserted_admitted_aspect_values()`, and `admitted_touched_aspects()`.
- Removed neutral touched-aspect aliases from mutation deltas, live patches,
  retained refresh contexts, batch write receipts, batch write inspection, batch
  component inspection, and graph touch descriptor rows. Downstream consumers
  now ask for admitted touches explicitly instead of treating a touch set as an
  unqualified terminal path list.
- Renamed the crate-local aspect-touch digest projection from
  `native_digest_part()` to `admitted_touch_digest_part()`, forcing digest,
  evidence, report, support, and test consumers to name the terminal digest role
  instead of depending on a generic native-looking string projection.
- Added aspect-native compile-fail coverage:
  `write_command_native_bag_aliases_removed.rs`,
  `backend_admissible_native_bag_aliases_removed.rs`,
  `mutation_delta_touched_aspects_alias_removed.rs`, and
  `aspect_touch_native_digest_alias_removed.rs`. Refreshed older aspect-value
  fixtures so they use `admitted_aspect_values()` before proving the
  aspect-value path/JSON aliases remain removed.
- Verification covered `cargo fmt -p worth-query`, deliberate red
  `cargo check -p worth-query --tests` passes at 12/70/143 reported errors,
  green final `cargo check -p worth-query --tests`, trybuild overwrite plus
  green normal `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, focused green
  `cargo test -p worth-query mutation::aspect_crud --lib`, green
  `cargo test -p worth-query --test public_bridge_runtime_bootstrap`, removed
  alias scans, and touched-file line counts.
- This is a Law 41 enforcement slice: aspect/touch proof can still be rendered
  into digest/report text, but consumers now cross a named admitted-proof or
  terminal-digest boundary. The compiler, not grep, owns the migration map.

## Query support document terminal JSON boundary fence

- Removed the neutral public support-document JSON vocabulary from consumer-kit
  support snapshot and support pinning APIs. `from_json(...)`,
  `to_canonical_json(...)`, `to_stable_json(...)`,
  `load_support_snapshot_document(...)`, and
  `load_support_pin_contract_document(...)` are no longer public or production
  API names.
- Replaced them with explicit terminal-boundary names:
  `from_terminal_json_document(...)`,
  `to_canonical_terminal_json_document(...)`,
  `to_stable_terminal_json_document(...)`,
  `load_support_snapshot_terminal_json_document(...)`, and
  `load_support_pin_contract_terminal_json_document(...)`.
- Support snapshot and pinning documents still use typed serde
  encode/decode at the external durable-document boundary. The semantic
  authority remains the typed support snapshot / support pin contract and their
  digest validation; JSON strings are not accepted by native mutation,
  materialization, effect, retained-row, or certification authority paths.
- Added aspect-native compile-fail coverage:
  `support_document_neutral_json_api_removed.rs` proves facade callers cannot
  import the old neutral loaders or call the old neutral document JSON methods.
- Verification covered `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query support_snapshot --lib`, focused green
  `cargo test -p worth-query support_pinning --lib`, and green normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`.
- This is a Law 41 naming and boundary-enforcement slice: surviving document
  JSON is explicitly terminal I/O, while support state remains sealed behind
  typed document validation and cannot be mistaken for an ordinary authority
  carrier.

## Query support document carrier visibility fence

- Removed `WORTHQuerySupportSnapshotDocument` and
  `WORTHQuerySupportPinContractDocument` from public `consumer_kit`, facade,
  and crate-root exports.
- Made both document structs crate-private and narrowed
  `WORTHQuerySupportSnapshot::to_document(...)` and
  `WORTHQuerySupportPinContract::to_document(...)` to crate-private helpers.
- Public callers now interact with native support snapshots / support pin
  contracts, or with explicit terminal JSON document text through
  `to_canonical_terminal_json_document(...)` and
  `load_*_terminal_json_document(...)`. They cannot name, traffic in, or
  attempt to construct the intermediate serde document carrier.
- Refreshed support snapshot/pinning compile-fail fixtures so the document
  boundary proof is now stronger than private fields: the document carrier is
  absent from the public facade entirely.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green normal
  `cargo test -p worth-query --test support_snapshot_facade`, green normal
  `cargo test -p worth-query --test support_pinning_facade`, green normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  green `cargo test -p worth-query support_snapshot --lib`, and focused green
  `cargo test -p worth-query support_pinning --lib`.
- This is a Law 41 visibility fence around the surviving terminal JSON
  exception: terminal durable document encoding still exists, but the
  serde-shaped document carrier no longer crosses the public/native boundary as
  a reusable authority object.

## Query support terminal document text proof fence

- Introduced explicit terminal-document text carriers:
  `WORTHQuerySupportSnapshotTerminalJsonDocument` and
  `WORTHQuerySupportPinContractTerminalJsonDocument`.
- Changed support snapshot and support pinning terminal loaders to accept those
  carriers instead of bare `&str`.
- Changed canonical/stable support document export methods to return the
  terminal-document carriers instead of anonymous strings.
- Test-only document mutation helpers now wrap mutated JSON text in the
  terminal carrier before calling the loader, making terminal ingress visible
  at the call site.
- Added `support_terminal_json_document_rejects_bare_string.rs` compile-fail
  coverage proving facade callers cannot pass a raw string directly into either
  terminal loader.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  green `cargo test -p worth-query support_snapshot --lib`, focused green
  `cargo test -p worth-query support_pinning --lib`, green
  `cargo test -p worth-query --test support_snapshot_facade`, and green
  `cargo test -p worth-query --test support_pinning_facade`.
- This is a Law 41 transition-order fence for the remaining support JSON
  exception: external terminal JSON text must be admitted as terminal-document
  evidence before it can be lowered into native support snapshot or support pin
  proof carriers.

## Query aspect closeout JSON authority wording fence

- Updated the executable aspect API closeout artifact and its closeout document
  so they no longer teach that JSON removal is merely a future internal
  rewrite underneath an otherwise-stable public facade.
- The closeout now says terminal document JSON and external reference artifacts
  are not native authority carriers, and downstream runtimes may build on Query
  only while JSON-shaped authority remains forbidden.
- Renamed the embedded Worth support-pin reference artifact constant to
  `WORTH_QUERY_SUPPORT_PINS_TERMINAL_JSON_DOCUMENT`, making the remaining
  `.json` reference visibly external/terminal instead of a neutral support-pin
  authority source.
- Verification covered green `cargo check -p worth-query --tests`, focused
  green `cargo test -p worth-query runtime_public_aspect_api_finalization_closeout --lib`,
  focused green `cargo test -p worth-query consumer_kit_closure --lib`, and
  a production scan showing non-test `serde_json` calls only in support
  snapshot/pinning terminal document encode/decode.
- This is a current-contract cleanup, not broad documentation polish: the
  runtime artifact that downstreams inspect now matches the aspect-native
  direction instead of preserving stale JSON-lowering guidance.

## Query phase-boundary raw JSON fixture cleanup

- Removed stale raw `serde_json` construction from top-level phase-boundary
  compile-fail fixtures that predated the aspect-native migration.
- `runtime_aspect_mutation_privates_forbidden.rs` now proves callers cannot
  construct `WORTHQueryAspectValue` through a struct literal instead of
  teaching the removed path-plus-JSON field shape.
- `runtime_payload_first_insert_command_missing.rs` now proves the
  payload-first `WORTHQueryWriteCommand::Insert` variant is absent without
  constructing a raw JSON payload.
- Refreshed the affected trybuild stderr snapshots, including already-stale
  compiler wording for graph composition and intent receipt phase-boundary
  fixtures.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, and green normal
  `cargo test -p worth-query --test phase_boundaries_compile_fail`.
- This is a test-harness teaching cleanup: the compile-fail suite still proves
  old authority paths are unavailable, but no longer demonstrates JSON payloads
  as the example of how callers try to construct those paths.

## Query support external terminal ingress proof split

- Split the remaining support terminal JSON boundary into two Law 41 states:
  native-derived terminal projection documents and weaker external terminal
  ingress documents.
- `WORTHQuerySupportSnapshotTerminalJsonDocument` and
  `WORTHQuerySupportPinContractTerminalJsonDocument` are now produced only by
  native support snapshot/contract export paths.
- New external ingress carriers,
  `WORTHQueryExternalSupportSnapshotTerminalJsonDocument` and
  `WORTHQueryExternalSupportPinContractTerminalJsonDocument`, are the only
  public types accepted by the support snapshot and support pin terminal
  loaders.
- Round-trip tests now explicitly cross from native terminal projection to
  external terminal ingress with `to_external_terminal_json_document()`, making
  the boundary visible instead of letting one JSON-shaped type satisfy both
  sides.
- The terminal support serde documents remain crate-private; public callers can
  neither construct them nor pass a raw `&str` as loader authority.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green normal
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, focused
  green `cargo test -p worth-query support_snapshot --lib`, focused green
  `cargo test -p worth-query support_pinning --lib`, green serial
  `cargo test -p worth-query --test support_snapshot_facade`, and green serial
  `cargo test -p worth-query --test support_pinning_facade`.
- This is a stronger Law 41 fence than the prior terminal document wrapper:
  exported terminal JSON, externally supplied terminal JSON, decoded support
  documents, and validated native support proof carriers are now distinct
  states in the type system.

## Query embedded support pin terminal JSON source typing

- Converted the embedded Worth `query_support_pins.json` reference from a
  neutral `&str` source constant into
  `WORTHQueryExternalSupportPinContractTerminalJsonDocument`.
- Added a static external-terminal constructor to the external support terminal
  document carriers, allowing embedded source artifacts to carry the same
  external-ingress proof type as runtime-loaded terminal documents.
- Changed consumer-kit closure certification source inventory rows to store
  `WORTHQueryConsumerKitCertificationSourceText` instead of raw source text.
  Ordinary source files use `StaticSource`; the Worth support pin JSON row uses
  `ExternalSupportPinContractTerminalJsonDocument`.
- Generic source digest code still projects text with `as_str()` because source
  inventories digest text, but the inventory no longer stores the JSON row as
  an anonymous source body.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query consumer_kit_closure --lib`, and focused green
  `cargo test -p worth-query consumer_residue --lib`.
- This is a narrow JSON authority exposure slice: the remaining embedded
  `.json` path is still present because it is a real terminal artifact, but it
  now enters Query through an external support-pin terminal document proof
  state instead of an untyped string constant.

## Query test backend schema aspect-touch storage

- Converted `WORTHQueryTestBackendSchema` from retaining aspect labels as
  `String` to retaining admitted `WORTHQueryAspectTouch` proof carriers.
- The public ergonomic `aspect(label, external_projection_path)` method still
  accepts authoring text, but it parses and admits the aspect touch at schema
  construction time instead of storing the string and reparsing it during
  workspace materialization.
- `memory_aspects()` now lowers directly from the stored touch proof into
  `WORTHQueryAspect::from_native_external_projection_path(...)`.
- Public schema iteration now returns `(&WORTHQueryAspectTouch,
  &CanonicalFieldPath)`, and the compile-fail fixture proves callers cannot
  recover either the aspect label or projection path as `&str` authority.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query test_backend --lib`, green
  `cargo test -p worth-query --test in_memory_test_backend_facade`, and green
  normal `cargo test -p worth-query --test aspect_native_query_compile_fail`.
- This is a Law 41 proof-retention slice: test backend schema admission now
  consumes weaker authoring text and stores the stronger native aspect touch
  state for downstream consumers.

## Query graph obligation support-matrix static aspect proof

- Converted support-matrix certification and selector-perturbation
  representative aspect touches from static raw aspect-path parsing to native
  `AspectKey` construction followed by `WORTHQueryAspectTouch::whole_aspect`.
- Removed the local `aspect_touch(&str)` / `set_operation(&str)` helpers from
  the support-matrix selector perturbation cases, so static `"capacity"` and
  `"boundary"` examples no longer flow through the authoring parser at the
  point where graph obligation proof fixtures are assembled.
- The remaining collection names in these files are still collection selector
  labels, not aspect authority. Aspect identity now enters the representative
  mutation descriptors as foundational aspect-key proof.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query graph_obligation --lib`, focused green
  `cargo test -p worth-query --test graph_obligation_consumer_kit_facade`, and
  a targeted scan proving the support-matrix folder no longer contains the
  removed `"capacity"` / `"boundary"` authoring-parser calls or raw aspect-path
  helper names.
- This is a Law 41 mechanical-enforcement slice: static certification fixtures
  now build stronger native aspect-touch proof directly instead of relying on
  late parsing of weaker dotted-string authoring text.

## Query support hostile terminal JSON test quarantine

- Removed the anonymous `TerminalSupport*DocumentJson = serde_json::Value`
  aliases from support snapshot and support pinning tests.
- Added test-only hostile terminal-document helpers under the support snapshot
  and support pinning test modules. Raw `serde_json::Value` mutation now lives
  only inside helpers named
  `HostileSupportSnapshotTerminalDocument` and
  `HostileSupportPinContractTerminalDocument`.
- Rewrote support snapshot schema/digest/vocabulary denial tests and support
  pinning stale/tampered/invalid contract document tests to use named terminal
  mutation methods such as `replace_top_level_string`,
  `replace_top_level_number`, `replace_first_row_string`, and
  `replace_first_requirement_string`.
- The tests still exercise malformed or hostile external terminal JSON, but
  ordinary test bodies no longer index arbitrary JSON values or teach
  `serde_json::Value` as a normal Query support substrate.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query support_snapshot --lib`, focused green
  `cargo test -p worth-query support_pinning --lib`, and a support
  snapshot/pinning scan showing raw `serde_json::Value` only in the production
  terminal encode/decode files and the two explicitly named hostile terminal
  document test helpers.
- This is a test-harness teaching cleanup for the surviving support terminal
  JSON exception: JSON mutation remains available only as hostile external I/O
  evidence, not as an ordinary authority helper in support tests.

## Query support terminal JSON codec boundary extraction

- Moved the remaining production support snapshot and support pinning
  `serde_json` encode/decode calls out of the semantic document roots and into
  explicitly named `terminal_json_codec` modules.
- `WORTHQuerySupportSnapshotDocument::from_terminal_json_document(...)` now
  delegates to `support_snapshot/document/terminal_json_codec.rs`, and
  `to_canonical_terminal_json_document(...)` delegates to the matching native
  terminal projection encoder.
- `WORTHQuerySupportPinContractDocument::from_terminal_json_document(...)` and
  `to_canonical_terminal_json_document(...)` now do the same through
  `support_pinning/document/terminal_json_codec.rs`.
- The semantic document types still own validated support snapshot / support
  pin document state and validation. Raw JSON parsing and pretty-printing are
  now isolated to files whose names state the external terminal contract.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query support_snapshot --lib`, focused green
  `cargo test -p worth-query support_pinning --lib`, green
  `cargo test -p worth-query --test support_snapshot_facade`, green
  `cargo test -p worth-query --test support_pinning_facade`, and a support
  scan showing production `serde_json` calls only in the two
  `terminal_json_codec.rs` modules.
- This is a production boundary isolation slice: the remaining support JSON
  exception is now physically boxed into terminal codec files instead of being
  embedded in the document validation modules.

## Query intent-admission certification native aspect touches

- Removed static `WORTHQueryAspectTouch::from_authoring_path(...)` calls from
  the intent-admission certification fixture directory.
- Added fixture-root helpers `identity_id_touch()` and `title_value_touch()`
  that construct `AspectKey`, `FieldKey`, `CanonicalFieldPath`, and then
  `WORTHQueryAspectTouch::aspect_field_path(...)`.
- Converted effect certification, routing probe/seed certification,
  authoritative intent execution fixtures, and legacy/canonical effect
  delegation parity fixtures to reuse those native touch helpers instead of
  reparsing `"identity.id"` or `"title.value"` at each proof site.
- The remaining `aspect_path` text in the fixture directory is a local
  reporting/digest variable derived from an admitted touch, not an authority
  constructor.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query intent_admission::certification --lib`, green
  `cargo test -p worth-query --test intent_admission_public_dx`, and a fixture
  directory scan showing no remaining `from_authoring_path` calls.
- This is a Law 41 fixture-hardening slice: certification examples that
  represent proven Query behavior now start from foundational aspect/field
  proof carriers, not static dotted-string authoring text.

## Query lower-runtime certification native aspect touches

- Removed static `WORTHQueryAspectTouch::from_authoring_path(...)` calls from
  the lower-runtime-routing certification surface fixture subtree.
- Added shared lower-runtime representative touch helpers for
  `title.value`, `status.value`, and `priority.value` that build
  `AspectKey`, `FieldKey`, `CanonicalFieldPath`, and then
  `WORTHQueryAspectTouch::aspect_field_path(...)`.
- Converted write-authority, signal-invalidation, projection-query-receipt,
  and causal-bridge representative fixtures to use those native helpers
  instead of parsing static dotted aspect paths at proof assembly sites.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query lower_runtime_routing::certification --lib`,
  focused green `cargo test -p worth-query lower_runtime_routing --lib`, and a
  fixture subtree scan showing no remaining `from_authoring_path` calls.
- This is another Law 41 fixture-hardening slice: lower-runtime certification
  rows that claim boundary proof now construct aspect touches from foundational
  proof carriers directly, rather than treating static authoring strings as
  the source of truth.

## Query backend receipt test native aspect touches

- Removed static `WORTHQueryAspectTouch::from_authoring_path(...)` calls from
  runtime backend receipt tests.
- Added receipt-test helpers for `title.value` and `status.value` that build
  `AspectKey`, `FieldKey`, `CanonicalFieldPath`, and then
  `WORTHQueryAspectTouch::aspect_field_path(...)`.
- Signal-invalidation routing receipt tests and bridge writeback effect-intent
  tests now build representative touched aspects from foundational proof
  carriers instead of parsing dotted strings at the assertion site.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query runtime::backend::receipts::tests --lib`, focused
  green `cargo test -p worth-query bridge_writeback_effect_intent --lib`, and
  a targeted scan showing no `from_authoring_path`, `title.value`, or
  `status.value` residue in `runtime/backend/receipts_tests.rs`.
- This is a test-fixture teaching cleanup: backend receipt tests now model
  admitted aspect proof directly instead of reintroducing the old authoring
  parser shape.

## Query aspect API finalization harness native aspect touches

- Removed static `WORTHQueryAspectTouch::from_authoring_path(...)` calls from
  the aspect API finalization certification harness.
- Added harness-level helpers for `identity.id`, `title.value`,
  `description.value`, and `ui.batch_summary` that build `AspectKey`,
  `FieldKey`, `CanonicalFieldPath`, and then
  `WORTHQueryAspectTouch::aspect_field_path(...)`.
- Converted canonical CRUD, typed-clear narrowing, batch, preview-batch, and
  duplicate-authoring rejection rows to use those native proof helpers instead
  of local `aspect_touch(&str)` parser helpers.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query aspect_api_finalization_certification --lib`,
  focused green
  `cargo test -p worth-query runtime_public_aspect_api_finalization_closeout --lib`,
  and targeted harness scans showing no `from_authoring_path`,
  `aspect_touch(...)`, or old dotted touch literal residue in the harness
  subtree.
- This is a Law 41 teaching-surface slice: the certification harness for the
  public aspect API now constructs representative aspect touches through
  foundational proof carriers, matching the story it certifies.

## Query runtime API stabilization transcript native aspect lowering

- Removed `WORTHQueryAspectTouch::from_authoring_path(...)` usage from the
  runtime API stabilization transcript harness.
- Added a shared `transcript_aspect_touch(...)` helper that lowers transcript
  fixture path text through foundational `AspectKey`, `FieldKey`,
  `CanonicalFieldPath`, and then
  `WORTHQueryAspectTouch::aspect_field_path(...)`.
- Converted golden transcript computed read/produce dependencies, effect
  trigger/condition dependencies, authoritative transcript writes, and preview
  proof writes to use that native lowering helper instead of local
  `aspect_touch(...)` parser helpers.
- Transcript specs still use text labels as fixture data because they model
  user-facing transcript families, but aspect-touch construction no longer
  delegates to Query's authoring-path parser as the authority source.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query runtime_api_stabilization --lib`, and a targeted
  runtime-stabilization harness scan showing no remaining
  `from_authoring_path` or `fn aspect_touch(...)` parser helpers.
- This is a Law 41 transcript-harness slice: golden DX certification now lowers
  transcript aspect data into native proof carriers before it reaches runtime
  declarations and writes.

## Query mutation evidence batch tests native aspect touch

- Removed the remaining static `WORTHQueryAspectTouch::from_authoring_path(...)`
  calls from mutation-evidence batch tests.
- Added a `title_value_touch()` helper that builds `AspectKey`, `FieldKey`,
  `CanonicalFieldPath`, and then
  `WORTHQueryAspectTouch::aspect_field_path(...)` for verified existing-truth
  assertion fixtures.
- Existing-truth mode summary tests now use the native touch helper instead of
  parsing `"title.value"` as authoring text.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query runtime::surface::mutation_evidence::batch --lib`,
  focused green
  `cargo test -p worth-query existing_truth_mode_summary --lib`, and a targeted
  scan showing no `from_authoring_path` or `title.value` residue in the batch
  evidence test file.
- This is a test-proof cleanup: batch mutation evidence tests now construct the
  touched aspect proof directly rather than re-teaching static dotted-string
  admission.

## Query graph-composition fixture native touch lowering

- Removed the remaining graph-composition test fixture dependence on
  `WORTHQueryAspectTouch::from_authoring_path(...)`.
- Converted touch-descriptor, graph-obligation registration, and
  graph-obligation index fixture helpers to lower fixture touch text through
  foundational `AspectKey`, `FieldKey`, and `CanonicalFieldPath` before
  constructing `WORTHQueryAspectTouch`.
- The helper still accepts fixture text because some graph selector cases model
  static aspect-field examples such as `topology.kind`, but the construction no
  longer delegates authority to Query's authoring-path parser.
- Cleaned the graph-composition fixture/test vocabulary so the subtree no
  longer retains `aspect_path` parameter names for these touch helpers.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query graph_composition --lib`, focused green
  `cargo test -p worth-query graph_obligation --lib`, and a targeted
  graph-composition scan showing no remaining `from_authoring_path` or
  `aspect_path` hits.
- This is a Law 41 fixture-hardening slice: static graph-composition proof
  examples now move through foundational touch proof construction instead of
  re-teaching the old dotted-string parser path.

## Query runtime support fixture native touch lowering

- Removed `WORTHQueryAspectTouch::from_authoring_path(...)` from the shared
  runtime test support helpers, graph-composition assertion support,
  existing-truth verification adapter, command builder helper, and stop-class
  representative error fixture.
- Added a shared runtime-test `test_aspect_touch(...)` lowering path that
  converts fixture touch text into foundational `AspectKey`, `FieldKey`, and
  `CanonicalFieldPath` before constructing `WORTHQueryAspectTouch`.
- Converted memory-workspace unit tests and consumer-kit test-backend behavior
  tests to the same native fixture lowering pattern instead of local
  authoring-parser helpers.
- The remaining source `from_authoring_path(...)` hits are now production
  authoring-ingress boundaries (`WORTHQueryTestBackendSchema`,
  memory-workspace declaration admission), private parser implementation, or
  compile-fail fixtures proving removed neutral helpers.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query runtime::tests::stop_class --lib`, focused green
  `cargo test -p worth-query graph_composition --lib`, focused green
  `cargo test -p worth-query memory_workspace --lib`, focused green
  `cargo test -p worth-query test_backend --lib`, and a global parser scan
  confirming no ordinary runtime/support/memory-workspace/consumer-kit
  behavior test helpers still call `WORTHQueryAspectTouch::from_authoring_path`.
- This is a test-surface teaching cleanup: broad runtime and consumer-kit
  helpers still accept compact fixture text, but their proof construction now
  flows through foundational native carriers instead of Query's legacy
  authoring parser.

## Query authored touch ingress boundary naming

- Removed production `from_authoring_path(...)` vocabulary from Query aspect
  touch admission.
- Renamed the crate-private aspect-touch authoring boundary to
  `WORTHQueryAspectTouch::admit_authoring_ingress_text(...)` and the parsed
  target parser to `WORTHQueryParsedAspectTarget::parse_authoring_ingress_text(...)`.
- Updated the two remaining production authoring-ingress callers,
  `WORTHQueryTestBackendSchema::aspect(...)` and memory-workspace declared
  aspect matching, to call the explicitly named ingress boundary.
- After this slice, global `from_authoring_path(...)` hits under
  `worth-query` are compile-fail fixtures for removed retained/projection
  neutral helpers, not production aspect-touch authority.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query test_backend --lib`, focused green
  `cargo test -p worth-query memory_workspace --lib`, and a global parser scan
  proving production no longer contains `WORTHQueryAspectTouch::from_authoring_path`
  or `WORTHQueryParsedAspectTarget::from_authoring_path`.
- This is a naming/visibility enforcement slice: ergonomic text parsing still
  exists where Query admits authored ingress, but the production API no longer
  presents it as a neutral authority-construction path.

## Query hostile support terminal test JSON carrier removal

- Removed raw `serde_json::Value` carriers from the hostile support snapshot
  and support pinning terminal-document test helpers.
- The hostile helpers now start from native-produced terminal documents and
  perform bounded scalar text substitutions for the small set of denial cases
  they need to exercise.
- Support tests still cover malformed external terminal JSON, stale vocabulary,
  digest drift, invalid facade families, invalid support statuses, invalid
  teaching postures, and blank required fields, but ordinary hostile helpers no
  longer carry a general-purpose JSON object model.
- After this slice, global `serde_json` hits under current Query source/tests
  are limited to the two production support terminal codec files and three
  compile-fail fixtures that intentionally try raw JSON APIs.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query support_snapshot --lib`, focused green
  `cargo test -p worth-query support_pinning --lib`, and a global JSON scan
  confirming hostile support test helpers no longer import `serde_json`.
- This is a test-boundary cleanup for the final support terminal exception:
  hostile external document tests remain, but they no longer teach
  `serde_json::Value` as an ordinary support-document mutation substrate.

## Query support terminal JSON allowlist enforcement

- Replaced the stale "no production JSON allowlist" table with the actual two
  surviving terminal codec exceptions:
  `support_snapshot/document/terminal_json_codec.rs` and
  `support_pinning/document/terminal_json_codec.rs`.
- Added `production_serde_json_is_confined_to_support_terminal_codecs` to the
  support snapshot runtime-boundary tests. The test scans production Rust
  source under `crates/worth-query/src`, skips test-only source files, and
  fails unless the only `serde_json` production files are the two named
  terminal codecs.
- The allowlist rows name the external terminal contract, the native authority
  carrier before/after the boundary, and the removal condition for each codec.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query production_serde_json_is_confined_to_support_terminal_codecs --lib`,
  focused green `cargo test -p worth-query support_snapshot --lib`, and a
  global JSON scan showing production JSON confined to the terminal codecs.
- This is mechanical containment for the remaining JSON island: new production
  JSON cannot appear silently while the larger terminal-format removal decision
  remains open.

## Query Foundational JSON Compatibility Guard

- Re-anchored the spec around WORTH Foundational as both the source of aspect
  truth and the source of transitional JSON compatibility. Query must not own
  a generic JSON-as-aspect lowering lane.
- Added
  `support_terminal_json_codecs_do_not_become_aspect_compatibility_bridges`.
  The test allows the two remaining `serde_json` terminal codecs to parse and
  print durable support documents, but fails if either codec starts importing
  foundational aspect contracts, aspect values, authoritative aspect carriers,
  or Foundational JSON compatibility lowering markers.
- After this slice, the production JSON position is explicit: Query currently
  has no approved production JSON-as-aspect compatibility boundary. If one is
  introduced later for a hard external contract, it must delegate immediately
  to `worth_foundational::compatibility().json()` and return native validated
  or authoritative artifacts.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green
  `cargo test -p worth-query
  support_terminal_json_codecs_do_not_become_aspect_compatibility_bridges
  --lib`, and a production scan showing `serde_json` still confined to the two
  support terminal codec files.
- This is the Foundational-compatibility correction: support document JSON is
  terminal I/O, not aspect truth. JSON that is aspect truth belongs to
  Foundational's compatibility bridge, not to Query-local lowering code.

## Query admitted aspect value proof-carrier rename

- Renamed the remaining mutation value proof carrier from
  `WORTHQueryAspectValue` to `WORTHQueryAdmittedAspectValue` across production
  Query, facade exports, certification fixtures, integration tests, and
  compile-fail fixtures.
- Did not leave a compatibility alias. Any consumer still reaching for the old
  weak name now fails to compile instead of being silently routed through a
  neutral "aspect value" vocabulary.
- The renamed carrier still stores parsed native desired-aspect state over
  foundational `AspectValue`; JSON is not part of its semantic authority.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused green `cargo test -p
  worth-query support_terminal_json_codecs_do_not_become_aspect_compatibility_bridges
  --lib`, and a source/stderr scan showing no remaining
  `WORTHQueryAspectValue` in Query code or compile-fail outputs.
- This is a Law 41 naming and public-surface break: the carrier now advertises
  the proof state it represents instead of reading like a generic aspect value
  that could be reconstructed from weaker JSON or dotted-string inputs.

## Query admitted aspect value scalar factory privacy

- Removed the public `WORTHQueryAdmittedAspectValue::string(...)` scalar factory
  that survived the proof-carrier rename. That method returned a raw
  foundational `AspectValue`, not an admitted Query proof value, so it made the
  admitted carrier look like a public weak-value constructor.
- Replaced internal call sites with the crate-local
  `WORTHQueryAdmittedAspectValue::native_string_value(...)` helper and updated
  external integration fixtures to construct Foundational `AspectValue`
  directly where they are asserting retained/public bridge scalar material.
- Added the compile-fail fixture
  `aspect_value_native_string_factory_private.rs`, proving facade consumers
  cannot use the admitted proof carrier as a scalar factory.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, and a residue scan showing external
  `native_string_value(...)` mentions only in the compile-fail fixture and its
  expected stderr.
- This is another Law 41 fence: public authoring text remains on
  `WORTHQueryAuthoredAspectValue::string(...)`; admitted mutation proof no
  longer doubles as a public lower-authority value factory.

## Query memory workspace native field path vocabulary

- Renamed the memory-workspace retained aspect carrier from
  `external_projection_path` to `native_field_path` and removed the public
  `external_projection_path()` accessor.
- Renamed the cross-module constructor from
  `from_native_external_projection_path(...)` to `from_native_field_path(...)`
  so test-backend lowering hands memory workspace a retained
  `CanonicalFieldPath` without preserving external/projection vocabulary as the
  authority name.
- Kept the test-backend `.aspect(label, projection_field_path_text)` parameter
  as explicitly authoring-ingress text, then immediately lowers it into
  `CanonicalFieldPath`; stored schema aspects and `schema.aspects()` expose the
  native field-path proof, not string text.
- Refreshed aspect-native compile-fail coverage so
  `WORTHQueryAspect::external_projection_path()` is proven absent and
  `native_field_path()` / test-backend schema aspect paths cannot be treated as
  `&str`.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green `cargo test -p worth-query
  --test aspect_native_query_compile_fail`, focused green `cargo test -p
  worth-query memory_workspace --lib`, focused green `cargo test -p
  worth-query test_backend --lib`, and scans showing production
  `external_projection_path`, `from_native_external_projection_path`, and
  `external projection path` residue removed.
- This is a Law 41 naming/enforcement slice: memory workspace and test-backend
  schema now advertise the proven carrier they hold, while authoring text stays
  visibly pre-proof ingress.

## Query projection consumption aspect-key extraction

- Changed query-read result row extraction so aspect values flow into
  projection fact path construction as `AspectKey` carriers instead of
  immediately projecting each key to an `aspect_path` string.
- Replaced the private
  `projection_fact_field_path_from_aspect_label(&str)` helper with
  `projection_fact_field_path_from_aspect_key(&AspectKey)`. Terminal text is
  now used only for diagnostic content after the native carrier reaches the
  deliberate projection fact field-path boundary.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query projection_consumption --lib`, and a projection-consumption scan
  showing the old `projection_fact_field_path_from_aspect_label`,
  `aspect_path = aspect_key.as_str()`, and `projection fact aspect label`
  vocabulary removed.
- This is a narrow Law 41 extraction cleanup: an already-admitted row aspect
  key is no longer weakened to string text before the field-path extraction
  transition.

## Query projection consumption native visibility key

- Added crate-private `ProjectionFactAspectFieldKey`, derived only when a
  requested `ProjectionFactFieldPath` proves the exact `aspect.field` shape.
- Changed projection-consumption visibility admission so authorized projection
  fields compare `AspectKey` and `FieldKey` carriers directly instead of
  comparing `native_*().as_str()` against canonical field path segments.
- Added
  `visibility_admission_requires_exact_native_aspect_field_key`, proving an
  authorized `profile.display_name` field does not admit a deeper
  `profile.display_name.extra` request through loose path text matching.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo test -p worth-query projection_consumption --lib`, green
  `cargo check -p worth-query --tests`, and a targeted scan showing the
  projection-consumption `native_aspect_key().as_str() == ...` /
  `native_field_key().as_str() == ...` visibility comparison is gone.
- This is a Law 41 visibility-gate slice: the request path must first prove it
  is a native aspect-field key before it can satisfy authorized projection
  visibility.

## Query projection consumption extraction carrier cleanup

- Added private `ProjectionMaterializedField` for row-like extraction so
  external relational/bridge field labels are lowered into
  `ProjectionFactFieldPath` at the source boundary before entering the shared
  row extractor.
- Added private `ProjectionGroupedMember` for grouped extraction so grouped
  membership and relation-endpoint extraction consume named row identity,
  member identity, and grouping value fields instead of positional
  `(&str, AspectValue, AspectValue)` tuples.
- The bridge/relational read contract surfaces still expose terminal field
  labels where those APIs require them, but the projection-consumption core no
  longer uses `(&str, AspectValue)` or `(&str, AspectValue, AspectValue)`
  iterator items as its retained extraction substrate.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo test -p worth-query projection_consumption --lib`, green
  `cargo check -p worth-query --tests`, and targeted scans showing the old
  row-like/grouped tuple iterator shapes removed from
  `crates/worth-query/src/projection_consumption`.
- This is a production Law 41 extraction cleanup: terminal field text is
  admitted into a typed projection field carrier before shared fact extraction
  consumes the row.

## Query read-composition native identity anchor key

- Replaced the read-composition scope classifier's direct
  `native_aspect_key().as_str() == "identity"` /
  `native_field_key().as_str() == "id"` checks with a local
  `NativeIdentityAnchorPredicateKey` carrying foundational `AspectKey` and
  `FieldKey` values.
- The classifier still treats predicate family and value kind as existing
  validated predicate metadata, but aspect identity no longer falls back to
  text comparison when deciding whether a predicate is the local identity
  anchor.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo test -p worth-query read_composition --lib`, green
  `cargo check -p worth-query --tests`, and a targeted scan showing the
  read-composition native-key string comparison was removed.
- This is a Law 41 read-classification slice: a validated predicate's native
  aspect and field carriers are now what prove identity-anchor status.

## Query native key string-equality residue cleanup

- Converted the intent-admission certification existing-truth probe fixture so
  it matches requested probe fields against admitted `WORTHQueryAspectTouch`
  fixtures (`identity_id_touch()` / `title_value_touch()`) instead of peeling
  touches back into aspect/field text.
- Updated policy delivery and policy narrowing assertions to compare
  `AuthorizedProjectionFieldPath` or `(AspectKey, FieldKey)` carriers instead
  of asserting absence through literal `native_*().as_str()` comparisons.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo test -p worth-query intent_admission --lib`, green `cargo test -p
  worth-query policy_delivery --lib`, green `cargo test -p worth-query
  policy_narrowing --lib`, green `cargo check -p worth-query --tests`, and a
  targeted scan with no remaining direct native aspect/field key string
  equality sites under `crates/worth-query/src`.
- This is mechanical enforcement support for Law 41: code may still project
  native keys for terminal reporting/digests, but this equality pattern can no
  longer hide authority decisions behind text comparisons.

## Query mutation metadata admitted-key storage

- Changed `WORTHQueryMutationMetadata` storage from
  `BTreeMap<String, WORTHQueryMutationMetadataValue>` to
  `BTreeMap<WORTHQueryMutationMetadataKey, WORTHQueryMutationMetadataValue>`.
- The public lookup and iterator fences already required typed metadata keys;
  the internal map now preserves that admitted key proof instead of dropping
  back to anonymous string storage after validation.
- Duplicate detection and iteration now operate on
  `WORTHQueryMutationMetadataKey` directly, projecting key text only for the
  human-readable duplicate denial message.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query mutation_metadata --lib`, focused green `cargo test -p
  worth-query write_receipt_inspection_retains_authored_mutation_metadata
  --lib`, green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, and a scan showing the raw metadata map
  shape removed from production.
- This is a Law 41 storage-shape closeout: metadata key admission is now
  reflected in the retained carrier, not just in public accessors.

## Query public bridge existing-truth native key harness

- Replaced the public bridge runtime test harness existing-truth map key from
  `(String, String, String)` to `PublicExistingTruthKey`, which retains the
  existing-truth binding digest, target collection projection, and admitted
  `WORTHQueryAspectTouch`.
- Changed `seed_backend_authoritative_truth(...)` to accept
  `WORTHQueryAspectTouch` instead of raw aspect-path text, so tests seed
  existing truth with the same admitted aspect proof that verification/probe
  requests use.
- Updated graph-composition and public bridge runtime bootstrap tests to pass
  native `touch(...)` fixtures into the seed helper. The returned seed record
  still exposes `terminal_aspect_path_projection()` for reporting assertions,
  but that projection no longer participates in the verification map key.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query --test public_bridge_runtime_bootstrap`, focused green
  `cargo test -p worth-query --test graph_composition_public_bridge`, focused
  green `cargo test -p worth-query --test
  graph_composition_public_bridge_existing`, and scans showing the old
  tuple-string existing-truth key plus raw-string seed calls removed.
- This is a test-harness Law 41 cleanup: the public bridge certification
  harness now models existing-truth verification over admitted aspect touches
  instead of terminal aspect-path strings.

## Query runtime test existing-truth native key adapter

- Replaced the runtime test `TestExistingTruthVerificationAdapter` backing map
  key from `(String, String, String)` to `TestExistingTruthKey`, retaining the
  existing-truth binding digest, target collection projection, and admitted
  `WORTHQueryAspectTouch`.
- Kept `with_value(binding, touch_fixture, value)` ergonomic for tests, but the
  fixture text now lowers once through `test_aspect_touch(...)` and storage
  retains the native touch proof instead of `admitted_touch_digest_part()` text.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query bridge_backed_verification_execution --lib`, focused green
  `cargo test -p worth-query graph_composition_verified_existing --lib`,
  focused green `cargo test -p worth-query graph_composition_edge_split
  --lib`, and scans showing the old tuple-key shape removed from the adapter.
- This is the same Law 41 harness tightening as the public bridge seed map:
  backend existing-truth verification tests now retain admitted aspect-touch
  keys instead of terminal digest strings.

## Query public bridge external-row native touch paths

- Replaced the public bridge runtime test harness external-row writer path from
  `WORTHQueryAspectTouch -> terminal dotted string -> CanonicalFieldPath` with
  direct `WORTHQueryAspectTouch -> CanonicalFieldPath` derivation.
- `apply_aspects_to_external_row(...)` now calls native touch setters/removers;
  the row writer uses `native_aspect_key()` plus any native field path carried
  by the admitted touch. Terminal dotted strings no longer participate in
  public bridge row mutation.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query --test public_bridge_runtime_bootstrap`, focused green
  `cargo test -p worth-query --test graph_composition_public_bridge`, focused
  green `cargo test -p worth-query --test
  graph_composition_public_bridge_existing`, focused green `cargo test -p
  worth-query --test public_submission_lane_replacements`, and a scan showing
  `terminal_aspect_path_projection`, string-path setters/removers, and
  `admitted_touch_digest_part()` are gone from
  `tests/support/public_bridge_runtime/external_row.rs`.
- This closes another Law 41 test-harness weakness: public bridge external
  rows are still a backend simulation artifact, but their mutation authority is
  now the admitted aspect touch rather than a terminal path string.

## Query intent-admission certification probe native matching

- Removed control-flow dependence on `admitted_touch_digest_part()` from the
  intent-admission certification runtime's existing-truth probe fixture.
- The certification adapter now matches probed fields by inspecting
  `WORTHQueryAspectTouch::native_aspect_key()` and
  `WORTHQueryAspectTouch::native_field_path()` directly, preserving the
  admitted touch as the decision carrier instead of formatting it into
  `identity:id` / `title:value` strings and branching on those projections.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query intent_admission --lib`, focused green `cargo test -p
  worth-query --test intent_admission_public_dx`, and a targeted scan showing
  `admitted_touch_digest_part()` is gone from
  `src/intent_admission/certification/fixtures/runtime.rs`.
- This keeps digest projections in proof/evidence/reporting lanes while
  removing one production-hosted certification fixture that had been using the
  digest string as semantic branch authority.

## Query graph-read predicate admission native lookup key

- Replaced boolean predicate admission's raw
  `BTreeMap<(String, String, String), ...>` lookup key with
  `AdmittedPredicateKey`, retaining Foundational `AspectKey`, Foundational
  `FieldKey`, and the local predicate family label.
- Admitted predicate rows now enter the index through their native aspect and
  field carriers. Declarative predicate filters are admitted into the same
  typed key shape before lookup instead of comparing anonymous string triples.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query --test graph_read_access_phase_four_requirements`, focused green
  `cargo test -p worth-query --test graph_read_access_phase_six_admission`,
  focused green `cargo test -p worth-query --test
  graph_read_access_phase_two_selectivity`, focused green `cargo test -p
  worth-query --test graph_read_access_phase_four_operator_matrix`, and a
  scan showing the raw `(String, String, String)` admitted predicate lookup
  shape removed from Query source/tests.
- This is a graph-read Law 41 storage/lookup cleanup: admitted schema
  predicate references now keep their aspect/field proof through boolean
  expression admission instead of being weakened to string tuples.

## Query effect intent input native aspect-touch value

- Added a private `WORTHQueryIntentInputValue::AspectTouch` variant so
  effect-derived intent inputs retain `WORTHQueryAspectTouch` values inside the
  input tree.
- `WORTHQueryIntentInput::from_effect_payload(...)` now stores
  `changed_aspects`, `input_aspects`, and `output_aspects` arrays as native
  touch values instead of generic string values containing
  `admitted_touch_digest_part()` projections.
- Digest material still projects aspect touches into stable text at the final
  digest boundary, but the pre-digest carrier is no longer an anonymous
  `String` inside intent input.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query intent_admission --lib`, focused green `cargo test -p
  worth-query effect --lib`, and a targeted scan showing the old
  `WORTHQueryIntentInputValue::String(...admitted_touch_digest_part...)`
  construction and `intent_input_string_array` helper are gone.
- This is a Law 41 transition-order cleanup for effect-triggered intent
  payloads: aspect touches remain native until digest serialization instead of
  becoming generic intent strings during payload shaping.

## Query batch touched-aspect native dedupe

- Replaced digest-string keyed touched-aspect dedupe maps with native
  `BTreeSet<WORTHQueryAspectTouch>` in batch write execution, batch receipt
  aggregate derivation, and unified batch component inspection.
- These paths still return ordered, deduplicated native touch vectors, but the
  uniqueness rule is now enforced by the admitted touch carrier itself instead
  of by `admitted_touch_digest_part()` string projections.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query batch --lib`, focused green `cargo test -p worth-query
  inspection --lib`, focused green `cargo test -p worth-query --test
  public_submission_lane_replacements`, and a scan showing digest-keyed
  touched-aspect insertions removed from the converted runtime files.
- This is a Law 41 mechanical enforcement slice for batch/inspection
  aggregation: touch identity is now a native set membership fact, not a string
  map key that happens to round-trip to a native touch afterward.

## Query preview affected-target native routing maps

- Replaced preview session affected-aspect propagation maps from
  `BTreeMap<String, Vec<WORTHQueryAspectTouch>>` to maps keyed by
  `WORTHQueryLiveArtifactTarget` and `WORTHQueryDerivedMaterializationTarget`.
- Preview live/computed/effect routing still reports handle names in public
  evidence, but internal propagation now routes changed aspect touches through
  the existing native target wrappers instead of anonymous view-name strings.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query preview --lib`, focused green `cargo test -p worth-query effect
  --lib`, focused green `cargo test -p worth-query --test
  public_submission_lane_replacements`, and a scan showing the old
  `BTreeMap<String, Vec<WORTHQueryAspectTouch>>` preview routing shape removed.
- This is a scoped Law 41 routing cleanup: aspect-touch propagation across
  preview live and computed surfaces is now indexed by typed Query artifact
  targets, while terminal handle text remains at the reporting boundary.

## Query focused-inspector denial native aspect retention

- Changed view-shape live focused-inspector projection filtering so rejected
  widening aspects are retained as Foundational `AspectKey` values until the
  final diagnostic string is rendered.
- `focus_projection(...)` already received a native focus aspect. Its failure
  branch now returns rejected native aspect keys instead of weakening each
  rejected key to `String` before the denial decision is built.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query focused_inspector_widening_is_denied_and_counted --lib`, and a
  targeted scan showing `native_aspect_key().as_str().to_string()` removed
  from `src/view_shape_live/execution.rs`.
- This is a Law 41 denial-path cleanup: focused inspector widening rejection is
  decided over native aspect identity, with string projection confined to the
  human-readable error message.

## Query retained evidence admitted-aspect-touch vocabulary

- Renamed production evidence identity tags that still described admitted touch
  material as `aspect_path` / `aspect_paths`.
- Computed patch inspection, effect delivery inspection, preview execution
  evidence, intent-admission symbolic aspect seeds, unified write/batch
  inspection identities, graph-composition evidence, and batch mutation
  evidence now use `admitted_aspect_touch` or `admitted_aspect_touches`.
- The digest values still project admitted touches to stable text at the
  evidence boundary, but retained evidence no longer teaches that those values
  are free-form aspect paths.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query batch --lib`, focused green `cargo test -p worth-query effect
  --lib`, focused green `cargo test -p worth-query preview --lib`, focused
  green `cargo test -p worth-query intent_admission --lib`, and a production
  scan showing `WORTHQueryEvidenceTag::new("aspect_path")` and
  `WORTHQueryEvidenceTag::new("aspect_paths")` removed from
  `crates/worth-query/src`.
- This is a retained-evidence Law 41 cleanup: evidence rows now name the proof
  state they are digesting instead of preserving the old dotted-path authority
  vocabulary.

## Query in-memory facade native row equivalence keys

- Replaced the in-memory test-backend facade live-row equivalence carrier from
  `Vec<(String, Option<AspectValue>)>` to
  `Vec<(CanonicalFieldPath, Option<AspectValue>)>`.
- The facade equivalence test still uses short authoring text to build
  fixtures, but the assertion key is now the same native field-path carrier
  used by retained rows.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query --test in_memory_test_backend_facade`, and a crate scan showing
  the old `Vec<(String, Option<AspectValue>)>` shape removed.
- This closes a test-harness Law 41 leak where bridge/in-memory row equality
  was being proven against string field labels instead of native field-path
  identity.

## Query grouped row fixture native value carriers

- Replaced grouped live/view-shape and milestone-eight certification
  `GroupedRowFixture = (String, String, String)` aliases with named
  `GroupedRowFixture` structs.
- Fixture member identity remains the terminal record selector needed by the
  bridge harness, but display and lane values are retained as foundational
  `AspectValue` values produced through Query's native scalar helper instead
  of anonymous string tuple slots.
- Snapshot read helpers now ask the row fixture for the value corresponding to
  the terminal bridge read contract. Terminal labels such as `identity.id`,
  `profile.display_name`, and `status.lane` remain at the bridge read-contract
  boundary, not as the retained row carrier shape.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo test -p worth-query view_shape_live --lib`, green `cargo test -p
  worth-query milestone_eight_certification --lib`, green `cargo check -p
  worth-query --tests`, and scans showing grouped row tuple aliases,
  destructures, and string-tuple literals removed from those grouped harnesses.
- This is a certification/test Law 41 cleanup: grouped fixtures now name the
  proof-bearing row fields and retain native scalar values instead of relying
  on positional string triples.

## Query projection-policy native assertion keys

- Replaced projection and policy test assertion helpers that converted
  authorized field paths into `(String, String)` tuples.
- `authorized_projection`, `policy_live`, and `policy_narrowing` tests now
  compare `(AspectKey, FieldKey)` tuples cloned from
  `AuthorizedProjectionFieldPath`.
- Fixture authoring still uses short string literals, but the proof assertions
  no longer weaken admitted projection fields to string labels before equality
  checks.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query authorized_projection --lib`, focused green `cargo test -p
  worth-query policy_live --lib`, focused green `cargo test -p worth-query
  policy_narrowing --lib`, and a scan showing
  `native_aspect_key().as_str().to_string()` /
  `native_field_key().as_str().to_string()` removed from those three test
  modules.
- This is a test-certification Law 41 cleanup: policy projection proof checks
  now assert over native aspect/field identity, not terminal string pairs.

## Query integration fixture authored-touch ingress

- Added a shared integration-test `support::aspect_touch(...)` helper as the
  single root-level fixture ingress from short authored touch text to
  `WORTHQueryAspectTouch`.
- Replaced duplicated local `fn touch(aspect_path: &str)` parsers across the
  public bridge, graph read, intent DX, in-memory backend, public submission,
  hostile graph fixture, and public-bridge hostile support tests.
- Renamed the remaining graph-obligation local wrapper vocabulary from
  `aspect_path` to `authored_touch_text`. That fixture keeps a local parser
  because its module tree cannot import the root `tests/support` module without
  pulling in submodules that rely on a different crate-root topology.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green integration runs for
  `graph_composition_public_bridge`,
  `graph_composition_public_bridge_existing`,
  `runtime_backed_read_bootstrap`, `public_bridge_runtime_bootstrap`,
  `graph_read_access_phase_eight_ephemeral_provisioning`,
  `graph_read_access_phase_nine_streaming_frontier`,
  `graph_read_access_phase_thirteen_live_maintenance`,
  `in_memory_test_backend_facade`, `public_submission_lane_replacements`,
  `graph_obligation_consumer_kit_facade`,
  `graph_obligation_hostile_certification`, and `intent_admission_public_dx`.
- A test-source scan now shows no local `fn touch(aspect_path: &str)`, no
  `fn set_operation(aspect_path: &str)`, no old `test/fixture aspect path`
  parser messages, and no `AspectKey::new(segment.to_string())` /
  `FieldKey::new(segment.to_string())` parser clones under
  `crates/worth-query/tests`.
- This is a mechanical test-harness cleanup: tests still get ergonomic authored
  touch fixtures, but the dotted-text lowering is centralized and named as
  authoring ingress instead of being reimplemented as ad hoc aspect-path
  authority in every integration test.

## Query boundary reporting admitted-touch cleanup

- Replaced the test-backend schema duplicate-aspect diagnostic helper from
  `terminal_projection_from_aspect_touch(...)` to
  `reporting_projection_from_admitted_touch(...)`.
- The schema already stored `WORTHQueryAspectTouch`; duplicate diagnostics now
  report the admitted touch digest from that carrier instead of rebuilding a
  terminal dotted aspect path from native keys.
- Renamed the public bridge existing-truth seed reporting helper from
  `terminal_aspect_path_projection(...)` to
  `admitted_aspect_touch_reporting_projection(...)` and changed the assertion
  to the admitted-touch reporting form.
- Simplified public bridge and stateful bridge external-row boundary lowering
  so `FieldKey::new(...)` receives the native aspect key text directly instead
  of explicitly allocating `native_aspect_key().as_str().to_string()`.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, focused green `cargo test -p
  worth-query test_backend --lib`, focused green `cargo test -p worth-query
  --test public_bridge_runtime_bootstrap`, focused green `cargo test -p
  worth-query --test public_submission_lane_replacements`, and focused green
  `cargo test -p worth-query bridge_backed_verification_execution --lib`.
- Source scans now show no
  `native_aspect_key().as_str().to_string()`,
  `native_field_key().as_str().to_string()`, or
  `terminal_aspect_path_projection(...)` under production or non-UI test
  sources. The only `terminal_aspect_path_projection(...)` mentions left are
  compile-fail fixtures that prove those public APIs remain removed.
- This closes the remaining boundary/reporting old-vocabulary leaks without
  adding a public terminal projection API or weakening the crate-private
  admitted-touch digest fence.

## Query crate-wide JSON residue enforcement

- Added
  `consumer_kit::support_snapshot::tests::runtime_boundary::rust_source_serde_json_residue_stays_terminal_or_compile_fail_only`.
  This scans every Rust source file under the `worth-query` crate root and
  fails unless `serde_json` appears only in the two approved support terminal
  codecs, the guard file itself, or the deliberately hostile compile-fail
  fixtures that prove raw JSON cannot enter retained rows, derived patches, or
  program operation inputs.
- Added
  `consumer_kit::support_snapshot::tests::runtime_boundary::query_sources_do_not_define_local_foundational_json_compatibility_bridge`.
  This keeps Query from quietly adding `JsonCompatibility*`,
  `compatibility().json()`, or `lower_json_*` usage as an unapproved local
  aspect-compatibility bridge. The current policy remains that external
  JSON-as-aspect lowering belongs in Foundational before Query receives native
  artifacts.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query
  rust_source_serde_json_residue_stays_terminal_or_compile_fail_only --lib`,
  and green focused `cargo test -p worth-query
  query_sources_do_not_define_local_foundational_json_compatibility_bridge
  --lib`.
- This is a mechanical enforcement slice: it does not remove the two approved
  durable terminal document codecs, but it prevents ordinary production or
  certification code from regrowing JSON authority without deliberately
  changing the checked allowlist.

## Query certification fixture aspect-path vocabulary cleanup

- Renamed the runtime API stabilization transcript touch helper parameter from
  `aspect_path` to `authored_touch_text` and its live field helper parameter to
  `authored_field_text`.
- Renamed the remaining computed-test update helper parameter from
  `aspect_path` to `authored_touch_text`.
- These helpers already lower into `AspectKey`, `FieldKey`,
  `CanonicalFieldPath`, `AspectFieldKey`, or `WORTHQueryAspectTouch`; the
  cleanup removes stale authority vocabulary from src-hosted certification and
  computed test support.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query runtime_api_stabilization --lib`, green focused
  `cargo test -p worth-query computed --lib`, green `cargo check -p
  worth-query --tests`, and a targeted scan showing no `fn .*aspect_path`,
  `aspect_path:`, `let aspect_path`, or old runtime-transcript aspect-path
  messages under Query Rust sources except the explicitly terminal computed
  projection helper.

## Query effect live-trigger target carrier cleanup

- Renamed `WORTHQueryRuntime::live_view_targets()` to
  `live_artifact_target_collections()` and changed its map key from raw live
  view-name `String` to `WORTHQueryLiveArtifactTarget`.
- Changed effect delivery routing so live-trigger lookup constructs the same
  `WORTHQueryLiveArtifactTarget` carrier used by mutation receipts, preview
  routing, and live artifact bindings before it reaches the declared collection
  comparison.
- This does not pretend the live artifact target owns the collection identity;
  the collection text still comes from the live declaration request and is used
  only at the receipt-delta boundary. The improvement is that live-trigger
  routing no longer treats arbitrary live-view strings as the map authority for
  affected live artifacts.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query effect --lib`, green `cargo check -p
  worth-query --tests`, and a targeted scan showing the removed
  `live_view_targets(...)` helper gone from the converted routing files.

## Query verified-assumption native touch breadth cleanup

- Changed `WORTHQueryVerificationReadSetBreadth::new(...)` so distinct
  asserted aspect touch counting dedupes `WORTHQueryAspectTouch` carriers
  directly instead of first projecting every touch through
  `admitted_touch_digest_part()`.
- Removed the graph-composition assumption summary helper named
  `native_asserted_aspect_touch_digest_parts(...)`; assumption summary evidence
  now keeps the collected `WORTHQueryAspectTouch` values and formats them only
  at the final `field_value_sequence(...)` digest boundary.
- This keeps verification-read-set breadth counters and graph-composition
  assumption summary construction native through counting/aggregation, while
  preserving existing digest material at the terminal evidence identity layer.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  graph_composition --lib`, green focused `cargo test -p worth-query
  bridge_backed_verification_execution --lib`, green focused `cargo test -p
  worth-query graph_composition_verified_existing --lib`, and a targeted scan
  showing the removed digest-helper/string-dedupe pattern gone.

## Query effect payload terminal touch digest vocabulary cleanup

- Renamed `native_touch_digest_sequence(...)` to
  `terminal_touch_digest_projection_sequence(...)` in the effect delivery
  helpers.
- `WORTHQueryEffectPayload` already retains input, output, and changed aspects
  as `WORTHQueryAspectTouch` values; the helper only joins admitted touch
  digest parts for terminal digest/evidence material. The helper name makes
  that final string projection explicit instead of calling it native.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query effect --lib`, green `cargo check -p
  worth-query --tests`, and a targeted scan showing the old helper name gone
  from runtime effect sources.

## Query terminal aspect-touch digest helper vocabulary cleanup

- Renamed remaining helper functions that projected admitted aspect touches into
  strings while calling the result `native`:
  `native_aspect_digest_parts(...)` became
  `terminal_aspect_touch_digest_parts(...)` in the consumer test-backend
  equivalence report and effect-triggered intent handoff binding identity.
- Renamed graph obligation registration
  `native_touch_digest_parts(...)` to `terminal_touch_digest_parts(...)`.
  Selector matching and lookup still retain `WORTHQueryAspectTouch` carriers;
  this helper is used only while composing selector evidence identity material.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query intent_admission --lib`, green focused
  `cargo test -p worth-query graph_obligation --lib`, green focused
  `cargo test -p worth-query test_backend --lib`, green `cargo check -p
  worth-query --tests`, and a targeted source scan showing the old helper names
  gone.

## Query same-batch symbolic target carrier cleanup

- Added internal `WORTHQuerySameBatchSymbolicTarget` with private fields and
  read-only accessors for resolved same-batch symbolic target identity and
  optional target collection.
- Converted authoritative batch symbolic admission, resolution, symbolic aspect
  evidence rebuilding, and backend atomic-batch deferred target tracking from
  `BTreeMap<String, (WORTHQueryEntityIdentity, Option<String>)>` to the named
  carrier.
- Converted preview batch symbolic admission and staged symbolic target
  resolution to the same internal carrier, so preview and authoritative lanes
  no longer maintain parallel anonymous tuple target folklore.
- This is a Law 41 enforcement slice: unresolved/mismatched symbolic target
  validation still happens at the resolution transition, and downstream update,
  delete, receipt, and symbolic-aspect evidence code now receives a proven
  resolved target object instead of reconstructing meaning from tuple slots.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query batch --lib`, green focused `cargo test -p
  worth-query preview --lib`, green `cargo check -p worth-query --tests`, and
  a targeted scan showing no remaining `(WORTHQueryEntityIdentity,
  Option<String>)` same-batch symbolic target maps under
  `crates/worth-query/src/runtime`.

## Query grouped execution lane identity carrier cleanup

- Changed `GroupedExecutionLaneValue` to retain the existing native
  `GroupedLaneIdentity` carrier instead of duplicating `AspectKey` plus lane
  text fields locally.
- Kept the public grouped execution accessors intact while adding a native lane
  identity accessor for downstream code that needs the complete proof-bearing
  lane value.
- Grouped execution, grouped baseline, desired-state construction, and grouped
  delta comparison now share the same lane identity vocabulary after truth-view
  materialization.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query view_shape_live --lib`, and green `cargo check
  -p worth-query --tests`.

## Query collection ordering native key constructor cleanup

- Added `OrderingKeyPath::from_native_keys(...)` for validated ordering entries
  that already carry Foundational `AspectKey` and `FieldKey` values.
- Changed collection planning to build ordering key paths from cloned native
  keys instead of projecting validated keys through `as_str()` and reparsing
  them with the weaker text constructor.
- The existing text constructor remains for authoring/default construction, but
  validated query-bundle lowering no longer recovers ordering identity from
  terminal key text after validation has already produced native carriers.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query collection --lib`, and green `cargo check -p
  worth-query --tests`.

## Query live relevance native field-key constructor cleanup

- Added `QueryFieldKey::from_native_keys(...)` for validated live relevance
  fields that already carry Foundational `AspectKey` and `FieldKey` values.
- Changed detail, ordered collection, and bounded materialization live
  relevance contracts to preserve native projection/ordering field keys from
  validated query bundles instead of projecting them through `as_str()` and
  reparsing with the weaker authoring constructor.
- Added `QueryFieldKey::terminal_digest_part(...)` and routed detail field
  patch digest formatting through that terminal formatter, so the remaining
  text projection is explicitly evidence material rather than relevance
  authority.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query live --lib`, green `cargo check -p worth-query
  --tests`, and a targeted scan showing the old live relevance
  `entry.native_*().as_str()` reparse pattern removed.

## Query effect aspect admission native-boundary cleanup

- Removed the `validate_declared_effect_aspects(...)` runtime helper that
  projected each `WORTHQueryAspectTouch` through
  `native_aspect_key().as_str().is_empty()` during effect declaration
  admission.
- Effect admission still rejects empty trigger/input/output aspect lists, but
  individual aspect validity now comes from the native `WORTHQueryAspectTouch`
  / Foundational `AspectKey` construction boundary instead of an impossible
  post-hoc string check.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query effect --lib`, green `cargo check -p
  worth-query --tests`, and a targeted scan showing the removed effect
  validator and native-key string emptiness check are gone.

## Query aspect-field native lowering boundary cleanup

- Added crate-private `AspectFieldKey::native_aspect_key()` and
  `AspectFieldKey::native_field_key()` as the single authoring-name to
  Foundational-key lowering boundary for admitted aspect field keys.
- Converted read obligation dispatch, preview aspect touch derivation, and live
  subscription delivery routing to use that named boundary instead of each
  consumer rebuilding `AspectKey` / `FieldKey` from `field.aspect().as_str()`
  and `field.field().as_str()`.
- This keeps the unavoidable authoring-to-native transition in one place and
  removes three runtime consumers that were reconstructing native proof from
  text after declarative read/live/preview admission.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query read_obligation_dispatch --lib`, green focused
  `cargo test -p worth-query preview --lib`, green focused `cargo test -p
  worth-query live --lib`, green `cargo check -p worth-query --tests`, and a
  targeted scan showing the repeated runtime `AspectKey::new(field.aspect()
  .as_str())` / `FieldKey::new(field.field().as_str())` pattern removed.

## Query graph composition terminal operation digest vocabulary cleanup

- Renamed the graph composition helper
  `native_declared_aspect_operation_digest_part(...)` to
  `terminal_declared_aspect_operation_digest_part(...)` in obligation
  registration selector helpers, lookup-key material, and touch descriptor row
  digest construction.
- Behavior is unchanged: `WORTHQueryAspectMutationOperation` remains the
  native operation carrier, and the produced string is terminal selector,
  digest, and evidence material rather than native authority.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query graph_obligation --lib`, green focused
  `cargo test -p worth-query graph_composition --lib`, green `cargo check -p
  worth-query --tests`, and a targeted scan showing no
  `native_declared_aspect_operation_digest_part` residue under graph
  composition.

## Query terminal digest material vocabulary root break

- Mechanically renamed the high-fanout `native_digest_material()` helper family
  to `terminal_digest_material()` across admitted aspect values, desired aspect
  values, effect payloads, mutation lowering, backend mutation authority,
  assertion construction, intent handoff bindings, and verification adapters.
- Renamed the coupled `aspect_value_native_digest_text(...)` formatter to
  `terminal_aspect_value_digest_text(...)`, making the `AspectValue` carrier
  native while the formatted string is explicitly terminal digest/evidence
  material.
- Renamed internal row and metadata formatter helpers from
  `native_digest_text`, `native_digest_parts`, and
  `native_result_digest_parts` to terminal vocabulary where they produce
  evidence strings from already-native carriers.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query mutation --lib`, green focused `cargo test -p
  worth-query effect --lib`, green focused `cargo test -p worth-query computed
  --lib`, green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, green `cargo check -p worth-query
  --tests`, and residue scans showing no remaining
  `native_digest_material`, `aspect_value_native_digest_text`,
  `native_digest_text`, `native_digest_parts`, or
  `native_result_digest_parts` helper names.

## Query existing-truth denial terminal value digest fence

- Renamed `WORTHQueryExistingTruthAssertionDenial`'s stored
  `expected_native_value_digest` and `found_native_value_digest` strings to
  `expected_terminal_value_digest` and `found_terminal_value_digest`.
- Renamed the public denial accessors to
  `expected_terminal_value_digest()` and `found_terminal_value_digest()`, so
  consumers cannot treat terminal digest strings as native value authority.
- Renamed the existing-truth denial evidence tags from
  `expected_native_value` / `found_native_value` to
  `expected_terminal_value` / `found_terminal_value`, and renamed the mutation
  lowering row value tag from `native_value` to `terminal_value`.
- Added `assertion_denial_native_value_digest_alias_removed.rs` compile-fail
  coverage proving facade callers cannot use the old native-value digest
  accessors and are directed to the terminal accessors instead.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query verify_existing --lib`, green focused
  `cargo test -p worth-query verified_update_existing --lib`, green focused
  `cargo test -p worth-query verified_delete_existing --lib`, green
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, green
  `cargo check -p worth-query --tests`, and residue scans showing no remaining
  `expected_native_value_digest`, `found_native_value_digest`,
  `native_value_digest`, `expected_native_value`, `found_native_value`, or
  `native_value` evidence tag usage in Query Rust sources.

## Query grouped binding native source-key constructor cleanup

- Added `AspectFieldKey::from_native_keys(...)` as the named crate-local bridge
  for constructing authoring field keys from already-native foundational
  `AspectKey` and `FieldKey` values.
- Replaced grouped planning identity-binding checks that compared
  `native_source_aspect_key().as_str()` and
  `native_source_field_key().as_str()` with equality against native
  `AspectKey` / `FieldKey` carriers.
- Replaced `QueryResultBindingProof::new(String/String, ...)` with
  `QueryResultBindingProof::from_native_source_keys(&AspectKey, &FieldKey, ...)`
  so grouped planning cannot rebuild binding proofs from arbitrary terminal
  text.
- Remaining string projection inside the constructor is localized to creating
  the terminal composite binding aspect key / authoring field key; grouped
  planning authority now flows through the native source keys.
- Verification passed: `cargo check -p worth-query --tests`, and a targeted
  scan showing no remaining `QueryResultBindingProof::new` or grouped planning
  `native_source_*().as_str()` comparisons.

## Query projection-consumption native grouping-aspect oracle cleanup

- Changed projection-consumption grouped oracle comparison helpers so membership
  entries accept `&AspectKey` and relation-endpoint entries accept
  `Option<&AspectKey>` instead of arbitrary grouping-aspect text.
- Localized the final string conversion behind
  `terminal_grouping_aspect_digest_part(...)`, making the terminal projection a
  digest boundary instead of a caller-supplied authority input.
- Converted the structured-content validation fixture to compare
  `native_source_aspect_key()` / `native_source_field_key()` against
  foundational `AspectKey` / `FieldKey` values instead of comparing their
  `.as_str()` projections to `"content"` / `"bio"`.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query projection_consumption --lib`, green focused
  `cargo test -p worth-query validation_cases --lib`, green
  `cargo check -p worth-query --tests`, and a targeted scan showing no old
  grouped oracle `grouping_aspect().as_str()` handoff or structured-content
  `native_source_*().as_str()` comparison residue.

## Query grouped support native claim-admission cleanup

- Converted grouped shared-posture support admission away from string decisions:
  `SharedMaterialPreview` now compares `WORTHQueryGroupedIntent::Authoritative`
  directly, and `SharedContinuity` now compares
  `WORTHQueryGroupedContinuityAssumption::PreserveNeighborhood` directly.
- Replaced grouped support `AspectFieldKey` matching through
  `key.aspect().as_str()` / `key.field().as_str()` with equality against an
  `AspectFieldKey` built from foundational `AspectKey` / `FieldKey` carriers.
- Removed the local `aspect_field_key_matches(...)` helper so support-claim
  admission cannot accidentally accept arbitrary aspect/field labels as proof.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query grouped_authoring --lib`, green
  `cargo check -p worth-query --tests`, and a targeted scan showing the old
  grouped-support string comparison patterns removed.

## Query memory workspace native declared-aspect alias cleanup

- Replaced memory workspace declared-aspect matching that reparsed
  `native_field_path` through a terminal dotted string with direct construction
  of a native `WORTHQueryAspectTouch::aspect_field_path(...)` alias.
- Removed the local `parsed_aspect_touch(...)` helper and deleted the now-dead
  workspace terminal field-path projection helper, so declared-aspect matching
  no longer recovers an admitted touch from display text after the workspace
  already has native `AspectKey` / `CanonicalFieldPath` carriers.
- Left `memory_workspace/entity_row.rs` terminal field-path formatting in place
  because it is only used for `terminal_result_digest_parts()` digest/report
  material, not for workspace declaration matching.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query memory_workspace --lib`, green focused
  `cargo test -p worth-query test_backend --lib`, green
  `cargo check -p worth-query --tests`, and targeted scans showing
  `parsed_aspect_touch(...)` gone from the memory workspace.

## Query stateful bridge grouped baseline native projection selection cleanup

- Changed the stateful bridge runtime test backend grouped baseline member
  extraction so it selects identity and grouping projection fields by native
  `AspectKey` values from `DeclarativeProjectionField::source_field_key()`
  instead of comparing `field.aspect()` against terminal grouping aspect text.
- Added `external_row_text_at_path(...)` for named external-row terminal lookup
  from an already-built native `CanonicalFieldPath`, then deleted the stale
  dotted-path `external_row_text(...)` / `native_external_field_path(...)`
  helpers from the stateful bridge write support.
- The remaining aspect-as-field conversion is localized inside
  `native_external_field_path_for_touch(...)`, which is the test backend's
  explicit external-row compatibility boundary.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo test -p worth-query --test public_bridge_runtime_bootstrap`, green
  `cargo test -p worth-query --test runtime_backed_read_bootstrap`, green
  focused `cargo test -p worth-query live_grouped --lib`, green focused
  `cargo test -p worth-query runtime::tests::live --lib`, green
  `cargo check -p worth-query --tests`, and a targeted scan showing the old
  grouped baseline `grouping_aspect_text` / dotted-path helper residue removed
  from the stateful bridge runtime support.

## Query same-batch symbolic collection carrier cleanup

- Changed `WORTHQuerySameBatchSymbolicTarget` so resolved same-batch symbolic
  targets retain `WORTHQueryMutationTargetCollectionIdentity` instead of
  `Option<String>` collection text.
- Converted authoritative batch execution, atomic-batch planning/rebuild,
  preview-batch preflight, preview symbolic receipt construction, and command
  declared-collection identity derivation to pass collection proof carriers
  through the symbolic target flow.
- Left string projection only at explicit bridge, memory-delta, receipt, and
  denial-message boundaries. Collection mismatch checks now compare the labels
  from admitted collection identity carriers because the evidence identity is
  role-scoped by construction.
- Added
  `symbolic_target_reference_denies_collection_mismatch_with_native_target_identity`,
  proving authoritative same-batch symbolic resolution denies a typed
  collection mismatch while preserving the native resolved-target carrier
  through admission.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query symbolic_reference --lib`, green focused
  `cargo test -p worth-query preview_branch_obligation_dispatch --lib`, green
  focused `cargo test -p worth-query batch --lib`, green `cargo check -p
  worth-query --tests`, and a targeted scan showing the old
  `BTreeMap<String, (WORTHQueryEntityIdentity, Option<String>)>` symbolic
  target shape no longer appears in the converted runtime symbolic paths.

## Query retained projection source target carrier cleanup

- Changed projection-consumption source construction for retained derived
  artifact bindings and live artifact bindings to build source-reference
  identities from `WORTHQueryDerivedMaterializationTarget` and
  `WORTHQueryLiveArtifactTarget` carriers instead of consuming
  `terminal_target_view_names_projection()` iterators.
- This aligns `ProjectionConsumptionSource::from_retained_derived_artifact_binding`
  and `ProjectionConsumptionSource::from_live_artifact_binding` with the
  retained/live extraction paths, which already iterate `binding.targets()`.
- The only remaining view-name string projection in this path is the explicit
  source-reference identity formatting boundary via each target's
  `terminal_view_name_projection()`.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query retained_binding_declaration_preserves_binding_identity_and_target_refs --lib`,
  green focused `cargo test -p worth-query retained_live --lib`, green
  focused `cargo test -p worth-query projection_consumption --lib`, green
  `cargo check -p worth-query --tests`, and a targeted scan showing
  `terminal_target_view_names_projection()` gone from projection-consumption
  source construction and extraction.

## Query graph composition collection carrier cleanup

- Changed `WORTHQueryGraphCompositionProgramStep` so declared collection state
  is stored as `Option<WORTHQueryMutationTargetCollectionIdentity>` instead of
  raw `String` text. `declared_collection_identity()` is now the native carrier
  accessor; `declared_collection()` remains only as a terminal/reporting
  projection.
- Converted graph-composition builder symbolic entity/relation declarations,
  symbolic followups/retirements, duplicate-symbol denials, existing-target
  lifecycle steps, retarget/supersession denial helpers, and workspace graph
  error mapping to pass target collection identities instead of terminal
  collection strings.
- Updated graph touch descriptor validation and touch-row derivation to compare
  and derive from command/program collection identity carriers, projecting to
  strings only for denial messages and descriptor rows.
- Converted graph obligation/touch descriptor fixtures and stop-class graph
  denial fixtures that directly constructed program steps or denials to use
  `WORTHQueryMutationTargetCollectionIdentity` carriers.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, green focused `cargo test -p worth-query
  graph_composition --lib`, green focused `cargo test -p worth-query
  graph_obligation --lib`, and a targeted scan showing the chased
  graph-composition collection string projections removed.

## Query mutation receipt target evidence carrier cleanup

- Changed `WORTHQueryMutationTargetDescriptor::new(...)` to accept
  `Option<WORTHQueryMutationTargetCollectionIdentity>` instead of
  `Option<String>`, so receipt target evidence no longer strips collection
  identities to labels and reconstructs weaker replacement identities.
- Updated write receipt target evidence construction to pass declared and
  resolved collection identity carriers directly.
- Updated symbolic target reference evidence, symbolic aspect resolution
  evidence, and symbolic aspect lowering digest rows to clone/use the
  collection identity already attached to the Query symbolic reference instead
  of rebuilding a target collection identity from
  `reference.target_collection()` terminal text.
- Updated preview symbolic entity identity construction to cite the target
  collection evidence identity rather than embedding collection text as an
  optional shape field.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, green focused `cargo test -p worth-query
  symbolic_reference --lib`, green focused `cargo test -p worth-query
  aspect_preview --lib`, and green focused `cargo test -p worth-query batch
  --lib`.

## Query naming and continuity target collection evidence cleanup

- Changed naming and continuity mutation evidence construction so receipt
  helpers accept `WORTHQueryMutationTargetCollectionIdentity` carriers rather
  than optional collection string slices.
- Authoritative, batch, and preview write receipt construction now passes the
  target collection identity carrier through naming/continuity evidence instead
  of projecting the collection label and rebuilding a weaker identity.
- Preview receipt construction now derives its target collection identity from
  `command.declared_collection_identity()` and no longer depends on command
  terminal collection projection helpers.
- Removed the now-dead command terminal declared-collection projection accessor
  and updated the compile-fail fixture to prove outside consumers cannot call
  it.
- Bridge bundle text remains an explicit bridge-boundary compatibility input;
  the bridge helpers convert it once into a target collection identity and
  native receipt construction stays on carriers after that.
- Verification covered green `cargo check -p worth-query --tests`, green
  `cargo test -p worth-query --test aspect_native_query_compile_fail`, green
  focused `cargo test -p worth-query naming --lib`, and green focused
  `cargo test -p worth-query continuity --lib`.

## Query receipt-summary collection carrier cleanup

- Changed `classify_receipt_mutation_summary(...)` so the single resolved
  receipt collection returns `WORTHQueryMutationTargetCollectionIdentity`
  instead of raw collection text.
- Updated authoritative write routing, batch component receipt construction,
  intent/effect execution routing, and same-batch symbolic target recording to
  pass that collection carrier through instead of projecting labels and
  rebuilding target identities.
- Updated bridge naming/continuity lowering and symbolic-target bridge lowering
  to accept Query target collection identity carriers. Bridge lowering still
  projects terminal text at the bridge facade boundary, but runtime callers no
  longer hand it anonymous strings.
- Updated preview symbolic write receipt attachment to retain collection
  identity carriers and removed the now-dead cloned terminal collection helper
  from same-batch symbolic targets.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green focused `cargo test -p
  worth-query bridge_lowered_continuity --lib`, green focused `cargo test -p
  worth-query batch --lib`, and green focused `cargo test -p worth-query
  aspect_preview --lib`.

## Query read-composition relation materialization cleanup

- Changed runtime read-composition materialization so traversal selection passes
  native `RelationName` carriers into relation-target lookup instead of passing
  `terminal_relation_projection_for_boundary()` text.
- Localized the legacy row-field compatibility mapping in
  `relation_target_field_path(...)`, which accepts a `RelationName` and builds
  the corresponding foundational `CanonicalFieldPath` from explicit
  `FieldKey`s.
- Changed identity anchor and identity ordering detection to compare
  `Declarative*::source_field_key()` through native `AspectFieldKey` lowering
  rather than comparing `aspect() == "identity"` / `field() == "id"` terminal
  aliases.
- A targeted scan now shows no
  `terminal_relation_projection_for_boundary()`, `ordering.aspect()`,
  `ordering.field()`, `filter.aspect()`, or `filter.field()` use in
  `read_composition_materialization.rs`.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo test -p worth-query read_composition --lib`, and green
  `cargo check -p worth-query --tests`.

## Query bridge row extraction native projection cleanup

- Changed bridge truth-view row-set projection fact extraction so bridge fields
  derive `ProjectionFactFieldPath` from
  `BridgeMaterializedFieldValue::projection()` metadata instead of parsing
  `BridgeMaterializedFieldIdentity::as_str()` as a dotted field path.
- Added `ProjectionMaterializedField::from_bridge_field_value(...)` as the
  bridge-specific constructor. It uses the bridge field locator's native
  `CanonicalFieldPath` when present, and falls back to the bridge projection's
  native aspect key for whole-aspect projections.
- Bridge field identity text remains available only for diagnostic field labels
  on invalid bridge field shapes; it no longer feeds consumed fact lookup keys.
- A targeted scan now shows the bridge row-set path no longer calls
  `key.as_str()` / `from_external_boundary(...)` for field authority; the
  remaining `key.as_str()` in `row_like.rs` is the separate relational row-set
  extraction root.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, and green focused
  `cargo test -p worth-query projection_consumption --lib`.

## Query memory workspace aspect constructor proof check

- Changed public `WORTHQueryAspect::new(...)` to return
  `Result<WORTHQueryAspect, WORTHQueryWorkspaceError>` and deny mismatches
  between the admitted `WORTHQueryAspectTouch` and the declared
  `CanonicalFieldPath`.
- The constructor now proves that every declared memory-workspace native field
  path is rooted at the admitted aspect key. For field-specific touches, it
  also proves the full path equals `aspect_key + touch field path`, so callers
  cannot combine `title.value` touch authority with an `identity.id` storage
  path and let the workspace carry that false proof downstream.
- The crate-local `from_native_field_path(...)` remains the trusted conversion
  used by test-backend schema lowering after that schema has already admitted
  the touch and field path pair.
- Added
  `memory_workspace_aspect_rejects_mismatched_native_field_path` and refreshed
  the aspect-native compile-fail fixture so the public constructor stays
  fallible while `native_field_path()` still cannot be treated as `&str`.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query memory_workspace --lib`, green
  `cargo check -p worth-query --tests`, and green
  `cargo test -p worth-query --test aspect_native_query_compile_fail`.

## Query authoring aspect-field native key storage cleanup

- Changed `AspectName` and `FieldName` in `authoring/names.rs` from raw
  `String` wrappers to foundational `AspectKey` and `FieldKey` wrappers.
- `AspectFieldKey` now retains foundational key proof through its existing
  name wrappers. Public authoring ergonomics and `aspect().as_str()` /
  `field().as_str()` display access stay intact, but admission now uses the
  same Foundational key constructors as downstream native authority paths.
- `AspectFieldKey::from_native_keys(...)` now clones existing `AspectKey` /
  `FieldKey` carriers directly instead of projecting them to text and
  reparsing. `native_aspect_key()` and `native_field_key()` also clone the
  retained carriers instead of reconstructing proof from strings.
- Added focused authoring tests proving invalid aspect/field text is rejected
  by foundational admission and native-key construction preserves the original
  carriers.
- Verification covered green `cargo fmt -p worth-query`, green focused
  `cargo test -p worth-query authoring::names --lib`, green
  `cargo check -p worth-query --tests`, a line-count check showing
  `authoring/names.rs` remains under the 400-line cap, and a targeted scan
  showing the removed native-key text reparse pattern is gone.

## Query write-command collection identity storage cleanup

- Changed `WORTHQueryWriteCommand::InsertAspects.collection` and
  `WORTHQueryWriteCommand::DeleteAspects.declared_collection` to retain
  `WORTHQueryMutationTargetCollectionIdentity` carriers instead of raw
  collection text.
- Insert/delete builders still accept ergonomic collection text, but they lower
  immediately into `"write-command-declared"` target collection identities
  before constructing command proof state.
- Changed `WORTHQueryBackendAdmissibleMutationShape::Insert.collection` and
  `DeleteDirect.declared_collection` to retain backend-admissible collection
  identities. Command-to-backend admission re-roles the carrier once at the
  proof-state transition boundary.
- Converted symbolic delete rebuilds and direct command-literal fixtures to
  carry collection identities. Preview memory deltas now project collection
  labels only at the terminal memory-delta boundary.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query mutation
  --lib`, green focused `cargo test -p worth-query batch --lib`, green
  focused `cargo test -p worth-query preview_branch_obligation_dispatch
  --lib`, line-count checks for the touched files, and targeted scans showing
  the old command/backend-admissible collection string fields are gone.

## Query graph touch selector collection carrier cleanup

- Changed graph touch selector collection variants to retain
  `WORTHQueryMutationTargetCollectionIdentity` carriers instead of collection
  text. Public selector constructors remain ergonomic text ingress, but lower
  immediately to selector-role collection identities.
- Changed graph touch descriptor rows to retain declared collection identity
  carriers. Row `declared_collection()` remains a terminal/reporting
  projection, while selector matching uses `touches_declared_collection(...)`
  against collection identity carriers.
- Added a private graph-obligation collection lookup identity for index keys,
  built only from target collection carriers. This removes the old
  `Collection(String)` lookup shape while preserving deterministic ordered
  lookup keys.
- Removed the dead string-based `touches_collection(...)` and collection-value
  selector accessors, so graph obligation matching no longer has a raw
  collection-label comparison path.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  graph_obligation --lib`, green focused `cargo test -p worth-query
  graph_touch_descriptor --lib`, line-count checks showing touched files under
  the 400-line cap, and a targeted scan showing old graph selector/row
  collection string shapes are gone.

## Query retained/live artifact bundle target carrier cleanup

- Changed `WORTHQueryLiveArtifactBundle` storage from
  `BTreeMap<String, WORTHQueryLiveReadResult>` to
  `BTreeMap<WORTHQueryLiveArtifactTarget, WORTHQueryLiveReadResult>`.
- Changed `WORTHQueryDerivedMaterializationBundle` storage from
  `BTreeMap<String, WORTHQueryDerivedMaterializationResult>` to
  `BTreeMap<WORTHQueryDerivedMaterializationTarget,
  WORTHQueryDerivedMaterializationResult>`.
- Converted bundle construction in live artifact reads, derived artifact
  materialization, published shared-read artifacts, retained/live projection
  tests, and retained scalar tests to insert target carriers as keys instead
  of view-name strings.
- Replaced bundle by-name lookups with carrier-based
  `read_for_target(...)` / `materialization_for_target(...)`. Binding exact-set
  validation now compares required target carriers against carrier-keyed bundle
  membership; terminal target names are projected only for digest/error text.
- Refreshed compile-fail expectations so public consumers now see removed
  by-name methods, not merely private string helpers.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  retained_live --lib`, green focused `cargo test -p worth-query
  projection_consumption --lib`, green focused `cargo test -p worth-query
  runtime::tests::live_artifacts --lib`, green focused `cargo test -p
  worth-query retained_scalar --lib`, green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, line-count checks showing touched bundle
  and binding files under the 400-line cap, and a targeted scan showing the old
  bundle map/by-name authority shapes removed from production Query.

## Query backend live lookup target carrier cleanup

- Changed `WORTHQueryRuntimeBackend` and `WORTHQueryRuntimeSourceAdapter` so
  live row reads and patch draining accept `WORTHQueryLiveArtifactTarget`
  carriers instead of terminal view-name `&str` values.
- Added `WORTHQueryLiveArtifactTarget::from_subscription_installation(...)`
  so ordinary live reads, patch drains, preview program reads, and retained
  live refreshes can pass the installed subscription proof through the backend
  lookup path. Materialized read-composition views use the same target carrier
  without an attached installation because they are backend-declared internal
  read views rather than runtime subscriptions.
- Converted bridge-backed backend forwarding, in-memory test backend,
  stateful bridge runtime support, public bridge runtime support, certification
  fixtures, transcript fixtures, and source-adapter fixtures to the target
  carrier contract. Source adapters may still project
  `terminal_view_name_projection()` internally as the hard external lookup
  boundary, but runtime callers no longer hand anonymous view-name strings to
  backend/source-adapter live lookup APIs.
- Moved the default unavailable snapshot identity helper out of
  `runtime/backend/contracts.rs` so the touched trait contract file stays under
  the 400-line cap.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  runtime::tests::live --lib`, green focused `cargo test -p worth-query
  retained_live --lib`, green focused `cargo test -p worth-query
  read_composition --lib`, green `cargo test -p worth-query --test
  phase_boundaries_bridge_truth_identity_compile_fail`, and a targeted scan
  showing the old `live_entities(&str)` / `drain_live_patches(&str)` method
  names gone from current Query source and tests.

## Query relational projection fact path ingress cleanup

- Removed the generic `ProjectionMaterializedField::from_external_boundary(...)`
  constructor from relational row-set extraction. Relational materialized rows
  now call `from_relational_projected_aspect_key(...)`, which accepts the
  `AspectKey` carrier retained by `RelationalProjectedAspectValueSet` instead
  of accepting arbitrary field-path text.
- Renamed the dotted lowering helper to
  `projection_fact_field_path_from_relational_projected_aspect_key(...)` and
  confined the remaining split to this named compatibility boundary. This
  documents the actual upstream contract: relational grouped truth currently
  exposes projected snapshot reads as foundational `AspectKey` values whose
  labels encode the projected field path.
- Bridge row-set extraction remains on
  `BridgeMaterializedFieldProjection` metadata and query-owned read/live rows
  remain on native entity field paths; neither path uses the relational
  compatibility helper.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  projection_consumption --lib`, a line-count check showing touched files under
  the 400-line cap, and a targeted scan showing the old
  `from_external_boundary` / `projection_fact_field_path_from_external_boundary`
  names removed from projection consumption.

## Query projection fact field-path segment carrier cleanup

- Changed `projection_fact_field_path_from_segments(...)` so it now accepts
  `FieldKey` carriers instead of raw string segments. This turns the helper
  from an unchecked string-segment constructor into a native foundational
  field-key assembly point.
- Mechanically converted the projection-consumption, domain-capability,
  intent-admission, retained/live, shared-read, hostile certification, and
  certification fixture call sites to construct foundational `FieldKey`
  segments before building a `ProjectionFactFieldPath`.
- The public/native `ProjectionFactFieldPath::from_canonical_field_path(...)`
  path remains the direct proof-carrying constructor. Bridge row extraction
  still uses bridge projection metadata, query/read/live extraction still uses
  native entity field paths, and relational row extraction is still fenced
  behind the named relational projected-aspect-key compatibility boundary.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  projection_consumption --lib`, and line-count checks showing touched files
  under the 400-line cap.

## Query predicate admission native key cleanup

- Added crate-internal authoring predicate constructors that accept
  `AspectFieldKey` carriers directly, so declarative predicate lowering no
  longer has to project an already-admitted field back into aspect/field text
  and reparse it.
- Changed declarative predicate projection normalization, duplicate field
  detection, and canonical predicate lowering to flow through
  `source_field_key()` instead of `filter.aspect()` / `filter.field()`.
- Removed the crate-local `aspect()` / `field()` aliases from
  `DeclarativeEqualityFilter`, `DeclarativeIntegerComparisonFilter`,
  `DeclarativeStringContainsFilter`, `DeclarativeSetMembershipFilter`,
  `DeclarativePresenceFilter`, and the aggregate
  `DeclarativePredicateFilter`, so future predicate consumers must use the
  proof carrier.
- Converted graph-read boolean predicate admission/evidence and schema
  reference admission to key admitted predicate lookups and schema rows by
  native `AspectKey` / `FieldKey` carriers. Terminal strings remain only for
  denial text.
- Refreshed the predicate string-alias compile-fail fixture from "private
  method" to "method not found", which is the stronger Law 41 fence: the
  lower-authority reconstruction API is gone instead of merely hidden outside
  the crate.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`, green focused `cargo test -p worth-query
  graph_read_access --lib`, and green focused `cargo test -p worth-query
  read_composition --lib`.

## Query AspectFieldKey and declarative carrier enforcement

- Renamed the neutral `AspectFieldKey::new(...)` constructor to
  `AspectFieldKey::from_authoring_parts(...)`, added
  `AspectFieldKey::from_native_keys(...)` as the named foundational
  `AspectKey` / `FieldKey` bridge, and added
  `aspect_field_key_neutral_constructor_removed.rs` so the old neutral
  constructor cannot return silently.
- Added native field-carrier constructors for projection, result-shape,
  ordering, and predicate authoring selectors. Declarative canonicalization and
  read-composition lowering now pass `AspectFieldKey` through
  `source_field_key()` / `target_field_key()` instead of decomposing fields
  into `(aspect, field)` strings and rebuilding them.
- Removed crate-local declarative raw helper roots for equality, integer,
  string-contains, set-membership, presence, ordering, projection, branch
  compare, and writeback string aliases where the compiler showed they were no
  longer needed. The surviving `from_authoring_parts` calls are now named
  authoring/test ingress points rather than hidden reparse steps in the
  lowering path.
- Refreshed compile-fail fixtures so the strongest fences now report "method
  not found" for removed aliases on `OrderingSelector`,
  `PredicateSelector`, `DeclarativeProjectionField`,
  `DeclarativeBranchCompareFieldDelta`, and
  `DeclarativeWritebackChange`.
- Verification covered green `cargo check -p worth-query --tests` and green
  `cargo test -p worth-query --test aspect_native_query_compile_fail`.
  `rustfmt` on the whole crate still intermittently hit Windows mapped-section
  locks, so touched-file formatting should be retried before closeout if the
  lock clears.

## Query mutation seed touch evidence carrier cleanup

- Changed runtime mutation lowering digest-row helpers to accept
  `WORTHQueryAspectTouch` carriers instead of caller-rendered admitted touch
  digest strings. Terminal text projection now happens inside the final
  evidence-row helper, not at every mutation-lowering call site.
- Changed authoritative mutation seed identity construction so declared aspect
  operations lower as structured evidence identities with operation kind plus
  admitted aspect touch evidence. The seed no longer stores operations as
  anonymous `kind:touch_digest` value strings.
- This is a narrow Law 41 improvement: the mutation seed/lowering callers must
  keep the native touch proof until the explicitly terminal evidence field.
  The remaining `admitted_touch_digest_part()` calls in the touched files are
  terminal evidence projection points rather than precomputed control values.
- Verification covered touched-file `rustfmt`, green `cargo check -p
  worth-query --tests`, green focused `cargo test -p worth-query
  intent_admission --lib`, and green focused `cargo test -p worth-query
  mutation --lib`. Both touched source files stayed under the 400-line cap.

## Query workspace declaration field carrier cleanup

- Changed workspace live declaration assembly so schema fields are retained as
  `AspectFieldKey` carriers instead of `(String, String)` label pairs.
- Live projection and ordering construction now use
  `DeclarativeProjectionField::new(...)` with the retained native field key
  instead of decomposing to aspect/field strings and calling
  `from_authoring_parts(...)` again.
- Removed the now-dead `schema_field_view(...)` string-to-name helper from the
  workspace declaration schema module. Schema relations still parse relation
  authoring text because relation names are not aspect-field authority.
- Verification covered touched-file `rustfmt`, green `cargo check -p
  worth-query --tests`, and green focused `cargo test -p worth-query
  workspace_declaration --lib`.

## Query canonical validation aspect-field carrier cleanup

- Changed canonical projection, predicate, ordering, and result-shape entries
  to retain `AspectFieldKey` carriers instead of separate `AspectName` /
  `FieldName` fields.
- Authorized projection, schema validation, predicate normalization, ordering
  validation, result-shape validation, and validated-artifact construction now
  consume the retained field carrier directly. Terminal labels are projected
  only for diagnostics, digest text, and event rows.
- Removed the obsolete `result_shape::source_projection_key(...)` helper
  because result-shape fields now keep their source `AspectFieldKey` proof.
- The deliberate `cargo check -p worth-query --tests` red exposed 65 old
  split-field consumers across validation and predicate-state normalization;
  those consumers now follow the native carrier boundary.
- Verification covered touched-file `rustfmt`, green `cargo check -p
  worth-query --tests`, green focused `cargo test -p worth-query
  canonicalization --lib`, green focused `cargo test -p worth-query
  validation --lib`, and green focused `cargo test -p worth-query
  authorized_projection --lib`. Touched source files stayed under the 400-line
  cap.

## Query mutation delta collection identity carrier cleanup

- Changed `WORTHQueryMutationDelta` to retain
  `WORTHQueryMutationTargetCollectionIdentity` instead of a raw collection
  `String`.
- Added a native collection-identity constructor for mutation deltas and routed
  memory-workspace production receipt construction through it. The existing
  collection text accessor now projects from the retained identity for terminal
  compatibility and existing string-keyed indexes.
- Live subscription maintenance evidence now consumes
  `delta.target_collection_identity().evidence_identity()` directly instead
  of rebuilding a collection identity from `delta.collection` text.
- The deliberate `cargo check -p worth-query --tests` red exposed direct
  `delta.collection` field consumers in production routing, inspection,
  preview relevance, signal invalidation, and critical harness adapters; those
  consumers now use either the native identity or the terminal accessor.
- Verification covered touched-file `rustfmt`, green `cargo check -p
  worth-query --tests`, green focused `cargo test -p worth-query
  memory_workspace --lib`, green focused `cargo test -p worth-query mutation
  --lib`, and green focused `cargo test -p worth-query preview --lib`.
  `cargo test -p worth-query live_subscription --lib` matched no tests, so
  live subscription routing is covered here by compile-time type enforcement.
  Touched source files stayed under the 400-line cap.

## Query live subscription index target carrier cleanup

- Converted `WORTHQueryRuntime::live_subscription_index` from a raw
  view-name cache to `BTreeMap<String, BTreeSet<WORTHQueryLiveArtifactTarget>>`.
  Registration now stores the artifact target derived from the admitted
  subscription installation.
- `route_live_subscription_delivery(...)` now consumes target carriers and
  returns affected `WORTHQueryLiveArtifactTarget` values. It projects
  `view_name()` only at the runtime state-map boundary and for delivery error
  reporting.
- `route_authoritative_mutation_summary(...)` now preserves the affected live
  targets returned by delivery instead of rebuilding them from raw strings.
  Computed candidate routing still projects target carriers to view names at
  the computed dependency boundary; that is the next routing-family cleanup, not
  part of this live-index slice.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, and focused green `cargo test -p worth-query
  aspect_crud --lib`.
- This is a Law 41 routing-carrier slice: once a live subscription has an
  admitted artifact target, downstream mutation routing and write receipts keep
  that proof carrier instead of reconstructing authority from a string name.

## Query computed dependency target carrier cleanup

- Converted `WORTHQueryComputedDependencyIndex` so live dependency keys are
  `WORTHQueryLiveArtifactTarget` values and computed dependency keys/values are
  `WORTHQueryDerivedMaterializationTarget` values.
- `computed_candidate_live_views(...)` now returns live artifact targets
  directly from the live subscription index instead of projecting candidate
  view-name strings.
- `route_derived_view_patches(...)` now consumes live target candidates and
  returns affected derived materialization targets. Authoritative mutation
  routing preserves those targets for write receipts and effect routing instead
  of rebuilding them from returned strings.
- Retained upstream live-row routing still stores rows under view-name keys
  because `WORTHQueryRetainedUpstreamInputs` and `WORTHQueryDerivedView`
  declarations are still string-addressed boundaries. That is now a localized
  boundary, not the computed dependency graph's authority carrier.
- Verification covered green `cargo check -p worth-query --tests`, green
  focused `cargo test -p worth-query computed --lib`, and green focused
  `cargo test -p worth-query retained_live --lib`.
- This is a Law 41 dependency-routing slice: the computed refresh graph now
  propagates typed live and derived artifact targets until the remaining
  declaration/materialization boundary explicitly projects terminal names.

## Query effect trigger index target carrier cleanup

- Converted `WORTHQueryEffectIndex` so live trigger keys are
  `WORTHQueryLiveArtifactTarget` values and computed trigger keys are
  `WORTHQueryDerivedMaterializationTarget` values.
- Effect declaration admission now checks declared trigger names by first
  admitting them into live/derived target sets derived from runtime state,
  rather than checking raw view-name sets.
- Effect candidate selection now consumes the affected live and derived target
  slices directly. It no longer projects affected write-route targets to view
  names before asking the effect index which effects should run.
- Live trigger change collection still projects the declaration's trigger name
  into a live target to look up the target collection, because
  `WORTHQueryEffectTrigger` remains the public declaration boundary. The effect
  index and candidate-selection authority no longer use raw names as their
  routing key.
- Verification covered green `cargo check -p worth-query --tests` and green
  focused `cargo test -p worth-query effect --lib`.
- This is a Law 41 effect-routing slice: affected live/derived target proofs
  now flow into effect candidate selection without being weakened to strings.

## Query graph descriptor collection identity cleanup

- `WORTHQueryGraphTouchDescriptorRow::touches_declared_collection(...)` now
  compares native `WORTHQueryMutationTargetCollectionIdentity` carriers through
  an explicit target-collection semantic comparator instead of projecting both
  sides to `&str`.
- `WORTHQueryGraphTouchDescriptorInventory` now retains declared collection
  identity carriers and dedupes them through that native semantic comparator
  instead of storing a `BTreeSet<String>`.
- Added deterministic ordering support for
  `WORTHQueryMutationTargetCollectionIdentity` so collection identity carriers
  can participate in ordered native collections where the semantic owner wants
  that shape.
- Verification covered green `cargo check -p worth-query --tests`, green
  focused `cargo test -p worth-query graph_touch_descriptor --lib`, and green
  focused `cargo test -p worth-query graph_obligation --lib`.
- This is a Law 41 selector-root cleanup: graph obligation selection can now
  ask whether a descriptor touches a target collection through a native carrier
  comparison, while terminal `declared_collection()` text remains a reporting
  projection.

## Query same-batch symbolic collection identity cleanup

- `record_same_batch_symbolic_target(...)` now matches created deltas against
  declared target collections through
  `WORTHQueryMutationTargetCollectionIdentity::same_target_collection_as(...)`
  instead of comparing terminal collection strings.
- `resolve_same_batch_symbolic_target(...)` now validates expected versus
  resolved collection through the same native semantic comparator.
- `classify_receipt_mutation_summary(...)` now collects mutation delta
  collection identities directly and returns the retained identity when the
  receipt has one semantic target collection, instead of collecting strings and
  rebuilding a new summary identity.
- Verification covered green `cargo check -p worth-query --tests`, green
  focused `cargo test -p worth-query symbolic_reference --lib`, and green
  focused `cargo test -p worth-query batch --lib`.
- This is a Law 41 batch-target cleanup: same-batch symbolic resolution and
  mutation summary classification keep collection identity carriers through the
  batch boundary instead of re-deriving authority from collection text.

## Query backend affected live target carrier cleanup

- `WORTHQueryRuntimeBackend::affected_live_view_ids(...)` and
  `WORTHQueryRuntimeSourceAdapter::affected_live_view_ids(...)` were replaced
  by `affected_live_view_targets(...)`, returning
  `Vec<WORTHQueryLiveArtifactTarget>`.
- Runtime batch receipt rows now carry backend/source affected-live targets
  directly instead of rebuilding target carriers from terminal live-view ids.
- Bridge-backed runtime forwarding, consumer-kit backends, runtime
  stabilization harnesses, intent-admission certification fixtures, stateful
  bridge runtime support, and public bridge runtime support all implement the
  native target-returning contract.
- Source-adapter declared view names enter through the explicit
  `WORTHQueryLiveArtifactTarget::from_source_adapter_declared_view_name(...)`
  boundary; ordinary raw target constructors remain private or unavailable.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green focused
  `cargo test -p worth-query batch --lib`, green
  `cargo test -p worth-query --test graph_composition_public_bridge`, and
  green `cargo test -p worth-query --test aspect_native_query_compile_fail`.
- This is a Law 41 backend-boundary cleanup: affected live-view identity now
  crosses the backend/source-adapter boundary as artifact target proof, while
  terminal live-view id projection remains an explicit reporting/accessor
  boundary.

## Query preview target collection carrier cleanup

- `DeclarativeLiveQueryRequest` now exposes
  `target_collection_identity()` so preview relevance can retain the request's
  target collection proof instead of comparing against `request.target()` text.
- `relevant_live_aspects(...)` now compares mutation delta target collection
  identities to the request target identity with
  `same_target_collection_as(...)`.
- Preview same-batch symbolic target resolution now validates expected versus
  resolved collection identities with the same native semantic comparator
  instead of comparing `as_str()` projections.
- `WORTHQueryMutationTargetCollectionIdentity` is exported through the facade
  and exposes the semantic comparator publicly, while its raw constructor
  remains sealed. Downstream runtime adapters can therefore stay native without
  gaining an arbitrary collection-token minting route.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green
  `cargo test -p worth-query preview --lib`, green
  `cargo test -p worth-query --test public_bridge_runtime_bootstrap`, green
  `cargo test -p worth-query intent_admission --lib`, green
  `cargo test -p worth-query --test graph_composition_public_bridge`, green
  `cargo test -p worth-query --test graph_composition_public_bridge_existing`,
  green `cargo test -p worth-query --test public_submission_lane_replacements`,
  and green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`.
- A focused residue scan over `crates/worth-query/src` and
  `crates/worth-query/tests/support` found no remaining
  `live_views: BTreeMap<String, String>`,
  `request.target().to_string()` live-view target storage,
  `*collection == delta.collection()`,
  `delta.collection() == target.as_str()`,
  `delta.collection() != request.target()`, or expected/resolved collection
  `as_str()` mismatch comparisons.
- This is a Law 41 preview and adapter-root cleanup: preview relevance and
  source-adapter affected-view routing now keep collection identity carriers
  through control decisions and only project text for terminal lookup/reporting.

## Query live subscription target collection index cleanup

- `WORTHQueryRuntime::live_subscription_index` now groups subscriptions by
  `WORTHQueryMutationTargetCollectionIdentity` entries rather than a
  `BTreeMap<String, ...>` keyed by target collection text.
- Live subscription registration stores the request's native target collection
  identity and dedupes entries through
  `same_target_collection_as(...)`. Unregistration prunes native index entries
  after removing the target carrier.
- Live delivery routing and computed candidate live-view discovery now find
  candidate subscriptions by comparing mutation delta target collection
  identities to index entry identities, instead of calling
  `.get(delta.collection())`.
- Effect routing's live-trigger collection map now carries
  `WORTHQueryMutationTargetCollectionIdentity` values and compares native
  mutation delta identities when collecting trigger changes.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green `cargo test -p worth-query live
  --lib`, green `cargo test -p worth-query computed --lib`, green
  `cargo test -p worth-query effect --lib`, and green `cargo test -p
  worth-query --test aspect_native_query_compile_fail`.
- Focused residue scans found no remaining runtime
  `live_subscription_index: BTreeMap`, no
  `BTreeMap<String, BTreeSet<WORTHQueryLiveArtifactTarget>>`, no
  runtime `.get(delta.collection())`, no
  `BTreeMap<WORTHQueryLiveArtifactTarget, String>` authority map, and no
  `delta.collection() != collection` live-trigger comparison.
- This is a Law 41 routing-index cleanup: live subscription and downstream
  effect/computed routing now preserve target collection identity as a control
  carrier, while terminal collection text remains only for receipts,
  inspection, and explicit compatibility/reporting boundaries.

## Query published artifact target registry cleanup

- `WORTHQueryPublishedArtifactRegistry` now stores generation entries under
  `WORTHQueryDerivedMaterializationTarget` keys instead of
  `BTreeMap<String, WORTHQueryPublishedArtifactEntry>`.
- Shared-read publication creates a derived materialization target for each
  runtime derived view before publishing the generation. Shared-read
  consumption resolves published artifacts by that target carrier, not by a
  view-name string.
- `WORTHQueryPublishedArtifactEntry` stores its target carrier and
  `bind_shared_read_artifact(...)` receives that target directly, avoiding the
  previous `WORTHQueryDerivedMaterializationTarget::new(view_name)` rebuild at
  the binding point.
- Terminal view names remain only for public handle reporting, error text, and
  existing shared-read evidence labels.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green `cargo test -p worth-query
  shared_read --lib`, green `cargo test -p worth-query runtime_boundary --lib`,
  and green `cargo test -p worth-query --test aspect_native_query_compile_fail`.
- This is a Law 41 shared-read/materialization cleanup: published artifact
  resolution no longer treats view-name text as the registry authority key.

## Query live subscription state target key cleanup

- `WORTHQueryRuntime::live_subscriptions` now stores subscription state under
  `WORTHQueryLiveArtifactTarget` keys instead of `BTreeMap<String,
  WORTHQueryRuntimeLiveSubscriptionState>`.
- Live declaration inserts state by the installed subscription target. Delivery
  routing, time-only delivery, mixed-cause delivery, async result projection,
  live reads, preview session routing, unified inspection, runtime state
  snapshots, downstream delivery, and critical test support now construct a
  target carrier at lookup boundaries.
- `live_artifact_target_collections(...)` now reuses the native map key instead
  of reconstructing `WORTHQueryLiveArtifactTarget` from a string key.
- Existing public APIs that accept view handles or view-name text remain as
  ergonomic/terminal boundaries; the runtime state table itself is no longer a
  raw-name authority map.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green `cargo test -p worth-query live
  --lib`, green `cargo test -p worth-query shared_read --lib`, green
  `cargo test -p worth-query runtime_boundary --lib`, and green `cargo test -p
  worth-query --test aspect_native_query_compile_fail`.
- Focused scans found no remaining runtime
  `live_subscriptions: BTreeMap<String, ...>`,
  `BTreeMap<String, WORTHQueryRuntimeLiveSubscriptionState>`,
  `BTreeMap<String, WORTHQueryPublishedArtifactEntry>`,
  string-keyed published-artifact entry map, or
  `request.target().to_string()` production runtime residue.
- This is a Law 41 live-state cleanup: runtime subscription state is keyed by
  live artifact target proof, with name text projected only at existing
  declaration, handle, inspection, and error-reporting boundaries.

## Query computed derived target authority cleanup

- `WORTHQueryRuntime::derived_views` now stores computed runtime state under
  `WORTHQueryDerivedMaterializationTarget` keys instead of view-name strings.
- Derived-view admission, cycle detection, topological ordering, retained
  upstream discovery, patch routing, shared-read publication, effect-trigger
  lookup, preview binding lookup, runtime read drains, and inspection
  materialization lookup now construct or receive derived materialization target
  carriers at the boundary where text enters.
- `WORTHQueryRetainedUpstreamInputs` now stores live rows by
  `WORTHQueryLiveArtifactTarget` and computed rows by
  `WORTHQueryDerivedMaterializationTarget`. Runtime authoritative mutation
  routing no longer projects affected live targets back into
  `BTreeMap<String, rows>` before refreshing computed materializations.
- Public handle/declaration helpers still provide ergonomic access, but raw
  view-name lookup and raw construction remain outside the public surface and
  are enforced by the aspect-native compile-fail suite.
- Verification covered green `cargo fmt -p worth-query`, green
  `cargo check -p worth-query --tests`, green focused `cargo test -p
  worth-query computed --lib`, green focused `cargo test -p worth-query effect
  --lib`, green focused `cargo test -p worth-query shared_read --lib`, green
  focused `cargo test -p worth-query runtime_boundary --lib`, and green `cargo
  test -p worth-query --test aspect_native_query_compile_fail`.
- Focused scans found no remaining runtime
  `BTreeMap<String, WORTHQueryDerivedViewRuntime>`, raw-name retained upstream
  input maps, or old `(String, Vec<WORTHQueryEntity>)` retained-upstream test
  fixture construction. Production `serde_json` under `crates/worth-query/src`
  remains confined to the two support terminal JSON codecs.
- This is a strong Law 41 materialization-root cleanup: computed refresh and
  downstream consumers now flow through typed target proof carriers instead of
  treating terminal view-name text as the runtime authority key.

## Query effect runtime target authority cleanup

- Added runtime-local `WORTHQueryEffectTarget` as the authority carrier for
  effect runtime registration and lookup.
- `WORTHQueryRuntime::effects` now stores effect runtime state under
  `WORTHQueryEffectTarget` keys instead of `BTreeMap<String,
  WORTHQueryEffectRuntime>`.
- `WORTHQueryEffectIndex` now indexes live/computed trigger candidates to
  `WORTHQueryEffectTarget` sets instead of raw effect-name strings. Effect
  routing therefore resolves candidates by target carrier after live/computed
  trigger matching.
- Public handle names, declaration names, delivery labels, receipt labels, and
  error text still expose ordinary effect-name text as ergonomic/terminal
  reporting boundaries. Runtime consumers lift those names into
  `WORTHQueryEffectTarget` at lookup boundaries for delivery draining,
  inspection, pending write-intent admission, pending delivery removal, and
  effect-triggered intent execution.
- The phase-four stale pending-delivery test now removes pending work through
  the effect target carrier instead of reusing the public handle name as the
  runtime table key.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, green focused `cargo test -p worth-query effect
  --lib`, green focused `cargo test -p worth-query intent_admission --lib`,
  and green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`.
- Focused scans found no remaining runtime `BTreeMap<String,
  WORTHQueryEffectRuntime>`, `effects: BTreeMap<String, ...>`, effect trigger
  indexes storing `BTreeSet<String>`, or unlifted `self.effects.get(...)` /
  `get_mut(...)` lookup boundaries. Production `serde_json` under
  `crates/worth-query/src` remains confined to the two support terminal JSON
  codecs.
- This is a Law 41 effect-routing cleanup: after effect declaration admission,
  runtime effect state and trigger routing use a typed effect target proof
  carrier instead of treating terminal effect-name text as registry authority.

## Query same-batch symbolic target key cleanup

- Added runtime-local `WORTHQuerySameBatchSymbolicTargetKey` as the authority
  key for same-batch symbolic target planning and resolution.
- Authoritative batch write routing now stores planned and resolved
  same-batch symbolic targets under `WORTHQuerySameBatchSymbolicTargetKey`
  instead of `BTreeMap<String, WORTHQuerySameBatchSymbolicTarget>`.
- Backend atomic-batch replay uses the same typed key while rebuilding
  symbolic aspect resolution evidence from deferred receipts, so the deferred
  path no longer reintroduces a raw-symbol map.
- Preview batch admission and preview write staging now use the same typed key
  for planned/resolved symbolic targets. Preview denial text still projects the
  public symbol string for human-readable diagnostics.
- Same-batch symbolic target resolution now constructs the key from
  `WORTHQuerySymbolicTargetReference` at the lookup boundary instead of using
  `reference.symbol()` directly as map authority.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check -p
  worth-query --tests`, green focused `cargo test -p worth-query
  symbolic_reference --lib`, green focused `cargo test -p worth-query preview
  --lib`, green focused `cargo test -p worth-query batch --lib`, and green
  `cargo test -p worth-query --test aspect_native_query_compile_fail`.
- Focused scans found no remaining production
  `BTreeMap<String, WORTHQuerySameBatchSymbolicTarget>`, no same-batch
  symbolic target map initialized as `BTreeMap::<String, ...>`, and no
  `symbolic_targets.get(reference.symbol())` lookup. Production `serde_json`
  under `crates/worth-query/src` remains confined to the two support terminal
  JSON codecs.
- This is a Law 41 mutation-authority cleanup: once a symbolic target
  reference is admitted, same-batch target resolution flows through a typed
  symbolic target key carrier instead of treating the terminal symbol string as
  the batch authority key.

## Query graph-composition symbolic declaration cleanup

- `WORTHQueryGraphCompositionBuilder` now tracks declared graph-composition
  symbols with `WORTHQueryMutationSymbolIdentity` instead of
  `BTreeSet<String>`.
- Graph symbolic entity and relation declaration commands now pass the already
  admitted `WORTHQuerySymbolicTargetReference` into command construction
  through `build_insert_symbolic_reference(...)` instead of projecting
  `reference.symbol().to_string()` and rebuilding the proof.
- The raw `WORTHQueryAspectMutationBuilder::build_insert_symbolic(...)` entry
  point was removed. Public batch authoring still accepts ergonomic symbol text
  at `WORTHQueryMutationBatchBuilder::insert_symbolic(...)`, but that boundary
  now lowers text into `WORTHQuerySymbolicTargetReference` before command
  construction.
- Symbolic insert command construction validates a reference's retained target
  collection identity against the insert collection identity, so a proof
  admitted for one collection cannot be silently attached to another.
- Added aspect-native compile-fail coverage proving facade callers cannot use
  the old raw symbolic insert builder method.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  graph_composition --lib`, green focused `cargo test -p worth-query
  symbolic_reference --lib`, green focused `cargo test -p worth-query batch
  --lib`, and green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`.
- Focused scans found no remaining production
  `declared_symbols: BTreeSet<String>`, no
  `reference.symbol().to_string()` symbolic insert command construction, and no
  raw `build_insert_symbolic(...)` method. Production `serde_json` under
  `crates/worth-query/src` remains confined to the two support terminal JSON
  codecs.
- This is a Law 41 symbolic-declaration cleanup: public authoring text is
  admitted once into symbolic reference proof, graph composition and batch
  command construction consume that proof, and terminal symbol text remains
  only for reporting/program evidence.

## Query materialized read target registry cleanup

- `WORTHQueryRuntime::materialized_read_views` now stores read materialization
  registrations under `WORTHQueryLiveArtifactTarget` keys instead of
  `BTreeMap<String, DeclarativeLiveQueryRequest>`.
- Runtime read materialization still derives the backend live-view declaration
  name from the read graph digest, but immediately lifts that name into a live
  artifact target before consulting the runtime cache or reading backend rows.
- Materialized-read cache collision checks and registration now consume the
  live artifact target carrier. Terminal view-name text remains only at the
  backend live-declaration boundary, which still requires a declared view name.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  read_composition --lib`, and green focused `cargo test -p worth-query
  materialized_read --lib`.
- Focused scans found no remaining production
  `materialized_read_views: BTreeMap<String, ...>`, no materialized-read cache
  lookup by `view_name`, no old materialized-read name-availability helper, and
  production `serde_json` under `crates/worth-query/src` remains confined to
  the two support terminal JSON codecs.
- This is a Law 41 read/materialization cleanup: once a read graph has a
  generated materialized live-view identity, runtime cache authority flows
  through the typed live artifact target instead of treating generated view-name
  text as the registry key.

## Query program registry identity cleanup

- Added sealed runtime surface carriers
  `WORTHQueryProgramInstallationIdentity` and `WORTHQueryProgramRunIdentity`
  for installed-program and retained-run-trace authority.
- `WORTHQueryRuntime::installed_programs` now stores installed programs under
  `WORTHQueryProgramInstallationIdentity` keys instead of raw program-id text.
  Public `WORTHQueryInstalledProgram::program_id()` remains a terminal display
  accessor, while `WORTHQueryInstalledOperation` retains the installation
  identity internally.
- `WORTHQueryRuntime::run_traces` now stores traces under
  `WORTHQueryProgramRunIdentity` keys instead of raw run-id text. Normal
  runtime execution and preview-session execution both allocate the run identity
  once, retain traces by that carrier, and expose `WORTHQueryRunReceipt::run_id()`
  only as a terminal display accessor.
- Added aspect-native compile-fail coverage proving facade callers cannot
  construct installed operations or run receipts by supplying raw `program_id`
  and `run_id` strings.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query program
  --lib`, green focused `cargo test -p worth-query preview --lib`, and green
  `cargo test -p worth-query --test aspect_native_query_compile_fail`.
- Focused scans found no remaining `installed_programs: BTreeMap<String, ...>`,
  no `run_traces: BTreeMap<String, ...>`, no installed-program lookup by
  `operation.program_id`, no run-trace lookup by `run.run_id()`, and no public
  struct-literal construction path using raw `program_id` or `run_id` fields.
- This is a Law 41 runtime-program cleanup: after a program is installed and a
  run is allocated, runtime lookup and trace retention flow through typed proof
  carriers; raw program/run text remains only for terminal display,
  diagnostics, and generated trace labels.

## Query backend live-view target map cleanup

- `WORTHQueryInMemoryTestBackend::live_views` now stores declared live views
  under `WORTHQueryLiveArtifactTarget` keys instead of raw view-name strings.
- Backend live reads and affected-live-view routing consume the retained target
  carrier directly; terminal view-name text remains only at the backend
  declaration/handle/activation boundary where the backend trait still receives
  names.
- The src-hosted runtime test source adapter and public bridge runtime support
  state now use the same target-keyed map, so critical adapter tests no longer
  preserve a raw live-view-name authority table beside the production
  consumer-kit backend.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  test_backend --lib`, green `cargo test -p worth-query --test
  public_bridge_runtime_bootstrap`, and green `cargo test -p worth-query
  --test aspect_native_query_compile_fail`.
- Focused scans found no remaining `live_views: BTreeMap<String, ...>` in the
  consumer-kit test backend, runtime source adapter support, or public bridge
  runtime support; no live entity lookup by
  `target.terminal_view_name_projection()` in those roots; and no affected-view
  projection from raw `(name, _)` map keys.
- This is a Law 41 backend/source-adapter cleanup: once a live declaration has
  been admitted, the backend support maps retain a live artifact target proof
  carrier rather than using terminal view-name text as lookup authority.

## Query read materialized row identity index cleanup

- Added private `WORTHQueryReadMaterializedRowIdentity` as the row-identity key
  carrier for read-composition materialization traversal and shared-neighborhood
  selection.
- `row_index(...)` now returns
  `BTreeMap<WORTHQueryReadMaterializedRowIdentity, WORTHQueryEntity>` instead
  of `BTreeMap<String, WORTHQueryEntity>`.
- Anchor identities, relation targets, traversal cursors, selected-row maps,
  and identity ordering now carry the private row identity type instead of
  moving anonymous identity strings through control flow.
- The private carrier deliberately exposes no string accessor; row identity text
  is read from native row fields at the extraction boundary and is not projected
  again for downstream selection.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  read_composition --lib`, and green focused `cargo test -p worth-query
  materialized_read --lib`.
- Focused scans found no remaining read-composition
  `row_index: &BTreeMap<String, WORTHQueryEntity>`, no `row_index(...) ->
  BTreeMap<String, WORTHQueryEntity>`, no `row_identity_label(...) ->
  Option<String>`, and no traversal cursor reassignment through
  `next_identity.to_string()`.
- This is a Law 41 read-materialization cleanup: once a materialized native row
  exposes an identity field, traversal/indexing uses a phase-local proof carrier
  instead of treating the scalar string as the read engine's authority key.

## Query causal decision trace lookup cleanup

- Added private `CausalDecisionTraceLookupKey` for the retained causal decision
  trace index.
- `CausalDecisionTraceIndex::lookup` now stores
  `HashMap<CausalDecisionTraceLookupKey, usize>` instead of
  `HashMap<String, usize>`.
- Trace rows still expose `key()` as terminal/reporting text, and public
  `row_for_key(&str)` remains the ergonomic lookup boundary, but the retained
  index immediately lifts that key text into the private lookup carrier before
  consulting the map.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query causal
  --lib`, and green focused `cargo test -p worth-query inspection --lib`.
- Focused scans found no remaining `lookup: HashMap<String, ...>` or
  production `HashMap<String, ...>` residue in the causal admission trace root.
- This is a Law 41 inspection-evidence cleanup: retained causal trace lookup no
  longer treats raw row-key text as the stored index authority, while terminal
  key strings remain available only at the public query/reporting boundary.

## Query program operation lookup cleanup

- Added `WORTHQueryProgramOperationIdentity` as the program operation lookup
  carrier.
- `WORTHQueryProgram::operations` now stores operations under
  `BTreeMap<WORTHQueryProgramOperationIdentity, WORTHQueryOperation>` instead
  of `BTreeMap<String, WORTHQueryOperation>`.
- `WORTHQueryInstalledOperation` now retains the operation identity carrier
  instead of raw `operation_id` text. Runtime execution and preview execution
  consume that carrier for program lookup, trace construction, error text, and
  run-id display projection.
- Existing public ergonomic boundaries still accept operation id text when a
  caller asks an installed program for an operation, and trace/error reporting
  still exposes terminal operation id strings.
- Strengthened the aspect-native compile-fail fixture for program runtime
  receipts: facade callers now fail when attempting to construct installed
  operations with raw `program_id`, raw `operation_id`, or run receipts with raw
  `run_id`.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query program
  --lib`, and green `cargo test -p worth-query --test
  aspect_native_query_compile_fail`.
- Focused scans found no remaining `operations: BTreeMap<String, ...>`, no
  runtime `operation.operation_id` lookup/display consumers, and no raw public
  installed-operation struct-literal path for `operation_id`.
- This is a Law 41 program-execution cleanup: after an operation is selected
  from an installed program, runtime execution carries a typed operation proof
  rather than a free operation-id string.

## Query source-hosted live-view target cleanup

- Converted the runtime API stabilization transcript source adapter,
  intent-admission certification runtime source adapter, and shared stateful
  bridge runtime test support from
  `BTreeMap<String, WORTHQueryMutationTargetCollectionIdentity>` live-view
  lookup tables to
  `BTreeMap<WORTHQueryLiveArtifactTarget, WORTHQueryMutationTargetCollectionIdentity>`.
- Live-view declaration still accepts the public view name and returns the
  public `WORTHQueryLiveViewHandle`, but the source adapters immediately lift
  the name into `WORTHQueryLiveArtifactTarget` before retaining lookup state.
- Affected-live-view routing now returns cloned live target carriers from the
  retained map instead of reconstructing targets from raw view-name map keys.
  The stateful bridge runtime live-read path now looks up by
  `WORTHQueryLiveArtifactTarget` directly instead of using
  `target.terminal_view_name_projection()` as an internal key.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  runtime_api_stabilization --lib`, green focused `cargo test -p worth-query
  intent_admission --lib`, and green focused `cargo test -p worth-query --test
  public_bridge_runtime_bootstrap`.
- Focused scans found no remaining
  `live_views: BTreeMap<String, WORTHQueryMutationTargetCollectionIdentity>`,
  no stateful bridge lookup by `target.terminal_view_name_projection()`, and no
  affected-view reconstruction from `(name, _)` live-view map keys in the
  converted source-hosted runtime roots.
- This is a Law 41 runtime/source-adapter cleanup: after live-view declaration,
  transcript, certification, and critical stateful bridge paths retain the live
  artifact target proof carrier instead of treating terminal view-name text as
  the routing table authority.

## Query bridge readmission fixture identity cleanup

- Converted the phase-six lower-runtime readmission certification fixture from
  terminal-string bridge identity maps to runtime-bridge identity carriers:
  `committed_patches` is now keyed by `TruthCommitIdentity`, `branch_heads`
  maps `TruthBranchIdentity` to `TruthCommitIdentity`, and `snapshots` is keyed
  by `TruthSnapshotIdentity`.
- The fixture now stores committed patch, branch-head, and snapshot authority
  using the bridge identities already present at the source boundary instead of
  projecting them through `bridge_admission_evidence().terminal_projection_for_reporting()`
  for internal lookup.
- Added the mechanical guard
  `bridge_readmission_fixtures_do_not_key_authority_by_terminal_strings` to the
  existing support snapshot runtime-boundary residue tests. It fails if the
  readmission fixture reintroduces `BTreeMap<String, ...>` authority maps for
  committed patches, branch heads, or snapshots, or if it stores lookup keys via
  terminal bridge evidence projection.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  bridge_readmission_fixtures_do_not_key_authority_by_terminal_strings --lib`,
  and green focused `cargo test -p worth-query lower_runtime_routing --lib`.
- Focused scans found no remaining `committed_patches: BTreeMap<String, ...>`,
  `branch_heads: BTreeMap<String, ...>`, `snapshots: BTreeMap<String, ...>`, or
  terminal bridge-evidence projection lookup in the readmission fixture.
- This is a Law 41 bridge-readmission cleanup: Query certification fixtures now
  retain the bridge-provided identity proof carriers for readmission authority
  instead of treating rendered identity strings as the bridge source of truth.

## Query production string-map residue classification guard

- Added the mechanical guard
  `production_string_map_residue_is_classified_grammar_or_reporting_only` to
  the support snapshot runtime-boundary residue tests.
- The guard recursively scans production Query source for `BTreeMap<String` and
  `HashMap<String` and fails unless the complete file-level residue set is:
  `composition/templates/instantiation.rs`,
  `consumer_kit/evidence_report/report.rs`,
  `consumer_kit/evidence_report_adoption/syntax.rs`, `program.rs`, and
  `runtime/intent/input.rs`.
- The classified survivors are grammar/reporting roots: template slot/binding
  grammar, evidence report field indexes, evidence report adoption symbol
  counters, program value/value-expression object grammar and bound inputs, and
  intent input object grammar. They are not approved runtime authority storage
  locations for aspect truth.
- This guard complements the existing production `serde_json` confinement
  guard. Together they make new JSON/string-map authority residue a test
  failure instead of a manual scan habit.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  production_string_map_residue_is_classified_grammar_or_reporting_only
  --lib`, and green focused `cargo test -p worth-query
  production_serde_json_is_confined_to_support_terminal_codecs --lib`.
- This is a Law 41 closeout enforcement slice: Query may keep explicit string
  grammar where the domain is literally named fields, slots, or program inputs,
  but production authority roots cannot quietly add raw string-keyed maps
  without changing the guard and explaining why.

## Query graph obligation digest-string projection guard

- Audited the graph-obligation index/selection path for admitted aspect-touch
  digest strings. Matching already uses native
  `WORTHQueryGraphObligationTouchLookupKey` values in
  `GraphObligationBuckets`, including `WORTHQueryAspectTouch` and
  `WORTHQueryAspectMutationOperation` carriers, rather than rendered digest
  strings.
- Renamed the internal lookup-key string projection helper from `value()` to
  `terminal_value_projection()` and renamed the public index-entry accessor
  from `touch_key_value()` to `terminal_touch_key_value_projection()`.
- Added the mechanical guard
  `graph_obligation_index_does_not_expose_touch_digest_as_lookup_key_value` to
  the support snapshot runtime-boundary residue tests. It fails if the graph
  obligation index reintroduces a public `touch_key_value()` accessor or an
  ambiguous lookup-key `value()` helper, and it requires terminal projection
  naming for the retained evidence text.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  graph_obligation_index_does_not_expose_touch_digest_as_lookup_key_value
  --lib`, and green focused `cargo test -p worth-query graph_obligation
  --lib`.
- This is a Law 41 digest-string closeout slice: graph-obligation dispatch
  keeps native lookup authority for selection, and any rendered touch digest
  exposed from the index is explicitly terminal evidence text, not a reusable
  lookup key.

## Query write receipt touch digest projection guard

- Audited the write-receipt replay outcome helper that renders admitted aspect
  touches into journal replay evidence. The rendered touch digest text feeds
  only replay evidence construction, not routing, matching, or mutation
  authority.
- Renamed the helper from `touched_aspect_digest_parts(...)` to
  `terminal_touched_aspect_digest_projections(...)` so the API describes the
  value as terminal evidence projection rather than reusable authority parts.
- Added the mechanical guard
  `write_receipt_touch_digest_helpers_are_terminal_projections_only` to the
  support snapshot runtime-boundary residue tests. It fails if the ambiguous
  helper name returns or if the terminal-projection helper disappears.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  write_receipt_touch_digest_helpers_are_terminal_projections_only --lib`, and
  green focused `cargo test -p worth-query write_receipt --lib`.
- This is a Law 41 digest-string closeout slice: write receipt replay may
  render admitted touch digest text for terminal evidence identities, but the
  helper name and guard prevent that text from being treated as a reusable
  mutation authority part.

## Query batch receipt touch digest projection guard

- Audited the batch receipt and unified batch write inspection digest helpers
  that render admitted aspect touches into evidence identities. These helpers
  feed only batch receipt / inspection evidence construction, not routing,
  matching, or mutation authority.
- Renamed the repeated `evidence_touch_identities(...)` helpers to
  `terminal_touch_projection_identities(...)` in the batch receipt identity,
  unified batch write digest, and unified batch write digest component helpers.
- Updated batch receipt construction to call the terminal-projection helper
  explicitly for admitted touched aspects.
- Added the mechanical guard
  `batch_receipt_touch_digest_helpers_are_terminal_projections_only` to the
  support snapshot runtime-boundary residue tests. It fails if the ambiguous
  helper name returns or if any guarded batch helper file stops naming rendered
  touch digest text as terminal projection evidence.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  batch_receipt_touch_digest_helpers_are_terminal_projections_only --lib`, and
  green focused `cargo test -p worth-query batch --lib`.
- This is a Law 41 digest-string closeout slice: batch receipts may render
  admitted touch digest text for terminal evidence identities, but batch
  evidence helpers no longer present that rendered text as generic evidence
  identity authority.

## Query computed/preview/verified/effect touch digest projection guard

- Audited the remaining computed inspection, preview execution evidence,
  verified-assumption summaries, graph-composition assumption summaries, and
  effect delivery payload digest material for admitted aspect-touch digest
  rendering.
- These roots retain native `WORTHQueryAspectTouch` carriers for runtime state
  and expose rendered touch digest text only at evidence identity / terminal
  digest material boundaries.
- Renamed the effect delivery helper from `terminal_touch_digest_sequence(...)`
  to `terminal_touch_digest_projection_sequence(...)` so the helper identifies
  its output as terminal projection text rather than a reusable touch authority
  sequence.
- Added the mechanical guard
  `summary_and_effect_touch_digest_rendering_stays_terminal_evidence_only` to
  the support snapshot runtime-boundary residue tests. It fails if the effect
  helper returns to the ambiguous name or if the guarded computed, preview, or
  verified-assumption summary files render touch digests outside evidence
  identity construction.
- Verification covered green `cargo fmt -p worth-query`, green `cargo check
  -p worth-query --tests`, green focused `cargo test -p worth-query
  summary_and_effect_touch_digest_rendering_stays_terminal_evidence_only
  --lib`, green focused `cargo test -p worth-query effect --lib`, and green
  focused `cargo test -p worth-query computed --lib`.
- This is a Law 41 digest-string closeout slice: the remaining summary/effect
  roots can still render touch digests for terminal evidence, but native touch
  carriers stay authoritative for runtime behavior.

## Remaining high-impact mechanical roots

The closeout audit did not identify another production authority root that
still decides mutation, read, projection, effect, replay, or certification
truth from JSON, dotted aspect strings, or untyped string maps.

- The digest-string authority audit is closed for the currently identified
  admitted-aspect-touch roots: graph-obligation lookup/selection,
  write-receipt replay outcome, batch receipt digest helpers, computed
  inspection, preview execution evidence, verified-assumption summaries,
  graph-composition assumption summaries, and effect delivery summaries all
  either retain native carriers or are guarded as terminal evidence
  projections.
- Production `serde_json` and production `BTreeMap<String, ...>` /
  `HashMap<String, ...>` residue are guarded by exact runtime-boundary tests.
  The remaining allowed string maps are grammar/reporting surfaces: program
  object fields, intent input object fields, template slots, and report
  indexes. They are not authority storage unless a future audit finds a
  concrete control-authority role.
- Do not continue converting strings only to make broad search output empty.
  Further work should start only from a concrete production authority leak or a
  failing Law 41 guard.

## Closeout status checkpoint - 2026-06-23

- The aspect-native closeout guard cluster is green:
  `cargo test -p worth-query runtime_boundary --lib` passes all 14 guarded
  support snapshot/runtime-boundary tests, including the production
  `serde_json`, local JSON compatibility bridge, production string-map
  residue, live-target identity, and terminal touch-digest projection guards.
- `cargo check -p worth-query --tests` is green after the latest closeout
  slices.
- A full `cargo test -p worth-query` run exposed one real concentrated test
  fixture issue: declaration aspect test keys were still split at the first
  dot even though foundational aspect keys may contain dotted namespaces and
  field keys may not contain dots. The test helper now splits at the last dot,
  and the previously failing declaration bridge-routing, relational-routing,
  signal-compatibility, binding-pipeline, and signal-orchestration clusters are
  green in focused runs.
- The subsequent full-suite failures observed were stale trybuild stderr
  expectations where the compiler now reports stronger/native boundary
  failures. Refreshed targeted suites:
  `phase_boundaries_bridge_truth_identity_compile_fail`,
  `phase_boundaries_compile_fail`, and
  `phase_boundaries_domain_capabilities_compile_fail`,
  `phase_boundaries_graph_read_access_compile_fail`,
  `phase_boundaries_runtime_receipts_compile_fail`, and
  `prohibition_registry_compile_fail`.
- Final closeout verification is green: `cargo test -p worth-query` completed
  successfully after the targeted trybuild refreshes.
- Current residue scans show production JSON compatibility terms only inside
  the mechanical guard source and hostile compile-fail fixtures, and broad
  string-map residue is covered by the runtime-boundary classification guard.

## Self-check

This spec is not a JSON cleanup wishlist. It is an authority-boundary
replacement plan.

The work is complete only when Query cannot accidentally recover mutation,
read, projection, effect, or certification truth from JSON rows, dotted
strings, or arbitrary projection maps in production. Query should instead
consume ergonomic authoring inputs, lower them once into foundational
contracts, locators, field paths, values, masks, authoritative state/patch
carriers, typed effects, typed failures, and typed certification artifacts.
Anything else is presentation, external I/O, or explicitly marked legacy test
debt.

The strict closeout check is: a new engineer should not be able to infer from
current production code, current facade APIs, or current certification fixtures
that Query ever treated `serde_json::Value`, dotted aspect paths, or external
row objects as authority. If they can, the migration is not done.
