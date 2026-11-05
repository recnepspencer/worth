# Ownership rerun at the phase-2 checkpoint (b78fc0c6b6)

Analysis only; no code moved. Same scripts as the stage-2 ownership analysis
(`ownership_units.py`, `ownership_analysis.py`, plain Python over the source), run on the current
tree at b78fc0c6b6 (the in-flight milestone fixes touch only facades and door paths, which no
unit classification reads), after every module below the runtime took a host trait and 188 of
221 runtime bridges went.

Two stacks per level:
- **greedy**: `module_coupling.py`'s current layering, one module per layer (the layering the
  `check` ratchet will freeze);
- **draft**: the stage-2 draft meaning stack, updated only for modules that changed:
  `application_inbound` sits below `application_attempt`, `runtime_host` sits in the identity
  layer, `workflow` is gone (nested in phase 1) and `request_execution` is at the bottom of
  execution resources.

The stack files are beside this README. Full output is in `<stack>/<level>/analysis.{json,md}`.

## Counts and estimates

Components are given as "largest component / references against the stack". "Every movable
unit moves" means every default ruling is taken and every movable unit moves whole to its
owner.

| level | stack | units | movable | behaviour-coupled | pinned | now | every movable unit moves | plus the five widest hubs behind contracts |
|---|---|---|---|---|---|---|---|---|
| `primary_graph` | greedy | 391 | 179 | 212 | 0 | 59 / 1504 | 58 / 618 | 58 / 598 |
| `primary_graph` | draft | 352 | 167 | 185 | 0 | 59 / 1248 | 58 / 530 | 57 / 483 |
| `domain_computation` | greedy | 198 | 89 | 109 | 0 | 12 / 581 | 8 / 103 | 8 / 103 |
| `domain_computation` | draft | 136 | 67 | 69 | 0 | 12 / 432 | 8 / 18 | 8 / 18 |

Compared with the stage-2 analysis (primary graph against the draft): units went from 333 to
352, movable from 158 to 167 and behaviour-coupled from 175 to 185. The estimate after every
movable unit moves was component 55 with 858 references; it is now 58 with 530.

Removing the runtime coupling cut the references the moves leave behind by 328, but it made
few more units movable. The component barely moves, because what holds the units in place is
behaviour of the types themselves, not the runtime.

At the domain level, every movable unit moving brings the component from 12 to 8 and the
references from 432 to 18 (draft) or from 581 to 103 (greedy).

## Why the behaviour-coupled units stay

A unit is behaviour-coupled when its declaration (fields, derives) or its own impls name items
that sit above its owner's destination.

- In the primary graph, against greedy, there are 212 coupled units holding 846 references.
- Their upward needs come from 216 inherent impls, 162 declarations and 23 trait impls.
- The upward items they need are 926 structs, 606 functions, 254 enums and 184 traits.

So most units are pinned by small helper functions and refusal enums that live in high modules,
not by the types they hold.

The top direct blockers are shown with the units each one holds in place and the references
behind them. Against greedy, they are almost all one-line classification or encoding helpers:
- `application_query::managed_index::provider_failure::equal_cause` (140 units);
- `application_contribution`'s readiness and native-field denial constructors (about 120 units each);
- `denial_value::mismatch_or`;
- `required_wave::capacity_denial`;
- `one_shot::denial::*`;
- `output_lineage::native_output_witness::*`.

```
== greedy primary_graph coupled 212 refs 846
 needs come from: {'declaration': 162, 'trait impl': 23, 'inherent impl': 216}
 upward item kinds needed: {'struct': 926, 'enum': 254, 'fn': 606, 'trait': 184, 'const': 12, 'type': 13, 'static': 4}
 top direct blockers (units held / refs held):
    application_query::managed_index::provider_failure::equal_cause 140 651
    application_contribution::conditional::output_readiness_contract::denial 121 487
    application_contribution::native_field_contract::field_denial 119 498
    application_contribution::conditional::output_readiness_contract::typed::field_denial 118 483
    application_contribution::producer::denial_value::mismatch_or 113 481
    application_contribution::producer::denial_value::declaration::encode_declared 110 475
    application_contribution::producer::registry::semantic_edition::posture_code 108 457
    application_contribution::producer::demand::progression::required_wave::capacity_denial 101 436
    application_query::observed_source::source_identity::material_storage::admit_footprint_comparison 100 429
    application_query::one_shot::denial::authorization_denial 97 415
    application_query::one_shot::denial::denial 97 415
    application_query::resource_lifecycle::lifecycle_count::release 96 423
    application_contribution::partitioned_computation::incremental::observed::observe_retained_full_tree 95 428
    output_lineage::native_output_witness::overflow 95 429
    output_lineage::native_output_witness::retirement::native_metadata 95 429
== greedy domain_computation coupled 109 refs 299
 needs come from: {'declaration': 87, 'inherent impl': 100, 'trait impl': 8}
 upward item kinds needed: {'struct': 555, 'enum': 132, 'type': 4, 'fn': 237, 'static': 4, 'trait': 132, 'const': 3}
 top direct blockers (units held / refs held):
    primary_graph::application_query::managed_index::provider_failure::equal_cause 61 183
    primary_graph::application_attempt::denial::request_read::relational_read_kind 54 164
    primary_graph::provider::relational_execution_denial::query_read_execution_kind 54 164
    provider_session::graph_provider::read_row::pair_vector_bytes 44 106
    provider_session::graph_provider::read_row::retained_arc_str_bytes 44 106
    provider_session::graph_provider::read_row::value_by_key 44 106
    primary_graph::application_contribution::conditional::output_readiness_contract::denial 43 129
    provider_session::graph_provider::call_identity::NEXT_GRAPH_CALL_AUTHORITY 43 102
    primary_graph::provider::relational_execution_denial::relational_execution_kind 41 93
    primary_graph::schema_layout::installation_primitives::planned_field_locator 41 125
    primary_graph::application_attempt::observation::observe_field_value 40 122
    primary_graph::application_attempt::workflow::actor::record::actor_record_corruption 40 122
    primary_graph::application_attempt::workflow::actor::record::decode_actor_components 40 122
    primary_graph::application_checkpoint::facts::payload::array_storage_bytes 40 98
    primary_graph::application_checkpoint::facts::payload::chunk::bytes 40 98
== draft primary_graph coupled 185 refs 758
 needs come from: {'declaration': 146, 'inherent impl': 178, 'trait impl': 26}
 upward item kinds needed: {'trait': 157, 'fn': 438, 'enum': 230, 'struct': 768, 'const': 13, 'static': 3, 'type': 9}
 top direct blockers (units held / refs held):
    application_query::managed_index::provider_failure::equal_cause 78 459
    application_contribution::producer::denial_value::mismatch_or 70 376
    application_contribution::conditional::output_readiness_contract::denial 69 338
    product_operation::document_history::step::NEXT_DOCUMENT 69 320
    product_operation::document_history::step::mint_document_ordinal 69 320
    application_resource::provider::ingress::backing::CHUNK_BYTES 67 309
    application_resource::provider::ingress::backing::lock 67 309
    product_operation::document_history::current_head::confirm_in_execution 67 350
    product_operation::document_history::current_head::confirm_in_native 67 350
    application_contribution::conditional::output_readiness_contract::typed::field_denial 66 332
    application_contribution::native_field_contract::field_denial 66 332
    application_resource::installation::denial 66 308
    application_resource::storage::lock 66 308
    application_runtime::operation_control::attempt_registration::arm 65 323
    application_runtime::operation_control::attempt_registration::clear 65 323

```

## Reading

The cycle is held by data types and the small functions their impls call: refusals,
observations, identities and evidence. Moving every movable unit whole leaves the primary
graph's component at 58.

Breaking it needs the blocker functions themselves to move down to the refusal or evidence
type they classify (most are pure functions of lower data), or contracts at the heaviest
seams. The per-pair ruling sheet in each `analysis.md` lists the defaults.
