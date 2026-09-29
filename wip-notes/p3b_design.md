# UI 3.17 Phase 3b: payload shaping (design, step 1)

Path prefixes: `dsl/` = `workspaces/worth-ui/crates/worth-ui-dsl/src/`,
`rt/` = `workspaces/worth-ui/crates/worth-ui-runtime/src/`. Every behavior claim
carries a citation I read; anything else is marked UNVERIFIED.

## 0. Headline conflict (decide before code)

The plan says "Compilation fails when a use site's role type differs". The DSL cannot
do that for payloads. It knows only the payload schema's identity and version
(`dsl/semantic/intent/declaration/mod.rs:17,79-99`). Field kinds live only in the runtime
registry (`rt/capability/registry/intent/payload_schema/field.rs:8-13,16-20`), and the
runtime already checks source-vs-kind at catalog preparation (`PayloadSourceKindMismatch`,
`rt/declaration/intent/denial.rs:58-63`; `require_kind`,
`rt/declaration/intent/payload_source/resolution.rs:295-311`).

**Recommendation:** split the check.
- The DSL rejects what it can know: an unknown expression, or the wrong role.
- The runtime rejects a result type that does not fit the field kind, as a typed
  catalog-preparation denial, before freeze.

Freeze prepares the expression catalog before the intent catalog
(`rt/facade/lifecycle/freeze.rs:72/82`, `:186/196`), so no generation ever activates with a
mismatched payload. The plan wording needs a coordinator ruling (Q5).

## 1. Grammar and spec

**Spelling (one canonical form, Law 8):**
- `payload <field> derived <derived-id>` adds `WorthUiIntentPayloadSource::Derived { expression }`.
- Revision kind `derived`, value = identity.

Parsing:
- It slots into the `parse_payload_source` match (`dsl/semantic/intent/declaration/parsing.rs:203-245`)
  beside the eight existing kinds (enum at `dsl/semantic/intent/payload_source.rs:7-18`;
  `revision_parts` at :115-129).
- The kind is a bare word, like the plan's use-site table. Existing payload kinds are
  hyphenated because each names a family plus a type (`projection-text`,
  `application-boolean`). An expression source takes no type suffix, because the type is
  the declaration's own.

**Boolean fields: `payload <field> condition <condition-id>`** adds `Condition { expression }`,
revision kind `condition`.
- Law 8 says booleans are conditions, and `derived` has no boolean result
  (`dsl/semantic/expression/expression_role.rs:1-9`).
- So the only spelling of a computed bool is a condition. Letting `derived` feed a bool
  would need a boolean derived result, which the law forbids.
- **Recommendation: add it in 3b** (Q1). Otherwise a Boolean payload field has no
  computed source at all, and authors get pushed into application facts.

**One source per field:**
- Already enforced at runtime by `DuplicatePayloadField`
  (`rt/declaration/intent/payload_source/resolution.rs:19-26`).
- The parser reads exactly `field kind value`. A second kind word on the same line fails as
  an unknown clause (`parsing.rs:35`, `:58-62`). UNVERIFIED: this is the exact message for a
  trailing token; a parse test pins it.
- No new rule is needed.

**Revision token and exact basis:**
- Unchanged mechanics. The token is `payload-source:{flen}:{field}:{kind}:{vlen}:{value}`
  (`payload_source.rs:95-105`).
- The declaration folds it (`dsl/semantic/intent/declaration/mod.rs:217-230`) and sorts
  sources by it (`:232-235`).
- The package exact basis folds it (`dsl/source/compile/semantic_package_exact_basis/declaration_basis.rs:236-240,252`).
- The tokens `derived` and `condition` are new, so the basis changes exactly when an
  author switches a field between sources. The expression's own identity and revision are
  already in the basis through the expression catalog (3a), so none of that is duplicated
  here.

**Rust authoring:**
- `UiAuthoredIntentPayloadSource::derived(field, identity)` / `::condition(field, identity)`
  in `rt/declaration/intent/authored_payload_source.rs` (typed constructors at :24-76,
  `into_dsl` at :86-121).
- These return `Result<_, UiIntentOperabilityContractIdentityError>`, mirroring the
  operability `condition(identity)` constructors
  (`rt/declaration/intent/operability_contract.rs:160-166,206-212,240-245`; validation at :312-321).
- UNVERIFIED: whether the existing payload constructors validate the field name the same way.
  Check this before copying the shape.

## 2. Use-site checks

**DSL (`dsl/source/compile/expression_admission/use_sites.rs`):**
- Today `validate_expression_use_sites` walks operability axes only (:14-45).
- `condition_use_denial` (:47-71) uses the two 3a codes
  (`dsl/source/compile/compile_diagnostic.rs:65-66`). A refused expression suppresses a
  cascade (:63).
- **Recommendation:** generalize it to `use_denial(consumer, site, identity, wanted)`:
  - `wanted = Condition` for operability axes and `payload … condition`;
  - `wanted = AnyDerived` for `payload … derived`.
- It takes a `site` label ("operability availability" / "payload field `x`") for the
  message. No new codes.
- The file is 72 lines, so there is room.

**Field kinds and admitted results.** This matrix is enforced at runtime in `resolve_source`,
using the result types from `expression_role.rs:4-9` and the kernel types from
`dsl/source/compile/expression_admission/operand_types.rs:26-34`.

| Payload field kind (`field.rs:8-13`) | Admits |
|---|---|
| Text | `derived text` only |
| Boolean | `condition` only |
| Unsigned64 | nothing in 3b |
| Selection | nothing (projection-selection only) |

- **Integer → Unsigned64 is denied.** Integer is signed INT64 (`operand_types.rs:26-34`;
  unsigned64 operands are UInt64 at :44-46). Admitting it would need a range check at read
  time, which is a coercion by another name.
- **Decimal and token are denied everywhere.** Token is an appearance value and must not
  become payload text (Q2). No decimal field kind exists (Q3).

**New catalog-preparation denials** go on `UiIntentCatalogPreparationDenial`
(`rt/declaration/intent/denial.rs:10`):
- `UnknownPayloadExpression { intent, field, expression }`;
- `PayloadExpressionKindMismatch { intent, field, field_kind, role }`. This one covers both
  "a derived feeding a Boolean field" and "a condition feeding a Text field", so it is one
  variant, not two.
- These mirror the existing condition denials (`denial.rs:138-147`).

## 3. Runtime

**Resolution:**
- `resolve_payload_sources(authored, fields, sources.query, sources.application_facts)`
  (`rt/declaration/intent/catalog/preparation.rs:123-128`) gains `sources.expressions`.
- The plans struct already carries it (`rt/declaration/intent/operability_contract/resolution.rs:15-19`;
  typed at `preparation.rs:27,78`).
- Resolution looks the identity up exactly like `resolve_condition` (`resolution.rs:131-162`).
- New resolved variants on `UiResolvedIntentPayloadSource`
  (`rt/declaration/intent/payload_source/resolved.rs:23-34`):
  - `DerivedText(UiResolvedIntentExpressionSource)`;
  - `Condition(UiResolvedIntentExpressionSource)`.
- `UiResolvedIntentConditionSource` (`operability_contract.rs:98-101`, identity-only
  `PartialEq` :305-311) is renamed to `UiResolvedIntentExpressionSource`. Identity-only
  equality keeps `semantic_comparison.rs:17-30` slot-agnostic, as 3a Decision 9 did for
  conditions.
- The derived `PartialEq` on the resolved enum (`resolved.rs:5,23`) then stays correct
  without hand-written code.
- The code goes in a new file `payload_source/expression_resolution.rs`, because
  `resolution.rs` is at 369 lines.

**Projection (typed read, no coercion):**
- New arms in `project` (`rt/runtime/intent/payload/projection.rs:93-128`) call a new
  module `payload/expression_input.rs`, because `projection.rs` is at 368 lines.
- The read is `view.expression(slot)` (`rt/runtime/intent/payload/input_basis/view.rs:89-97`,
  generation-filtered). `None` means Stale, matching 3a's operability mapping
  (`rt/runtime/intent/operability/basis/axis_observation.rs:155-171`).
- `Condition` reads through the record's `condition()` (`rt/runtime/expression/owner/record.rs:137-139`,
  outcome `condition()` at `outcome.rs:130-137`), giving `Ok(bool)` or `Err(withholding)`.
- `DerivedText` uses a new typed read `record.derived_text() -> Result<&str, withholding>`.
  It is built on `ExpressionValue::as_str` (`crates/worth-foundational/src/expressions/evaluation/value.rs:280-285`).
  The field's byte budget is checked before `Arc<str>` is allocated, reusing `text()`
  (`projection.rs:335-349`).
- **Owner fix, needed for "wrong type is a denial":**
  - `role_outcome` (`rt/runtime/expression/owner/evaluation.rs:152-160`) stores any Derived
    value unchecked.
  - Extend it so a Derived value whose kernel type differs from the declared result type
    records `Denied(RoleMismatch{role})`, symmetric with the Condition arm.
  - The payload read then never sees a wrong-type `Value`. `derived_text()` treats
    `as_str() == None` as unreachable-by-construction. Recommend returning the withholding
    `Denied` there rather than panicking; a test pins the owner's denial instead.

**Posture reaches the payload (Law 2):**
- One new stop on `UiIntentPayloadStop` (`rt/runtime/intent/payload/stop.rs:1-65`):
  `ExpressionWithheld { field, expression, posture: UiExpressionWithholding }`.
  - `posture` is {Denied(reason), Unavailable, Stale}, using the 3a enum
    (`rt/runtime/expression/owner/outcome.rs:34-42`).
  - No default value, and a Boolean is never set to `false`.
- The stop flows out through `Payload(stop)` (`rt/facade/entry/native_intent.rs:50`) and
  `prepare_intent_payload` (`rt/facade/entry/intent_payload.rs:4-29`).
- The inspection map must list it too: it maps payload stops exhaustively to
  `PayloadInputUnavailable` (`rt/runtime/intent/operability/standing_observation/inspection/denial.rs:8-34`).
  Carrying posture there is out of scope; the stop itself keeps it.
- Rename `UiExpressionConditionWithholding` to `UiExpressionWithholding`, because it now
  withholds derived values too (Q4, public API).

**Owner revisions:**
- `UiIntentInputOwnerRevision` (`rt/runtime/intent/payload/input_basis/owner_revision.rs:1-5`,
  {Query, Application, Draft}) gains
  `Expression(UiIntentExpressionInputRevision { field, reference: UiExpressionResultReference })`.
- The reference carries identity, slot, generation and outcome revision
  (`rt/runtime/expression/owner/record.rs:48-54`).
- It is pushed where the other owners push (`projection.rs:149,190,245,326`). The basis
  (`input_basis.rs:20-26`) gains `expression_inputs`.
- It is recorded for Stale, Unavailable and Denied reads only if the stop is not returned.
  Since every withholding returns a stop, only a successful read records.

**Currentness refuses a drifted payload:**
- `payload_inputs_are_current(mounted, application_facts, generation)`
  (`input_basis.rs:113-128`; delegate `prepared.rs:91-99`) gains `expressions`.
- It checks each `expression_inputs` reference with `is_current_result`
  (`rt/runtime/expression/owner/state.rs:97-108`): same generation, same slot record, same
  outcome revision, same identity.
- Both callers already hold `expressions`:
  - admission (`rt/runtime/intent/admission/currentness.rs:17`, check 8 → `PayloadInputChanged`
    at :214-223, execution revalidation at :82-91);
  - confirmation (`rt/runtime/intent/confirmation/validation.rs:15`, :104 →
    `PayloadInputChanged` :109).
- No new stop is needed.

**Ripple:**
- The cost struct (`input_basis.rs:1-7`) gains `expression_inputs_read`.
- `retained_owner_reference_count` (`:79-82`) counts expression inputs.
- The causal trace's exhaustive `primary_revision` (`rt/runtime/intent/causal_trace.rs:43-51`)
  and `owner_revision_digest` (tags 1-3, :53-79) gain tag 4.
- Certification uses of `UiIntentInputOwnerRevision` are refutable let-else patterns
  (`payload/ia_05.rs:125,191`, `selection_identity.rs:179`, `application_coherence.rs:140`),
  so they do not break.

## 4. Push/pull

**Strictly pull at preparation. Currentness at admission, confirmation and execution is enough.**
- Standing observation projects operability only, not payloads
  (`rt/runtime/intent/operability/standing_observation.rs:141-160`).
- The standing fact retains no payload values (`standing_fact.rs:12-21`).
- A prepared payload is checked again at every consuming step (§3).
- 3a's push path, the condition consumer index, indexes `operability().conditions()` only
  (`rt/declaration/intent/catalog/condition_consumers.rs:15-35`). Payload expression sources
  must stay out of it, or a derived change would re-observe operability for nothing.
- A test pins zero re-observations when only a payload-read derived changes.

## 5. Generation succession

Nothing beyond 3a's prepare/commit path.
- A carried prepared payload is refused by the generation checks (`currentness.rs:176-187`,
  `validation.rs:33-38`) and by `is_current_result`'s `reference.generation() == active`
  (`state.rs:97-108`).
- The evidence-only commit builds its successor identity from the successor authority
  (`rt/facade/entry/rebind_execution/evidence_only.rs:42-50`). UNVERIFIED: whether that
  prepared generation differs from the predecessor's. Both refusal paths hold either way,
  because the expression reference names the generation it was read at. A test covers it.
- The application-publication no-op requires `expression_contract_unchanged`
  (`rt/runtime/activation/application_publication.rs:70-86`). A payload source change is
  an intent-declaration change, so it is already non-no-op through the declaration
  comparison. UNVERIFIED: the declaration comparison participates in that no-op predicate.
  Confirm by test.

## 6. Tests, with one mutation per guard

Parse and basis (DSL):
- `payload_derived_source_round_trips`. Mutation: parse `derived` as `ProjectionText`.
- `payload_condition_source_round_trips`. Mutation: drop the `condition` arm.
- `payload_source_trailing_kind_is_refused`.
- `payload_expression_source_changes_exact_basis`. Mutation: `revision_parts` returns `""`
  as the value for Derived.

Use sites (DSL):
- `payload_derived_unknown_expression_is_refused`. Code `UnknownExpressionUseSite`.
  Mutation: return `None` for the missing case.
- `payload_derived_reading_condition_is_role_mismatch`. Mutation: the derived arm accepts
  Condition.
- `payload_condition_reading_derived_is_role_mismatch`.
- `payload_use_site_of_refused_expression_is_silent`. Mutation: delete the skip at `:63`.

Catalog preparation (runtime):
- `derived_integer_into_unsigned64_is_kind_mismatch`. Mutation: admit Integer for Unsigned64.
- `derived_token_into_text_is_kind_mismatch`. Mutation: admit Token for Text.
- `condition_into_text_field_is_kind_mismatch`.
- `unknown_payload_expression_is_denied`.
- `payload_expression_declaration_compares_by_identity`. Mutation: derive `PartialEq` over
  the slot.

Projection:
- `derived_text_projects_value`.
- `condition_projects_boolean`.
- `derived_text_over_budget_is_refused_before_allocation`. Mutation: allocate before the
  budget check; the cost counter asserts.
- `withheld_expression_stops_with_posture`, parameterized over Denied, Unavailable and
  Stale; asserts the posture is carried. Mutation: map Unavailable to Stale.
- `false_condition_is_not_withheld_and_withheld_is_not_false`. Mutation: `unwrap_or(false)`.
- `derived_value_of_wrong_kernel_type_is_denied_by_owner`. Mutation: revert the
  `role_outcome` Derived check.

Currentness:
- `admission_refuses_payload_after_derived_recompute`, returning `PayloadInputChanged`.
  Mutation: drop the expression clause in `payload_inputs_are_current`.
- `confirmation_refuses_payload_after_derived_recompute`.
- `execution_revalidation_refuses_drifted_payload`.
- `unchanged_outcome_revision_keeps_payload_current`. Mutation: compare generation only.
- `payload_across_rebind_is_refused`, covering both framed and evidence-only.

Push:
- `payload_only_derived_change_does_not_reobserve_operability`. Mutation: index payload
  sources in `condition_consumers.rs`.

Ripple:
- `causal_trace_digests_expression_owner_revision`. Mutation: reuse tag 3.
- `expression_inputs_counted_in_cost_and_retained_references`.

## 7. Risks and conflicts

- **Plan conflict:** compile-time type matching is impossible (§0, Q5).
- **File size:**
  - `payload_source/resolution.rs` is 369 lines and `payload/projection.rs` is 368; new
    code goes in new sibling files.
  - `operability_contract.rs` is 322 lines; the rename is line-neutral.
  - `input_basis.rs` is 216 lines and `causal_trace.rs` is 112, both fine.
- **Public API changes:**
  - new variants on `UiIntentPayloadStop` (`stop.rs:1-65`), `UiIntentInputOwnerRevision`
    (`owner_revision.rs:1-5`) and `UiIntentCatalogPreparationDenial` (`denial.rs:10`);
  - new variants on `WorthUiIntentPayloadSource` (DSL public, `payload_source.rs:7-18`);
  - new authored constructors;
  - the optional rename of `UiExpressionConditionWithholding` (exported from
    `rt/runtime/expression/mod.rs:11-16`, `rt/facade/expression.rs:4`,
    `worth-ui/src/facade/intent.rs:4`; 27 occurrences in 11 files).
  - UNVERIFIED: whether certification matches on `UiIntentCatalogPreparationDenial`
    exhaustively (`certification/tests/.../declaration/denials.rs`,
    `ia_05/projection_posture.rs`). Downstream crates must be rebuilt either way.
- **Bank-server:** no public Query enum changes, so no check is needed.
- **Owner behavior change:** the `role_outcome` Derived type check turns a previously
  retained wrong-type value into `Denied`. Appearance consumers of derived values (3a/3c)
  would see a denial where they saw a value. UNVERIFIED: whether any current consumer
  relies on a mistyped value. Since the admission type checker should make this
  unreachable, the risk is low.

## Open questions

- **Q1.** Add `payload <field> condition <id>` for Boolean fields in 3b?
  Recommend yes (Law 8: the only computed bool).
- **Q2.** Deny `derived token` into Text at runtime preparation (recommended), or refuse it
  in the DSL as a role mismatch? The DSL can know token is never a payload type.
  Recommend doing both: a DSL refusal for token, which is kind-independent, and the runtime
  matrix as the backstop.
- **Q3.** Add Integer64 or Decimal payload field kinds so `derived integer|decimal` have a
  home? Recommend deferring. 3b admits text and condition only, and Integer→Unsigned64
  stays denied (no narrowing).
- **Q4.** Rename `UiExpressionConditionWithholding` to `UiExpressionWithholding` (public,
  11 files)? Recommend yes, in the same phase, with no alias.
- **Q5.** The plan says "Compilation fails when a use site's role type differs". Accept
  the split: role at compile, field-kind fit at catalog preparation (before freeze
  activates)? Recommend yes, and amend the plan wording (coordinator-owned; I will not edit
  the milestone file).
