//! Select genuine source occurrences before any corrective disclosure.
use super::*;
use worth_query_execution_lineage::facade::ports::CommitReceiptInternalPort as _;
use worth_query_execution_lineage::facade::ports::{
    CommitAuthorityBindingInternalPort as _, RetainedGovernedInputInternalPort as _,
};
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn select_correction_source(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        receipt: &WorthQueryApplicationCommitReceipt,
        maximum: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        WorthQuerySelectedCorrectionSource<'runtime, Schema>,
        WorthQueryCorrectionSelectionDenial,
    > {
        let application = self.application();
        let current = application
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
            .owner_identity();
        let prior = receipt
            .committed_product_publication()
            .composite_commit()
            .owner_identity();
        if let (Some(prior_home), Some(current_home)) = (prior.durable(), current.durable())
            && prior_home.home() == current_home.home()
            && prior
                .home_incarnation_fence()
                .zip(current.home_incarnation_fence())
                .is_some_and(|(prior, current)| prior.admit(current).is_err())
        {
            return Err(WorthQueryCorrectionSelectionDenial::StaleIncarnation);
        }
        if receipt.runtime_authority() != application.inner.runtime.authority_identity() {
            return Err(WorthQueryCorrectionSelectionDenial::ForeignRuntime);
        }
        if application
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
            .owner_identity()
            .durable()
            .is_some()
        {
            let coordinates = receipt.correction_source().coordinates();
            return self.select_home_correction_source(
                phase,
                principal,
                request,
                &coordinates,
                Some(receipt.committed_product_publication().composite_commit()),
                maximum,
                maximum_results,
                None,
            );
        }
        let occurrence = self.observe_member(
            receipt.commit_reference(),
            receipt.correction_source().member(),
        )?;
        let performed = occurrence.committed_product_publication();
        let claimed = receipt.committed_product_publication();
        if performed.composite_commit() != claimed.composite_commit()
            || performed.publication_attempt() != claimed.publication_attempt()
            || performed.product_incarnation() != claimed.product_incarnation()
            || occurrence.commit_reference() != receipt.commit_reference()
        {
            return Err(WorthQueryCorrectionSelectionDenial::SourcePublicationMismatch);
        }
        self.select_retained_occurrence(
            phase,
            principal,
            request,
            occurrence,
            maximum,
            maximum_results,
        )
    }

    /// Resolves descriptive coordinates through the completed-evidence owner.
    pub fn select_recorded_correction_source(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        locator: &crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource,
        maximum: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        WorthQuerySelectedCorrectionSource<'runtime, Schema>,
        WorthQueryCorrectionSelectionDenial,
    > {
        self.select_correction_source_coordinates(
            phase,
            principal,
            request,
            &locator.coordinates(),
            maximum,
            maximum_results,
        )
    }

    /// Resolves untrusted wire coordinates without constructing an owner-issued World identity.
    pub fn select_correction_source_coordinates(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        locator: &crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates,
        maximum: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        WorthQuerySelectedCorrectionSource<'runtime, Schema>,
        WorthQueryCorrectionSelectionDenial,
    > {
        if self
            .application()
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
            .owner_identity()
            .durable()
            .is_some()
        {
            return self.select_home_correction_source(
                phase,
                principal,
                request,
                locator,
                None,
                maximum,
                maximum_results,
                None,
            );
        }
        let occurrence = self.observe_member(locator.relational(), locator.member())?;
        let actual = occurrence
            .committed_product_publication()
            .composite_commit();
        let owner = match actual.owner_identity().durable() {
            Some(owner) => worth_foundational::facade::WorldOwnerCoordinate::Home { owner },
            None => worth_foundational::facade::WorldOwnerCoordinate::MemoryOnly {
                observation: actual.owner_identity().get(),
            },
        };
        if owner != locator.world_owner() || actual.ordinal() != locator.world_commit() {
            return Err(WorthQueryCorrectionSelectionDenial::SourcePublicationMismatch);
        }
        self.select_retained_occurrence(
            phase,
            principal,
            request,
            occurrence,
            maximum,
            maximum_results,
        )
    }

    /// The named writer of a completed commit's coupled group (public #368).
    fn observe_member(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
        member: crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember,
    ) -> Result<WorthQueryPrimaryGraphCommittedApplication, WorthQueryCorrectionSelectionDenial>
    {
        let group = (self.application().inner.runtime.primary_provider_ref())
            .observe_completed_application(commit)
            .ok_or(WorthQueryCorrectionSelectionDenial::SourceUnavailable)?;
        group
            .publication_members()
            .find(|writer| {
                member.names_writer(
                    writer
                        .commit_evidence()
                        .idempotency()
                        .authored_stage_identity(),
                )
            })
            .cloned()
            .ok_or(WorthQueryCorrectionSelectionDenial::SourceMemberUnavailable)
    }

    fn select_retained_occurrence(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        occurrence: WorthQueryPrimaryGraphCommittedApplication,
        maximum: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        WorthQuerySelectedCorrectionSource<'runtime, Schema>,
        WorthQueryCorrectionSelectionDenial,
    > {
        let application = self.application();
        let execution = phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|_| WorthQueryCorrectionSelectionDenial::ForeignPhase)?;
        let performed = occurrence.committed_product_publication();
        let original = occurrence.commit_evidence().original_authority();
        let retained = original
            .retained_governed_input_carrier()
            .ok_or(WorthQueryCorrectionSelectionDenial::SourceNotCorrectable)?;
        if retained.correction().is_none() || retained.original_scope().is_none() {
            return Err(WorthQueryCorrectionSelectionDenial::SourceNotCorrectable);
        }
        let ancestry = application
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
            .observation_port()
            .select_retained_product_branch_ancestor(
                &execution,
                self.product().observation(),
                performed.composite_commit(),
                maximum,
            )
            .map_err(WorthQueryCorrectionSelectionDenial::Ancestry)?;
        let source_type = original
            .original_mutation_binding()
            .ok_or(WorthQueryCorrectionSelectionDenial::SourceNotCorrectable)?;
        let source = WorthQuerySelectedCorrectionSource {
            application,
            ancestry,
            occurrence: CorrectionOccurrence::Memory(std::sync::Arc::new(occurrence)),
        };
        Schema::visit_correction(
            source_type,
            DiscloseSelection {
                target: self,
                phase,
                principal,
                request,
                source,
                maximum,
                maximum_results,
            },
        )
    }
}
