# Stage 2 seams: index

Design only; no source changed. Read at `43d28b12e9` (phase-2 milestone). Inputs:
`ownership/simulation/helpers-carry-types/*/simulation.md` (the seams left after the scripted
helper moves), the other agent's untracked `ownership/simulation/helpers-carry-types-draft-stack/`
run, and fresh reference counts from `scripts/query_split/module_coupling.coupling()` (plain
Python, no cargo).

- [`contracts.md`](contracts.md) — the 13 primary-graph and 4 domain-level "contract (3+ items)"
  seams, regrouped into six families (A–F) plus the layering artifacts.
- [`merges.md`](merges.md) — the three nest-or-merge seams. None is one responsibility.
- [`proposal-03-plan.md`](proposal-03-plan.md) — option A for the public context types.

## Decide first: which order governs

Every "lower module owns the contract" answer depends on the module order, and the three orders
in play disagree:

| pair | greedy (simulation) | draft stack | host-supertrait DAG (compiles today) |
|---|---|---|---|
| product_operation vs application_attempt | po below | po above | po above (`ProductOperationHost <: ApplicationAttemptHost`) |
| product_operation vs application_query | po below | po above | po below (`ApplicationQueryHost <: ProductOperationHost`) |
| application_attempt vs provider | attempt below | attempt below | attempt above (`ApplicationAttemptHost <: ProviderHost`) |
| application_runtime vs conditional_operation, application_installation | runtime above both | runtime below both | runtime above all (ruling a: it implements every host) |
| convergence_epoch vs managed_run | epoch below | epoch above | — (source has 0 refs managed_run → epoch) |

Two consequences:
- Six primary-graph seams and one domain seam exist **only** under the greedy order
  (`contracts.md` §0.1). Two of the three nest-or-merge seams are greedy artifacts too.
- The draft stack places `application_runtime` below `conditional_operation`,
  `application_invariant_preparation` and `application_installation`, which contradicts ruling
  (a). It needs that correction before it can govern.

The in-flight helper moves follow the greedy order. Three of their moves conflict with these
designs: `Suppression` → invariant_projection (family A wants it with `PriorAbsence`), the 66
managed_run units → convergence_epoch (`merges.md` §3), and `WorthQuerySelectedProductOperation`
→ application_program (ruled to keep its module; `proposal-03-plan.md` §8).

**Also in flight (seen untracked while these were written):** a `primary_graph::vocabulary`
module (one child per family, held below every family) and `ownership/vocabulary/*/simulation.md`.
Where these designs say "lower X into a new module", that family's `vocabulary::<family>` child
is the natural destination if the owner adopts it (family A's retention envelope →
`vocabulary::application_contribution`; family E's evaluation evidence →
`vocabulary::application_output_demand`). Its domain-level run leaves authorization →
primary_graph on the same foundational items `merges.md` §2 lowers (layout, principal, workflow
actor observation, entity identity; it skips them as "not vocabulary: struct"), and reports no
convergence_epoch seam, which matches `merges.md` §3.

## All seams designed

Refs are "up after the helper moves" from the greedy simulation; current counts are in the
documents. "Mechanical" means a non-expert agent can do it with `module_move.py`, rustc and the
listed checks; "careful" needs someone holding the whole design.

| seam (low → high) | refs / items | owner of the contract or move | kind | est. lines touched | who |
|---|---|---|---|---|---|
| **Family F** `execution_denial` in attempt→contribution, invariant_projection→contribution, execution_runtime→primary_graph, product_publication→primary_graph | 1 + 1 + 2 + 2 | `request_execution::execution_stop` | hand move of two refusal translators (simulation missed it: module-path reference) | ~60 | mechanical |
| **Family A** retention: output_lineage→contribution, attempt→contribution, invariant_projection→contribution, output_demand→contribution (retention part) | 22/8, 12/5, 9/7, 3/2 | new `pg::computation_retention` below the attempt; `ProducerEditionAdmission` (contribution) and `NativeCarriageReads` (attempt) | lower the retention envelope, invert two impls, one read port | ~700 incl. moved files | careful |
| **Family B** attempt→provider | 18/8 | `pg::application_attempt::provider_port::AttemptProviderPort`, impl in `provider/attempt_port.rs` | move provider half of registration to provider, then a `dyn` port; host supertrait change | ~600 incl. one 377-line file | careful |
| **Family C** attempt→invariant_projection | 10/5 | `pg::application_attempt::read_set::DecisionFactExpectations`, `MutationProjectionSource` | read port (`dyn`; iterator questions instead of `impl Iterator`) | ~100 | careful |
| **Family D** attempt→output_demand | 12/4 | registration field removed (after B); `WorkflowSettlementObservation` (attempt) | authority transfer + read port | ~80 | mechanical after B, one careful step |
| **Family E** output_demand→contribution (non-retention), causal_inspection→contribution | 5/3, 3/3 | `application_output_demand::denial` (bridge translators); `ProducerOutputFieldScope` (output demand) | hand move + lower evaluation evidence with a `dyn` field scope | ~200 | step 1 mechanical, step 2 careful |
| product_operation→contribution | 16/3 | — | greedy artifact; if kept: `GeneratedOutputProducer<S>` port (generic, recorded) | ~80 if kept | decide layering |
| application_discovery→attempt | 15/9 | — | greedy artifact; if kept: lower workflow fact readers (mostly settled by `merges.md` §2 workflow layout lowering) | ~10 files if kept | decide layering |
| application_output_demand→query | 10/4 | — | greedy artifact; if kept: lower `observed_source::source_identity` | large if kept (47 current refs) | decide layering |
| application_entry→attempt | 4/4 | — | greedy artifact (host DAG already puts entry above) | 0 | — |
| conditional_operation→contribution | 3/3 | — | greedy artifact; if kept: `ConditionalReadinessInstallation` port | ~40 if kept | decide layering |
| causal_inspection→contribution | 3/3 | — | greedy artifact and **no cycle in source**; settled by family E | 0 | — |
| provider_session→primary_graph (domain) | 22/9 | `merges.md` §2 lowering; closure-capture change in `graph_obligation`; `PrimaryReadSetCapture` port | payload pass-through removed, vocabulary lowered, one port | ~100 + §2 | careful |
| application_aftermath→primary_graph (domain) | 8/3 | idempotency binding to `application_outcome_identity` with `OperationScopeIdempotency` inversion; `CorrectionCausality` port | lower a value type, invert one impl, one port | ~150 | careful (second inversion likely) |
| application_aftermath→authorization (domain) | 8/3 | — | draft-stack artifact; if kept: `RecoveryAuthorityBasis` port | ~40 if kept | decide layering |
| execution_runtime→primary_graph (domain) | 5/3 | family F; `ProductSourceInvalidation` port; root wiring | port + wiring choice | ~30 | mechanical after F |
| product_operation↔application_attempt (nest/merge) | 19/18 | product_operation above the attempt | no merge: layering + workflow preparations move up + 3 hand moves | ~12 files, ~60 lines | careful (step 2 shared with proposal 03) |
| authorization↔primary_graph (nest/merge, domain) | 120/42 | new domain siblings `graph_layout`, `application_principal`; progression steps move into primary_graph | no merge: lower foundational vocabulary, move orchestration up | ~7,000 moved (mostly `schema_layout`) | careful; the layout move itself is mechanical once 3 refs are cut |
| convergence_epoch↔managed_run (nest/merge, domain) | 32/22 | `provider_session::graph_provider::convergence_port` | no merge, not a cycle: layering + move the provider port family (~600 lines) | ~650 | mechanical once the layering is recorded |
| product_operation→application_query, product_operation→application_installation (hub) | 17/10, 9/6 | proposal 03 option A | generic host parameter with `dyn` default, aliases in `application_runtime` | ~137 impl headers, ~38 blocks moved | careful at layering and public-method choice; rest mechanical |

## Where the classification is wrong

1. **Six primary-graph "contract" seams are greedy-layering artifacts**
   (product_operation→contribution, discovery→attempt, output_demand→query, entry→attempt,
   causal_inspection→contribution, conditional_operation→contribution), and so is
   aftermath→authorization at the domain level. The draft-stack run has none of them.
2. **causal_inspection↔contribution is not a cycle in the source** (contribution →
   causal_inspection: 0 references); the simulation creates it by moving 16 query units into
   causal_inspection.
3. **convergence_epoch↔managed_run is not a cycle in the source** (managed_run →
   convergence_epoch: 0 references; 119 the other way). The simulation creates the seam by moving
   66 managed_run units down. The real edge is provider_session → convergence_epoch (2 refs to
   `WorthQueryConvergenceDomainProvider`). It is a one-item contract, not a merge.
4. **product_operation↔application_attempt is not a merge**: under the draft stack it is one
   item (`HistoryProducerInvocation`), and the compiled host DAG already ranks product_operation
   above the attempt.
5. **Four "contract" seams are one family** (A: computation retention). Designing four
   contracts would give one vocabulary four owners.
6. **`execution_denial` was reported as unmovable** because callers import the module; its two
   functions name only `request_execution` vocabulary and move in one step (family F).
7. **The "hub or host order" bucket undercounts host-order seams**: attempt→provider's
   classification flips with the host DAG (`contracts.md` family B, direction check).

## Verification for every step

`cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`,
`cargo run --manifest-path tools/agent-context/Cargo.toml -- check`, the owning court crates,
`scripts/ci/check_workspace_rust_line_caps.sh dirty`, and `module_coupling.py report` at both
levels (refs up must fall; no new upward edge). Steps that add generics also run the llvm-lines
ratchet.
