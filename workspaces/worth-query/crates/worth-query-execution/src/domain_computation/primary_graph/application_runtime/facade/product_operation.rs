//! The runtime's consumer methods over `domain_computation::primary_graph::product_operation`; each only calls its subsystem.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::product_operation::WorthQueryApplicationProductBranches;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::product_operation::WorthQueryBranchSetAdoptionPreparationDenial;
use crate::domain_computation::primary_graph::product_operation::WorthQueryBranchSetAdoptionRecovery;
use crate::domain_computation::primary_graph::product_operation::WorthQueryBranchSetAdoptionRecoveryFailure;
use crate::domain_computation::primary_graph::product_operation::WorthQueryBranchSetAdoptionRecoveryOutcome;
use crate::domain_computation::primary_graph::product_operation::WorthQueryBranchSetAdoptionRecoveryReleaseFailure;
use crate::domain_computation::primary_graph::product_operation::WorthQueryBranchSetAdoptionResumeFailure;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryOperationConstraint;
use crate::domain_computation::primary_graph::product_operation::WorthQueryGeneratedOutputReconstruction;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputAuthority;
use crate::domain_computation::primary_graph::product_operation::WorthQueryPreparedBranchSetAdoption;
use crate::domain_computation::primary_graph::product_operation::WorthQueryPreparedProgramMigration;
use crate::domain_computation::primary_graph::product_operation::WorthQueryProductEntry;
use crate::domain_computation::primary_graph::application_attempt::migration::preparation_denial::WorthQueryProgramMigrationPreparationDenial;
use crate::domain_computation::primary_graph::product_operation::WorthQueryStoppedBranchSetAdoption;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::product_operation::WorthQueryAdmittedProgramMigration;
use crate::domain_computation::primary_graph::product_operation::WorthQueryBranchSetAdoptionCancellation;
use crate::domain_computation::primary_graph::product_operation::WorthQueryGeneratedOutputReconstructionFailure;
use crate::domain_computation::primary_graph::product_operation::WorthQueryGeneratedOutputRestorationFailure;
use crate::domain_computation::primary_graph::product_operation::WorthQueryGeneratedOutputRestorationRecovery;
use crate::domain_computation::primary_graph::product_operation::WorthQueryGeneratedOutputRestorationRecoveryFailure;
use crate::domain_computation::primary_graph::product_operation::WorthQueryGeneratedOutputSuspensionRecovery;
use crate::domain_computation::primary_graph::product_operation::WorthQueryGeneratedOutputSuspensionRecoveryFailure;
use worth_query_installation::facade::ApplicationSchema;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use crate::domain_computation::primary_graph::product_operation::WorthQueryOrderedProgramAdoptionCoverage;
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::product_operation::document_history::WorthQueryDocumentHistoryDenial as Denial;
use crate::domain_computation::primary_graph::product_operation::generated_materialization::WorthQuerySuspendedGeneratedOutput;
use crate::domain_computation::primary_graph::WorthQueryApplicationProducerBinding;
use crate::domain_computation::primary_graph::product_operation::generated_materialization::WorthQueryCompletedGeneratedOutputReconstruction;
use crate::domain_computation::primary_graph::product_operation::generated_materialization::restoration::WorthQueryRestoredGeneratedOutput;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use crate::domain_computation::primary_graph::product_operation::program_adoption::custody::recovery::WorthQueryBranchAdoptionRecovery;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use crate::domain_computation::primary_graph::application_entry::mutation::WorthQueryCompletedMutationCandidate;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt;
use crate::domain_computation::primary_graph::product_operation::program_adoption::custody::WorthQueryBranchAdoptionRecoveryReleaseFailure;
use crate::domain_computation::primary_graph::WorthQueryBranchAdoptionRecoveryFailure;
use crate::domain_computation::primary_graph::WorthQueryBranchAdoptionRecoveryOutcome;
use crate::domain_computation::primary_graph::product_operation::query::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
impl<S: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn validate_history_execution_predecessor<SOp, Input, Model>(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        restriction: &WorthQueryDocumentHistoryOperationConstraint,
        admitted: &WorthQueryAdmittedApplicationOperation<S, SOp, Input, Model>,
    ) -> Result<(), Denial> {
        <Self as crate::domain_computation::primary_graph::product_operation::ConstraintOps<S>>::validate_history_execution_predecessor(self, _access, restriction, admitted)
    }

    pub fn validate_history_operation<SOp, Input, Model>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        restriction: &WorthQueryDocumentHistoryOperationConstraint,
        admitted: &WorthQueryAdmittedApplicationOperation<S, SOp, Input, Model>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<(), Denial> {
        <Self as crate::domain_computation::primary_graph::product_operation::ConstraintOps<S>>::validate_history_operation(self, phase, _access, restriction, admitted, binding)
    }

    pub fn execute_history_root_output_query<'control, 'authority, 'phase, 'scope, B>(
        &'control self,
        phase: &WorthQueryAdvancementPhase<'_>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        binding: &'control WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        control: crate::domain_computation::primary_graph::application_query::HistoryOutputControlPlan<'control, S, B>,
    ) -> Result<
        WorthQueryHistoryRootOutputRead<'authority, 'phase, 'scope, B::Query, <<B as worth_query_declaration::facade::application_query::ApplicationQueryBinding<S>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value>,
        WorthQueryHistoryOutputReadDenial,
    >
    where
        B: ApplicationQueryBinding<S>,
    <<B as worth_query_declaration::facade::application_query::ApplicationQueryBinding<S>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, B::Query>,{
        <Self as crate::domain_computation::primary_graph::product_operation::HistoryOutputsOps<
            S,
        >>::execute_history_root_output_query(
            self, phase, authority, binding, external, control
        )
    }
}

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn current_world(&self) -> WorthQueryProductBranch {
        <Self as crate::domain_computation::primary_graph::product_operation::BranchesOps<Schema>>::current_world(self)
    }

    pub fn branches(&self) -> WorthQueryApplicationProductBranches<'_, Schema> {
        <Self as crate::domain_computation::primary_graph::product_operation::BranchesOps<Schema>>::branches(self)
    }

    pub fn product_runtime(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &self.runtime.product_runtime
    }

    pub fn on_branch(&self, branch: WorthQueryProductBranch) -> WorthQueryProductEntry<'_, Schema> {
        <Self as crate::domain_computation::primary_graph::product_operation::ContextOps<Schema>>::on_branch(self, branch)
    }

    pub fn reconstruct_generated_output<Producer>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        suspended: WorthQuerySuspendedGeneratedOutput,
    ) -> Result<
        WorthQueryGeneratedOutputReconstruction<'_, Schema, Producer>,
        WorthQueryGeneratedOutputReconstructionFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::product_operation::ReconstructionOps<
            Schema,
        >>::reconstruct_generated_output(self, phase, suspended)
    }

    pub fn restore_generated_output<Producer>(
        &self,
        completed: WorthQueryCompletedGeneratedOutputReconstruction<Schema, Producer>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryRestoredGeneratedOutput, WorthQueryGeneratedOutputRestorationFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::product_operation::RestorationOps<
            Schema,
        >>::restore_generated_output(self, completed, request)
    }

    pub fn continue_generated_output_restoration_recovery(
        &self,
        recovery: WorthQueryGeneratedOutputRestorationRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        <Self as crate::domain_computation::primary_graph::product_operation::AdvancementOps<
            Schema,
        >>::continue_generated_output_restoration_recovery(self, recovery, request)
    }

    pub fn continue_generated_output_suspension_recovery<Producer>(
        &self,
        recovery: WorthQueryGeneratedOutputSuspensionRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::product_operation::RecoveryAdvancementOps<Schema>>::continue_generated_output_suspension_recovery::<Producer>(self, recovery, request)
    }

    #[doc(hidden)]
    pub fn integration_recover_branch_adoption(
        &self,

        branch: crate::basis::WorthQueryProductBranch,
        recovery: WorthQueryBranchAdoptionRecovery,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>,
        (
            crate::basis::WorthQueryProductBranchAdmissionDenial,
            WorthQueryBranchAdoptionRecovery,
        ),
    > {
        <Self as crate::domain_computation::primary_graph::product_operation::RecoveryOps<Schema>>::integration_recover_branch_adoption(self, branch, recovery, principal, request)
    }

    pub fn release_branch_adoption_recovery(
        &self,
        recovery: WorthQueryBranchAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryBranchAdoptionRecoveryReleaseFailure,
    >{
        <Self as crate::domain_computation::primary_graph::product_operation::ReleaseOps<Schema>>::release_branch_adoption_recovery(self, recovery, minimum_age_ticks)
    }

    pub fn admit_program_migration<Binding>(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<
        WorthQueryAdmittedProgramMigration<Schema, Binding>,
        WorthQueryProgramMigrationPreparationDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::product_operation::MigrationOps<
            Schema,
        >>::admit_program_migration(self, target)
    }

    pub fn seal_program_migration<Binding>(
        &self,
        admitted: WorthQueryAdmittedProgramMigration<Schema, Binding>,
        candidate: WorthQueryCompletedMutationCandidate<Schema, Binding>,
    ) -> Result<WorthQueryPreparedProgramMigration, WorthQueryProgramMigrationPreparationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::product_operation::MigrationOps<
            Schema,
        >>::seal_program_migration(self, admitted, candidate)
    }

    pub fn prepare_branch_set_adoption(
        &self,

        coverage: WorthQueryOrderedProgramAdoptionCoverage,
        target: &ApplicationProgramRevision,
        maximum_selection_work_per_branch: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchSetAdoption, WorthQueryBranchSetAdoptionPreparationDenial>
    {
        <Self as crate::domain_computation::primary_graph::product_operation::BranchSetOps<
            Schema,
        >>::prepare_branch_set_adoption(
            self,
            coverage,
            target,
            maximum_selection_work_per_branch,
            principal,
            request,
        )
    }

    pub fn authorize_branch_set_adoption(
        &self,
        coverage: &WorthQueryOrderedProgramAdoptionCoverage,
        target: &ApplicationProgramRevision,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<(), WorthQueryBranchSetAdoptionPreparationDenial> {
        <Self as crate::domain_computation::primary_graph::product_operation::AdmissionOps<
            Schema,
        >>::authorize_branch_set_adoption(self, coverage, target, principal, request)
    }

    pub fn recover_branch_set_adoption(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryBranchSetAdoptionRecovery,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryBranchSetAdoptionRecoveryOutcome,
        WorthQueryBranchSetAdoptionRecoveryFailure,
    > {
        <Self as crate::domain_computation::primary_graph::product_operation::BranchSetRecoveryOps<Schema>>::recover_branch_set_adoption(self, phase, recovery, principal, request)
    }

    pub fn release_branch_set_adoption_recovery(
        &self,
        recovery: WorthQueryBranchSetAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        (
            WorthQueryBranchSetAdoptionCancellation,
            WorthQueryProductBranchOwnerCleanupReceipt,
        ),
        WorthQueryBranchSetAdoptionRecoveryReleaseFailure,
    > {
        <Self as crate::domain_computation::primary_graph::product_operation::BranchSetRecoveryOps<Schema>>::release_branch_set_adoption_recovery(self, recovery, minimum_age_ticks)
    }

    pub fn resume_branch_set_adoption(
        &self,

        adoption: WorthQueryStoppedBranchSetAdoption,
        coverage: WorthQueryOrderedProgramAdoptionCoverage,
        maximum_selection_work_per_branch: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchSetAdoption, WorthQueryBranchSetAdoptionResumeFailure> {
        <Self as crate::domain_computation::primary_graph::product_operation::ResumeOps<Schema>>::resume_branch_set_adoption(self, adoption, coverage, maximum_selection_work_per_branch, principal, request)
    }
}
