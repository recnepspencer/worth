# Runtime subsystem proposal 01: output production

Source: `hub-WorthQueryPrimaryGraphApplicationRuntime.{json,md}` in this directory, made by
`scripts/query_split/hub_map.py` at the end of phase 1 (`--expand runtime runtime.product_runtime
runtime.primary_provider`). Analysis only; nothing here has moved.

## The hub, in numbers

`WorthQueryPrimaryGraphApplicationRuntime<Schema>` has 270 production impl blocks and 606 methods.
By module: application_query 67 blocks, application_contribution 33, product_operation 29,
application_attempt 27, authorization 20, application_runtime's own children 51 (inbound_occurrence
16, certification_controls 14, inbound_publication 6, ...), application_output_demand 9, and a
tail of 14 more modules.

Most methods touch only shared state: `runtime.product_runtime` (101 methods),
`runtime.authority_identity()` (62), `installed_schema` (45), `runtime.primary_graph()` (42),
`runtime.primary_provider.graph` (26). Those are the runtime's identity and wiring, not a
subsystem. Seventeen field clusters have a dominant module (at least half of the methods touching
each field sit in one module). Ranked by owned-field accesses:

| cluster (dominant module) | methods | owned-field accesses | fields |
|---|---|---|---|
| **application_contribution: output production** | **54** | **64** | `runtime.output_demands`, `runtime.recovered_outputs`, `runtime.output_producer_routes`, `runtime.output_readiness_routes`, `installed_producers`, reads of `runtime.product_runtime.recovered_root_authority` |
| application_runtime::inbound_occurrence | 21 | 26 | `runtime.inbound_custody`, `runtime.inbound_verifiers`, `primary_provider.lookup_completed_inbound()` |
| application_attempt | 19 | 25 | `program_support`, `runtime.external_effect_transport`, `runtime.installed_operations`, commit lanes |
| application_query | 20 | 21 | `managed_indexes`, readiness schema token, source admission, live delivery |
| application_runtime::inbound_publication | 10 | 18 | `runtime.transport_completion_custody`, inbound completion publication marks |
| application_runtime::operation_control | 15 | 15 | attempt operation control, product activations, owner cleanup reservation |
| authorization | 13 | 13 | `runtime.authorization` |

The heaviest cluster is output production. (The two inbound clusters together, 31 methods and 44
accesses, are the natural proposal 02; they already call each other through 12 named methods.)

## 1. The state it takes

`WorthQueryOutputProduction<Schema>`, owned by the runtime, takes:

| field today | type | access in the 54 methods |
|---|---|---|
| `runtime.output_demands` | `WorthQueryOutputDemandRegistry` (application_output_demand) | 43 methods: 32 read, 11 mutate (registry insert/settle/queue) |
| `runtime.recovered_outputs` | `WorthQueryRecoveredOutputs` (application_output_demand) | 3 read |
| `runtime.output_producer_routes` | `WorthQueryInstalledOutputProducerRoutes` (application_contribution) | 3 read |
| `runtime.output_readiness_routes` | `WorthQueryInstalledOutputReadinessRoutes` (application_contribution) | 3 read |
| `installed_producers` | `WorthQueryInstalledApplicationProducerRegistry<Schema>` (application_contribution) | 10 read |
| `runtime.next_output_producer_attempt` | `AtomicU64` | 6 methods over 3 modules: moves with it (producer attempt ordinals are this subsystem's) |

Four of these sit today on the non-generic `WorthQueryApplicationRuntimeCore`; `installed_producers`
sits on the generic runtime. The subsystem is therefore generic over `Schema` and owned by the
runtime struct, not by the core. `recovered_root_authority` stays on the product runtime and is
read through the host trait below.

## 2. Its operations

The 54 methods, by where they live today (3,775 lines in all):

- application_contribution, 33: demand admission (`admit_output_demand_with_source*`,
  `admit_settled_at_observation`, `admit_recovered_output_demand`,
  `admit_required_fresh_successor`), progression (`advance_caller_pass`, `advance_validated_ready`,
  `refresh_output_demand`, `refresh_ready_output_demand`, source custody and source recovery),
  checkpoint delivery (`advance_unpublished_checkpoint`, `deliver_output_checkpoint_with_finish`,
  `finish_output_checkpoint`), and producer selection and scheduling
  (`select_output_producer_with_remaining_core`, `schedule_selected_output_producer_with_truth`,
  `evaluate_current_output_readiness_core`, `certify_ready`).
- application_output_demand, 13: the required-work queue (6, `registry::required_work::queue`),
  join observation (4) and settlement reads.
- application_runtime::certification_controls, 3: output-demand certification controls.
- product_operation, 3: generated-output restoration and admission with observation room.
- application_checkpoint, 1: checkpoint capture of the demand registry sections.
- application_runtime, 1: installation wiring.

## 3. Traits it needs and offers

**Needs from the rest of the runtime** (one host trait owned by the subsystem, implemented by the
runtime; uses counted in the 54 methods):

- identity: `authority_identity()` (17), `installed_schema` (8), `primary_graph()` (2);
- graph and provider: `primary_provider.graph` (9) and the four readiness hooks
  `take_delayed_output_readiness_delivery`, `output_comparison_completed`,
  `take_failed_output_readiness_evaluation`, `take_readiness_snapshot_pressure` (1 each),
  `application_branch_commit_lane_for_occurrence` (1);
- product runtime: `on_branch` (5 calls), `root_identity()` (3), `admit_product_occurrence`,
  `default_occurrence`, `owner`, `recovered_root_authority` (1 or 2 each);
- other runtime operations it calls: `output_demand_resource_profile` (2), `publication`,
  `granular_invalidation_installation`, `own_publication_is_selected`,
  `select_application_read_observation_with_request`, `publish_restoration`,
  `hold_world_snapshot_pressure_for_test` (1 each); `bridge` (1 field read).

These group into three narrow traits: `OutputProductionIdentity` (authority, schema, graph),
`OutputProductionProduct` (branch, occurrence and root reads on the product runtime) and
`OutputProductionPublication` (publication, invalidation installation, own-publication
selection, read observation). The readiness hooks are provider methods and can be passed as the
provider itself.

**Offers**: 23 call sites from outside the cluster, all through methods that stay public on the
runtime as one-line delegations until their callers are moved: from the rest of
application_contribution (`select_output_producer_with_remaining_core` ×4,
`advance_unpublished_checkpoint` ×2, `deliver_output_checkpoint_with_finish` ×2,
`admit_output_demand_with_source_admitted` ×2, `certify_ready` ×2,
`evaluate_current_output_readiness_core` ×2, and six single calls), from application_checkpoint
(`capture_application_checkpoint_with_sections`) and from product_operation
(`restore_generated_output_in_phase`, `admit_with_observation_room`).

## 4. How the runtime composes it

```rust
pub struct WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    // identity and wiring stay
    installed_schema: ..., runtime: WorthQueryApplicationRuntimeCore, ...
    // replaces runtime.output_demands, runtime.recovered_outputs, runtime.output_producer_routes,
    // runtime.output_readiness_routes, runtime.next_output_producer_attempt, installed_producers
    output: WorthQueryOutputProduction<Schema>,
}
```

The subsystem's methods are `impl<Schema> WorthQueryOutputProduction<Schema>` taking
`&self` plus `host: &impl OutputProductionHost<Schema>`; the runtime implements the host traits
in application_runtime and keeps today's public methods as delegations
(`self.output.admit_output_demand_with_source(self, ...)`). Because the subsystem owns its state
and borrows the host only through traits, the borrow of `self.output` and of the host never
overlap mutably: every field access today is `&self` (reads, or interior mutation through the
registry's own locks), so no `&mut` split is needed.

**Correction (accepted):** the subsystem type lives in `application_contribution`, not in
`application_output_demand`. It holds the producer registry and the routes, which are
`application_contribution`'s, beside the demand registry and recovered outputs, which are
`application_output_demand`'s and sit below it; placing it in `application_output_demand` would
have added an output_demand → contribution edge. Its registry-only test doors become methods on
`WorthQueryOutputDemandRegistry` itself.

## 4a. The rulings that bind every runtime proposal

- **Facade (ruling A).** The runtime's 196 public non-test methods stay as inherent methods in
  `application_runtime`. Each is a short composition over the subsystem that owns the work and
  holds no domain logic (composition law 13). Internal forwarders are temporary: each is deleted
  once its library callers move, and at the end no runtime method exists only for
  library-internal callers.
- **Test doors** move onto the subsystems; courts reach them through a sealed door that hands
  the subsystem over, with callers registered as usual.
- **Host traits.** Every module below `application_runtime` owns a host trait listing only what
  it needs from above; the runtime implements each one in `application_runtime`. Context types
  and parameters that name the runtime hold or take `&'r dyn <Module>Host<Schema>` (or
  `&'r mut dyn`), so every public type keeps its name, definition and module, and its body is
  compiled once in its own module (iteration law 7). A generic `R: <Module>Host<Schema>` is used
  only where a trait object cannot work (generic methods that cannot take `&mut dyn FnMut` or be
  split, `Self: Sized` returns, a demonstrated per-row or per-fact hot path); each such generic is
  recorded with its reason, and a public type made generic gets its composed alias in
  `application_runtime`.
- **Internal runtime methods** become extension traits in their own modules, one per original
  visibility, implemented for every host, so no method is widened.
- Conversions are scripted per module, bottom-up, with rustc's suggestions supplying the trait
  imports, and doors regenerated after each.

Under these rulings the output-production subsystem is: the state struct
`WorthQueryOutputProduction<Schema>` in `application_contribution` (the six fields), the
contribution host trait (identity, product, publication and readiness needs), and the
extension traits holding the 54 methods, which reach the state through the host's
`output_production()` accessor.

## 5. Migration steps

1. Add `WorthQueryOutputProduction<Schema>` with the six fields and constructors; install it
   from the existing installation path; the old fields become reads through it
   (`self.runtime.output_demands` → `self.output.demands`), by script over the 58 files that
   reach them.
2. Add the host traits and implement them on the runtime.
3. Move the 13 application_output_demand methods first (they already sit with the registry),
   then the 33 application_contribution methods, cluster by cluster (admission, progression,
   checkpoint delivery, selection), each as `impl WorthQueryOutputProduction`, keeping
   delegations on the runtime for the 23 outside call sites.
4. Move the remaining 8 (certification controls, product_operation, checkpoint capture,
   installation wiring) or leave them as runtime methods calling the subsystem.
5. Retire delegations whose callers have moved; regenerate item and member doors
   (`src_court_doors.py items`, `members`) for the court paths that named runtime methods.
6. Re-measure with `module_coupling.py` and `hub_map.py`.

Each step compiles on its own and is committed as script output plus hand fixes, as in phase 1.

## 6. Risks

- **Doors.** Courts reach runtime methods through `*_for_test` member doors; moving a method
  off the runtime breaks those door paths. Keep the runtime delegation until the doors are
  regenerated, and run the courts after each cluster.
- **Field reach outside runtime methods.** `output_demands` is named on 128 lines in 58 files,
  most as `self.runtime.output_demands` split over lines, some through a borrowed core passed
  into helpers. Step 1 rewrites them all to the new field before any method moves.
- **Core versus runtime ownership.** Four fields leave the non-generic core for the generic
  runtime. Any code that holds only `&WorthQueryApplicationRuntimeCore` (authorization's
  admission does) and reads them needs the subsystem passed explicitly; none of the 54 methods
  does today, but the step-1 script must report any such site.
- **Interior mutation.** Eleven methods mutate the registry through its locks. Moving them does
  not change lock order, but the queue methods (`required_work::queue`) and checkpoint delivery
  both lock the registry; keep each moved method's lock scope byte for byte.
- **Shared identity reads.** `authority_identity()` is read 17 times; if the host trait returns
  it by value it must stay the same `Copy` type, so the subsystem never caches a stale identity.
- **Generic surface.** Making the subsystem generic over `Schema` duplicates monomorphisations
  only where the runtime already was generic; no new public type is exposed outside the crate
  (the subsystem is `pub(in crate::domain_computation::primary_graph)`).

## Summary maps of the other two hubs

**`WorthQuerySelectedProductOperation<'runtime, Schema>`** (`hub-WorthQuerySelectedProductOperation.*`):
56 impl blocks, 132 methods, spread over application_attempt 7, application_query 7,
application_output_demand 6, application_discovery 5, entity_resolution 5, resolution 3,
conditional_operation 1 and twelve product_operation children (23 blocks). Its six fields
(`application`, `product`, `application_basis`, `producer`, `selection_origin`,
`selected_program`) are touched directly by only 9 methods, all in `product_operation::context`;
every other method reaches the selection only through accessors. It is a context object, not a
state hub: the split is to move each module's impl blocks to that module as extension traits (or
free functions over `&WorthQuerySelectedProductOperation`), with no state to divide.

**`WorthQueryPrimaryGraphBootstrap<Schema>`** (`hub-WorthQueryPrimaryGraphBootstrap.*`): 12 impl
blocks, 25 methods, almost all in `bootstrap`'s own children (advancement_publication,
seed_batches, managed_source_initialization, publication_target, checkpoint, truth_partition,
unique_seed_values) plus typed_bootstrap, handler, invariant_projection and application_runtime
(one each). Six small clusters: seed rows (`rows`, `entity_rows`, `relation_rows`) owned by
seed_batches, commit counters and recovered publication by advancement_publication, source
initialization by managed_source_initialization, entity keys by bootstrap/typed_bootstrap. Four
cross-cluster calls (`take_source_initialization` ×2, `validate_publication_target`,
`commit_seed_batch_once`). A builder with phases rather than a hub; a seed-batch subsystem
(rows plus commit counters) is the only split worth making, and only after the runtime.
