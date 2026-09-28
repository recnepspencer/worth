# worth-signals-wasm Forms Product Surface Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Web runtime parent:** [web_runtime_spec.md](./web_runtime_spec.md)
>
> **Composition prerequisite:** [composition-api-plan.md](./composition-api-plan.md)
>
> **Resource/API closeout:** [api_surface_closeout.md](./api_surface_closeout.md)
>
> **Resource effects closeout:**
> [resource_response_lens_contracts_plan.md](./resource_response_lens_contracts_plan.md)
>
> **Resource mutation-response closeout:**
> [resource_mutation_response_reconciliation_plan.md](./resource_mutation_response_reconciliation_plan.md)
>
> **Certification parent:** [test-requirements.md](./test-requirements.md)

## Goal

Build a first-class forms product surface for `worth-signals-wasm` that gives app
authors one obvious form story while preserving runtime-owned truth.

The target product shape is:

```ts
const profileForm = signals.form({
  source: userDetail.line({ userId: "u1" }),
  fields: ({ field }) => ({
    displayName: field("displayName"),
    email: field("email").normalize((value) => value.trim()),
    timezone: field("settings.timezone"),
  }),
  validate: ({ effective, field, valid, invalid }) => ({
    email: field.email.includes("@")
      ? valid()
      : invalid("email.format", "Enter a valid email address"),
    displayName: effective.displayName.length > 0
      ? valid()
      : invalid("displayName.required", "Display name is required"),
  }),
  submit: ({ resourcePatch }) => resourcePatch(userDetail, ({ draft }) =>
    userDetail.patch.replace(draft),
  ),
});

profileForm.fields.email.set("ada@example.com");

if (profileForm.readiness().canSubmit) {
  await profileForm.submit();
}
```

The example is illustrative, not a locked final spelling. The locked product
meaning is that source, draft, effective value, validation, readiness,
submission, rollback, diagnostics, history, and host-derived facts are all
runtime-visible products rather than component-local conventions.

## Why This Milestone Exists

Forms are the place where otherwise disciplined web runtimes commonly regress
into convenience-era architecture:

- component-local draft stores become a second source of truth
- validation becomes a pile of callback folklore
- submit lifecycle invents `isSubmitting`, `submitError`, and `success` state
  beside the runtime async substrate
- host-derived facts such as focus, visibility, online status, viewport, and
  persistence availability become ambient browser reads
- rollback and reset semantics are implemented as ad hoc object copies
- diagnostics stop at "field X invalid" instead of explaining the runtime graph
  that produced readiness or submit denial

The roadmap explicitly says forms must come after host capability, composition,
graph lifecycle, opaque identity, and async runtime truth. That sequencing is
the architectural point: the forms product is not allowed to define those
semantics for itself. It must consume them.

This milestone exists to make the humane form story real without weakening the
substrate that now exists underneath it.

## Governing Source Summaries

- `MENTALITY.md`
  Protects hostile-proof design. This plan starts with the form race and drift
  conditions that would break a naive local-store implementation rather than
  with a pleasant field helper.
- `arch_laws.md`
  Protects boundary honesty and proof-bearing progression. A form edit must
  lower through declared field authority into draft/effective/validation/submit
  artifacts, not mutate an untyped bag and explain it later.
- `composition_laws.md`
  Protects semantic compilation units. Form declaration, field addressing,
  validation planning, submit execution, host-fact admission, and diagnostics
  must be separate responsibilities, not a single `form` helper file.
- `domain_structure_laws.md`
  Protects authority and derivation boundaries. Source truth, draft truth,
  effective projection, validation artifacts, readiness summaries, host facts,
  and submit effects must occupy distinct structural spaces.
- `perf_laws.md`
  Protects breadth honesty. Field writes, validation, dirty checks, readiness,
  and diagnostics must name their touched field/dependency breadth instead of
  hiding whole-form scans behind cheap-looking reads.
- `web_runtime_spec.md`
  Protects the framework-agnostic app runtime. Forms must be a product surface
  over `createSignals()`, `input`, `computed`, `output`, `watch`, `effect`,
  transactions, diagnostics, history, and branch truth, not a React-shaped
  adapter.
- `wasm_product_roadmap.md`
  Protects sequencing and runtime ownership. Milestone 5 belongs after
  composition/graph lifecycle/opaque identity and before future UI products
  because it should consume those semantics instead of compensating for them.
- `test-requirements.md`
  Protects certification rigor. Form closeout must provide hostile convergence
  packages, type denials, replay/restore parity, diagnostics proof, and cost
  evidence, not only example tests.
- `composition-api-plan.md`
  Protects controller-first authoring and explicit graph publication. Forms
  should be controller/graph-native products with published outputs, not hidden
  mini-graphs.
- `api_surface_closeout.md`
  Protects the closed resource line model. Resource-backed form sources and
  submits must consume line value, patch, lifecycle, diagnostics, history, and
  branch semantics instead of duplicating them.
- `resource_response_lens_contracts_plan.md`
  Protects branch-native resource effects. Form submits that patch resources
  must lower into resource effects with the same rollback, confirmation, merge,
  diagnostics, and performance posture as ordinary resource writes.

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived application with multi-section forms, resource-backed source
> values, local drafts, field normalization, sync and async validation,
> cross-field readiness, host-derived availability facts, repeated submit
> attempts, cancellation, supersession, server canonicalization, field-level
> permissions, schema-version drift, approval gates, electronic-signature
> requirements, reason-for-change prompts, unit/locale formatting, attachment
> evidence, failure rollback, branch restore, replay, rematerialization,
> multi-actor source drift, concurrent draft collaboration, declared secondary
> actions, and UI subscription churn must converge to one effective value, one
> dirty/readiness truth, one action lifecycle, and one diagnostics/history
> explanation without creating a component-local form store or a second async
> engine.

If semantically equivalent histories can produce:

- different effective values
- different dirty or touched-field truth
- different validation or readiness explanations
- different action lifecycle truth
- field-level validation work that widens silently to whole-form scans
- host facts read ambiently instead of through host capability
- resource submits that bypass resource effect envelopes
- permission or role changes that silently mutate draft/source truth
- schema version changes that strand old drafts without migration or typed
  unavailable posture
- approval, signature, and reason-for-change requirements that cannot be tied
  to the exact submitted patch and actor evidence
- non-submit actions such as save draft, approve, reject, route, assign,
  duplicate, archive, export, or request changes that bypass readiness,
  admission, patch, diagnostics, or history truth
- concurrent actors whose edits, locks, presence, approvals, or comments cannot
  be explained as branch/resource/admission facts
- locale, timezone, unit, or formatting differences that change semantic dirty
  truth without an explicit conversion artifact
- reset/rollback behavior that depends on live object copies instead of recorded
  source/draft proof
- or diagnostics/history that cannot reconstruct why the form was submittable,
  blocked, submitted, superseded, rolled back, or canonicalized

then this milestone has failed.

## Product Decision Lock

- A form is a runtime-owned product controller, not a component-local object.
- A form source is authoritative input from one of:
  - a signal readable/input
  - a graph public input
  - a resource line value
  - an explicitly declared external value boundary
- A form draft is runtime-owned writable state with declared field loci. It is
  not arbitrary component state.
- The effective form value is derived from source plus draft through declared
  field merge semantics.
- Form field loci and nested lenses must reuse runtime/resource locus proof
  machinery where it exists. Resource-backed fields must not create a parallel
  form-only path proof for topology the resource effect system already knows
  how to lower.
- Form input handling must distinguish raw input text, parsed draft value,
  formatted display value, and canonical effective/source value. Partial user
  input such as `-`, `1.`, incomplete dates, IME composition, and masked values
  must not be collapsed into invalid domain values or silently discarded.
- Dirty, touched, visited, focused, readiness, and validation are derived facts.
  They must be rebuildable from source, draft, host facts, and validation
  declarations.
- Dirty truth is semantic by default. If a user changes text and then restores
  the canonical source-equivalent value, the form must become unchanged again.
  The default save/submit readiness posture must deny unchanged submits unless
  the form explicitly opts into unchanged submission.
- Semantic dirty truth derives from a declared patch plan or equivalence proof,
  not from "field was touched" or referential object inequality. Raw input that
  has not parsed into an admitted draft may block readiness, but it must not
  WORTH a saveable domain patch.
- Error visibility is derived policy, not validation truth. A field may be
  invalid before its message is user-visible, and that distinction must be
  inspectable for adapters, summaries, focus management, and tests.
- Validation is a declared phase that emits structured artifacts:
  `valid`, `invalid`, `warning`, `pending`, `blocked`, and `unavailable`.
  Binary valid/invalid is not enough.
- Validation and submit messages must carry stable codes, target loci, severity,
  audience, visibility posture, and recovery hints. Human text is presentation
  data over a structured artifact, not the artifact identity.
- Async validation and submit lifecycle consume runtime async nodes. Forms must
  not invent a private pending/fulfilled/rejected grammar.
- Host-derived facts such as focus, visibility, online status, viewport,
  persistence availability, and autofill/credential availability must flow
  through host capability lanes where they affect readiness or validation.
- Submit is a lowered plan with declared input, source snapshot, draft snapshot,
  validation/readiness proof, async policy, and optional resource effect
  binding.
- Submit planning must emit the exact effective patch, empty-patch posture,
  omitted-field posture, attachment operation posture, and broad-replacement
  posture before execution. Save buttons and submit buttons consume this plan
  instead of guessing from touched state.
- Form actions are declared runtime actions, not arbitrary buttons. Save draft,
  submit, approve, reject, request changes, route, assign, claim, release,
  duplicate, archive, export, generate document, add evidence, and cancel/reset
  may each declare their own patch requirement, validation breadth, readiness
  policy, admission policy, async policy, resource/external effect binding, and
  history artifact.
- Submit execution is one runtime async node whose terminal artifact projects
  into form submit lifecycle and, for resource-line submits, resource effect
  lifecycle. The two views must not be backed by separate completion records.
- Resource-line submit uses resource patch/effect semantics. It must not call
  resource mutation helpers as an opaque side effect after form validation.
- Resource-line submit and resource-line action confirmation may consume
  mutation-response reconciliation, but forms still declare the intended write
  at the form boundary. The form surface must not hide resource mutation
  response planning behind feature-local commit callbacks.
- Resource-backed forms must preserve the completed resource/API surface:
  family identity, member identity, line identity, detail/collection/paged
  shape, request posture, lifecycle/freshness/status posture, history,
  verification packages, downloads, uploads, processing, delivery basis, and
  external-definition/delivery compatibility. Forms may consume these artifacts;
  they must not flatten them into generic fetch/cache state.
- Resource-backed form sessions may be bound to explicit branch/snapshot basis
  when the source line exposes one. Draft reset, rollback, and replay must name
  that basis instead of treating branch locality as hidden implementation state.
- Resource-backed submit/action bindings must inherit or select typed resource
  effect profiles. Preconfigured branch-native optimistic, server-canonical,
  pessimistic/no-optimism, delivery-authoritative, non-reversible, and
  sensitive-data profiles must stay available to forms where the resource line
  supports them; bespoke profiles remain typed advanced declarations, not
  form-local option bags.
- Dynamic enabled, disabled, hidden, required, readonly, and omitted states are
  derived control-availability facts with declared dependency breadth. Any
  field, control, group, section, host fact, resource fact, or validation result
  may influence availability only through declared dependencies and typed
  readiness artifacts.
- Permission, role, ownership, lock, review, approval, and signature facts are
  derived admission facts. They may block edit, patch, submit, or confirmation,
  but they must not silently erase draft/source/effective truth.
- Multi-actor editing is explicit collaboration posture. The runtime must name
  whether the form is single-writer locked, field-lease based,
  branch-per-actor, optimistic merge, reviewer-comment-only, or unavailable for
  collaboration. Remote changes must enter as source/resource/branch/admission
  facts, not as silent draft mutations.
- Regulated submit requirements such as electronic signature, actor identity,
  approval step, reason-for-change, policy attestation, and attachment evidence
  must bind to the exact submit plan and semantic patch digest they authorize.
- Form declarations and drafts carry schema/version posture. Version drift must
  either migrate through declared migration evidence, preserve a compatible
  draft, or emit typed unavailable/blocked artifacts before submit planning.
- Locale, timezone, unit, precision, and display-format conversions are input
  and normalization artifacts. They cannot change semantic dirty truth without
  declared conversion/equality evidence.
- Reset and rollback derive from recorded source/draft proof. They must not
  depend on mutating retained object references.
- UI events such as toasts, banners, focus movement, analytics, and navigation
  are consumers of typed form lifecycle facts. The form runtime does not execute
  UI policy.
- Form presentation lifecycle is a declared adapter-visible lane over form
  truth. Entry, interaction, availability changes, message reveal, layout
  settlement, action busy state, canonicalization, resource drift,
  collaboration, attachment/media work, navigation handoff, and exit may each
  expose presentation pending/busy/settling/ready/failed/unavailable posture.
  Presentation lifecycle may wait for adapter acknowledgements where declared,
  but it must not become source, draft, validation, patch, readiness, admission,
  action, or resource truth.
- UI adapters may consume runtime-authored interaction facts for input,
  composition, blur, focus, paste, drop, autofill, and submit intent, but those
  facts must lower into the same form controller artifacts as direct API calls.
  DOM event vocabulary must not become a second form API.
- Form diagnostics are first-class. A caller must be able to ask why a field,
  section, submit button, or whole form is dirty, blocked, pending, valid,
  invalid, unavailable, or submitted.
- Form API shape must reveal cost. Whole-form validation and field-local
  validation are different public postures.

## Scope

### In Scope

- `signals.form(...)` or equivalent form namespace authoring
- source/draft/effective/dirty/readiness/submission vocabulary
- field declarations with stable field identity and optional nested lenses
- repeated fields and field collections with stable item identity, keyed field
  loci, add/remove/reorder operations, and error preservation across reorder
- field groups and sections as declared readiness/diagnostics regions
- field normalization and parse/format posture
- raw input, parsed draft, formatted display, and canonical value posture
- semantic equality, semantic dirty, empty-patch denial, and save-readiness
  posture
- patch planning for nested JSON/object graphs, repeated collections, nullable
  fields, optional fields, omitted fields, broad replacements, and resource
  response-lens-backed patches
- attachment fields with declared metadata, blob/file identity, upload posture,
  staged add/remove/replace operations, digest/equality policy, and submit
  coordination with resource/external effects
- dynamic availability for controls, groups, sections, and submit actions:
  enabled, disabled, hidden, readonly, required, omitted, blocked, and
  unavailable
- controller-local multi-step forms: declared steps, step groups, step
  readiness, step validation gates, next/back/custom step actions, skipped/
  optional/blocked steps, dynamic step insertion/removal, progress summaries,
  and draft preservation while one form controller remains mounted
- declared form action extension points beyond submit/cancel. The runtime must
  support custom domain actions such as workflow, review, document, evidence,
  routing, assignment, and lifecycle commands without baking those examples into
  a closed action enum.
- per-action readiness, admission, validation, patch, async lifecycle,
  resource/external effect, confirmation, destructive-action, idempotency, and
  history policy
- custom input adapters with declared capability tiers: signal-native inputs,
  signal-bridged inputs, and external imperative inputs with explicit
  unavailable artifacts for unsupported behavior
- permission, role, lock, ownership, review, approval, signature, and
  reason-for-change admission facts
- schema-version, draft migration, compatibility, and typed unavailable posture
- locale, timezone, unit, precision, and display-format conversion posture
- accessibility-facing artifacts for labels, descriptions, required/invalid/
  disabled/read-only posture, error/message relationships, section summaries,
  first-blocker focus targets, announcement priority, and adapter-consumed
  reading/focus order hints
- optional generated-layout metadata for config-driven renderers, including
  section order, row/column grouping, label/control/message track hints,
  density, alignment, min-height, grow/wrap posture, and presentation-position
  hints. Renderers may combine these hints with live measurement for dynamic
  label, input, help, and message growth. Measurement is renderer feedback only;
  position, coordinates, grid placement, viewport geometry, or visual layout
  must not become source, validation, readiness, dirty, action, or submit
  authority.
- DOM measurement for generated layouts must run through a renderer-owned
  measurement controller, not through form signals. Resize, font-load,
  viewport, message-growth, and control-growth measurement may publish
  coalesced layout snapshots for renderers, but those snapshots are
  presentation artifacts outside the semantic form graph.
- multi-actor source drift, stale draft, stale permission, and stale approval
  handling where the source is resource-backed or externally versioned
- multi-actor collaboration posture, including locks, field leases,
  branch-per-actor drafts, optimistic merge, remote source updates, reviewer
  comments, presence/advisory facts, and typed unavailable posture when true
  collaboration is not supported
- derived visible-error policy separate from validation truth
- structured message artifacts for validation, readiness, submit, resource, and
  host-capability denials
- sync validation, cross-field validation, and async validation
- host-capability integration for browser-local readiness facts
- resource-line source and resource-line submit integration
- resource-line source support for detail, collection, and paged line shapes
  while preserving family/member/line identity and request posture
- resource freshness, status, reload, revalidate, retry, timeout,
  supersession, delivery basis, and visible-branch selection posture as
  readiness/action inputs where declared
- resource binary/download, upload, processing, and transfer posture for
  attachment/evidence fields where declared
- typed resource effect profile inheritance/selection for resource-line
  submit/actions
- reset, rollback, server canonicalization, supersession, and failed-submit
  semantics
- form diagnostics, summaries, history, and verification packages
- public TypeScript types and compile-denial fixtures
- product docs and examples that teach one obvious form story

### Explicitly Out Of Scope

- React hooks, components, or adapter-specific rendering APIs
- HTML form event adapters beyond the runtime-facing facts they may consume
- owning CSS, collision detection, responsive placement, or absolute/relative
  coordinates as form semantics. Generated-layout hints and measured-layout
  feedback are allowed only as typed renderer configuration.
- schema-library adapters as the primary validation model
- localization frameworks, copywriting systems, or design-system components
  beyond structured message artifacts and stable message codes
- persistence, offline queues, or cross-tab sync beyond existing runtime and
  host-capability truth
- replacing resource semantics, async semantics, or host capability semantics
- arbitrary DOM reads as reactive form truth
- route-coupled multi-step forms before router integration. URL-addressed
  steps, browser back/forward step authority, route guards, deep links, resume
  links, route-local step resources, route remount preservation, and
  branch-native speculative navigation belong to the later router integration
  lane. Forms may expose typed unavailable/deferred posture for these before
  router support exists.

## Public API Model

The final spelling should follow implementation evidence, but the product must
expose these conceptual surfaces:

```ts
const form = signals.form({
  source,
  fields,
  validate,
  actions,
  host,
  policy,
});
```

### Form Handle

A form handle must expose:

- `source()`
- `draft()`
- `effective()`
- `dirty()`
- `readiness()`
- `validation()`
- `submission()`
- `actions.<name>`
- `actionReadiness(name)`
- `patchPlan()`
- `admission()`
- `presentation()`
- `presentationLifecycle(lane?)`
- `diagnostics()`
- `diagnosticsSummary()`
- `history()`
- `reset(...)`
- `submit(...)`
- `runAction(name, input?)`
- `fields.<name>`
- optional `sections.<name>`

Cheap-looking reads such as `dirty()` and `readiness()` must return summaries
with cost and dependency posture where breadth could otherwise be hidden.
By default, `readiness().canSubmit` is false when `patchPlan()` is semantically
empty, even if fields were touched and later restored to their source-equivalent
values.

Each action handle must expose readiness, admission, required patch posture,
validation posture, semantic lifecycle, presentation lifecycle, result artifact,
diagnostics, and history. A UI button is merely an adapter over an action
handle; it must not carry hidden action policy.

Presentation lifecycle lanes must be addressable by adapters. At minimum the
model must be able to represent entry, interaction, availability, message,
layout, action, canonicalization, resource drift, collaboration, attachment,
navigation, and exit presentation posture. The exact public spelling can change,
but the form cannot collapse these into one boolean `isLoading`.

### Field Handle

A field handle must expose:

- `value()`
- `rawValue()`
- `sourceValue()`
- `draftValue()`
- `effectiveValue()`
- `set(value)`
- `input(rawValue, options?)`
- `commitInput(options?)`
- `clearDraft()`
- `dirty()`
- `touched()`
- `validation()`
- `availability()`
- `visibleMessages()`
- `readiness()`
- `diagnostics()`

Field writes must lower to a declared field locus, not to an arbitrary object
replacement unless the declaration explicitly says broad replacement is the
only legal posture.

Field collection handles must preserve stable item identity across insert,
remove, move, and replace operations. Collection-level validation may name the
collection locus, an item locus, or a child field locus, but it must not fall
back to unstable array-index-only identity when a stable key has been declared.

### Patch Planning Model

Forms must plan writes before they submit. A patch plan is the semantic bridge
between source/draft/effective truth and resource or external mutation:

```ts
type FormPatchOperation =
  | { kind: "set"; locus: FieldLocus; valueDigest: string }
  | { kind: "unset"; locus: FieldLocus }
  | { kind: "insert"; locus: CollectionLocus; itemKey: string }
  | { kind: "remove"; locus: CollectionLocus; itemKey: string }
  | { kind: "move"; locus: CollectionLocus; itemKey: string; before?: string }
  | { kind: "replace"; locus: FormLocus; replacementDigest: string }
  | { kind: "attach"; locus: FieldLocus; attachmentDigest: string }
  | { kind: "detach"; locus: FieldLocus; attachmentDigest: string };

type FormPatchPlan = {
  semanticDirty: boolean;
  empty: boolean;
  operations: readonly FormPatchOperation[];
  broadReplacement: boolean;
  omitted: readonly FieldLocus[];
  blocked: readonly FormMessageArtifact[];
  equivalenceDigest: string;
};
```

The exact names may change, but the capability cannot collapse into touched
field lists. Patch planning must support nested JSON, optional/null fields,
collection item identity, attachment operations, and broad replacement as a
visible cost posture.

### Action Model

Forms must treat every meaningful command as a declared action:

```ts
type FormActionCategory =
  | "mutation"
  | "workflow"
  | "review"
  | "assignment"
  | "evidence"
  | "document"
  | "navigation"
  | "local"
  | "custom";

type FormActionPlan = {
  actionId: string;
  category: FormActionCategory;
  domainKind?: string;
  sourceDigest: string;
  draftDigest?: string;
  patchPlanDigest?: string;
  validationDigest?: string;
  readinessDigest: string;
  admissionDigest: string;
  asyncPolicyDigest?: string;
  effectBindingDigest?: string;
  idempotencyDigest?: string;
  destructive: boolean;
};
```

Actions may be patching or non-patching, synchronous or asynchronous,
destructive or reversible, local-only or resource/external-effect backed. The
important invariant is that each action emits a plan and lifecycle artifact
before it produces side effects. Built-in convenience actions may exist, but
the contract is the action protocol, not a closed list of button names.

### Custom Input Adapter Model

Forms must not require every visual input component to be authored with the
signal library first. They must, however, require every input integration to
declare what it can prove:

```ts
type FormInputCapabilityTier =
  | "signalNative"
  | "signalBridge"
  | "externalImperative";

type FormInputAdapterProof = {
  field: FieldId;
  tier: FormInputCapabilityTier;
  canReportRawInput: boolean;
  canReportComposition: boolean;
  canReportFocusBlur: boolean;
  canReportSelection: boolean;
  canReportAutofill: boolean;
  canApplyFormattedValue: boolean;
  canPreserveUncommittedInput: boolean;
  unavailable: readonly FormMessageArtifact[];
};
```

Signal-native inputs can participate in the richest form behavior: raw input,
composition, focus/blur, selection, formatted display, host facts, and replay
can all lower through runtime artifacts. Signal-bridged inputs may adapt an
external component into those artifacts where possible. External imperative
inputs are allowed, but unsupported behaviors must become typed unavailable
posture rather than being silently assumed.

### Validation Model

Validation declarations must produce structured validation artifacts:

```ts
type FormMessageArtifact = {
  code: string;
  message?: string;
  severity: "info" | "warning" | "error";
  target?: FieldId | SectionId | FormId;
  audience: "user" | "developer" | "system";
  visibility: "hidden" | "visible" | "summary" | "blocked";
  accessibility?: {
    describedBy?: readonly string[];
    announce?: "off" | "polite" | "assertive";
    focusTarget?: FieldId | SectionId | FormId;
  };
  recovery?: readonly FormRecoveryAction[];
};

type FormValidationArtifact =
  | { kind: "valid"; field?: FieldId; digest: string }
  | { kind: "warning"; field?: FieldId; message: FormMessageArtifact }
  | { kind: "invalid"; field?: FieldId; message: FormMessageArtifact }
  | { kind: "pending"; field?: FieldId; asyncValidationId: string }
  | { kind: "blocked"; reason: string; blockers: readonly string[] }
  | { kind: "unavailable"; reason: string; detail: string };
```

The exact public type names may change, but the semantic variants may not
collapse into a boolean without violating the milestone.

### Submit Model

Submit declarations must lower to a plan:

- source snapshot digest
- actor/admission digest where permissions, approval, or signature affect
  submit
- draft snapshot digest
- effective value digest
- semantic patch plan digest
- validation digest
- readiness digest
- submit input digest
- async policy digest
- resource effect binding when resource-line authority is present
- host fact digest when host facts affect submission
- schema/version digest
- rollback and canonicalization posture

Submit execution must consume that plan. It must not re-decide eligibility,
validation, host facts, or resource effect posture at execution time.
Default submit readiness must require a non-empty semantic patch unless policy
explicitly enables unchanged submissions.

Other form actions must lower to action plans with equivalent proof. For
example, `saveDraft` may allow incomplete validation while still requiring
parseable draft regions; `approve` may require no semantic patch but require
signature/admission proof; `reject` may require a reason artifact; `assign` may
require role and route proof; `export` may require a stable snapshot digest;
and `reset` may require rollback proof rather than validation proof.

## Phases

### Phase 1: Form Kernel And Field Loci

Purpose:

- establish form identity, source authority, draft authority, effective
  projection, field loci, and adapter proof before derived behavior exists

Must ship:

- `signals.form(...)` product namespace or equivalent
- form declaration records
- source categories for signal, graph public input, resource line, and explicit
  external boundary
- field declaration records with stable field ids and optional nested lenses
- repeated-field declaration records with stable item identities and keyed loci
- attachment field declaration records with file/blob identity and metadata
- input adapter declaration records with capability tier, event proof,
  unavailable behavior, display proof, and replay posture
- runtime-owned draft storage
- raw input slots for parse/format and composition-sensitive fields
- effective projection from source plus draft
- kernel diagnostics that name source, draft, effective, raw input, adapter,
  and field-locus proof

Must preserve:

- source remains authoritative
- draft is explicitly separate from source
- effective is derived and rebuildable
- raw input truth does not masquerade as parsed draft truth
- field writes do not mutate source directly
- non-signal input adapters participate only through declared proof

Proof obligations:

- equivalent source + draft declarations converge to the same effective value
- field writes update only declared draft loci
- raw input writes update raw input posture without forging parsed draft values
- repeated-field add/remove/reorder preserves stable item identity
- attachment fields preserve identity and metadata before any submit behavior
- unsupported raw input, composition, focus, selection, autofill, formatting, or
  replay behavior emits unavailable posture
- malformed field paths deny before draft mutation
- type denials reject fields not declared in the form contract

### Phase 2: Semantic Dirty And Patch Planning

Purpose:

- make "changed" mean semantic source-equivalence delta, not touched state,
  object identity, or broad object replacement

Must ship:

- semantic equality declarations and default canonical-source equivalence
- dirty summary derived from source/draft/effective equality proof
- empty-patch posture and default save/submit denial for unchanged forms
- patch planning from source/draft/effective into semantic operations
- nested JSON/object graph patch planning with exact loci where declared
- nullable, optional, omitted, unset, and broad-replacement patch posture
- repeated collection insert/remove/move/replace patch operations
- attachment add/remove/replace patch operations with digest/equality policy
- diagnostics that name dirty/equality/patch breadth and broad replacement cost

Must preserve:

- dirty truth is semantic by default, not touch-state or reference inequality
- edit-then-revert histories clear semantic dirty
- empty semantic patches deny default save/submit readiness
- broad replacement is explicit and cost-visible
- attachment operations are distinct from JSON value patches

Proof obligations:

- changing a field and then restoring its source-equivalent canonical value
  clears semantic dirty and disables default save readiness
- nested JSON/object patches name exact declared loci or visibly fall back to a
  broad replacement posture
- optional/null/omitted/unset fields produce distinguishable patch operations
- attachment add/remove/replace operations are planned separately from JSON
  value changes and carry file/blob identity evidence
- repeated-field reorder preserves item-level dirty and patch identity when
  stable keys exist
- dirty summary names touched fields, semantic changed fields, and breadth

### Phase 3: Validation, Messages, And Readiness

Purpose:

- make parse, validation, message visibility, and readiness runtime-derived
  structured facts with explicit dependency breadth

Must ship:

- validation declaration surface
- parse/format failure artifacts distinct from domain validation
- field-local validation
- cross-field validation with declared dependencies
- warning, invalid, pending, blocked, unavailable, and valid artifacts
- message catalog posture with stable codes, severity, target loci, audience,
  visibility, accessibility metadata, and recovery hints
- visible-message derivation from validation, touched/visited/focused state,
  submit/action attempts, and policy
- readiness summary derived from dirty, validation, messages, host placeholders,
  admission placeholders, action policy, and submit policy
- validation diagnostics and history entries
- cost counters for field-local, dependency-region, and whole-form validation

Must preserve:

- validation cannot mutate source or draft
- parse failures cannot mutate parsed draft truth until the parser admits a
  value
- message visibility cannot change validation truth
- readiness is derived from artifacts and policy, not hand-authored booleans
- cross-field validation declares dependency breadth
- warnings do not block submit unless policy says so

Proof obligations:

- field-local edits revalidate only the declared field/dependency region
- IME composition, partial numeric input, incomplete date input, masked input,
  paste, and autofill produce explicit input artifacts instead of lossy values
- first visible field error, section summary, and form summary derive from the
  same message artifacts
- cross-field validators name their dependency set
- async-shaped pending validation artifacts can exist before async execution
  is wired, but cannot unblock readiness by convention
- invalid, pending, blocked, and unavailable remain distinct
- type denials reject validators that return undeclared artifact shapes

### Phase 4: Availability And Admission

Purpose:

- make dynamic control topology and authority gates explicit runtime facts
  before action execution exists

Must ship:

- dynamic availability declaration surface for fields, controls, groups,
  sections, and actions
- controller-local step declaration surface with stable step ids, step groups,
  step order, optional/skipped/blocked posture, and dynamic insertion/removal
  policy
- enabled, disabled, hidden, readonly, required, omitted, blocked, and
  unavailable control states
- draft preservation, clearing, freezing, and omission policy for unavailable or
  disabled fields
- admission declaration surface for edit, patch, submit, action execution,
  approval, signature, review, reason-for-change, permission, role, ownership,
  and lock posture
- availability and admission summaries with declared dependency breadth
- step readiness, step validation, step dirty/patch, step message, and step
  progress summaries derived from existing field/section/action artifacts
- stale approval/signature/reason invalidation rules bound to actor, policy,
  patch, schema, and source digests

Must preserve:

- availability and omission are runtime-derived facts, not adapter state
- admission facts block capabilities through typed artifacts instead of
  disabling controls by convention
- permission, lock, approval, signature, and review posture do not mutate source
  or draft truth
- disabled, hidden, readonly, required, omitted, blocked, and unavailable are
  distinct
- controller-local step state is derived from form artifacts and does not depend
  on route state
- dependency cycles are denied before derived topology executes

Proof obligations:

- arbitrary control/group enablement changes recompute only declared dependent
  readiness, validation, message, and patch-plan regions
- a field disabled by another input preserves, clears, freezes, or omits draft
  state only according to declared policy
- role/permission/lock changes update editability, patchability, and
  actionability without mutating source or draft
- approval/signature/reason requirements bind to the current semantic patch and
  become stale when patch, actor, policy, schema, or source digests change
- action visibility/enabled/blocked state derives from declared readiness and
  admission facts, not button-local booleans
- next/back/custom step actions consume declared step readiness and preserve
  draft/patch truth across skipped, optional, blocked, inserted, and removed
  steps

### Phase 5: Actions And Submit Planning

Purpose:

- generalize submit into a declared action protocol so custom buttons and
  workflow commands cannot bypass form truth

Must ship:

- action declaration surface for built-in conveniences and custom domain
  actions
- step navigation actions for controller-local multi-step forms, including
  next, back, jump, skip, revisit, and custom step commands
- per-action readiness, admission, validation, patch, schema, host, idempotency,
  destructive-action, and effect-binding policy
- action planning phase with source/draft/effective/validation/readiness/
  admission/patch/schema/effect proof
- submit as a first-class action plan with non-empty semantic patch default
- action result artifact shapes for accepted, denied, unavailable, cancelled,
  superseded, rejected, fulfilled, and no-op outcomes
- regulated action artifacts for actor identity, approval, electronic
  signature, reason-for-change, and policy attestation where declared
- recovery actions for retry, edit field, reset field, accept canonical value,
  reveal section, and focus first actionable blocker
- diagnostics/history for action and submit planning

Must preserve:

- every effectful action consumes a lowered action plan
- submit execution consumes a lowered submit/action plan
- action planning cannot secretly re-run validation with different dependency
  breadth
- unchanged semantic patch denial happens before external/resource side effects
  unless unchanged submission is explicitly declared
- custom domain actions are labels over the action protocol, not hidden button
  callbacks
- controller-local step navigation must not invent URL, browser history, or
  route transition semantics

Proof obligations:

- default submit is denied when the semantic patch is empty, even after
  touch/edit/revert histories
- non-submit actions such as save draft, approve, reject, assign, route,
  archive, export, reset, and custom domain actions emit readiness, admission,
  plan, diagnostics, and history artifacts
- destructive and idempotent actions expose distinct posture before execution
- repeated action attempts with the same action/idempotency digest collapse,
  supersede, queue, or deny according to declared policy
- approval/signature/reason artifacts bind to the exact action plan digest
- step navigation actions produce action plans and diagnostics just like other
  form actions

### Phase 6: Async Lifecycle And Server Results

Purpose:

- bind async validation, action execution, submit execution, server results, and
  canonicalization to runtime-owned async truth

Must ship:

- async validation declarations
- async validation trigger policy for input, blur, idle, debounce, explicit
  validation, and action/submit admission
- action execution through runtime async node semantics where asynchronous or
  effect-backed
- cancellation, retry, timeout, supersession, stale completion, and rejection
  posture
- server rejection mapping into form-level, section-level, field-level, action,
  and resource-locus messages
- canonicalization hooks for server-returned transformed values
- schema-version and draft-migration posture for long-lived drafts once forms
  can bind a long-lived draft to explicit source-schema compatibility proof
- source drift handling before async action execution
- typed deferred/unavailable posture for route-coupled step behavior before the
  router milestone
- diagnostics/history for async validation, action execution, server results,
  and canonicalization

Must preserve:

- async validation and action lifecycles use runtime async semantics
- server canonicalization is explicit and replay-visible
- server field errors are messages over declared loci, not opaque submit errors
- stale async completions cannot rewrite newer source, draft, validation,
  readiness, action, or admission truth
- schema drift cannot silently rewrite or discard a long-lived draft once
  schema-version posture is admitted
- route-coupled multi-step behavior cannot be faked with form-local route state
  once router integration begins; until then the form lane must stay on typed
  deferred/unavailable posture rather than pretending route semantics exist

Proof obligations:

- stale async validation completion cannot unblock a newer invalid draft
- stale submit/action completion cannot rewrite newer form truth
- cancellation, timeout, retry, rejection, supersession, and fulfillment produce
  typed lifecycle artifacts
- server-side field rejection maps to the same field/section summaries as local
  validation without pretending it was produced by a local validator
- schema drift either migrates with evidence, blocks with typed unavailable
  posture, or preserves a compatible draft with a compatibility digest
- fulfilled canonicalization updates source/draft/effective truth through
  declared posture
- async denial happens before external/resource side effects
- URL-addressed step transitions, route guards, browser back/forward behavior,
  deep links, resume links, route remount preservation, and route-local step
  resources deny with typed deferred posture that requires route authority

Current implementation note:

- the current forms branch already covers async validation, action execution,
  stale completion handling, retry/timeout/cancellation/supersession posture,
  server rejection mapping, and canonicalization
- schema-version/draft-migration posture and route-coupled step deferred
  posture remain explicit Phase 6 obligations rather than closed work

### Phase 7: Host Facts And Input/Renderer Capabilities

Purpose:

- admit browser-local facts, custom input capabilities, accessibility artifacts,
  generated-layout hints, DOM measurement, and presentation lifecycle without
  polluting semantic form truth

Must ship:

- host fact bindings for focus, visibility, viewport, online status,
  persistence availability, and credential/autofill availability where
  supported
- typed host fact posture in readiness, validation, availability, and action
  summaries
- accessibility artifact surface for labels, descriptions, message
  relationships, summaries, required/invalid/disabled/read-only posture,
  announcement priority, and first actionable focus target
- presentation-order hints for reading order, tab/focus order, section order,
  and summary order
- generated-layout hints for section/row/column grouping, label/control/help/
  message tracks, density, alignment, min-height, grow/wrap posture, and
  responsive tokens
- non-signal DOM measurement controllers with ResizeObserver/font-load/
  viewport/content-growth/animation-frame batching policy
- coalesced layout snapshot artifacts for renderer subscribers
- presentation lifecycle lanes for entry, interaction, availability, messages,
  layout, action busy/settlement, canonicalization, resource drift,
  collaboration, attachments/media, controller-local step navigation,
  navigation handoff, and exit
- default presentation policies for delayed busy reveal, minimum busy duration,
  settlement acknowledgement, settlement timeout, supersession handoff, and
  unavailable adapter acknowledgement
- presentation scope policy for field, section, action, button/control, whole
  form, step, modal, route, and external handoff scopes
- unavailable posture when adapters cannot honor host, accessibility, input, or
  layout/presentation capabilities

Must preserve:

- no ambient DOM or browser reads become reactive form truth
- host fact absence is explicit and does not masquerade as false
- host fact changes cannot mutate source/draft directly
- accessibility artifacts derive from canonical form artifacts and do not
  execute DOM policy
- visual position and viewport geometry do not decide validation, readiness,
  dirty, availability, admission, action, submit, patch, or source truth
- DOM measurement remains outside `signals.form(...)` semantic graph state
- presentation lifecycle remains adapter-visible presentation state and cannot
  rewrite source, draft, effective, validation, dirty, patch, readiness,
  admission, action, resource, or history truth
- semantic action fulfillment and visible presentation settlement are distinct

Proof obligations:

- online/offline changes block or unblock action/submit readiness through host
  capability artifacts
- focus/visited/touched facts are diagnostics-visible and replay-honest where
  supported
- labels, descriptions, required posture, invalid posture, message
  relationships, summary relationships, announcement priority, and focus targets
  derive from form artifacts
- generated layouts align row label/control/message heights for mixed controls
  without changing form truth
- page resize, font loading, translated copy expansion, async message arrival,
  and textarea growth produce bounded renderer layout snapshots without
  invalidating form signals or re-running validation/readiness/action planning
- entry presentation can remain pending until source admission, draft restore,
  schema migration, host facts, adapter capability, initial validation/readiness,
  focus target, and declared layout measurement settle
- action presentation can remain busy after semantic fulfillment until declared
  canonicalization, message, focus, banner/toast, layout, navigation, or route/
  modal handoff acknowledgement occurs
- interaction, availability, message, resource drift, collaboration, attachment,
  step, and exit presentation lanes expose scoped busy/settling/ready/failed
  posture without changing semantic form truth

### Phase 8: Resource, Branch, And Collaboration Integration

Purpose:

- make resource-backed forms, branch-native effects, and multi-actor
  collaboration consume existing resource/branch truth instead of inventing
  form-local optimistic or collaboration engines

Must ship:

- resource line source adapters
- resource family/member/line identity preservation for detail, collection, and
  paged resource-line forms
- request posture, lifecycle/freshness/status posture, reload/revalidate/retry/
  timeout/supersession posture, delivery basis posture, and external delivery
  compatibility as form-readable artifacts
- binary/download/upload/processing posture for attachment and evidence fields
  that bind to resource transfer surfaces
- visible branch selection proof for committed, speculative, confirmed,
  restored, merged, or unavailable resource-line visible truth
- resource response-lens/locus proof reuse for resource-line field loci
- lowering from form patch plans into resource effect operations
- lowering from resource-line form actions into resource effect, delivery,
  branch, or external action artifacts where declared
- resource mutation-response reconciliation as a consumable form-facing
  substrate for resource-line submit/action completion:
  exact reconciliation, partial reconciliation, stale-target denial,
  refetch-required, delivery-awaited, identity migration, create placement,
  delete/tombstone posture, and multi-family target outcomes where the backing
  resource declaration admits them
- typed resource effect profile inheritance/selection for resource-line
  submit/actions, including branch-native optimistic, server-canonical,
  pessimistic/no-optimism, delivery-authoritative, non-reversible, and
  sensitive-data profiles where supported
- server confirmation and failure posture for resource-line submit/actions
- rollback through resource branch restore or inverse effect where available
- mapping from form field loci to resource effect loci
- projection from resource merge/rebase conflicts back to declared form fields,
  sections, messages, readiness, and diagnostics regions
- collaboration posture for resource-line forms: single-writer lock,
  field-lease, branch-per-actor, optimistic merge, reviewer-comment-only, or
  typed unavailable
- remote source drift, collaborator events, reviewer comments, lock/lease
  changes, and advisory presence artifacts

Must preserve:

- resource line remains authoritative for resource source truth
- resource family/member/line identity and request posture remain
  resource-owned, not form-owned
- resource freshness, lifecycle, download, upload, processing, delivery,
  history, restore, replay, and external compatibility truth remain
  resource-owned artifacts
- mutation-response plans, fallback posture, lifecycle proof, history proof,
  verification packages, and closeout matrices remain resource-owned artifacts
  when the form submit/action is governed by a resource line
- form draft remains separate until submit/confirmation policy admits it
- resource effect envelopes remain the canonical artifact for resource-line
  writes and actions
- branch/merge/rebase proof remains resource-owned, not form-owned
- branch-local speculative form posture remains attributable to native signal
  branch proof
- collaborator presence and comments are advisory unless admitted through
  lock/lease/review/approval/action artifacts

Proof obligations:

- resource-line submit emits the same resource effect posture as equivalent
  direct resource patch
- resource-line form source materialization preserves detail, collection, and
  paged identity rather than collapsing into anonymous objects
- stale freshness, retry/timeout, supersession, delivery basis drift, and
  resource status changes can block or advise form readiness/action posture
  through typed artifacts
- attachment/evidence fields backed by binary/download/upload/processing
  resource posture do not invent a parallel transfer lifecycle
- resource-line visible source/effective truth exposes committed,
  speculative, confirmed, restored, merged, or unavailable branch selection
  proof where the line carries it
- complex patch plans over nested JSON, collections, and attachments lower into
  resource effects without hidden broad replacement
- server canonicalization updates source/effective truth without preserving
  stale draft values
- failure rollback restores exact source/effective/draft posture
- branch restore and replay preserve form diagnostics/history truth
- form field to resource locus mapping denies when no declared resource locus
  exists
- resource-line form certification consumes the underlying resource line
  verification package, `resource.effects.closeoutMatrix(profile)` evidence,
  and resource mutation-response closeout evidence such as
  `signals.resource.mutationResponses.closeoutMatrix()` where available
- resource-line submit/action completion that relies on canonical server
  responses consumes the same mutation-response confirmation, fallback,
  replay/restore, and stale-denial posture the resource lane already ships; the
  form layer must not restate those outcomes as a second local submit cache
- simultaneous actor edits isolate by branch/lease, merge through native
  branch/resource proof, or block with typed conflict/unavailable artifacts
- remote source changes rebase, conflict, block, or preserve local draft truth
  through declared branch/resource/admission evidence

### Phase 8.5: Public Contract And Vocabulary Closure

Purpose:

- close Phase 8 by removing public-surface contract language that normalizes
  partial implementation as durable product truth

Must ship:

- public forms/resource-facing vocabulary that names declared capability,
  admitted posture, lowering contract, compatibility boundary, or unavailable
  runtime proof directly rather than encoding unfinished implementation as
  stable product nouns
- cleanup of facade, diagnostics, verification, and certification language
  where `unsupported`, `currently`, or roadmap-leaking `until integration
  exists` wording would otherwise teach the wrong product contract
- explicit separation between:
  - product capability unavailable by declared/native/runtime proof
  - current lowering contract admission/denial
  - typed deferred posture where an external authority boundary is intentionally
    outside the current milestone
- hostile certification proving that renamed or reclassified posture did not
  weaken authority, readiness, execution, or replay truth

Must preserve:

- typed unavailable and typed deferred posture where the spec intentionally
  requires them
- declaration/runtime denial honesty; this phase is contract correction, not
  euphemism
- resource-owned, host-owned, route-owned, and history-owned authority
  boundaries
- replay, restore, diagnostics, verification, and performance evidence parity
  after vocabulary correction

Proof obligations:

- no form-facing resource shape or recovery surface encodes "not built yet" as
  a stable capability noun when the real contract is "different declared
  capability" or "outside the current lowering path"
- unavailable posture means capability or proof is genuinely absent at the
  declared/runtime boundary, not merely that implementation work remains
- deferred posture means responsibility belongs to an external authority
  boundary, not that local implementation was postponed casually
- equivalent runtime behavior before and after vocabulary cleanup preserves the
  same authority, readiness, lifecycle, replay, and verification truth
- docs, public types, diagnostics, and tests all teach the same corrected
  contract language

Terminology note:

- broad product-category wording such as `resource-backed forms` may remain in
  roadmap or product-family descriptions
- public contract, proof, diagnostics, verification, and certification lanes
  should name the actual authority boundary directly, such as `resource line`,
  `resource-line submit`, `resource-line action`, `resource proof`, or
  `route authority`

### Phase 9: Diagnostics, History, Replay, And Verification Packages

Purpose:

- close observability and reconstructability for forms as product artifacts

Must ship:

- form diagnostics summary
- full form diagnostics
- form history entries for source changes, raw input, draft writes, dirty/patch
  changes, validation changes, availability/admission changes, host facts,
  renderer capability posture, action planning, action execution, async results,
  resource effects, collaborator events, lock/lease changes, reset, rollback,
  canonicalization, and schema migration
- form verification package
- exact restore and replay posture where runtime support exists
- retained-history unavailable artifacts where exact replay is unsupported
- boundary performance envelope

Must preserve:

- diagnostics derive from canonical form artifacts
- diagnostics and history cannot change operational truth
- replay/restore posture stays explicit rather than silently best-effort
- verification packages compose resource effect and branch proof where relevant

Proof obligations:

- equivalent forward and replay histories produce the same effective, dirty,
  patch, readiness, validation, action, admission, and submission truth
- diagnostics summary and full diagnostics agree on current state
- verification packages preserve source/draft/effective/validation/readiness/
  action/admission/patch/resource/collaboration digests
- retained-history unavailability is explicit and does not rewrite proof
- performance envelope names field write, raw input, validation, readiness,
  action, submit, renderer measurement, resource, collaboration, and diagnostics
  breadth

### Phase 10: Public Types, Docs, And Certification Closeout

Purpose:

- make the forms surface library-grade, teachable, type-hardened, and
  certifiably complete

Must ship:

- public TypeScript declaration files
- type-smoke usage examples
- compile-denial fixtures for invalid field, validation, readiness,
  availability, admission, action, host, accessibility, renderer capability,
  resource, and submit shapes
- feature docs and recipes for ordinary forms, resource-backed forms, async
  validation, host facts, action lifecycle, generated renderer layout,
  collaboration posture, and submit lifecycle
- closeout matrix tying form families to runtime tests, type denials,
  diagnostics/history proof, resource/branch proof, renderer proof, and
  performance evidence
- full hostile suite-0-style convergence scenario

Must preserve:

- docs show the runtime-owned story, not component-local workarounds
- type surface makes invalid phase transitions uncallable where possible
- examples do not rely on tribal knowledge or ambient state

Proof obligations:

- package docs examples execute against the real product facade
- public types reject undeclared fields and illegal action/submit/validation/
  availability/admission/host/resource/renderer shapes
- hostile convergence covers ordinary signal forms, resource-backed forms,
  async validation, host facts, custom inputs, generated layout, collaboration,
  failure rollback, reset, branch restore, and replay

## Must Ship

- a first-class form product namespace
- source/draft/effective/dirty/readiness/submission vocabulary
- field declarations with stable loci and optional lenses
- raw/parsed/display/canonical value posture for input-heavy fields
- repeated-field and collection handling with stable item identity
- semantic dirty and empty-patch default save denial
- patch planning for nested JSON, optional/null/omitted fields, collections,
  attachments, resource effects, and broad replacement posture
- dynamic control/group/section availability with declared dependency breadth
- declared action vocabulary and lifecycle for commands beyond submit/cancel
- per-action readiness, admission, idempotency, destructive-action, and effect
  policy
- runtime-owned draft state
- structured validation artifacts
- structured message artifacts with derived visibility and recovery hints
- admission artifacts for permissions, locks, approvals, signatures, review
  gates, and reason-for-change requirements
- schema-version and draft-migration evidence for long-lived forms
- locale/timezone/unit/precision conversion evidence for semantic equality
- accessibility-facing label, description, relationship, summary,
  announcement, and focus artifacts derived from form truth
- generated-layout hints for config-driven renderers, including row/column
  grouping, label/control/message track consistency, density, alignment,
  min-height, grow/wrap posture, and presentation order/position hints that
  adapters may consume without granting layout authority to the form runtime
- measured-layout feedback for renderers that need dynamic row/track
  synchronization under arbitrary label, input, help, and error growth
- non-signal DOM measurement controllers with coalesced renderer layout
  snapshots for resize/content/font/message/input growth
- presentation lifecycle lanes for entry, interaction, availability, message
  reveal, layout settlement, action busy/settlement, canonicalization, resource
  drift, collaboration, attachment/media work, controller-local step
  navigation, navigation handoff, and exit guards
- collaboration posture for multi-actor forms, including locks, leases,
  branch-per-actor drafts, merge/conflict projection, remote source drift, and
  advisory presence/comment artifacts
- controller-local multi-step forms with step readiness, step validation gates,
  step actions, step progress summaries, skipped/optional/blocked steps,
  dynamic step insertion/removal, and route-coupled deferred posture
- async validation and submit lifecycle through runtime async truth
- resource-line submit integration through resource effect envelopes
- resource-line actions consume resource/branch/effect truth where applicable
- resource-line forms preserve completed resource/API identity, lifecycle,
  freshness, transfer, delivery, external compatibility, history, restore,
  replay, and verification truth
- resource-line submit/actions inherit or select typed resource effect
  profiles rather than declaring form-local resource effect options
- host-capability integration for browser-local facts
- reset, rollback, canonicalization, cancellation, timeout, retry, and
  supersession posture
- diagnostics, history, replay/restore posture, and verification packages
- public TypeScript surface and compile-denial coverage
- docs and examples that teach one obvious forms story

## Must Preserve

- forms do not become a second local store
- source truth remains authoritative and structurally distinct from draft truth
- effective form truth is derived and rebuildable
- raw input truth does not masquerade as parsed draft truth
- semantic dirty truth does not masquerade as touched truth
- default save/submit readiness denies unchanged semantic patches
- validation/readiness/submission are runtime-derived facts
- action readiness and lifecycle are runtime-derived facts
- availability and omission are runtime-derived facts, not ad hoc adapter state
- permission, lock, approval, signature, and review posture are runtime-derived
  admission facts, not adapter-only disabled buttons
- approval/signature/reason evidence cannot authorize a different patch than
  the one it was bound to
- schema drift cannot silently rewrite or discard a long-lived draft
- visible error messages do not masquerade as validation truth
- accessibility artifacts derive from canonical form truth and do not execute
  DOM or layout policy
- visual position does not masquerade as semantic order unless declared as a
  form-order field/locus
- generated-layout hints remain presentation metadata and do not affect source,
  draft, effective, dirty, validation, readiness, action, submit, patch, or
  admission truth
- measured-layout feedback remains renderer state and cannot become source,
  draft, effective, dirty, validation, readiness, action, submit, patch,
  admission, or resource truth
- DOM measurement remains outside `signals.form(...)` semantic graph state and
  cannot create form signal invalidation storms during resize or content growth
- presentation lifecycle does not masquerade as semantic action, readiness, or
  resource truth
- async lifecycle remains runtime-owned
- resource-line submit consumes resource effect truth
- resource-line forms do not flatten detail/collection/paged resource lines,
  freshness/status, delivery, binary/download/upload/processing, or external
  compatibility into generic form state
- resource effect profiles, visible branch selection, branch restore,
  merge/rebase, and closeout-matrix evidence remain resource/effect-owned
- non-submit actions cannot bypass readiness, admission, diagnostics, history,
  or resource/external effect envelopes
- collaborative edits cannot silently mutate another actor's draft truth
- controller-local step state does not masquerade as route or browser-history
  truth
- host-derived facts flow through typed host capability
- UI behavior consumes lifecycle facts and is not executed by the form runtime
- cheap-looking form APIs do not hide broad scans or rich diagnostics work

## Acceptance Evidence

This milestone is complete only when the package can prove all of the following
through runtime tests, type-smoke tests, compile denials, docs examples, and
closeout verification packages.

### Form Kernel Evidence

- equivalent signal-backed, graph-backed, resource-backed, and external-boundary
  form sources preserve explicit source authority
- draft writes are runtime-owned and field-locus-bound
- signal-native, signal-bridged, and external imperative input adapters expose
  distinct capability evidence and unsupported behavior artifacts
- raw input, parsed draft, formatted display, and canonical effective/source
  values remain distinct and replay-visible
- semantic dirty clears after edit-then-revert histories and default
  save/submit readiness becomes false again
- nested JSON, optional/null fields, omitted fields, and broad replacement
  posture emit explicit patch-plan evidence
- attachment add/remove/replace operations preserve identity, metadata, digest,
  upload posture, and rollback/canonicalization evidence
- repeated-field add/remove/reorder preserves stable item identity and does not
  strand field messages or dirty truth on stale indexes
- effective values derive from source plus draft without mutating source
- malformed field paths and undeclared fields deny before draft mutation
- dirty summaries name exact touched fields and breadth

### Validation, Availability, Renderer, And Readiness Evidence

- field-local validation stays field-local
- declared cross-field validation widens only to declared dependencies
- parse failures, domain validation failures, readiness blockers, and host
  unavailability remain distinguishable
- disabled, hidden, readonly, required, omitted, blocked, and unavailable
  control states remain distinguishable
- accessibility labels, descriptions, required posture, invalid posture,
  disabled/read-only posture, message relationships, summary relationships,
  announcement priority, and focus targets derive from form artifacts
- adapter-declared reading, section, summary, and focus order hints remain
  separate from semantic value/patch order
- dynamic control/group/section availability recomputes only declared
  dependency regions and denies dependency cycles
- role, permission, ownership, lock, approval, signature, review, and
  reason-for-change admission facts block the correct edit/patch/submit
  capability with typed diagnostics
- controller-local steps derive readiness, validation, dirty/patch, message,
  and progress summaries from existing form artifacts without route state
- skipped, optional, blocked, inserted, removed, revisited, next, back, and jump
  step postures preserve draft and patch truth
- locale, timezone, unit, precision, and display-format conversions preserve
  semantic equality unless a declared conversion changes domain meaning
- visible field messages, section summaries, form summaries, and first-blocker
  focus targets derive from the same structured message artifacts
- labels, descriptions, error relationships, required/invalid/disabled/read-only
  posture, summary relationships, announcement priority, and focus targets are
  derived from form artifacts and remain adapter-consumable
- changing visual order or layout position does not alter semantic dirty,
  validation, readiness, action, or submit truth unless an explicit declared
  form-order locus changes
- generated layouts can align row label/control/message heights consistently
  for mixed control types such as textarea, select, custom input, and attachment
  fields without changing field value, dirty, validation, or patch truth
- arbitrary label, input, help, and error message growth can update measured
  layout artifacts and synchronize row tracks without altering canonical form
  truth
- page resize, font loading, translated copy expansion, async message arrival,
  and textarea growth produce bounded renderer layout snapshots without
  invalidating form signals or re-running validation/readiness/action planning
- presentation lifecycle lanes expose pending/busy/settling/ready/failed/
  unavailable posture for entry, interaction, availability, messages, layout,
  actions, canonicalization, resource drift, collaboration, attachments,
  navigation, and exit without changing semantic form truth
- warning, invalid, pending, blocked, unavailable, and valid artifacts remain
  distinct
- readiness explains all blockers and advisories
- async validation stale completions cannot unblock newer drafts
- type denials reject malformed validation declarations

### Action And Submit Evidence

- every declared action emits readiness, admission, plan, lifecycle, result,
  diagnostics, and history artifacts
- every effectful action can emit distinct semantic lifecycle and presentation
  lifecycle artifacts
- controller-local step navigation actions emit ordinary action plans,
  diagnostics, history, and presentation lifecycle artifacts
- non-submit actions such as save draft, approve, reject, request changes,
  route, assign, claim, release, archive, export, generate, add evidence, reset,
  abandon, and custom domain actions cannot bypass declared action policy
- destructive actions and idempotent actions expose distinct posture before
  execution
- submit planning emits source, draft, effective, validation, readiness, host,
  policy, semantic patch, admission, schema, and submit-input digests
- submit execution consumes the lowered plan
- empty semantic patch submit denial happens before external/resource effects
  by default
- approval/signature/reason artifacts bind to the exact submit plan digest and
  become stale when the patch, actor, policy, schema, or source digest changes
- cancellation, timeout, retry, rejection, supersession, and fulfillment produce
  typed lifecycle artifacts
- action presentation can remain busy after semantic fulfillment until declared
  UI settlement acknowledgements or timeout artifacts resolve
- stale submit completions cannot rewrite newer form truth
- server canonicalization is explicit and replay-visible
- server-side field, section, form, and resource-locus rejections map into
  structured messages without becoming local validation folklore
- submit denials happen before resource or external effects

### Resource, Branch, And Collaboration Evidence

- resource-line submit lowers to the same resource effect evidence as an
  equivalent direct resource patch
- detail, collection, and paged resource-line forms preserve
  family/member/line identity, request posture, lifecycle/freshness/status, and
  verification package identity
- resource freshness/reload/revalidate/retry/timeout/supersession/delivery
  posture flows into readiness/action artifacts without inventing form-local
  resource lifecycle truth
- attachment/evidence fields backed by binary/download/upload/processing
  resource surfaces preserve resource transfer posture
- resource-line submit/actions inherit or select typed resource effect
  profiles and deny form-local WORTHd profile posture
- failed resource submit can roll back through branch restore or inverse effect
  where available
- resource locus mapping is explicit and can deny with a typed unavailable
  artifact
- visible resource-line form truth is attributable to committed,
  speculative, confirmed, restored, merged, or unavailable branch selection
  proof where the line exposes branch visibility
- resource-line form closeout embeds or links resource line verification
  packages, `resource.effects.closeoutMatrix(profile)` evidence, and resource
  mutation-response closeout evidence such as
  `signals.resource.mutationResponses.closeoutMatrix()`
- branch restore, exact replay, retained-history unavailability, and merge proof
  preserve form diagnostics/history truth
- resource drift and collaboration presentation lanes can show settling or
  conflict UI without mutating another actor's draft truth
- route-coupled multi-step features emit typed deferred/unavailable posture
  that requires route authority
- multi-actor edits use explicit lock, lease, branch-per-actor, optimistic
  merge, reviewer-comment, or typed unavailable posture
- remote source changes rebase, conflict, block, or preserve local draft truth
  through declared branch/resource/admission evidence
- collaborator presence and comments remain advisory unless admitted through
  lock, lease, review, approval, or action artifacts

### Host And Renderer Capability Evidence

- online, visibility, focus/touched, viewport, persistence, and credential or
  autofill availability affect forms only through host capability artifacts
- unsupported host facts remain unavailable rather than false or absent
- host invalidation breadth is declared and tested

### Documentation And Type Evidence

- package docs include ordinary forms, resource-backed forms, async validation,
  host facts, custom inputs, generated renderer layout, action lifecycle,
  submit lifecycle, rollback/reset, collaboration, and diagnostics examples
- docs examples execute against the real package facade
- public TypeScript surfaces reject undeclared fields, illegal validator
  returns, illegal action/submit plans, WORTHd host fact posture, WORTHd
  accessibility posture, WORTHd renderer capability posture, and illegal
  lifecycle transitions where the type system can encode them

## Required Certification Suites

1. The Form Source Authority And Draft Isolation Test
2. The Field Locus And Dirty Breadth Test
3. The Effective Value Derivation And Reset Test
4. The Raw Input Parse Format And Canonical Value Test
5. The Semantic Dirty And Empty Patch Denial Test
6. The Nested JSON Patch Planning Test
7. The Attachment Operation Patch Test
8. The Repeated Field Stable Identity Test
9. The Dynamic Availability Dependency Test
10. The Permission Lock And Admission Fact Test
11. The Approval Signature And Reason Binding Test
12. The Schema Version Drift And Draft Migration Test
13. The Locale Unit And Precision Equality Test
14. The Controller-Local Multi-Step Flow Test
15. The Route-Coupled Step Deferred Posture Test
16. The Accessibility Artifact And Focus Target Test
17. The Presentation Order Hint Boundary Test
18. The Generated Layout Hint Consistency Test
19. The Presentation Lifecycle Entry And Settlement Test
20. The Presentation Busy Policy And Timeout Test
21. The Secondary Action Planning And Lifecycle Test
22. The Destructive And Idempotent Action Policy Test
23. The Multi-Actor Lock Lease And Branch Collaboration Test
24. The Remote Source Drift And Draft Conflict Test
25. The Reviewer Comment And Advisory Presence Test
26. The Validation Artifact Topology Test
27. The Message Visibility And Recovery Test
28. The Cross-Field Dependency Breadth Test
29. The Async Validation Supersession Test
30. The Readiness Explanation Test
31. The Submit Plan Lowering Test
32. The Submit Async Lifecycle Test
33. The Stale Submit Completion Denial Test
34. The Server Rejection Message Mapping Test
35. The Resource-Line Submit Effect Test
36. The Resource Family Line And Freshness Preservation Test
37. The Resource Transfer And Attachment Posture Test
38. The Resource Effect Profile Inheritance Test
39. The Resource Visible Branch Selection Test
40. The Resource Submit Rollback And Canonicalization Test
41. The Host Capability Form Fact Test
42. The Form Diagnostics And History Parity Test
43. The Form Replay Restore And Retained-History Test
44. The Form Boundary Performance Envelope Test
45. The Public Form Type Surface Test
46. The Forms Documentation Happy Path Test
47. The Full Forms Hostile Convergence Test

Suite 47 is the closeout suite. It must combine signal-backed forms,
resource-backed forms, async validation, host facts, repeated submit attempts,
failure rollback, reset, branch restore, dynamic availability, complex patch
planning, attachment operations, controller-local multi-step flow, route-coupled
deferred posture, accessibility artifacts, presentation-order hint boundaries,
generated layout hint consistency, presentation lifecycle, secondary actions,
destructive/idempotent action policy, permission/admission changes, approval and
signature binding, schema drift, multi-actor source drift, resource family/line/
freshness preservation, resource transfer posture, resource effect profile
inheritance, visible branch selection, collaboration posture, replay, and
rematerialization into one canonical verification package.

## Verification Package Standard

Every broad form certification package must include:

- form declaration digest
- source authority digest
- field contract digest
- input adapter capability digest
- collection identity digest where repeated fields exist
- raw input digest where parse/format posture exists
- draft digest
- effective value digest
- semantic equality digest
- patch plan digest
- attachment operation digest where attachment fields exist
- availability dependency digest
- step declaration/progress digest where controller-local steps exist
- route-coupled step deferred digest where route-addressed behavior is requested
- admission policy digest where permissions, locks, approval, review, or
  signature affect the form
- actor/approval/signature/reason binding digest where regulated submit is
  declared
- action catalog digest where declared actions exist
- action readiness/admission digest where declared actions exist
- action lifecycle digest where actions execute
- collaboration posture digest where multi-actor behavior is declared
- lock/lease/branch/collaborator-event digest where collaboration is active
- schema/version/migration digest
- locale/timezone/unit/precision conversion digest where declared
- dirty/touched/visited digest
- validation digest
- message visibility and recovery digest
- accessibility artifact digest
- presentation order hint digest where declared
- generated layout hint digest where declared
- measured layout feedback digest where renderer feedback is declared
- measurement controller digest where DOM measurement is enabled
- presentation lifecycle digest where presentation lanes are declared
- presentation settlement acknowledgement digest where adapters participate
- readiness digest
- host fact digest
- submit plan digest
- submit lifecycle digest
- resource effect digest where resource-line authority is present
- resource family/member/line identity digest where resource-line authority is
  present
- resource request/lifecycle/freshness/status digest where resource-line
  authority is present
- resource delivery basis digest where resource-line authority is present
- resource transfer/download/upload/processing digest where resource-line
  authority is present
- resource effect profile digest where resource-line authority is present
- resource visible branch selection digest where branch-backed
- resource locus/lens proof digest where resource-line authority is present
- branch/snapshot basis digest where branch-bound
- resource verification package link or embedded digest where resource-line
  authority is present
- resource effect closeout-matrix row digest where resource-line authority is
  present
- reset/rollback digest
- diagnostics/history digest
- replay/restore digest
- boundary performance envelope
- typed denial or unavailable artifacts

Equivalent histories must match exactly except for fields explicitly declared
non-semantic.

## Performance And Cost Contracts

The forms surface must name and test these cost boundaries:

- field write breadth: `O(touched_field_locus + declared_normalization_cost)`
- raw input write breadth: proportional to the input field locus and declared
  parse/format posture, not to whole-form validation unless explicitly declared
- input adapter breadth: proportional to the adapter's declared event,
  formatting, host-fact, and replay capabilities
- repeated-field operation breadth: proportional to the affected item/collection
  locus plus declared dependent regions
- semantic patch planning breadth: proportional to declared dirty/equality
  regions, changed loci, attachment operation metadata, and explicitly declared
  broad replacement regions
- dirty summary breadth:
  - field-local when the caller asks for one field
  - declared section-local when the caller asks for one section
  - whole-form only when the caller asks for whole-form summary
- validation breadth:
  - field-local validators run for their field
  - cross-field validators run only for declared dependency regions
  - whole-form validators must be named as broad validators
- readiness breadth: derived from validation/readiness dependencies rather than
  rescanning all fields unless declared broad
- action readiness breadth: derived per declared action from its validation,
  admission, patch, schema, host, and effect dependencies
- availability breadth: derived from declared condition dependencies, with
  cycles denied and broad availability checks visibly broad
- controller-local step breadth: proportional to declared step fields, step
  actions, step messages, step progress, and dependent availability/admission
  regions
- admission breadth: proportional to declared actor, role, lock, approval,
  signature, review, reason, resource, and policy dependencies
- schema migration breadth: proportional to declared migration regions and
  changed schema fields, with broad draft migration visibly broad
- submit planning breadth: proportional to declared submit input, validation
  proof, host proof, admission proof, schema proof, semantic patch proof, and
  source/draft digests
- action planning breadth: proportional to the declared action input,
  validation proof, readiness proof, admission proof, schema proof, patch proof,
  idempotency proof, and effect binding
- resource-line form breadth: proportional to declared resource family/member/
  line identity, request posture, freshness/status reads, delivery basis,
  transfer posture, effect profile, visible branch selection, response-lens
  proof, and effect-locus proof
- collaboration breadth: proportional to changed actor, lock, lease, branch,
  remote source, comment, and declared merge/conflict regions
- diagnostics summary breadth: summary-shaped and not equivalent to full
  history materialization
- message summary breadth: field-local, section-local, or form-wide according
  to the requested target, with broad summaries visibly broad
- accessibility artifact breadth: proportional to the affected field, section,
  message, summary, action, and order-hint dependencies
- presentation lifecycle breadth: proportional to declared entry, interaction,
  availability, message, layout, action, canonicalization, resource drift,
  collaboration, attachment, navigation, and exit lane dependencies
- generated layout hint breadth: proportional to declared section, row, column,
  field, label, control, help, and message track metadata
- measured layout feedback breadth: proportional to changed measured regions,
  synchronized row/track groups, and renderer-declared throttling or batching
  policy; never to semantic form graph breadth
- DOM measurement breadth: proportional to observed renderer regions and
  coalesced snapshot groups, with explicit batching, throttling, deduplication,
  and no semantic signal invalidation

Any public API that can perform whole-form work must make that posture visible
in its name, return artifact, diagnostics, or performance envelope.

## Architectural Notes

- Field paths are not just strings. They are declared form loci that must carry
  enough proof for draft writes, validation dependency tracking, dirty
  summaries, submit planning, diagnostics, and type denials.
- Semantic equality is part of the form contract. Default equality is
  canonical-source equivalence, not raw text equality, touched status,
  referential equality, JSON stringification, or deep scans hidden behind a
  cheap API.
- Patch planning is a runtime artifact. Forms must be able to explain whether a
  submit will be an exact patch, a collection operation, an attachment
  operation, an omitted-field posture, a broad replacement, or a typed
  unavailable plan before any external/resource effect runs.
- Actions are runtime artifacts. A visible button may invoke an action, but the
  action's readiness, admission, idempotency, destructive posture, required
  patch, validation breadth, effect binding, lifecycle, and history belong to
  the form runtime. Action kinds are extension labels over a typed protocol,
  not a closed framework-owned enum.
- Input components do not have to be signal-native, but their guarantees do.
  A signal-native input can expose full runtime truth directly. A bridged or
  imperative input must declare which facts it can report and which behaviors
  are unavailable. The form runtime must never infer composition, selection,
  autofill, focus, display formatting, or replay fidelity from an opaque
  component.
- Nested JSON and other hard-to-normalize structures should use declared loci,
  resource response lenses, stable collection keys, and semantic equality
  digests. Falling back to whole-object replacement is legal only when the broad
  cost and rollback posture are explicit.
- Attachments are not plain JSON fields. File/blob identity, metadata, digest,
  upload state, staged addition/removal, replacement, cancellation, failure,
  rollback, and server canonicalization must be represented as attachment
  operations that can coordinate with resource or external effects.
- Raw input is user interaction truth, not domain truth. It exists to preserve
  ergonomic editing states, IME composition, paste/autofill provenance, parse
  failures, masks, and formatting without corrupting source, draft, or
  effective values.
- Repeated fields require stable item identity once declared. Indexes may be a
  display order, but they must not be the only identity used for dirty truth,
  validation artifacts, messages, diagnostics, or replay.
- The draft store is authoritative only for draft truth. It is not authoritative
  for source truth, effective truth, validation truth, or submit lifecycle.
- Effective value is a derived projection. Destroying and rebuilding derived
  form state from source, draft, host facts, and declarations must be possible.
- Validation artifacts are domain facts. UI components may render them, but UI
  components do not decide what validation means.
- Message artifacts are communication facts over validation, readiness, submit,
  resource, and host-capability truth. Stable codes and target loci are the
  durable contract; localized copy and design-system rendering are adapter
  concerns.
- Availability is graph truth. Enabled, disabled, hidden, readonly, required,
  omitted, blocked, and unavailable states must be derived from declared
  dependency regions and must explain draft preservation/clearing/freezing and
  submit omission policy. Arbitrary UI conditions without declared dependency
  breadth are out of spec.
- Admission is authority truth. Permissions, roles, locks, ownership, review
  state, approval gates, electronic signatures, reason-for-change prompts, and
  policy attestations must produce typed artifacts that bind to the exact
  capability and digest they authorize or deny.
- Collaboration is branch/resource/admission truth, not shared component state.
  Multi-actor forms must name whether they use locks, field leases,
  branch-per-actor drafts, optimistic merge, reviewer-comment-only posture, or
  typed unavailable collaboration; remote source changes cannot silently rewrite
  local drafts.
- Regulated workflow evidence belongs in the verification package. A caller
  should be able to prove who could edit, who could submit, what changed, why it
  changed, which approvals/signatures applied, what schema version was used,
  which attachments were included, and why the runtime accepted or denied the
  action.
- Schema evolution is not an adapter problem. Long-lived drafts must carry
  version posture, declared migration evidence, compatibility proof, or typed
  unavailable artifacts before submit.
- Accessibility-facing artifacts derive from form truth. First blocker,
  summary messages, described-by relationships, required/invalid/disabled
  posture, and focus targets must be consumable without the runtime executing
  DOM policy.
- Position is not form truth. The form runtime may expose adapter hints for
  declared presentation order, reading order, summary order, and focus order,
  but it must not own CSS layout, absolute coordinates, viewport collision,
  responsive placement, or visual measurement. If visual order changes domain
  meaning, it must be modeled as a declared form value or collection order
  locus, not inferred from pixels.
- Generated-layout hints are a legitimate product surface when they are typed,
  tokenized, and adapter-owned. They may describe row/column grouping,
  label/control/help/message tracks, consistent row heights, min heights,
  grow/wrap behavior, density, and responsive presentation tokens so a config
  generator can render a polished form without bespoke HTML.
- Measured-layout feedback is allowed and expected for realistic generated
  forms. Labels, translated copy, async messages, custom inputs, textareas,
  attachments, and help content may grow arbitrarily. A renderer may measure
  those regions, synchronize row tracks, reserve or release message space,
  stretch peer controls, and reflow rows. That feedback must stay in the
  renderer layout lane, be throttled/batched where needed, and never own
  semantic form truth.
- DOM measurement should be implemented as a renderer-owned measurement lane,
  not as form signal state. A renderer may use observers, animation-frame
  scheduling, font readiness, and viewport events to produce coalesced layout
  snapshots. Those snapshots may drive row heights, wrapping, stretching, and
  generated-layout reflow, but they must not invalidate form semantics or run
  validators, patch planners, action planners, or readiness derivations.
- Presentation lifecycle is user-visible settlement truth, not semantic form
  truth. It may keep an entry skeleton, field pending state, button spinner,
  section overlay, attachment progress, route handoff, or exit guard active
  until declared runtime artifacts and adapter acknowledgements settle. Defaults
  should prevent spinner flicker with delayed reveal/minimum duration, prevent
  stuck busy states with timeouts, and distinguish semantic fulfillment from UI
  settlement.
- Multi-step forms are split deliberately. Controller-local steps are forms
  truth: declared step ids, step groups, step readiness, step validation gates,
  step actions, progress summaries, skipped/optional/blocked posture, and
  dynamic step topology can ship here. Route-coupled steps are router truth:
  URL step authority, browser history, route guards, deep links, resume links,
  route-local resources, remount preservation, and branch-native speculative
  navigation must wait for router integration or emit typed deferred posture.
- Submit lifecycle belongs to runtime async truth. Form submit is an authored
  product lane over that substrate, not a new async subsystem.
- Resource-backed forms are consumers of resource line/effect truth. Any form
  shortcut that bypasses the resource effect envelope is out of spec.
- Resource-backed form field loci should collapse onto resource response-lens
  and effect-locus proof when the source topology is already known to the
  resource product surface. Forms may add field semantics, labels, grouping,
  validation, and readiness posture; they may not duplicate resource topology
  lowering.
- Branch-native resource submit posture must remain visible through form
  diagnostics. A failed merge, branch mismatch, unavailable inverse, or
  speculative branch disposal should explain the affected form field or region
  when the resource locus can be projected that far.
- Resource delivery and server canonicalization are not broad draft overwrite
  permissions. They must pass through source/draft/effective proof and expose
  whether a newer draft was preserved, rebased, denied, or reset.
- Host facts are admitted through host capability. Any direct browser read that
  changes readiness or validation without host capability evidence is out of
  spec.

## Sequencing Notes

This milestone belongs at roadmap Milestone 5.

The original roadmap placed forms before the API surface. The repository has
since completed the API/resource surface and branch-native resource effects
first. That does not invalidate the form milestone; it makes the form spec
stricter:

- forms still must consume composition, graph lifecycle, opaque identity, async,
  and host capability truth
- resource-backed forms must now also consume the completed resource line and
  resource effect model
- forms must not reopen resource semantics or define a second submit/effect
  engine
- controller-local multi-step forms belong here because they are section/action/
  readiness/presentation truth inside one form controller
- route-coupled multi-step forms are intentionally deferred until the router
  milestone because URL step authority, browser history, route guards,
  deep links, resume links, route remount preservation, and route-local
  resources must consume router truth rather than being invented by forms

Implementation should start with Phase 1 and proceed in order. Later phases are
not interchangeable because each one consumes proof from the prior phases.

## Self-Check

- Does this milestone solve a real structural problem?
  Yes. It prevents forms from becoming the second local store and second async
  engine that the roadmap explicitly warns against.
- Is the adversarial constraint precise and load-bearing?
  Yes. It names multi-section forms, resource-backed sources, drafts,
  validation, host facts, async submit, rollback, branch restore, replay, and
  rematerialization as one convergence problem.
- Does the milestone preserve crate authority boundaries?
  Yes. Runtime async, resource effects, host capability, graph publication, and
  diagnostics remain owned by their existing product lanes.
- Does the milestone define proof obligations?
  Yes. Each phase has required runtime, diagnostic, type, denial, and
  performance evidence.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The spec names declaration records, field loci, validation artifacts,
  submit plans, host fact posture, verification packages, and certification
  suites.
- Does the milestone belong in this roadmap sequence?
  Yes. It is still Milestone 5 by product meaning, even though resource/API
  work has already landed; the spec explicitly adapts to the stronger completed
  substrate.
