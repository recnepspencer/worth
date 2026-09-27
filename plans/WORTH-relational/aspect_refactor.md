# Aspect Refactor Rules

Rules and reference for the `worth-relational` aspect-native refactor.

The steering **goal** is separate: see `ASPECT_REFACTOR_GOAL.md` (or the goal
prompt you pin for the agent). This file does not restate that goal.

Read this document at the start of every turn before writing code.

## Immediate Foundational Read Gate

Before you plan, inspect crate structure, or touch code, you must immediately
read:

- `crates/worth-foundational/docs/aspect-contracts-values-and-authoritative-state/README.md`

Treat that README as the entry point into the authoritative aspect contract.
If the batch touches masks, struct fields, authoritative admission, patches,
compatibility lowering, digest semantics, or grouped publication behavior, read
the relevant sibling pages from that folder **before** planning the batch.

Do not postpone this reading until after you have already chosen a subsystem or
started sketching a migration shape. Foundational docs come first so the batch
plan is written from real `worth-foundational` meaning instead of remembered or
invented relational semantics.

## Core Rules

- Native aspect meaning is `worth-foundational` meaning.
- **Aspects aggressively.** If a value, failure, diagnostic, conflict, witness,
  or report field names or carries aspect keys, contract fields, patch targets,
  or validated values, it must use foundational aspect carriers and masks â€” not
  JSON bags, loose strings, or relational field maps. Human text is presentation;
  machine truth is aspect-shaped.
- `serde_json` is compatibility debt, not canonical authority.
- **One semantic core.** Never maintain relational and foundational truth
  systems side by side in the same batch.
- **Rewrite touched slices in one motion.** If you touch a slice, move that
  slice onto foundational truth, clean up the touched code, and delete the old
  relational path in the same turn, even if the change feels more like a
  rewrite than a refactor.
- Prefer **deletion** of obsolete paths over adaptation layers.
- Do not add bridge code "until later."
- Do not add new JSON-era shortcuts in freshly migrated code.
- Renaming JSON-era symbols without replacing them with foundational contracts
  is wasted work. **Replace, don't rename.**

## Rewrite Pattern

Use this as the default move whenever you touch a slice:

- replace the live relational truth path with foundational meaning at the
  choke point
- delete or bypass the old relational duplicate path in that same rewrite
- refactor any touched god files, helper buckets, tests, or local boundaries
  at the same time so the rewritten subsystem is coherent
- if a slice contains god files or weak structure, rewrite that structure as
  part of the same motion that lands aspects, even when the work feels more
  like a rewrite than a refactor
- keep pushing until the touched slice reads like a foundational subsystem,
  even if the work feels more like a rewrite than a refactor

Do not treat structure work, file splitting, helper cleanup, or test reshaping
as separate categories of work. They are expected parts of the same rewrite
that lands foundational adoption.

## Foundational Usage Rules

`worth-foundational` is not an optional helper crate. It is the **only**
authority for aspect meaning inside migrated `worth-relational` code.

Read the foundational docs README before choosing APIs. Do not invent
relational stand-ins because they feel easier in the moment.

### Aggressive aspect policy

Default stance: **aspect-native unless proven external-only.**

If touched code names an aspect key, contract, field path, validated value,
patch target, denial reason, or â€œwhat changed on the record,â€ it must be
represented with foundational aspect law â€” not a JSON object, not a generic
string map, not a relational duplicate enum â€œfor convenience.â€

This applies as hard as production truth:

- commit conflicts and `ConflictClass` payloads
- validation failures, invariant violations, witness keys
- diagnostic artifacts and structured diagnostic fields
- merge/validation/authority preparation failures
- strategy/domain rejections that cite aspect keys or field values
- traces, reports, and summaries that describe aspect truth

Rules:

- **Machine-readable failure context is aspect-shaped.** Use
  `ContractValidationDenial`, foundational locators, masked diagnostic
  projections, and typed `AspectValue` (or struct/reference/opaque carriers) for
  the values that matter. Do not use `serde_json::Value`, `json!`,
  `StructuredPayloadDocument`, or open `Map<String, â€¦>` as the canonical error
  payload.
- **Diagnostic masks are mandatory for diagnostic emission.** Select what to
  expose through `DiagnosticMask` against the contract surface. Forbidden:
  `json_fields.rs`-style â€œserialize every enum variant to JSONâ€ as the core
  diagnostic model.
- **Human strings are downstream.** Plain-language `detail` strings may exist
  for logs/UI, but they must be rendered from aspect artifacts â€” not the only
  storage of meaning. Forbidden: stuffing all structured failure data into one
  formatted string because it is easier.
- **Denials stay typed end to end.** When a foundational API returns
  `TransitionOutcome::Denied`, preserve denial category and contract/aspect
  identity through the relational boundary. Do not flatten to `"failed: {:?}"`.
- **Same-turn rewrite for failure paths.** When migrating a slice, rewrite its
  success path **and** its error/diagnostic/conflict paths in the same batch.
  Leaving JSON-era failure payloads while moving success onto foundational does
  not count as migrated.
- **Ingress-only exception unchanged.** JSON/compatibility lowering remains
  allowed only where external input enters. Error **output** inside the crate is
  not an ingress exception.

When unsure, ask: â€œCould this field be an aspect contract, mask-selected value,
or locator?â€ If yes, it must be â€” not a JSON field.

### Public surfaces you must use

| Relational job | Foundational surface | Do not substitute |
| --- | --- | --- |
| Declare aspect law | `worth_foundational::aspects()` â†’ contract builders | Relational aspect key enums, `StructuredDataContract*`, payload schema declarations |
| Carry typed values | `AspectValue`, `StructAspectValue`, reference/opaque carriers | `serde_json::Value`, canonical strings, relational scalar enums |
| Validate before authority | `validate_aspect_value` / validation front door | Ad-hoc JSON field checks, relational contract validators |
| Hold commit truth | `AuthoritativeRecordAspectState`, admission artifacts | Payload rescans, `BindingEvidence` strings, relational delta-only truth |
| Express commit diffs | `AuthoritativeRecordAspectPatch` (whole-aspect and field-level builders) | Relational patch records built directly from payload diffs |
| Apply commit diffs | patch application / state front doors on `aspects()` | Manual merge of aspect maps, compatibility-only patch assembly |
| Target reads/writes/diagnostics | `ProjectionMask`, `MutationMask`, `DiagnosticMask` | One generic â€œaspect setâ€, implicit full-record projection |
| Ingress from JSON/payload bytes | `worth_foundational::compatibility()` lowering | `structured_field_observation`, document rescans, loose object merge |
| Deny/ admit transitions | `worth_proof::TransitionOutcome` on foundational APIs | Invented `Result` errors that hide denial categories |
| Diagnostic / failure payloads | `DiagnosticMask` + masked aspect projections; typed denials | `json!`, `StructuredPayloadDocument`, variantâ†’JSON serializers |
| Conflict / witness / report fields | Aspect locators, keys, validated values, denial artifacts | Open JSON objects, stringly field bags, relational witness maps |

Import from `worth_foundational::facade` or the documented front doors. Do not
re-export foundational types under relational names unless the relational name
is a thin newtype over the same artifact with no alternate semantics.

### Required flow by layer

**Schema / registration**

- Every declared aspect binding stores a real `AspectContract` (or derives one
  through `aspects().contract()`), keyed by foundational `AspectKey`.
- Lowered plans carry the same contract through to execution; do not lower to
  relational-only keys and re-derive law later.
- Struct-shaped aspects use foundational field paths and field declarations â€”
  not JSON object field names as the authority model.

**Payload ingress (only place JSON may enter)**

- External payload/document input lowers through `compatibility()` into
  validated foundational values or admitted state fragments.
- After ingress, core code sees `AspectValue` / authoritative state â€” not
  `serde_json::Value` or relational structured documents.

**Authority / commit evaluation**

- Mutation evaluation must produce foundational patch/state meaning at the
  choke point, not relational evidence that is â€œconverted later.â€
- Build patches with foundational patch builders
  (`whole_aspect`, `field_level` as contract shape requires).
- Validate every set with `validate_aspect_value` (or equivalent validation
  front door) using the **typed value matching the contract shape** â€” not a
  stringified stand-in unless the contract is actually scalar string.
- Use `admit_authoritative_record_aspect_state` when a slice owns live
  authoritative state, not a parallel relational aspect map.

**Publication / durable artifacts**

- Durable commit/publication truth must be encoded from foundational patch/state
  artifacts â€” not from a separate compatibility-only reconstruction that
  discards the foundational patch after validation.
- Relational `PatchRecord` / envelope fields may exist only as **projection or
  transport** of foundational truth for a not-yet-migrated consumer. They must
  not be the only place patch meaning survives.
- Forbidden: construct `AuthoritativeRecordAspectPatch`, then throw it away and
  emit only `compatibility_record()` with no foundational artifact retained.

**Read / query / visibility / grouped truth**

- Project from authoritative state or canonical patch history through
  **projection masks** declared against contracts.
- Forbidden: re-parse payloads or JSON documents to â€œrecoverâ€ aspect meaning
  after commit.

**Validation / merge / diagnostics / errors**

- Invariants evaluate foundational contracts and authoritative surfaces.
- Diagnostic emission uses **diagnostic masks**, not raw payload access or JSON
  field extraction in the core evaluator.
- Invariant violations, witness keys, conflict `fields`, and diagnostic
  artifact bodies carry **aspect keys, locators, and typed values** when the
  failure is about aspect meaning.
- Delete JSON projection layers (`json_fields`, structured document error
  envelopes, enumâ†’JSON witness serializers) when rewriting a slice; replace with
  mask-driven diagnostic projections from authoritative/aspect artifacts.
- Merge and validation rejections that cite undeclared aspects, field mismatch,
  or contract denial must name the **`AspectContract` / `AspectKey` / field
  path** â€” not a free-form JSON snapshot of the record.

**Wire codecs (e.g. commit strategies)**

- Strategy canonical bytes encode foundational carriers (`AspectValue`, native
  ids) â€” not JSON object bytes.
- A custom native codec is allowed at outer wire boundaries, but it must round-
  trip the same `AspectValue` families the contract admits; do not embed
  `serde_json::Value` inside native bytes except through explicit compatibility
  lowering at the boundary.
- Forbidden: `StrategyRequestCanonicalization::JsonStableObjectOrderV1` for new
  or rewritten strategy surfaces.

### Proof and denial handling

- Foundational admit/validate/construct/apply APIs return proof-bearing outcomes.
  Handle denials explicitly; do not collapse them into generic strings.
- Do not wrap foundational artifacts in relational error types that erase denial
  kind unless the relational layer is purely presentation.

### Forbidden hybrid patterns

These are hard failures for a migrated slice, not acceptable interim states:

| Forbidden | Required instead |
| --- | --- |
| Relational + foundational contract systems in the same slice | One contract system: foundational only |
| Payload / document rescans after ingress | Authoritative state or patch artifacts |
| Stringifying all aspect values for patch sets | Typed `AspectValue` / struct/reference/opaque carriers matching contract shape |
| Foundational patch built then discarded; only relational `PatchRecord` kept | Foundational patch/state is the source; relational views are projections |
| `StructuredDataContract*` parallel to `AspectContract` | Delete relational contract vocab; use foundational struct/scalar contracts |
| `BindingEvidence` / materialized payload state as long-lived truth | Evidence lowers to patch/state once at choke point, then deleted |
| Generic masks or implicit full-record reads | Explicit projection / mutation / diagnostic masks |
| `compatibility()` inside core commit/history/query paths | `compatibility()` at ingress/egress only |
| JSON helpers (`json!`, `Value`, structured document types) in authority path | Foundational values/state/patches |
| Encode panics for unhandled `AspectValue` variants on production paths | Explicit supported-family matrix or denial at boundary |
| JSON/object error payloads (`StructuredPayloadDocument`, `json_fields`) | Diagnostic masks + typed denials + aspect locators |
| Failure paths left on JSON while success path uses foundational | Success and failure rewritten in the same slice |
| Human `detail` string as the only structured failure data | Aspect artifacts first; strings rendered from them |

### Minimum bar for a slice to count as â€œon foundationalâ€

A touched slice is not migrated until all of the following are true **in that
slice**:

1. Contracts come from `aspects()` / `AspectContract`, not relational duplicates.
2. Values entering authority are validated foundational values, not JSON fields.
3. Commit or read truth is expressed as patch/state artifacts, not payload diffs.
4. Any retained relational envelope/view is documented as projection of
   foundational truth, not a second source of meaning.
5. Old relational duplicate types and JSON authority paths in that slice are
   deleted, not bridged.
6. Failure, conflict, diagnostic, and witness paths in that slice are
   aspect-shaped â€” not JSON bags with a foundational success path bolted on.

### Doc map (read before using an API)

| Topic | Foundational doc |
| --- | --- |
| Contracts and scalar shapes | `aspect-keys-values-and-scalar-contracts.md` |
| Value carriers | `aspect-shapes-and-value-carriers.md` |
| Struct fields / paths | `struct-contracts-fields-and-field-paths.md` |
| Masks | `projection-mutation-and-diagnostic-masks.md` |
| Admission | `validation-and-authoritative-state-admission.md` |
| Patches / apply | `authoritative-patches-and-apply-flow.md` |
| JSON ingress only | `compatibility-lowering-and-json-bridges.md` |

Examples of relational surfaces that must not survive as parallel truth:

- payload/schema contract vocabularies that duplicate foundational struct
  contracts, field paths, or value typing
- relational-only aspect evaluation state that re-implements admission,
  materialization, or patch apply already present in foundational
- JSON document helpers used as authority outside compatibility lowering
- JSON/object diagnostic or error envelopes where diagnostic masks and typed
  denials could carry the same meaning

## Migration Shape (Speed First)

Migrate in **large vertical rewrites**, not horizontal sweeps.

### Default batch unit

One proof-bearing slice that moves a coherent truth path onto foundational in a
single turn â€” production, coupled surfaces, and that slice's tests together:

```
schema registration / lowering
  -> authority commit evaluation
  -> publication patch encoding
  -> tests/fixtures for that slice
```

Treat the touched slice as a **single rewrite boundary**:

- wire `worth_foundational` into the live choke point now
- delete or bypass the old relational duplicate path now
- perform any cleanup needed to make the rewritten slice coherent now

Do not split those moves across separate "prep", "cleanup", and "integration"
turns.

Pick the order that **minimizes total work**. The usual fast path:

1. Semantic spine â€” `schema` + `authority` + `publication/patch`
2. Direct consumers â€” `history`, `replay`, `transactions`
3. Read/project paths â€” `visibility`, `query`, `grouped_truth`, `inspection`
4. Policy/validation â€” `validation`, `merge`, `lineage`
5. Infrastructure â€” `storage`, `durability`, `indexes`, `snapshots`
6. Outer surfaces last â€” `presentation`, `facade.rs`, `commit_strategies`,
   `diagnostics`, `performance`, `simulation`

### Rewrite semantics

The rewrite owns the whole touched slice:

- foundational adoption
- deletion of the old truth path
- structural reshaping of the touched subsystem
- test and fixture reshaping needed to keep the rewritten slice honest

If a touched slice contains god files, broad helper buckets, or weak local
structure, refactor that structure in the same rewrite that lands aspects.
Splitting god files, renaming weak boundaries, and reorganizing helpers are
all normal parts of this pattern when they happen in the same motion as the
aspect adoption and old-path deletion.

### Anti-patterns

- Crate-wide rename passes before foundational wiring
- Building relational evaluation layers instead of foundational
  admission/state/patch flow
- Partial migration of a touched slice where structural reshaping and
  foundational adoption are separated into different turns
- "Cleanup first, integrate foundational later"
- Touching god files or broad subsystem structure without also landing real
  foundational adoption in that same vertical slice
- Horizontal edits across unrelated subsystems without a vertical truth claim

If a rewrite improves structure but does not change the slice to foundational
truth in the same turn, that work does **not** count as progress for this
refactor. After a valid rewrite, apply `code-quality-qa` to touched production
code.

## Aspect Semantics

Follow **Foundational Usage Rules** and the foundational docs. Non-negotiables:

- Contracts are law, not metadata.
- Projection, mutation, and diagnostic masks stay distinct.
- Absence, null, default, and clear keep explicit foundational semantics.
- Struct aspects use declared field paths â€” not loose JSON object behavior.
- No post-authority payload rescans anywhere in a migrated slice.
- Failures and diagnostics about aspect meaning use aspect carriers and diagnostic
  masks â€” not JSON side channels.

## Testing Rule (Refactor Override)

This refactor **overrides** the usual "test after every batch" habit from
`implementation-batch`.

- **Do not run tests** until the full planned batch for the turn is implemented.
- Expect compile breaks and widespread test failure mid-batch. Keep going.
- At the **end of the turn only**, run one focused verification pass for the
  slice you moved.
- Use a **10 minute timeout** per command.
- Do not burn turns fixing unrelated failures outside the batch unless they
  block compiling the slice under active work.

## Turn Startup (Every Turn)

1. Read the goal (`ASPECT_REFACTOR_GOAL.md` or pinned goal prompt).
2. Read this file.
3. Immediately read `crates/worth-foundational/docs/aspect-contracts-values-and-authoritative-state/README.md`.
4. Immediately read any sibling foundational pages the batch will rely on.
5. Only after that, read the `implementation-batch` skill and write a batch
   plan (template below).
6. Read the `code-quality-qa` skill.
7. Implement the batch without running tests until the batch is complete.
8. Apply `code-quality-qa` to touched production code before ending the turn.

## Subsystem Map

Reference layout under `crates/worth-relational/src`. Use to choose the next
batch â€” not as mandatory serial order.

**Spine:** `symbols`, `identity`, `payloads`, `schema`, `authority`,
`transactions`, `publication`, `logic`

**Consumers:** `history`, `replay`, `query`, `visibility`, `inspection`,
`grouped_truth`, `lineage`, `merge`, `validation`

**Infrastructure:** `storage`, `snapshots`, `durability`, `indexes`,
`capabilities`, `config`, `errors`

**Late / outer:** `presentation`, `facade.rs`, `commit_strategies`,
`diagnostics`, `performance`, `simulation`, `testing`, `tests`

## Completion Checklist

Mark complete only when the slice is on foundational truth **and** old paths are
deleted (not bridged).

- [ ] Spine: `schema` + `authority` + `publication/patch` on foundational
- [ ] `payloads` ingress-only; core no longer payload-shaped
- [ ] `transactions` + `logic` on foundational aspect truth
- [ ] `history` + `replay` without rescans
- [ ] `visibility` + `inspection` + `grouped_truth` project authoritative state
- [ ] `query` + `lineage` + `merge` + `validation` on contracts and truth
- [ ] errors, conflicts, diagnostics, and witnesses aspect-shaped (no JSON bags)
- [ ] `storage` + `durability` + `indexes` + `snapshots` without legacy seams
- [ ] `capabilities` + `config` + `errors` reconciled
- [ ] `presentation` + `facade.rs` hard-break public shape
- [ ] `commit_strategies` + `diagnostics` + `performance` + `simulation`
- [ ] `tests` mirror final topology; no JSON-compatibility residue
- [ ] Final pass: no shims, legacy branches, or old-path helper names anywhere

## Batch Plan Template (Required Each Turn)

```markdown
### Batch name
<vertical slice or subsystem>

### Foundational surfaces used
<List each API: aspects(), validate_aspect_value, admit_â€¦, patch builders, masks, compatibility() â€” and where in the slice>

### Forbidden hybrids removed
<Which rows from the forbidden-hybrid table were deleted in this slice?>

### Failure/diagnostic paths rewritten
<conflicts, validations, diagnostics, witnesses â€” now aspect-shaped how?>

### Choke point replaced this turn
<the exact relational truth path that now runs through foundational meaning>

### Relational surfaces deleted
<types/modules/paths removed outright>

### Coupled surfaces in this turn
<schema, authority, publication, tests, ...>

### Out of scope
<explicitly deferred>

### End-of-turn verification (10m timeout)
<exact cargo test/check commands â€” run only after batch is complete>
```
