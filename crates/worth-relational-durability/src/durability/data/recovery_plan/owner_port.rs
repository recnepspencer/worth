//! Runtime custody of the plan and its admitted recovery inputs.
use super::*;

#[doc(hidden)]
pub trait RecoveryPlanOwnerPort: Sized {
    fn branch_transitions(&self) -> &super::super::RecoveryBranchTransitions;
    /// Medium recovery read its checkpoint from the selected durable source.
    fn acknowledge_selected_checkpoint(&mut self);
    fn selected_checkpoint_acknowledged(&self) -> bool;
    fn attach_original_source(
        &mut self,
        source: Option<
            std::sync::Arc<
                crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
            >,
        >,
    );
    fn retain_transition_prefix(
        &mut self,
        prefix: Option<std::sync::Arc<crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputs>>,
    );

    fn select_home_world(
        &mut self,
        selection: Option<
            std::sync::Arc<
                worth_proof::VerifiedHomeWorldSelection<
                    std::sync::Arc<
                        crate::durability::home_world_catalog::RelationalWorldSourceCatalog,
                    >,
                >,
            >,
        >,
    );
    fn cold_source(
        &self,
    ) -> Option<
        &std::sync::Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>,
    >;
    fn retained_transition_prefix(
        &self,
    ) -> Option<
        &crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputs,
    >;
    fn tail_log(&self) -> &[crate::durability::migration::ReadmittedCanonicalCommit];
    fn commit_strategy_executors(
        &self,
    ) -> &merge::commit_strategies::execution::registry::FrozenCommitStrategyExecutorRegistry;

    fn new(
        config: contracts::config::data::runtime_config::RelationalRuntimeConfig,
        checkpoint: Option<DurableCheckpoint>,
        tail_log: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
        branch_transitions: super::super::RecoveryBranchTransitions,
        authority_continuity: RecoveryAuthorityContinuityCheck,
        verification_mode: RecoveryVerificationMode,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        restore_authoritative_envelope_commit_ids: Vec<CommitId>,
        cold_source: Option<
            std::sync::Arc<
                dyn storage::durability::medium::cold_source::RelationalDurableColdSource,
            >,
        >,
    ) -> Self;

    fn with_commit_strategy_executors(
        self,
        executors: merge::commit_strategies::execution::registry::FrozenCommitStrategyExecutorRegistry,
    ) -> Self;

    fn tail_commit_count(&self) -> usize;

    fn tail_envelopes_in_stream_order(
        &self,
    ) -> Vec<&branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>;

    fn should_restore_authoritative_envelope(&self, commit_id: CommitId) -> bool;
}

impl RecoveryPlanOwnerPort for RecoveryPlan {
    fn branch_transitions(&self) -> &super::super::RecoveryBranchTransitions {
        &self.branch_transitions
    }

    fn acknowledge_selected_checkpoint(&mut self) {
        self.selected_checkpoint_acknowledged = self.checkpoint.is_some();
    }

    fn selected_checkpoint_acknowledged(&self) -> bool {
        self.selected_checkpoint_acknowledged
    }

    fn attach_original_source(
        &mut self,
        source: Option<
            std::sync::Arc<
                crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
            >,
        >,
    ) {
        self.original_source = source;
    }

    fn retain_transition_prefix(
        &mut self,
        prefix: Option<std::sync::Arc<crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputs>>,
    ) {
        self.retained_transition_prefix = prefix;
    }

    fn select_home_world(
        &mut self,
        selection: Option<
            std::sync::Arc<
                worth_proof::VerifiedHomeWorldSelection<
                    std::sync::Arc<
                        crate::durability::home_world_catalog::RelationalWorldSourceCatalog,
                    >,
                >,
            >,
        >,
    ) {
        self.home_world_selection = selection;
    }

    fn cold_source(
        &self,
    ) -> Option<
        &std::sync::Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>,
    > {
        self.cold_source.as_ref()
    }

    fn retained_transition_prefix(
        &self,
    ) -> Option<
        &crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputs,
    > {
        self.retained_transition_prefix.as_deref()
    }

    fn tail_log(&self) -> &[crate::durability::migration::ReadmittedCanonicalCommit] {
        &self.tail_log
    }

    fn commit_strategy_executors(
        &self,
    ) -> &merge::commit_strategies::execution::registry::FrozenCommitStrategyExecutorRegistry {
        &self.commit_strategy_executors
    }

    fn new(
        config: contracts::config::data::runtime_config::RelationalRuntimeConfig,
        checkpoint: Option<DurableCheckpoint>,
        tail_log: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
        branch_transitions: super::super::RecoveryBranchTransitions,
        authority_continuity: RecoveryAuthorityContinuityCheck,
        verification_mode: RecoveryVerificationMode,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        restore_authoritative_envelope_commit_ids: Vec<CommitId>,
        cold_source: Option<
            std::sync::Arc<
                dyn storage::durability::medium::cold_source::RelationalDurableColdSource,
            >,
        >,
    ) -> Self {
        Self::new(
            config,
            checkpoint,
            tail_log,
            branch_transitions,
            authority_continuity,
            verification_mode,
            descriptor_semantics_version,
            restore_authoritative_envelope_commit_ids,
            cold_source,
        )
    }

    fn with_commit_strategy_executors(
        self,
        executors: merge::commit_strategies::execution::registry::FrozenCommitStrategyExecutorRegistry,
    ) -> Self {
        self.with_commit_strategy_executors(executors)
    }

    fn tail_commit_count(&self) -> usize {
        self.tail_commit_count()
    }

    fn tail_envelopes_in_stream_order(
        &self,
    ) -> Vec<&branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope> {
        self.tail_envelopes_in_stream_order()
    }

    fn should_restore_authoritative_envelope(&self, commit_id: CommitId) -> bool {
        self.should_restore_authoritative_envelope(commit_id)
    }
}

#[doc(hidden)]
pub trait RecoveryPlanFixturePort {
    fn replace_tail_log_for_test(
        &mut self,
        tail: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
    );
}
impl RecoveryPlanFixturePort for RecoveryPlan {
    fn replace_tail_log_for_test(
        &mut self,
        tail: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
    ) {
        self.tail_log = tail;
    }
}
