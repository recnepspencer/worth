//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationCommitReceipt>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_receipt/mod.rs", "WorthQueryApplicationCommitReceipt", 128 * 1024);
        owner.field::<crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding>("authoritative_provider_session");
        owner.field::<Option<super::super::super::WorthQueryApplicationCommitOutcomeIdentity>>(
            "outcome_identity",
        );
        owner.field::<u64>("provider_runtime_instance_id");
        owner.field::<RelationalCommitReceipt>("commit");
        owner.field::<super::super::WorthQueryCommittedProductPublication>(
            "committed_product_publication",
        );
        owner.field::<worth_relational::facade::branch::RelationalBranchBasisDescriptor>(
            "basis_descriptor",
        );
        owner.field::<usize>("changed_record_count");
        owner.field::<usize>("emitted_effect_count");
        owner.field::<Option<super::super::super::super::provider::WorthQueryPrimaryMutationWorkEvidence>>("mutation_work");
        owner.field::<WorthQueryMutationPreconditionComparisonEvidence>("precondition_comparison");
        owner.field::<WorthQueryCanonicalWorkPhases>("canonical_work");
        owner.field::<WorthQueryApplicationCommitTerminalEvidence>("terminal");
        owner.field::<Option<super::super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding>>("committed_dispatch_outbox");
        owner.field::<Option<WorthQueryExternalEffectDispatch>>("external_dispatch");
        owner.field::<Option<super::super::super::WorthQueryExternalDispatchPreparationDenial>>(
            "external_dispatch_preparation_denial",
        );
        owner.field::<WorthQueryApplicationCommitAuthorityBinding>("authority_binding");
        owner.field::<Option<WorthQueryRetainedPreImage>>("retained_preimage");
        owner.field::<Option<WorthQueryCommittedAftermathCausality>>("aftermath_causality");
        owner.field::<Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >>("performed_product_change");
        owner.field::<WorthQueryApplicationCommittedChanges>("committed_changes");
        owner.field::<Option<
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        >>("performed_input_facts");
        owner.field::<Option<Result<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload, crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure>>>("performed_output_fields");
        owner.field::<std::sync::Arc<WorthQueryApplicationOutputCorrespondence>>(
            "output_correspondence",
        );
        owner.field::<Option<std::sync::Arc<
        crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    >>>("exact_output_settlement");
        owner
            .field::<Option<crate::domain_computation::primary_graph::provider::OwnEffectOnReads>>(
                "unrebased_own_effect",
            );
    }
}
