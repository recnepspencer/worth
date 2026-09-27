# Milestone 3.10.1 Phase 6 Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Objective

Delete every predecessor route and make the remaining ownership mechanically exact.

The closed ordinary lifecycle is:

`WorthUi::app() -> WorthUiApplicationBuilder -> WorthUiApp -> WorthUiActiveApplicationSession -> execute_mounted_frame`

No public alias, feature-gated inherent method, forwarding wrapper, or intermediate
authority constructor may create another ordinary entry. Certification keeps access to
framework-turn mechanics only through the existing non-production support gate.

## Boundary Review

### Current authority

- Phase 5 exactly inventories product facade modules and re-exported symbol names, but
  it does not inventory inherent public methods on re-exported runtime types.
- `WorthUiActiveApplicationSession::execute_framework_turn` remains a public inherent
  method under `cfg(any(test, feature = "certification-support"))`. Feature unification
  therefore restores an ordinary-looking midpoint entry.
- Thirty-three certification files use that method. Runtime-internal callers are
  legitimate, but external certification callers need support authority rather than a
  production method.
- `WorthUiBuilder` is the concrete builder and `WorthUiAppBuilder` is a public alias,
  while the governing inventory names `WorthUiApplicationBuilder` as canonical. The
  current surface therefore has two real names and one documentary name.
- `facade::source` mixes DSL-owned authored input and diagnostic types with
  runtime-owned filesystem, watcher, settlement, and candidate-ingress types.
- `source/mod.rs` and `source/lower/mod.rs` are private runtime aggregation modules.
  Their current contents perform runtime admission, artifact assembly, dependency
  derivation, and inspection projection over a sealed DSL handoff; they do not parse
  or reinterpret authored source. Their Phase 1 `split` rows can close only after the
  public mixed source facade is split and the inventory records this private posture.
- Existing boundary enforcement already denies DSL dependencies on runtime, Query,
  host, inspection, product-session, and mounted owners and denies known DSL language
  identifiers inside runtime. It does not yet enforce the inverse ownership rule or
  exact callable lifecycle reachability.

### Real callers

- Ordinary product compile journeys use `WorthUi::app`, builder registration/freeze,
  app launch, `execute_mounted_frame`, typed outcome continuations, and shutdown.
- File and watcher journeys use runtime-owned source ingress through
  `worth_ui::facade::source`.
- Rust-authored journeys use DSL-owned input types and should import them from
  `worth_ui_dsl`, their real owner.
- Framework-turn, raw lane execution, plan availability, raw graph/runtime
  observation, and midpoint mounted preparation callers are certification or
  runtime-internal callers.
- Host mechanics belong to `worth-ui-host-contract`; cross-crate fixtures belong to
  `worth-ui-test-support`.

### Destination authority

- `WorthUiApplicationBuilder` is the only public builder type. The
  `WorthUiBuilder` and `WorthUiAppBuilder` aliases are absent.
- `execute_mounted_frame` is the only ordinary mounted-frame start on the active
  session.
- Framework-turn execution is crate-private production machinery. External
  certification uses a sealed extension trait exported only by
  `worth-ui-test-support`.
- `facade::source` publishes runtime source transport and candidate-ingress values
  only. Authored input, spans, and compiler diagnostics are consumed from
  `worth_ui_dsl`.
- Cross-audience inherent methods on product lifecycle types are either:
  - made crate-private;
  - exposed through a named audience extension trait owned by the relevant facade; or
  - retained only when the callable manifest proves they are lifecycle operations or
    immutable product observations with a named ordinary caller.
- Runtime source aggregation remains private and is recorded as runtime admission and
  artifact composition, not as a language facade.
- Ownership checks scan all production source regardless of feature configuration.

No compatibility alias or forwarding wrapper is part of the destination.

## Public DX Contract

```rust
use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline, WorthUi,
    WorthUiApplicationBuilder,
};
use worth_ui_dsl::{
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};

fn configure(builder: WorthUiApplicationBuilder) -> WorthUiApplicationBuilder {
    builder.with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_component("workspace.component.dashboard"),
    ]))
}

let app = configure(WorthUi::app()).freeze()?;
let mut session = app.launch()?;
let outcome = session.execute_mounted_frame(
    UiMountedFrameRequest::all_bound_surfaces(),
    UiPresentationDeadline::at_tick(1),
    0,
    |_| {},
)?;

match outcome {
    UiMountedFrameOutcome::Published(receipt) => observe(receipt),
    UiMountedFrameOutcome::Unchanged(receipt) => observe(receipt),
    UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => recover(rejected),
    UiMountedFrameOutcome::InFlight(in_flight) => continue_from(in_flight),
    UiMountedFrameOutcome::PresentationIndeterminate(frame) => reconcile(frame),
    UiMountedFrameOutcome::AdmissionDenied(denial) => retry(denial),
    UiMountedFrameOutcome::RetentionDenied(denial) => retry(denial),
    UiMountedFrameOutcome::CompletionDenied(denial) => reject_foreign(denial),
    UiMountedFrameOutcome::Reconciled(receipt) => observe(receipt),
}
```

The product path does not import `worth_ui_runtime`, `worth_ui_test_support`, raw
framework-turn types, or a compatibility builder name.

## Implementation Batches

### Batch 1 - Exact callable and predecessor manifests

1. Add `plans/worth-ui/milestone-3.10.1-phase-6-callable-surface.toml`.
2. Inventory:
   - every ordinary lifecycle type;
   - every inherent public method reachable through those types;
   - named audience extension methods;
   - every forbidden predecessor symbol, alias, wrapper, and midpoint entry;
   - feature posture and named real caller for every retained callable.
3. Extend the syntax-aware Phase 5 audit to follow product reexports to runtime
   definition sites and compare inherent/trait callables exactly.
4. Fail on unmanifested methods, public type aliases, public wildcard forwarding,
   undocumented production features, and callable predecessor names.
5. Add hostile unit fixtures for inherent-method growth, renamed forwarding,
   aliases, and feature-gated routes.

### Batch 2 - Canonical builder and source-owner split

1. Rename the concrete builder to `WorthUiApplicationBuilder`.
2. Remove `WorthUiBuilder` and `WorthUiAppBuilder`; migrate all real callers.
3. Remove DSL-owned authored input, compiler diagnostic, source span, and semantic
   handoff input reexports from `worth_ui::facade::source`.
4. Migrate authored callers to `worth_ui_dsl`; retain only runtime transport and
   candidate-ingress values in the product source audience.
5. Close the Phase 1 `facade/mod.rs`, `facade/source.rs`, runtime source root, and
   runtime source lowering-facade transition rows with current exact ownership and
   fingerprints.

### Batch 3 - Certification-only framework-turn authority

1. Make `execute_framework_turn` crate-private and remove its feature-gated public
   inherent form.
2. Add one `WorthUiFrameworkTurnCertificationExt` support trait under
   `worth-ui-runtime::certification_support`.
3. Re-export the trait only through `worth-ui-test-support`.
4. Migrate external certification callers; keep runtime-internal callers on the
   crate-private owner method.
5. Compile-fail ordinary imports and calls while compile-passing the canonical
   certification route.

### Batch 4 - Callable audience closure

1. Classify every public method on `WorthUi`, `WorthUiApplicationBuilder`,
   `WorthUiApp`, `WorthUiActiveApplicationSession`, and their affine continuation
   types.
2. Keep ordinary lifecycle operations inherent.
3. Move graph, admission, source-ingress, host-measurement, and inspection operations
   behind named audience traits when they are real product capabilities.
4. Move certification-only raw plan, lane, runtime, host, and query-residue
   observations behind support authority.
5. Make intermediate prepared-application, graph mutation, plan, allocation,
   mounted-attempt, publication, and host-session constructors unreachable.

### Batch 5 - Mechanical ownership and reachability enforcement

1. Extend the runtime language-owner audit with inverse DSL authority ownership.
2. Extend boundary-check configuration so DSL cannot define or import runtime
   generation, execution-plan, allocation, mounted, publication, or host-session
   authority identifiers.
3. Add production-source reachability that scans default and non-default
   production-feature source.
4. Add exact one-entry validation for ordinary mounted execution.
5. Add runtime subsystem matrix and no-catch-all checks to the integrated topology
   owner rather than a parallel script.
6. Add hostile fixtures for aliases, forwarding wrappers, feature flags, test
   constructor leakage, inverse ownership, and guard mutation.

### Batch 6 - Consolidated compiler and behavioral proof

1. Add Phase 6 compile cases to the canonical runner without increasing its two Cargo
   sessions or adding a fixture workspace.
2. Preserve one app-only compile-pass journey and add one certification-support
   compile-pass journey.
3. Run application, source ingress, replacement, mounted, inspection, topology, and
   product suites.
4. Run default and certification builds, strict clippy, boundary-check,
   agent-context, line-cap, composition, and test-evidence review.
5. Close the Phase 6 proof ledger only after every row has fresh evidence.

## Proof Strategy

- Syntax/AST audits prove ownership, visibility, aliases, cfg posture, callable
  reachability, and exact manifest membership.
- Cargo dependency and boundary checks prove DSL direction independently of source
  naming.
- Compile-fail cases prove that ordinary downstream callers cannot recover removed
  aliases, call framework turns, mint intermediate authority, or enter a midpoint.
- Compile-pass cases prove the ordinary mounted journey and the gated certification
  journey.
- Behavioral suites prove that moving access paths does not change lifecycle,
  replacement, publication, recovery, or inspection semantics.
- Runner instrumentation proves the evidence remains consolidated into two Cargo
  sessions.

## Causal Reopen Rules

- Any lifecycle-type reexport or public method change reopens callable exactness,
  one-entry proof, alias hostility, and ordinary DX.
- Any builder rename or app-construction change reopens all product compile journeys
  and application preparation behavior.
- Any source facade or runtime source-root change reopens DSL/runtime ownership,
  file-versus-Rust parity, watcher ingress, and source diagnostics.
- Any certification extension change reopens test-authority isolation and every
  framework-turn certification caller.
- Any feature or cfg change reopens feature hostility and production reachability.
- Any compile inventory or runner change reopens target ownership and the two-session
  guarantee.

## Non-Goals

- Phase 7 steady-frame cost certification.
- Phase 8 documentation and later-milestone insertion closeout.
- New Query, snapshot, rebind, service, intent, interaction, or appearance semantics.
- A compatibility deprecation window, prelude, or broad advanced runtime audience.
- Crate extraction or a second compile-contract workspace.
