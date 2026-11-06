# Boundary closure requires owner handoff

Raw lane: funding-merge-boundary.log, exit1. Owner probes:
funding-merge-boundary-owner-audit.log and funding-merge-boundary-handoff.json.
All17 findings are retained; no enforcement configuration is changed.

P-A keeps the six explicitly granted Native topology sites from the latest
brief. These are not excused by the merge. In addition, the following six
source courts need their current owner's migration or a scope ruling:

- crates/worth-relational/src/runtime/home_program_transition/body_digest/tests.rs
- crates/worth-relational/src/runtime/home_program_transition/catalog_digest/tests.rs
- crates/worth-runtime-world/src/durability/artifact/retention_tests.rs
- crates/worth-runtime-world/src/durability/occurrence_custody/retention_tests.rs
- crates/worth-runtime-world/src/lifecycle/owner/execution_service/tests/public_ports/released_source.rs
- workspaces/worth-query-bank-world/crates/bank-http-adapter/src/http/server/authorization_denial/instruction_tests.rs

Each is changed on Lifecycle relative to7d0e5189bc; Bank is also explicitly held
by integration. Their actual runtime/request-root construction must move to
public-flow courts or split genuinely pure logic. A baseline addition is not a fix.

The other five findings are exact caller seals. The narrow entries below are
proposals for coordinator/owner ruling, NOT applied configuration. The paths
already exist in14b16fa292; no missing entry was dropped by our merge resolution.
The current constructor caller file differs from incoming only by the four
accepted P-A funding entries (and one blank line).

Proposed control-callers.txt row:

```text
workspaces/worth-query/crates/worth-query-execution/ workspaces/worth-query/crates/worth-query-court-execution-primary-graph-application-attempt/tests/worth_query_court_execution_primary_graph_application_attempt/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared/running/affinity_tests/retained_program.rs
```

Proposed certification-construction-callers.txt rows:

```text
native-world-readmission workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_catalog_readmission.rs
native-durable-initial-installation workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_compilation/fresh_installation.rs
native-home-query-preparation workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_compilation/fresh_installation.rs
query-world-commit-preparation workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect/native_original.rs
```

The factory keys come from seal_rules/factory_ownership.rs, not guessed names.
Alternatively the owner can cut over those callers through an already legal
door. No new production or test observer door is proposed.

Exact entries preserved while resolving the constructor-list conflict:

```text
native-home-complete-checkpoint crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/capture_cost.rs
native-fresh-initialization crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/contract_work.rs
native-durable-initial-installation crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/contract_work.rs
native-home-original-source crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/cold_materialization/selected_image.rs
installation workspaces/worth-query/crates/worth-query-court-execution-primary-graph-application-query/tests/worth_query_court_execution_primary_graph_application_query/domain_computation/primary_graph/application_query/projection/retained_projection/tests/reads/mod.rs
```

The first four are retained P-A rulings; the last is the incoming relocated
projection court. Incoming master caller relocations elsewhere are kept intact.

Current format posture is unchanged by these resolutions: no new version or
Home retirement. D13's required authenticated World-original section and format
bump have not been implemented or qualified in this merge slice.
