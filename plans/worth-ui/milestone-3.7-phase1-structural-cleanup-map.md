# Milestone 3.7 Phase 1: Structural Cleanup Map

> **Status:** Phase 1 closeout artifact
>
> **Parent spec:** [milestone-3.7.md](./milestone-3.7.md)
>
> **Mechanical inventory:** `worth_ui_certification::topology::audit_milestone_37_structural_inventory`
>
> **Parity tests:** `worth-ui-certification/tests/milestone_37_structural_inventory_audit.rs`

Phase 1 freezes cleanup scope and produces the map later phases must consume.
It is evidence work, not runtime feature implementation.

## Concept Freeze

### In scope for milestone 3.7

Structural cleanup of shipped 3.1–3.6b runtime lanes across:

- `worth-ui-runtime`
- `worth-ui-inspection`
- `worth-ui-query-binding`
- `worth-ui-certification`

### Frozen out (do not land under 3.7 cleanup cover)

- Milestone 3.8 allocation receipts
- Incremental replanning semantics
- Scroll/portal churn invalidation
- Continuous interaction measurement
- Any new user-facing Worth UI behavior

## Target Directory Skeleton

Prescriptive end-state for phases 2–8.

### `worth-ui-runtime/src/`

```
lib.rs                          # pub mod facade only

facade/
  mod.rs                        # lifecycle-grouped re-exports (no runtime::*)
  entry/                        # app, builder
  lifecycle/                    # bootstrap, support_inventory
  inspection/                   # bridge + boundaries/
  admission/ declaration/ graph/ query_binding/

runtime/
  mod.rs                        # lane routers only
  launch/                       # host shell + seal/derive/build
  replacement/                  # compare → classify → narrow → match → reconcile → rebind → stage
  planning/                     # allocation_planning, plan_topology, plan_equivalence, plan_inspection
  activation/                   # staging, frame gate, atomic swap
  execution/                    # handle_allocation, execution_plan_input, lanes/
  host_observation/             # move runtime/host_*.rs here
  source_ingress/
  tests/                        # colocate *_boundary_tests.rs

evidence/
  mod.rs                        # routing only
  construction/                 # identity, slice, expansion helpers
  measurement/                  # basis, projection, dependency
  planning/                     # neighborhood, constraint_set, constraint_propagation, allocation_solve
  obligation/
  layout_operator/

graph/                          # tighten handoffs to evidence/planning
host/                           # measurement observation intake only
```

### `worth-ui-inspection/src/`

Keep contract vocabulary ownership. Single `facade/scope_inventory.rs` authority.

### `worth-ui-query-binding/src/`

Preserve sealed reference shape:

`WorthUiQueryBindingSubsystem::bootstrap().prerequisites()`

### `worth-ui-certification/src/topology/`

Group audits by boundary family: `admission/`, `declaration/`, `graph/`, `inspection/`, `measurement/`, `planning/`, `legacy/`.

## Intended Public Facade Shape

Lifecycle-ordered exports replace alphabet soup and `pub use crate::runtime::*`.

1. Definition — `WorthUi`, `WorthUiBuilder`, capability registration
2. Freeze outputs — `CapabilitySnapshot`, declaration/graph read surfaces
3. Runtime handoff — `WorthUiRuntimeHost`, `WorthUiRuntimeLaunch`
4. Replacement lifecycle — ordered transition types only
5. Inspection bridge — narrow runtime bridge, not full `worth_ui_inspection` mirror
6. Host contract — translator types from `worth_ui_host_contract`

**Removed from facade root:** `runtime::*`, `certify_*_suite`, full inspection vocabulary mirror.

## Proof-Flow Grammar

### App freeze

`register → validate → freeze_declaration → admit_graph → build_indexes → bootstrap_inspection → WorthUiApp`

### Runtime launch

`seal_launch_artifact → derive_launch_execution_plan → build_active_runtime_state → WorthUiRuntimeHost`

### Replacement lifecycle (canonical order)

`admit → compare → classify_impact → narrow → identity_match → node_replace → reconcile → query_compare → query_rebind → prepare_lowering → stage_activation → lower_plan → plan_allocation → allocate_handles → activate/swap/execute`

### Evidence expansion

`preflight_expansion_classify → [admitted] assemble_slice → materialize → UiEvidenceExpansion`

### Inspection

`admit_relevance → classify_dispatch_lane → boundary_inspect → assemble_receipt`

### Certification (support authority)

`worth_ui_certification::topology::<suite> → consume_retained_evidence → run_scenario → report`

## Classified Findings

| ID | Failure mode | Path / surface | Owner phase | Closeout evidence |
|----|--------------|----------------|-------------|-------------------|
| F-01 | facade_leakage | `worth-ui-runtime/src/facade/mod.rs` wildcard `runtime::*` | 2 | export diff + compile-fail deep import |
| F-02 | facade_leakage | `facade/mod.rs` lifecycle-ungrouped pub use blocks | 2 | export diff |
| F-03 | facade_leakage | `facade/mod.rs` mirrors `worth_ui_inspection` | 2, 6 | export diff + direct inspection import |
| F-04 | facade_leakage | `facade/mod.rs` exports `certify_*_suite` | 2, 6 | export diff + cert topology entry |
| T-01 | topology_sinkhole | `runtime/` 99 same-level `.rs` files | 3 | directory skeleton |
| T-02 | topology_sinkhole | `runtime/` 15 `host_*` files mixed with production | 3, 5 | `host_observation/` lane |
| T-03 | topology_sinkhole | `evidence/` 90 same-level `.rs` files | 4 | transition-family subdirs |
| T-04 | topology_sinkhole | `runtime/` 47 `*_boundary_tests.rs` at root | 3, 6 | colocated test topology |
| H-01 | helper_swamp | `evidence/mod.rs` inline construction helpers | 4, 7 | `construction/` relocation |
| H-02 | helper_swamp | `facade/runtime_bridge.rs` bootstrap bag | 2, 7 | lifecycle split diff |
| A-01 | authority_mixing | `evidence/allocation_planning_certification.rs` → `planning_pair_for_certification_suite` | 6 | cert scenario relocation |
| A-02 | authority_mixing | `runtime/allocation_planning/certification_fixture*` | 6 | cert fixture fence |
| A-03 | authority_mixing | `facade/app.rs` index build + inspection dispatch | 6 | inspection bridge split |
| A-04 | authority_mixing | `lifecycle/support_inventory.rs` `PHASE3_*` naming | 6 | inventory rename |
| O-01 | function_overload | `runtime/host.rs` (518 lines) | 3, 7 | decomposition + parity test |
| O-02 | function_overload | `facade/app.rs` (487 lines) | 2, 7 | dispatch classifier split |
| O-03 | function_overload | `matching/worth_ui_identity_match_graph_builder.rs` (455 lines) | 7 | collect/classify/build split |
| O-04 | function_overload | `evidence/measurement_inspection_receipt.rs` (432 lines) | 4, 7 | named projector split |
| S-01 | file_size | 8 runtime files >400 lines | 7 | split or exemption |
| B-01 | test_bypass | `runtime/mod.rs` `#[cfg(test)] pub use touch_origin_certification_support` | 6 | visibility fence |
| B-02 | test_bypass | `runtime/mod.rs` `runtime_test_modules` pub(crate) use | 6 | scope narrowing |
| B-03 | test_bypass | `certification/topology` imports `certify_*` via runtime facade | 6 | typed cert consumption |

## Rejected Cosmetic Candidates

Not load-bearing; excluded from critical inventory:

| ID | Reason rejected |
|----|-----------------|
| COSMETIC-01 | `worth-ui-query-binding` already uses sealed narrow facade — positive reference, not a blocker |
| COSMETIC-02 | `worth-ui-inspection` receipt/query subdir topology is already lifecycle-shaped |
| COSMETIC-03 | Individual long type names in capability registry — naming verbosity without boundary impact |
| COSMETIC-04 | Test file count in `graph/` subdirs — colocated under real owners, not runtime-root swamp |
| COSMETIC-05 | `worth-ui-certification/topology` flat audit module list — organizational preference until phase 6 grouping |

## Function Decomposition Targets

### `WorthUiRuntimeHost::plan_allocation` (O-01)

| Step | Target function |
|------|-----------------|
| Orchestration | `plan_allocation` |
| Evidence collection | `collect_planning_measurement_basis` |
| Classifier | `classify_constraint_set_admission` |
| Verification | `admit_planning_inputs`, `lower_execution_plan_for_planning` |
| Denial receipt | `build_constraint_set_denial_planning` |
| Success receipt | `WorthUiAllocationPlanner::plan_from_lowered_input` |

### `WorthUiApp::inspect` (O-02)

| Step | Target function |
|------|-----------------|
| Orchestration | `inspect` |
| Evidence collection | `collect_inspection_authority_generation` |
| Classifier | `classify_inspection_dispatch` → `InspectionDispatchLane` |
| Transition | `dispatch_lane.inspect` per `inspection/boundaries/*` |
| Receipt | `UiInspectionReceipt::from_*` |

### `preflight_evidence_expansion` (H-01)

Move to `evidence/construction/expansion.rs` with `classify_expansion_admission` decision table.

## Receipt And Counter Construction Points

| Transition | Receipt | Counters |
|------------|---------|----------|
| Launch | `build_active_runtime_state` | lifecycle init |
| Replacement compare | `WorthUiRuntimeArtifactComparison` | comparison counters |
| Impact classify | `WorthUiReplacementImpactClassification` | impact counters |
| Activation stage | `WorthUiPendingActivation` / denial | staging counters |
| Plan allocation | `WorthUiAllocationPlanning` / denial | planning counters (including denial path) |
| Handle allocation | `WorthUiRuntimeHandleAllocation` | allocation counters |
| Inspection | `UiInspectionReceipt` | cost receipt at boundary |
| Evidence expansion | `UiEvidenceExpansion` | outcome enum as proof |

## Phase Consumption Index

| Phase | Consumes finding IDs |
|-------|---------------------|
| 2 | F-01, F-02, F-03, F-04, H-02, O-02 |
| 3 | T-01, T-02, T-04, O-01 |
| 4 | T-03, H-01, O-04 |
| 5 | T-02, graph/planning/host seams from skeleton |
| 6 | F-03, F-04, A-01–A-04, B-01–B-03 |
| 7 | O-01–O-04, S-01, H-01 |
| 8 | All IDs — closeout bundle + 3.8 readiness |

## Verification

```bash
cargo test -p worth-ui-certification milestone_37_structural_inventory -- --nocapture
```

Expected:

- Parity: two scans produce identical findings and digest
- Critical blocker set: all IDs F-01 through B-03 present
- Failure modes: all seven modes represented
- Cosmetic rejection: COSMETIC-01..05 absent from critical set
- Concept freeze: no 3.8 scope tokens in finding summaries

## Preserved Behavior

- Shipped 3.1–3.6b runtime behavior unchanged by phase 1
- Query-owned truth boundaries
- Host adapters as mechanics translators only
- Runtime-owned diagnostics posture
- Compatibility re-exports may persist until phase 2 narrows facade