# Stage 2 ownership analysis: summary

Analysis only; no code moved. A unit is a type with every inherent impl and every trait impl the crate holds for it
(an impl of one of this crate's traits may stay with a higher trait). Scripts: `scripts/query_split/ownership_units.py`,
`ownership_analysis.py` (plain Python over the source). Full output per parent: `primary_graph/analysis.{json,md}`,
`domain_computation/analysis.{json,md}`; the stacks are `stack-*.json`.

## `domain_computation::primary_graph` (79 modules)

Now: largest component 57; 2,063 references against the draft stack (the coupling report's greedy layering has 2,313 against its own order).

### Draft stack, bottom to top

1. **identity_keys_and_refusals**: `application_branch`, `entity_key`, `principal_key`, `aggregate_identity`, `output_binding_identity`, `output_family_identity`, `admitted_capture`, `exact_basis_access`, `index_maintenance_budget`, `request_local`, `request_allocation_control`, `record_reservations`, `selected_read_indexes`, `read_observation`, `program_occurrence`, `publication_request_denial`, `value_decode`, `content`, `entity_resolution_denial`, `denial`, `initial_schema_denial`, `resolution_denial`
2. **schema_and_native_facts**: `schema_layout`, `index_currency`, `structural_walk`, `derived_source`, `source_facts`, `aggregate_observation`, `root_path_selection`, `component_selection`, `source_read_verification`, `source_read_equivalence`, `fact_verification_observation`, `aggregate_projection`
3. **principals_and_resolution**: `authenticated_principal`, `observations`, `freshness`, `principal_currentness_capture`, `resolution`, `entity_resolution`, `source_disclosure`
4. **graph_root_and_workflow_facts**: `workflow`, `root`, `granular_invalidation`, `managed_bridge`, `expression`
5. **attempt_and_commit**: `application_attempt`, `live_delivery`, `provider`, `application_checkpoint`, `application_program`
6. **query**: `access_strategy`, `application_query`, `application_discovery`, `application_invariant`
7. **contribution_and_output_demand**: `output_reuse`, `output_lineage`, `invariant_projection`, `application_output_demand`, `application_contribution`
8. **operation_execution**: `application_entry`, `operation_body`, `handler`
9. **product_operations**: `product_activation`, `product_operation`
10. **runtime_and_installation**: `typed_bootstrap`, `bootstrap_publication`, `bootstrap`, `invariant_installation`, `application_runtime`, `application_resource`, `conditional_operation`, `application_invariant_preparation`, `application_installation`
11. **surface**: `causal_inspection`, `merge_unique_values`, `index_refresh`, `ordinary_read`, `facade`

Greedy and meaning disagree for 10 modules: `content` (stack rank 17, greedy 63), `initial_schema_denial` (stack rank 20, greedy 60), `source_read_equivalence` (stack rank 31, greedy 69), `expression` (stack rank 45, greedy 65), `application_attempt` (stack rank 46, greedy 67), `application_program` (stack rank 50, greedy 21), `application_discovery` (stack rank 53, greedy 32), `product_activation` (stack rank 63, greedy 25), `application_runtime` (stack rank 69, greedy 35), `application_resource` (stack rank 70, greedy 40). Each placement's one-line justification is in the analysis file.

### Units referenced against the stack

- 333 units (1961 references): **158 movable**, **175 behaviour-coupled**, 0 pinned
- 49 functions, constants and modules (102 references)
- widest impl spread: `WorthQueryPrimaryGraphApplicationRuntime` (310 impl blocks in 19 modules), `WorthQuerySelectedProductOperation` (65 impl blocks in 9 modules), `WorthQueryPrimaryGraphBootstrap` (18 impl blocks in 5 modules), `WorthQueryPrimaryGraphIntegrationHandle` (13 impl blocks in 4 modules), `WorthQueryProgramApplicationRuntime` (37 impl blocks in 3 modules)

### Ruling sheet

| pair (lower ↔ higher) | refs low→high / high→low | against the stack | movable / coupled / fn | nest cost | default | note |
|---|---|---|---|---|---|---|
| `workflow` ↔ `application_attempt` | 386 / 200 | 200 | 11 / 15 / 8 | 18590 lines, 13 refs exposed | nest workflow under application_attempt (18590 lines); its other users then go through application_attempt | agree: workflow facts are the attempt's vocabulary in practice; every crossing type (attempt denial, observed fact, realized effect) is held by attempt behaviour, and only 13 references from other lower modules would point up |
| `application_output_demand` ↔ `application_contribution` | 197 / 198 | 198 | 6 / 10 / 1 | 19445 lines, 63 refs exposed | move the 6 movable units by script; a contract owned by application_output_demand for the 11 items that stay behaviour-bound in application_contribution | owner ruling: `WorthQueryOutputDemandDenial`/`Kind` (161 of the 198) are output demand's refusals declared under `contribution::producer::demand`; make `application_output_demand` their owner and put the one bridge-denial classification behind the contract |
| `application_attempt` ↔ `application_contribution` | 142 / 75 | 142 | 9 / 9 / 3 | 67706 lines, 765 refs exposed | move the 9 movable units by script; a contract owned by application_attempt for the 12 items that stay behaviour-bound in application_contribution | owner ruling: `WorthQueryAdvancementPhase`/`WorthQueryAdvancementDenial` (83 of 142) are request-execution custody used from query, attempt and resolution; carve request execution out of contribution below attempt, or keep the contract |
| `application_query` ↔ `application_contribution` | 153 / 56 | 56 | 1 / 5 / 3 | 55420 lines, 170 refs exposed | move the 1 movable units by script; a contract owned by application_query for the 8 items that stay behaviour-bound in application_contribution | same seam as the row above (advancement phase 37 of 56) plus the demand denials; settles with those two rulings |
| `application_query` ↔ `product_operation` | 118 / 89 | 118 | 18 / 22 / 1 | 55420 lines, 329 refs exposed | move the 18 movable units by script; a contract owned by application_query for the 23 items that stay behaviour-bound in product_operation | `WorthQuerySelectedProductOperation` (65 impl blocks in 9 modules) is a hub; its impls must regroup before any move; the 18 history denials and selections move by script |
| `application_attempt` ↔ `provider` | 59 / 147 | 59 | 14 / 10 / 3 | 67706 lines, 235 refs exposed | move the 14 movable units by script; a contract owned by application_attempt for the 13 items that stay behaviour-bound in provider | agree; `WorthQueryPrimaryGraphProvider` and committed-application handles stay with provider behind the contract |
| `application_contribution` ↔ `product_operation` | 38 / 145 | 38 | 5 / 2 / 0 | 40345 lines, 653 refs exposed | move the 5 movable units by script; a contract owned by application_contribution for the 2 items that stay behaviour-bound in product_operation | agree; `SharedSelectedProductOperation` (20) moves by script |
| `application_attempt` ↔ `invariant_projection` | 35 / 111 | 35 | 4 / 9 / 0 | 67706 lines, 606 refs exposed | move the 4 movable units by script; a contract owned by application_attempt for the 9 items that stay behaviour-bound in invariant_projection | agree; consumed-output evidence and verification stops are the seam |
| `output_lineage` ↔ `invariant_projection` | 98 / 45 | 45 | 4 / 4 / 2 | 19001 lines, 49 refs exposed | move the 4 movable units by script; a contract owned by output_lineage for the 6 items that stay behaviour-bound in invariant_projection | agree; the same consumed-output seam as the row above, from the lineage side |
| `application_contribution` ↔ `application_runtime` | 72 / 67 | 72 | 0 / 1 / 0 | 40345 lines, 813 refs exposed | move the 0 movable units by script; a contract owned by application_contribution for the 1 items that stay behaviour-bound in application_runtime | not a pair to move: all 72 are the runtime hub `WorthQueryPrimaryGraphApplicationRuntime` (310 impl blocks, 28,109 lines, in 19 modules); needs its own ruling (below) |

### Estimate

- every default above: largest component **56**, **1109** references against the stack; 61 units by script, 85 items by hand, 1 nest
- plus every movable unit anywhere: component **55**, **858** references; 158 units by script, 238 items by hand
- plus the five widest hubs behind contracts: component **54**, **537** references

## `domain_computation` (20 modules)

Now: largest component 12; 534 references against the draft stack.

### Draft stack, bottom to top

1. **identity_time_and_observation**: `runtime_time`, `artifact_identity`, `application_outcome_identity`, `evidence_material`, `evidence_identity`, `execution_observation`, `application_contract_admission`
2. **execution_resources**: `execution_runtime`, `invalidation_equivalence_control`, `product_publication`, `artifact_owner`, `operation_binding`, `domain_evidence_binding`
3. **sessions_authority_and_aftermath**: `provider_session`, `managed_run`, `authorization`, `application_aftermath`
4. **primary_graph**: `primary_graph`
5. **convergence_and_attempts**: `convergence_epoch`, `execution_resource_attempt`

Greedy and meaning disagree for 2 modules: `runtime_time` (stack rank 0, greedy 6), `application_contract_admission` (stack rank 6, greedy 0). Each placement's one-line justification is in the analysis file.

### Units referenced against the stack

- 136 units (479 references): **68 movable**, **68 behaviour-coupled**, 0 pinned
- 23 functions, constants and modules (55 references)
- widest impl spread: `WorthQueryExecutionRuntime` (9 impl blocks in 5 modules), `WorthQueryPrimaryGraphApplicationRuntime` (310 impl blocks in 4 modules), `WorthQueryProductRuntime` (32 impl blocks in 2 modules), `WorthQueryApplicationRuntimeCore` (32 impl blocks in 2 modules), `WorthQueryExecutionInstallationAuthority` (6 impl blocks in 2 modules)

### Ruling sheet

| pair (lower ↔ higher) | refs low→high / high→low | against the stack | movable / coupled / fn | nest cost | default | note |
|---|---|---|---|---|---|---|
| `authorization` ↔ `primary_graph` | 321 / 669 | 321 | 28 / 42 / 19 | 30136 lines, 15 refs exposed | nest authorization under primary_graph (30136 lines); its other users then go through primary_graph | the tool says nest; the size (30,136 lines) and meaning argue instead for an authorization contract owned below primary_graph; owner ruling |
| `provider_session` ↔ `primary_graph` | 657 / 61 | 61 | 8 / 14 / 0 | 16886 lines, 311 refs exposed | move the 8 movable units by script; a contract owned by provider_session for the 14 items that stay behaviour-bound in primary_graph | agree |
| `application_aftermath` ↔ `primary_graph` | 41 / 244 | 41 | 15 / 6 / 1 | 6004 lines, 4 refs exposed | move the 15 movable units by script; a contract owned by application_aftermath for the 7 items that stay behaviour-bound in primary_graph | agree; 15 of 21 units move by script |
| `execution_runtime` ↔ `primary_graph` | 17 / 253 | 17 | 3 / 5 / 2 | 5743 lines, 73 refs exposed | move the 3 movable units by script; a contract owned by execution_runtime for the 7 items that stay behaviour-bound in primary_graph | agree; the execution and product runtimes are hubs at this level too |
| `provider_session` ↔ `managed_run` | 215 / 18 | 18 | 9 / 2 / 0 | 16886 lines, 16 refs exposed | move the 9 movable units by script; a contract owned by provider_session for the 2 items that stay behaviour-bound in managed_run | agree |
| `provider_session` ↔ `authorization` | 78 / 3 | 3 | 1 / 1 / 0 | 16886 lines, 231 refs exposed | move the 1 movable units by script; a contract owned by provider_session for the 1 items that stay behaviour-bound in authorization |  |
| `product_publication` ↔ `primary_graph` | 70 / 9 | 9 | 0 / 4 / 1 | 622 lines, 8 refs exposed | move the 0 movable units by script; a contract owned by product_publication for the 5 items that stay behaviour-bound in primary_graph |  |
| `managed_run` ↔ `primary_graph` | 30 / 38 | 30 | 3 / 2 / 0 | 17529 lines, 33 refs exposed | move the 3 movable units by script; a contract owned by managed_run for the 2 items that stay behaviour-bound in primary_graph |  |
| `operation_binding` ↔ `provider_session` | 13 / 12 | 13 | 3 / 4 / 0 | 1752 lines, 6 refs exposed | move the 3 movable units by script; a contract owned by operation_binding for the 4 items that stay behaviour-bound in provider_session |  |
| `execution_runtime` ↔ `managed_run` | 1 / 13 | 1 | 0 / 1 / 0 | 5743 lines, 32 refs exposed | move the 0 movable units by script; a contract owned by execution_runtime for the 1 items that stay behaviour-bound in managed_run |  |

### Estimate

- every default above: largest component **11**, **26** references against the stack; 42 units by script, 43 items by hand, 1 nest
- plus every movable unit anywhere: component **8**, **18** references; 68 units by script, 100 items by hand
- plus the five widest hubs behind contracts: component **8**, **18** references

## Reading

- Unit moves alone do not collapse either cycle. The pair defaults, every movable unit and the five widest hubs together
  still leave a 54-module component in primary_graph: the remaining references are behaviour-coupled units and functions
  spread over many edges, not a few heavy seams.
- The deciding fact is impl spread. `WorthQueryPrimaryGraphApplicationRuntime` carries 310 impl blocks (28,109 lines) in 19
  modules; under the unit rule it cannot leave `application_runtime` or move to another crate until those impls are
  regrouped, for example as extension traits owned by each module over a runtime contract. That is the first owner ruling;
  `WorthQuerySelectedProductOperation` (65 blocks, 9 modules) and `WorthQueryPrimaryGraphBootstrap` are the next.
- Second: the owner of three vocabularies now declared in the wrong module: output-demand refusals (in contribution),
  advancement phase and denial (in contribution, used from below attempt), and the attempt refusal and observed-fact
  family (in attempt, used from workflow).

