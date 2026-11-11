//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPreparedApplicationCommit<(), (), (), ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared.rs", "WorthQueryPreparedApplicationCommit", 128 * 1024);
        owner.field::<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        (),
        (),
        (),
        (),
    >>("admission");
        owner.field::<WorthQueryApplicationSnapshotLease>("lease");
        owner.field::<WorthQueryPreparedApplicationProviderAttempt>("provider_attempt");
        owner.field::<super::super::super::super::WorthQueryApplicationCommitOutcomeIdentity>(
            "outcome_identity",
        );
        owner
            .field::<Option<LocalWorkflowSettlementPublication>>("workflow_settlement_publication");
        owner.field::<crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture>("effect_posture");
        owner.field::<worth_query_installation::facade::WorthQueryInvariantExecutionContract>(
            "native_reference_invariants",
        );
        owner.field::<WorthQueryProviderCommitAuthorization>("authorization");
        owner.field::<WorthQueryApplicationIdempotencyBinding>("idempotency");
        owner.field::<Option<
            crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >>("aftermath_causality");
    }
    {
        let _owner = census.owner::<WorthQueryApplicationCommitPreparation<(), (), (), ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared.rs", "WorthQueryApplicationCommitPreparation", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryApplicationCommitPreparationRequest<'static, 'static, 'static, (), (), (), ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared.rs", "WorthQueryApplicationCommitPreparationRequest", 128 * 1024);
        owner.field::<CurrentCommitProgram<
            'static,
            'static,
            'static,
            (),
            WorthQueryApplicationEffectProgram<(), (), (), ()>,
        >>("entry");
        owner.field::<WorthQueryApplicationIdempotencyBinding>("idempotency");
        owner.field::<Option<WorthQueryElevationCommitCurrentness>>("elevation_currentness");
        owner.field::<Option<WorthQueryPendingAftermathCausality>>("aftermath_causality");
    }
    {
        let mut owner = census.owner::<WorthQueryProviderAttemptPreparation>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared.rs", "WorthQueryProviderAttemptPreparation", 128 * 1024);
        owner.field::<Option<crate::domain_computation::primary_graph::workflow::operation_output::WorkflowOperationOrigin>>("workflow_origin");
        owner.field::<Option<
            super::super::super::super::workflow_transition_program::LocalActionSettlementProof,
        >>("local_action_settlement");
        owner.field::<Option<crate::domain_computation::primary_graph::operation_body::emission::BodyExternalEffectSelection>>("body_external_emission");
        owner
            .field::<Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>>(
                "required_output_demand",
            );
        owner.field::<crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes>("installed_read_scopes");
        owner.field::<super::super::super::super::CompletedFactRoutes>("effect_routes");
        owner.field::<Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedFactProjection>>>("application_projection");
        owner.field::<Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedOriginalRead>>>("original_read");
        owner.field::<Vec<
            crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence,
        >>("consumed_outputs");
        owner.field::<Vec<WorthQueryApplicationRealizedEffect>>("effects");
        owner.field::<worth_relational::facade::content::RelationalContentAttachments>(
            "content_attachments",
        );
        owner.field::<usize>("application_effect_count");
        owner.field::<Option<worth_query_installation::facade::InstalledPreImageDemand>>(
            "preimage_demand",
        );
        owner.field::<Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>>("conditional_definition");
        owner.field::<crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture>("effect_posture");
        owner.field::<super::super::super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate>("output_correspondence");
        owner.field::<bool>("retain_output_demand_observation");
        owner.field::<bool>("retain_client_observation");
        owner.field::<&'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement]>("producer_required_invariants");
        owner.field::<Option<super::super::super::super::OutputCurrentnessFacts>>(
            "output_currentness_facts",
        );
        owner.field::<Option<
        crate::domain_computation::primary_graph::application_attempt::read_set::WorkflowOperationBindingProof,
    >>("workflow_settlement");
    }
    {
        let mut owner = census.owner::<WorthQueryCurrentApplicationCommit<(), (), (), ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared.rs", "WorthQueryCurrentApplicationCommit", 128 * 1024);
        owner.field::<WorthQueryAdmittedApplicationOperation<(), (), (), ()>>("admission");
        owner.field::<WorthQueryApplicationSnapshotLease>("lease");
        owner.field::<WorthQueryProviderAttemptPreparation>("provider");
        owner.field::<WorthQueryApplicationIdempotencyBinding>("idempotency");
        owner.field::<Option<WorthQueryPendingAftermathCausality>>("aftermath_causality");
        owner.field::<Option<
            crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
        >>("workflow_approval_authority");
    }
    super::running::inline_type_budget::collect(census);
}
