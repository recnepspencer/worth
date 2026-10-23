//! Exact performed-source selection; retained payload stays inside Query.
use std::num::NonZeroUsize;

use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    ProductBranchAncestorSelection, ProductBranchAncestorSelectionDenial, RuntimeWorldServiceDenial,
};

use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod compatibility;
mod derivation;
mod inspection;
mod intent;
mod key;
mod preparation;
mod replay;
pub use intent::WorthQueryCorrectionIntent;
pub use key::WorthQueryCorrectionCommandKey;
pub use preparation::WorthQueryPreparedCorrectionSource;
pub use replay::WorthQueryCorrectionInputSelection;
mod disclosure;
pub use disclosure::WorthQueryCorrectionDerivationDenial;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_aftermath::{
    ApplicationCorrectionBinding, ApplicationCorrectionVisitor, ApplicationInverseBinding,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationScopeResolution,
};

/// A locator backed by the target's retained World ancestry and performed evidence.
/// Preparing a correction still requires a fresh target and current authority.
pub struct WorthQuerySelectedCorrectionSource<'runtime, Schema> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ancestry: ProductBranchAncestorSelection,
    occurrence: WorthQueryPrimaryGraphCommittedApplication,
}

#[derive(Debug)]
/// Why an exact performed occurrence could not be selected as a correction source.
pub enum WorthQueryCorrectionSelectionDenial {
    ForeignPhase,
    ForeignRuntime,
    SourceUnavailable,
    SourceNotCorrectable,
    SourcePublicationMismatch,
    Disclosure(WorthQueryCorrectionDerivationDenial),
    Ancestry(RuntimeWorldServiceDenial<ProductBranchAncestorSelectionDenial>),
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    /// Read-only retained-input selection; fresh inverse admission must follow replay.
    pub fn select_correction_input<C: ApplicationCorrectionBinding<Schema>>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<WorthQueryCorrectionInputSelection<Schema, C>, WorthQueryCorrectionDerivationDenial>
    {
        replay::select_input::<Schema, C>(
            self,
            phase,
            principal,
            request,
            source,
            key,
            maximum_history,
            maximum_results,
        )
    }
    /// Observes source compatibility under fresh disclosure, without admitting an inverse.
    pub fn inspect_correction_source(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<(), WorthQueryCorrectionDerivationDenial> {
        inspection::inspect_source(
            self,
            phase,
            principal,
            request,
            source,
            maximum_history,
            maximum_results,
        )
    }
    /// Publication's typed dispatch derives input only after a new disclosure read.
    pub fn derive_correction_inverse<C: ApplicationCorrectionBinding<Schema>>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        (
            ApplicationEncodedInput<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>,
            std::sync::Arc<WorthQueryPreparedCorrectionSource>,
        ),
        WorthQueryCorrectionDerivationDenial,
    >
    where
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    {
        derivation::derive_correction_inverse::<Schema, C>(
            self,
            access,
            phase,
            principal,
            request,
            source,
            key,
            maximum_history,
            maximum_results,
        )
    }

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
        let execution = phase
            .execution_request_for(&application.product_runtime)
            .map_err(|_| WorthQueryCorrectionSelectionDenial::ForeignPhase)?;
        if receipt.runtime_authority() != application.runtime.authority_identity() {
            return Err(WorthQueryCorrectionSelectionDenial::ForeignRuntime);
        }
        let occurrence = application
            .primary_provider
            .observe_completed_application(receipt.commit_reference())
            .ok_or(WorthQueryCorrectionSelectionDenial::SourceUnavailable)?;
        let performed = occurrence.committed_product_publication();
        let claimed = receipt.committed_product_publication();
        if performed.composite_commit() != claimed.composite_commit()
            || performed.publication_attempt() != claimed.publication_attempt()
            || performed.product_incarnation() != claimed.product_incarnation()
            || occurrence.commit_reference() != receipt.commit_reference()
        {
            return Err(WorthQueryCorrectionSelectionDenial::SourcePublicationMismatch);
        }
        let original = occurrence.commit_evidence().original_authority();
        let retained = original
            .retained_governed_input_carrier()
            .ok_or(WorthQueryCorrectionSelectionDenial::SourceNotCorrectable)?;
        if retained.correction().is_none() || retained.original_scope().is_none() {
            return Err(WorthQueryCorrectionSelectionDenial::SourceNotCorrectable);
        }
        let ancestry = application
            .product_runtime
            .owner
            .observation_port()
            .select_product_branch_ancestor(
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
            occurrence,
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

struct DiscloseSelection<'a, 'runtime, 'phase, Schema> {
    target: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    phase: &'a WorthQueryAdvancementPhase<'phase>,
    principal: &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &'a WorthQueryRequestScope,
    source: WorthQuerySelectedCorrectionSource<'runtime, Schema>,
    maximum: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
}
impl<'runtime, Schema: ApplicationSchema> ApplicationCorrectionVisitor<Schema>
    for DiscloseSelection<'_, 'runtime, '_, Schema>
{
    type Output = Result<
        WorthQuerySelectedCorrectionSource<'runtime, Schema>,
        WorthQueryCorrectionSelectionDenial,
    >;
    fn unavailable(self) -> Self::Output {
        Err(WorthQueryCorrectionSelectionDenial::SourceNotCorrectable)
    }
    fn declared<C: ApplicationCorrectionBinding<Schema>>(self) -> Self::Output {
        disclosure::disclose_correction_source::<Schema, C>(
            self.target,
            self.phase,
            self.principal,
            self.request,
            &self.source,
            self.maximum,
            self.maximum_results,
        )
        .map_err(WorthQueryCorrectionSelectionDenial::Disclosure)?;
        Ok(self.source)
    }
}

impl<Schema> WorthQuerySelectedCorrectionSource<'_, Schema> {
    /// Native declaration dispatch retained by the original performed occurrence.
    pub fn source_binding_type(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> std::any::TypeId {
        self.occurrence
            .commit_evidence()
            .original_authority()
            .original_mutation_binding()
            .expect("selected correction retained its native binding")
    }

    pub(in crate::domain_computation::primary_graph) fn application(
        &self,
    ) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.application
    }

    pub(in crate::domain_computation::primary_graph) fn ancestry(
        &self,
    ) -> &ProductBranchAncestorSelection {
        &self.ancestry
    }

    pub(in crate::domain_computation::primary_graph) fn occurrence(
        &self,
    ) -> &WorthQueryPrimaryGraphCommittedApplication {
        &self.occurrence
    }
}
