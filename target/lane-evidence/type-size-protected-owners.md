# Native and World carrier inventory

Measured only; no product or census mount edit in protected storage/root/World files. Native rows outside that protected subset are listed for completeness; this table does not assign those files to P-A. These values are baseline dev observations. H10 ordinary-main success remains integration-owned; residual ceilings belong to C2.

|Bytes|Type|Definition|Owner|
|---:|---|---|---|
|12752|`worth_relational::authority::commit::pipeline::mutation_execution::MutatedCommitExecution`|crates/worth-relational/src/authority/commit/pipeline/mutation_execution.rs|Native; P-A only for storage/checkpoint/root subset|
|12640|`worth_relational::authority::commit::pipeline::draft_execution::PreparedCommitExecution`|crates/worth-relational/src/authority/commit/pipeline/draft_execution.rs|Native; P-A only for storage/checkpoint/root subset|
|11584|`worth_relational::authority::commit::pipeline::artifact_execution::AssembledCommitExecution`|crates/worth-relational/src/authority/commit/pipeline/artifact_execution.rs|Native; P-A only for storage/checkpoint/root subset|
|10736|`worth_relational::authority::commit::pipeline::draft_execution::PreparedCommitState`|crates/worth-relational/src/authority/commit/pipeline/draft_execution.rs|Native; P-A only for storage/checkpoint/root subset|
|10480|`worth_relational::authority::commit::pipeline::execution_admission::AdmittedCommitState`|crates/worth-relational/src/authority/commit/pipeline/execution_admission.rs|Native; P-A only for storage/checkpoint/root subset|
|6872|`worth_runtime_world::branch::currentness_scope::ProductBranchCurrentnessFailure`|crates/worth-runtime-world/src/branch/currentness_scope.rs|P-C|
|6352|`aggregate_identity::funding::navigation::NavigationLoan`|crates/worth-relational/src/aggregate_identity/funding/navigation.rs|Native; P-A only for storage/checkpoint/root subset|
|5760|`worth_relational::facade::merge::MergeExecutionOutcome`|crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs|Native; P-A only for storage/checkpoint/root subset|
|4592|`worth_relational::facade::merge::PreparedMergeExecution`|crates/worth-relational/src/merge/data/execution.rs|Native; P-A only for storage/checkpoint/root subset|
|4480|`worth_relational::facade::transactions::CommitResult`|crates/worth-relational/src/transactions/data/outcomes/commit_result.rs|Native; P-A only for storage/checkpoint/root subset|
|4480|`worth_relational::transactions::data::outcomes::commit_result::CommitResult`|crates/worth-relational/src/transactions/data/outcomes/commit_result.rs|Native; P-A only for storage/checkpoint/root subset|
|3184|`worth_relational::merge::data::execution::CompiledMergeExecution`|crates/worth-relational/src/merge/data/execution.rs|Native; P-A only for storage/checkpoint/root subset|
|3016|`worth_runtime_world::lifecycle::owner::RuntimeWorldOwnerState`|crates/worth-runtime-world/src/lifecycle/owner.rs|P-C|
|2408|`worth_relational::facade::commit_strategies::LoweredStrategyCommitPlan`|crates/worth-relational/src/commit_strategies/data/lowering.rs|Native; P-A only for storage/checkpoint/root subset|
|2136|`worth_relational::facade::durability::RelationalRecoveredPublicationRetirement`|crates/worth-relational/src/durability/authority/recovered_publication/retirement.rs|Native; P-A only for storage/checkpoint/root subset|
|2136|`worth_relational::facade::durability::RetiredRelationalRecoveredPublication`|crates/worth-relational/src/durability/authority/recovered_publication/retirement.rs|Native; P-A only for storage/checkpoint/root subset|
|2112|`worth_relational::facade::merge::MergePlanningArtifactCore`|crates/worth-relational/src/merge/data/artifacts/planning_artifact_core.rs|Native; P-A only for storage/checkpoint/root subset|
|2096|`worth_relational::transactions::data::outcomes::commit_result::CommitExecution`|crates/worth-relational/src/transactions/data/outcomes/commit_result.rs|Native; P-A only for storage/checkpoint/root subset|
|2048|`worth_relational::facade::runtime::CustomInvariantExecutionContext`|crates/worth-relational/src/validation/custom_rule/execution_context.rs|Native; P-A only for storage/checkpoint/root subset|
|2048|`worth_relational::validation::custom_rule::execution_context::CustomInvariantExecutionContext`|crates/worth-relational/src/validation/custom_rule/execution_context.rs|Native; P-A only for storage/checkpoint/root subset|
|2032|`worth_relational::facade::runtime::CustomInvariantScopePlanner`|crates/worth-relational/src/validation/custom_rule/scope_planner.rs|Native; P-A only for storage/checkpoint/root subset|
|2032|`worth_relational::validation::custom_rule::scope_planner::CustomInvariantScopePlanner`|crates/worth-relational/src/validation/custom_rule/scope_planner.rs|Native; P-A only for storage/checkpoint/root subset|
|1904|`worth_relational::authority::commit::phases::applied_mutation::AppliedMutationDraft`|crates/worth-relational/src/authority/commit/phases/applied_mutation.rs|Native; P-A only for storage/checkpoint/root subset|
|1888|`worth_relational::facade::runtime::RuntimeComplexityCounters`|crates/worth-relational/src/performance/data/runtime_complexity_counters.rs|Native; P-A only for storage/checkpoint/root subset|
|1888|`worth_relational::performance::data::runtime_complexity_counters::RuntimeComplexityCounters`|crates/worth-relational/src/performance/data/runtime_complexity_counters.rs|Native; P-A only for storage/checkpoint/root subset|
|1720|`worth_runtime_world::publication::custody::conditional_definition::ConditionalDefinitionAttemptCustody`|crates/worth-runtime-world/src/publication/custody/conditional_definition.rs|P-C|
|1608|`worth_runtime_world::publication::outcome::OwnerExecutionOutcome`|crates/worth-runtime-world/src/publication/outcome.rs|P-C|
|1608|`worth_runtime_world::publication::owner_execution::OwnerExecutionSettlement`|crates/worth-runtime-world/src/publication/owner_execution.rs|P-C|
|1448|`worth_relational::mvcc::retained_entity_selection::RelationalRetainedEntitySelection`|crates/worth-relational/src/mvcc/retained_entity_selection.rs|Native; P-A only for storage/checkpoint/root subset|
|1424|`worth_relational::authority::commit::pipeline::publication_execution::preparation::PreparePublicationPhaseInput`|crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation.rs|Native; P-A only for storage/checkpoint/root subset|
|1424|`worth_relational::facade::mvcc::BranchBoundRelationalTransaction`|crates/worth-relational/src/mvcc/transaction/bound.rs|Native; P-A only for storage/checkpoint/root subset|
|1424|`worth_relational::mvcc::transaction::bound::BranchBoundRelationalTransaction`|crates/worth-relational/src/mvcc/transaction/bound.rs|Native; P-A only for storage/checkpoint/root subset|
|1408|`worth_relational::merge::data::execution::PreparedMergeMutationPlan`|crates/worth-relational/src/merge/data/execution.rs|Native; P-A only for storage/checkpoint/root subset|
|1312|`worth_relational::config::data::runtime_config::RelationalRuntimeConfig`|crates/worth-relational/src/config/data/runtime_config.rs|Native; P-A only for storage/checkpoint/root subset|
|1312|`worth_relational::facade::runtime::RelationalRuntimeConfig`|crates/worth-relational/src/config/data/runtime_config.rs|Native; P-A only for storage/checkpoint/root subset|
|1200|`worth_relational::transactions::data::outcomes::commit_result::CommitPublication`|crates/worth-relational/src/transactions/data/outcomes/commit_result.rs|Native; P-A only for storage/checkpoint/root subset|
|1192|`worth_relational::facade::transactions::MergeExecutionSummary`|crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs|Native; P-A only for storage/checkpoint/root subset|
|1160|`worth_relational::authority::commit::pipeline::publication_execution::preparation::PreparedPublicationPhase`|crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation.rs|Native; P-A only for storage/checkpoint/root subset|
|1112|`worth_relational::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle`|crates/worth-relational/src/commit_strategies/data/artifacts/artifact_bundle.rs|Native; P-A only for storage/checkpoint/root subset|
|1112|`worth_relational::facade::commit_strategies::StrategyCommitArtifactBundle`|crates/worth-relational/src/commit_strategies/data/artifacts/artifact_bundle.rs|Native; P-A only for storage/checkpoint/root subset|
|1104|`worth_relational::validation::candidate_source::observation::ManagedIndexSourceRead`|crates/worth-relational/src/validation/candidate_source/observation.rs|Native; P-A only for storage/checkpoint/root subset|
|1088|`worth_relational::validation::candidate_source::preparation::SourceViews`|crates/worth-relational/src/validation/candidate_source/preparation.rs|Native; P-A only for storage/checkpoint/root subset|
|1040|`worth_runtime_world::branch::reference_cell::ProductBranchReferenceImage`|crates/worth-runtime-world/src/branch/reference_cell.rs|P-C|
|1024|`worth_runtime_world::publication::conditional_recovery::PreparedConditionalDefinitionRecovery`|crates/worth-runtime-world/src/publication/conditional_recovery.rs|P-C|
