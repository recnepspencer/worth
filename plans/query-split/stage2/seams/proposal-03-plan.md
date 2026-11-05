# Proposal 03, option A: implementation plan

Plan only. Source state `43d28b12e9`. Paths are relative to
`workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/`
unless they start with a crate name.

Option A (proposal 03): the public context types take a host type parameter with a `dyn`
default; impl blocks are written over any host that satisfies the owning module's host trait;
consumers name the context through composed public aliases in `application_runtime`; one erase
step turns a runtime-typed context into the `dyn` default where internal code expects it.

## 0. What option A must solve, measured

- **Bridges.** 36 call sites of the identity host's temporary `application_runtime()` remain
  (`grep -rn '\.application_runtime()' worth-query-execution/src`): product_operation 21
  (document history 6, query 8, branches 2, correction source 3, context 1, relational change
  delivery 1), application_attempt 5, application_installation 3, conditional_operation 2,
  application_entry 2, handler 1, invariant_projection 1, provider 1, authorization 2
  (`delegation_progression/replay.rs:192`, `elevation_progression/approval/replay.rs:145`).
- **Public accessors that hand out the concrete runtime.**
  `WorthQueryDocumentHistory::application()` (`product_operation/document_history/mod.rs:133`,
  the only fully `pub` one; `worth-query-publication` calls `.application()` 45 times in 26
  files: 21 `let runtime = …application()` uses and 11 `std::ptr::eq(…application(), request.application)`
  ownership checks), and `WorthQueryProgramApplicationRuntime::runtime()` plus its `Deref`
  (`application_installation/program/program_runtime.rs:58,279`). The other `application()`
  accessors are `pub(in crate::domain_computation::primary_graph)`
  (`context.rs:329` `into_parts`, `correction_source.rs:367`, `branches/owned_head.rs:47`,
  `branches/expected_source.rs:121`).
- **Higher modules' methods on a lower context.** `WorthQuerySelectedProductOperation` has 66
  inherent impl blocks today: product_operation 28, application_attempt 8,
  application_query 8, application_output_demand 7, application_discovery 5, entity_resolution 5,
  resolution 4, conditional_operation 1 (counted by header scan; the phase-1 hub map's 56 predates
  phase 2). 38 of them sit outside the type's module, and 18 of those 38 contain fully `pub`
  methods (23 methods; 3 more foreign blocks are `*_for_test` doors).

Inherent impls outside the defining module do not survive a later crate split (owner
ruling). So option A is implemented together with moving each foreign block to an owner that can
keep it: the type's own module (inherent), or the calling module as an extension trait.

## 1. The types that take the parameter

Parameter name `Host` (`WorthQueryDocumentHistory` already uses `H` for its history binding).
Spelling: `Host: ?Sized + 'r = dyn <OwningModule>Host<Schema> + 'r`, field `application: &'r Host`.

| type | file:line of the host field | today | needs the parameter because |
|---|---|---|---|
| `WorthQueryProductEntry<'r, S>` | `product_operation/context.rs:51` | `&'r dyn ProductOperationHost<S>` | creates the selected operation (chain below) |
| `WorthQuerySelectedProductOperation<'r, S>` | `product_operation/context.rs:34` | same | creates document history and correction sources; carries higher modules' methods |
| `WorthQueryApplicationProductBranches<'r, S>` | `product_operation/branches.rs:40` | same | creates owned heads and expected branches |
| `WorthQueryOwnedProductHead<'r, S>` | `product_operation/branches/owned_head.rs:42` | same | its runtime becomes the document history's (`application_query/document_history/mod.rs:148,170`) |
| `WorthQueryExpectedProductBranch<'r, S>`, `…Fork<'r, S>` | `product_operation/branches/expected_source.rs:25,34` | same | public, created by branches |
| `WorthQueryDocumentHistory<'r, S, H>` | `product_operation/document_history/mod.rs:114` | same | `pub fn application()` |
| `WorthQueryDocumentHistoryClosing<'r, S>` | `product_operation/document_history/disposal.rs:46` | same | created from the history |
| `WorthQueryProductHistory<'r, S>` | `product_operation/history.rs:19` | same | created from the branches (`history.rs:30,46` copies `self.application`) |
| `WorthQuerySelectedCorrectionSource<'r, S>` | `product_operation/correction_source.rs:58` | same | created from the selected operation; bridges at `correction_source.rs:370`, `disclosure.rs:199,209` |
| `NavigationProgress<'r, S>` (private) | `product_operation/document_history/navigation/custody.rs:64` | same | built from the history; stays private |
| `WorthQuerySelectedProgramOwner<'r, S>` | `application_installation/program/program_owner.rs:45` | `&'r dyn ApplicationInstallationHost<S>` | bridge at `program_owner/contract.rs:24` |
| `WorthQuerySupportedProgramHandle<'s, S, P>` | `application_installation/program/supported_program.rs:113` | same | bridge at `supported_program.rs:145` |
| `WorthQueryWorkflowVocabulary<'v, S, Spec>` | `application_installation/program/workflow_runtime/vocabulary.rs:26` | same | bridge at `vocabulary.rs:53` |
| `WorthQueryProgramApplicationRuntime<S, P>` | `application_installation/program/program_runtime.rs:36` | owns the concrete runtime in `Arc<WorthQueryProgramRuntimeBacking>` | `pub fn runtime()`, `Deref<Target = Runtime>` |

`WorthQueryProgramApplicationRuntime` owns its runtime by value, so its parameter is a sized
`R: ApplicationInstallationHost<S>` with **no** `dyn` default (a trait object cannot be held by
value; recorded as a ruling exception). Its alias fixes `R`. Under the draft stack
(`stack-primary_graph.json`) installation sits above application_runtime and this type would not
need the parameter, but that order contradicts ruling (a) — the runtime implements
`ApplicationInstallationHost` — so the plan follows the ruling.

Types that only take a host as a function parameter (`generated_materialization.rs:120,131`,
`document_history/current_head.rs`, `navigation/output_authority*.rs`, `group/mod.rs:72`,
`program_adoption/**`, `conditional_definition/**`, installation `signing.rs`, `assembly.rs`,
`workflow_assessment.rs`, `checkpoint_lineage.rs`) keep `&dyn`. They are internal and are
reached after the erase step.

### The creation chain (why the parameter cascades)

`runtime.branches()` (`BranchesOps::branches`, `branches.rs:50`, today
`application: self.product_operation_host()`, which erases) → owned head → the query host's
document-history attach (`application_query/document_history/mod.rs:170`,
`let application = head.application()`) → `WorthQueryDocumentHistory`. Likewise
`ContextOps::on_branch` (`context.rs:58`, also erasing) → entry → selected operation →
correction source and product history. Under option A each constructor in that chain stores
`self` (typed `Self`) instead of `self.product_operation_host()`, so a consumer calling through
the runtime gets runtime-typed contexts, and internal callers holding `&dyn` get the default.

## 2. Aliases and the public facade

New file `application_runtime/public_contexts.rs` (one responsibility: the consumer spelling of
each context type over the composed runtime), one alias per type:

```rust
pub type WorthQuerySelectedProductOperation<'r, S> =
    product_operation::WorthQuerySelectedProductOperation<'r, S, WorthQueryPrimaryGraphApplicationRuntime<S>>;
pub type WorthQueryDocumentHistory<'r, S, H> =
    product_operation::WorthQueryDocumentHistory<'r, S, H, WorthQueryPrimaryGraphApplicationRuntime<S>>;
pub type WorthQueryProgramApplicationRuntime<S, P> =
    application_installation::WorthQueryProgramApplicationRuntime<S, P, WorthQueryPrimaryGraphApplicationRuntime<S>>;
// … one per row of the table above except NavigationProgress
```

`worth-query-execution/src/facade/product.rs` (lines 34-46) and the primary-graph facade (`facade/product_operation.rs:7-54`, `mod.rs:97-101`), re-export the
aliases in place of the structs. This changes what the public name points at; it is the public
path, not a shim at an old path. Internal code keeps naming the struct, and gets the default.

Rust caveat: a type alias cannot be used as a constructor in patterns for tuple structs, and
defaults do not drive inference in expression position. Neither matters here: all fields are
private and every value comes from a constructor method.

## 3. Impl blocks: rewrite and move

**Every existing impl block must be rewritten**, not only the moved ones. An
`impl<'r, S> WorthQuerySelectedProductOperation<'r, S>` written today silently becomes an impl
for the `dyn` default only; a consumer's runtime-typed context would lose those methods. Each
block becomes `impl<'r, S, Host: ProductOperationHost<S> + ?Sized + 'r> … <'r, S, Host>` (or the
owning module's host bound, section 3.2). rustc does not report the omission at the definition;
it reports "no method" at the consumer. A grep check belongs in the step:
`grep -rnE 'impl<[^>]*> *[A-Za-z:]*(WorthQuerySelectedProductOperation|WorthQueryDocumentHistory|…)<[^>]*>' | grep -v Host`.

Counts of inherent blocks per type (header scan): SelectedProductOperation 66,
DocumentHistory 20, ProductEntry 4, ApplicationProductBranches 4, OwnedProductHead 2,
ExpectedProductBranch 2, ExpectedProductBranchFork 1, SelectedCorrectionSource 1,
ProductHistory 1, DocumentHistoryClosing 1, SelectedProgramOwner 3, ProgramApplicationRuntime 32
(installation 28, product_operation 3, output_demand 1). About 137 blocks in all.

### 3.1 Blocks in the type's own module (product_operation 28 + 34; installation 31)

Rewrite in place over `Host: ProductOperationHost<S> + ?Sized` (installation:
`R: ApplicationInstallationHost<S>`). Bodies that pass `self.application` to a function taking
`&dyn ProductOperationHost<S>` call `self.application.product_operation_host()` (the host's own
erase method, already on every host trait). Mechanical; rustc's errors list each site.

### 3.2 Blocks in other modules (38 on the selected operation, 4 elsewhere)

The destination depends on whether the module is above or below `product_operation`. The
three orders disagree (see `README.md`); the table uses the host-supertrait DAG that compiles
today (`ApplicationQueryHost <: ProductOperationHost <: ApplicationAttemptHost <: OutputDemandHost`,
`ConditionalOperationHost <: ApplicationQueryHost`), and lists the draft-stack answer where it
differs.

| module | blocks (pub methods) | files | relation (host DAG) | destination |
|---|---|---|---|---|
| application_query | 8 (8) | `retained_read.rs:141`, `readiness.rs:24`, `live/lease/open{,/governed}.rs`, `observed_source/workflow_capture.rs:16`, `observed_source/retained_dependencies{,/coverage,/comparison}.rs` | above | extension trait `SelectedQueryOps<S>` in `application_query`, `impl<'r, S, Host: ApplicationQueryHost<S> + ?Sized> SelectedQueryOps<S> for WorthQuerySelectedProductOperation<'r, S, Host>`. Draft stack: below (move up instead). |
| conditional_operation | 1 (1) | `clock_observation/product_admission.rs:8` (2 bridges) | above | extension trait `SelectedClockAdmission<S>` in `conditional_operation`, bound `Host: ConditionalOperationHost<S>` |
| application_attempt | 8 (1) | workflow transition, instance, proposal programs; `workflow/captured_input.rs:120`; `workflow/definition/{retirement,preparation}.rs` | below | move up into `product_operation/workflow_preparation/` as inherent blocks (`merges.md` §1 step 2) |
| application_output_demand | 7 (4) | `currentness{,/settlements,/consumed}.rs`, `completed_{set,outputs,read}.rs` | below | move up into `product_operation/output_currentness/` as inherent blocks, calling output-demand functions that take the parts |
| application_discovery | 5 (6) | `workflow/{read,enumeration,definition,history_page,subscription}.rs` | unranked (greedy and draft: below) | move up into `product_operation/workflow_discovery/` |
| entity_resolution | 5 (1) | `admitted_scope.rs:21`, `product{,/issued_scope,/admitted_mutation,/retained_scope}.rs` | unranked (greedy and draft: below) | move up into `product_operation/entity_scope/` |
| resolution | 4 (2) | `selected_product{,/admitted,/admitted/native}.rs` and a door | unranked (greedy: above; draft: below) | decide with the layering; default: move up |
| application_output_demand (program runtime) | 1 (1) | `original_outputs.rs` | below installation | move into installation's program demand |

"Move up" keeps the blocks inherent in the type's module (survives a split) and leaves the
lower module with functions over the parts it needs (`product()`, `application_basis()`,
`snapshot_lease`), so the lower module stops naming the context at all. "Extension trait" puts
the methods in the higher module; the trait is the higher module's, so it survives a split.

**Consumer-visible consequence.** The 9 `pub` methods in the two "above" rows (query 8,
conditional 1) become trait methods. A consumer calling them must import the trait. Two ways
to keep consumers unchanged; the owner chooses:
- export the extension traits through the consumer facades (`worth-query-host`'s prelude and the
  `facade/product.rs` re-exports), so `use worth_query_host::prelude::*` brings them in; or
- keep a thin inherent method on the context in product_operation that takes its higher host
  explicitly (`fn query(&self, host: &dyn ApplicationQueryHost<S>, …)`), which changes the call
  shape for consumers instead.
Count the consumer call sites of those 9 methods before choosing (`grep` the method names over
`worth-query-publication`, `worth-query`, testkits and courts).

**A note on the rulings.** For internal (non-`pub`) methods of higher modules, an explicit
higher-host argument (`&dyn ConditionalOperationHost<S>`) works with `dyn` and needs no type
parameter; ruling (a) prefers it. The type parameter is strictly needed only for the runtime
handed out by public accessors (problem 2) and for public methods that consumers call without a
host in hand. The plan therefore records the generic as justified by `WorthQueryDocumentHistory::application()`,
`WorthQueryProgramApplicationRuntime::{runtime, deref}` and the 9 public higher-module methods.

## 4. The erase step

Two forms, both owned by product_operation:

- **By value.** `impl<'r, S, Host: ProductOperationHost<S> + ?Sized + 'r> WorthQuerySelectedProductOperation<'r, S, Host> { pub(in crate::domain_computation::primary_graph) fn into_host_erased(self) -> WorthQuerySelectedProductOperation<'r, S> }`
  moves every field and replaces `application` with `self.application.product_operation_host()`.
  One per type in the table. Used where a runtime-typed context is handed by value to an
  internal function that takes the default (the runtime facade methods in
  `application_runtime/facade/product_operation.rs` and `facade/application_query.rs` that pass
  a context into an internal pipeline).
- **By reference (generic shell, concrete core).** Internal functions that take
  `&WorthQuerySelectedProductOperation<'r, S>` and are reached from a runtime-typed context
  become `fn f<Host: ProductOperationHost<S> + ?Sized>(selected: &WorthQuerySelectedProductOperation<'_, S, Host>, …)`
  whose body extracts the parts (`selected.product()`, `selected.application_basis()`,
  `selected.application().product_operation_host()`) and calls a non-generic core
  (iteration law 7). Do not make whole pipelines generic.

Instantiation budget: each generic shell compiles for `dyn ProductOperationHost<S>` and for
`WorthQueryPrimaryGraphApplicationRuntime<S>` per schema. Measure with `cargo llvm-lines` on the
worth-query-execution lib before and after; the ratchet (iteration law 7) must not move by more
than the shells.

## 5. Bridges, by class, and how each goes

| class | sites | resolution |
|---|---|---|
| public accessor returning the runtime | `document_history/mod.rs:133` | returns `&'r Host` |
| internal accessors returning the runtime | `context.rs:337` (`into_parts`), `correction_source.rs:370`, `branches/owned_head.rs:50`, `branches/expected_source.rs:124` | return `&'r Host`; their callers bound `Host` by what they call |
| query blocks reaching query-host methods | `query/{batch,preview,history_outputs}.rs` (8), `relational_change_delivery.rs:87` | become `SelectedQueryOps` (section 3.2): `Host: ApplicationQueryHost<S>` gives the methods directly |
| document-history navigation | `navigation/{eligibility,output_completion,progression}.rs` (4) | `Host` bound by the host the navigation calls (query host for output reads); these blocks are product_operation's own, so the bound names a higher host only if query stays above product_operation — then they move to query as an extension trait too |
| correction disclosure | `correction_source/disclosure.rs:199,209` | same as the query blocks (proposal 03's "correction disclosure") |
| conditional clock admission | `conditional_operation/clock_observation/product_admission.rs:44,55` | `SelectedClockAdmission` over `Host: ConditionalOperationHost<S>` |
| installation program owners | `supported_program.rs:145`, `workflow_runtime/vocabulary.rs:53`, `program_owner/contract.rs:24` | installation contexts take `Host: ApplicationInstallationHost<S>` |
| attempt, entry, handler, invariant projection, provider, authorization | 12 sites (listed in section 0) | not context-type problems: each is an upward host call; replace with a method on that module's own host trait (host-method work proposal 03 already lists), e.g. `retained_commit.rs:76` needs `application_query_host()` on the attempt's host |

## 6. Consumer site classes

| class | where | change |
|---|---|---|
| type mentions through the facade | `worth-query-publication` (`WorthQuerySelectedProductOperation` 65 mentions in 35 files across publication, `worth-query`, 3 testkits; `WorthQueryDocumentHistory` 5 files; correction source 4; owned head 4; expected branch 4; program owner 5) | none: the alias keeps the two- or three-parameter spelling |
| `WorthQueryProgramApplicationRuntime<S, P>` | 98 library mentions, 200 files outside (`worth-query-host`, `worth-query-publication`, 9 testkits, 4 courts, certification fixtures) | none through the alias; any site that writes the struct path directly (not the facade) must switch to the alias — grep for `application_installation::` in those crates |
| `.application()` then runtime methods | publication, 21 sites | none (`&Runtime` through the alias) |
| `std::ptr::eq(doc.application(), request.application)` | publication, 11 sites; `ptr::addr_eq(self, head.application())` at `application_query/document_history/mod.rs:148` | none for consumers; the internal one compares `&Self` with `&Host` and needs `ptr::addr_eq` (already used) |
| calls to the 9 public higher-module methods | to be counted (section 3.2) | trait import or explicit host, per the owner's choice |
| testkits and courts naming internal paths | `worth-query-testkit-execution-primary-graph-application-{attempt-2,contribution}`, `worth-query-testkit-host` | re-check: if they name `crate::…::product_operation::WorthQuerySelectedProductOperation` through a door they get the default; doors that hand a runtime-typed context need the alias |
| generic consumer code over contexts | none found by name; re-check after the rewrite | — |

## 7. Steps

Each step compiles alone and is committed as script output plus hand fixes.

1. **Layering decision** for query, conditional_operation, discovery, entity_resolution and
   resolution relative to product_operation (section 3.2 depends on it).
2. **Add the parameter** to the 14 types (one commit per type family: entry/selected/history/
   correction; branches/heads; document history/closing/navigation; installation's three plus
   the program runtime). Each existing impl block gets the `Host` parameter in the same commit
   (section 3 check). Constructors in the creation chain store `self` instead of
   `self.<module>_host()`.
3. **Aliases** in `application_runtime/public_contexts.rs`; switch the facade re-exports.
   Build `worth-query-publication`, `worth-query-host` and the testkits.
4. **Move the "below" blocks up** into product_operation (attempt, output demand, discovery,
   entity resolution, resolution if below). One module per commit; doors regenerated
   (`src_court_doors.py members`).
5. **Convert the "above" blocks** to extension traits in their modules (query, conditional
   operation); apply the owner's choice for the 9 public methods.
6. **Erase step** at the remaining sites rustc reports (by value at facade hand-offs, by
   generic shell at reference sites).
7. **Remove the 24 context bridges** (section 5, first seven classes); the other 12 go with the
   host-method work.
8. **Remove `application_runtime()`** from `runtime_host.rs:17` and its implementation
   (`application_runtime/host/identity.rs:49`) once the count is 0.
9. **Record each generic** (the 14 types, every generic shell) in the ruling record with its
   reason, as ruling (a) requires.
10. Verify: boundary-check, agent-context check, `cargo check --workspace --tests` over
    `workspaces/worth-query`, the publication and host courts, line caps (`dirty`), and the
    llvm-lines ratchet.

## 8. Size and risks

Size: about 137 impl headers rewritten, 14 struct definitions, 14 aliases, about 40 constructor
sites, 24 bridge sites, about 38 blocks moved (roughly 2,500 lines moved across ~40 files),
and re-checks in ~25 publication files and ~9 testkit files (proposal 03's estimate holds). Needs
careful design at steps 1 and 5; steps 2, 3 and 6 are mechanical with rustc's help.

Risks:
- Silent loss of methods on runtime-typed contexts if an impl block keeps the default
  (section 3). Gate step 2 on the grep check.
- `ContextOps` and `BranchesOps` are blanket-implemented for `HostRuntime: ProductOperationHost<S> + ?Sized`;
  returning `WorthQueryProductEntry<'_, S, Self>` from a `?Sized` `Self` is fine (the field is a
  reference) but the trait methods must not require `Self: Sized`.
- `Send`/`Sync`: the runtime-typed context is `Send` iff the runtime is `Sync`; the `dyn`
  default carries whatever auto-trait bounds the host trait object has (none today). Check the
  publication types that move contexts across threads.
- Monomorphization: every public method of the 14 types now compiles twice per schema. Keep
  their bodies as shells (law 7); measure.
- Interaction with the in-flight helper moves: both simulations move `WorthQuerySelectedProductOperation`
  out of product_operation (greedy: to `application_program`; draft: to `application_attempt`).
  Under the rulings the public type keeps its module; exclude it (and `SharedSelectedProductOperation`,
  `SelectedQueryOrigin`, the document-history identity and denial types the draft run moves to
  query) from the scripted moves until this plan lands.
