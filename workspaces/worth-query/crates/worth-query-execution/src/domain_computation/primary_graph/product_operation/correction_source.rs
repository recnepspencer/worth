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

mod alternatives;
mod cold_selection;
mod exact_original;
mod occurrence;
mod selection;
use occurrence::CorrectionOccurrence;
mod authored_identity;
pub use alternatives::{
    WorthQueryCorrectionAlternativeRef, WorthQueryCorrectionAlternativeResolution,
    WorthQueryCorrectionAlternativesContinuation, WorthQueryCorrectionAlternativesPage,
    WorthQuerySelectedCorrectionAlternative,
};
pub(in crate::domain_computation::primary_graph) use authored_identity::AuthoredCorrectionCandidateDenial;
mod compatibility;
pub(crate) mod restored_support;
pub use restored_support::WorthQueryCorrectionRestoredSupportDenial;
mod derivation;
mod inspection;
mod intent;
mod key;
mod lineage;
pub(crate) mod material;
mod preparation;
mod reapplication;
mod replay;
mod required_output;
pub use intent::WorthQueryCorrectionIntent;
pub use key::WorthQueryCorrectionCommandKey;
pub use material::WorthQueryCorrectionOutputProjectionDenial;
pub use preparation::WorthQueryPreparedCorrectionSource;
pub use replay::{WorthQueryCorrectionActionInputSelection, WorthQueryCorrectionInputSelection};
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
    occurrence: CorrectionOccurrence,
}

#[derive(Debug)]
/// Why an exact performed occurrence could not be selected as a correction source.
pub enum WorthQueryCorrectionSelectionDenial {
    StaleIncarnation,
    Material(crate::facade::correction_material::QueryCorrectionMaterialDenial),
    ExactHistory(RuntimeWorldServiceDenial<worth_runtime_world::facade::ProductExactHistoryDenial>),
    HistoryLimit,
    HistorySource(RuntimeWorldServiceDenial<worth_runtime_world::facade::WorldReadmissionDenial>),
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
            None,
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
    /// Native World parent links followed when selecting this source.
    /// Zero means the source was the selected target occurrence itself.
    pub fn ancestry_parent_links_followed(&self) -> usize {
        self.ancestry.parent_links_followed()
    }

    /// Native declaration dispatch retained by the original performed occurrence.
    pub fn source_binding_type(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> std::any::TypeId {
        self.occurrence
            .original_binding()
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

    pub(in crate::domain_computation::primary_graph) fn occurrence(&self) -> &CorrectionOccurrence {
        &self.occurrence
    }
}
#[cfg(test)]
#[path = "correction_source_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
