//! Bounded lineage projection carries complete active-reversal resolution into selection.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::application_aftermath::{
    WorthQueryCorrectionSourceCoordinates, WorthQueryRecordedCorrectionSource,
};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryActiveReversalProgress, WorthQueryCanonicalCorrectionFact,
};

/// Descriptive reference to one canonical historical reversal in a lineage.
///
/// It remains inspectable when optional source material expires. Selecting it
/// separately resolves retained native evidence and freshly checks disclosure;
/// possession of the reference grants no permission to reapply an operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionAlternativeRef {
    fact: WorthQueryCanonicalCorrectionFact,
}
impl WorthQueryCorrectionAlternativeRef {
    /// The exact performed source occurrence that this historical reversal operated on.
    pub fn source(&self) -> &WorthQueryCorrectionSourceCoordinates {
        &self.fact.source
    }
    /// The original ordinary occurrence shared by this correction lineage.
    pub fn root(&self) -> &WorthQueryCorrectionSourceCoordinates {
        &self.fact.root
    }
    /// The reversal's own co-committed Relational occurrence.
    pub fn performed_commit(&self) -> &worth_relational::facade::history::RelationalCommitReceipt {
        &self.fact.child
    }
}

/// Opaque continuation of a bounded traversal of one lineage's reversal index.
///
/// It carries partial active-reversal resolution and binds the exact target,
/// selector and native index generation. Continuing requires fresh disclosure;
/// a changed basis or generation stops traversal instead of replacing its meaning.
pub struct WorthQueryCorrectionAlternativesContinuation {
    root: WorthQueryRecordedCorrectionSource,
    target: WorthQueryProductBranchReadIdentity,
    progress: WorthQueryActiveReversalProgress,
}
/// Completed native classification of the active reversal on one exact target.
///
/// Only a complete lineage traversal produces this value. It permits selection
/// of an alternative from any page of that traversal, while selection and later
/// preparation still require current disclosure and ordinary action admission.
pub struct WorthQueryCorrectionAlternativeResolution {
    root: WorthQueryRecordedCorrectionSource,
    target: WorthQueryProductBranchReadIdentity,
    active: Option<WorthQueryCanonicalCorrectionFact>,
}
/// One bounded page of canonical historical alternatives after fresh disclosure.
///
/// Entries describe historical facts; they do not imply current applicability.
/// Continue until a resolution is available before selecting an alternative.
pub struct WorthQueryCorrectionAlternativesPage {
    entries: Vec<WorthQueryCorrectionAlternativeRef>,
    continuation: Option<WorthQueryCorrectionAlternativesContinuation>,
    resolution: Option<WorthQueryCorrectionAlternativeResolution>,
}
impl WorthQueryCorrectionAlternativesPage {
    /// Historical reversal references returned by this page's native posting range.
    pub fn entries(&self) -> &[WorthQueryCorrectionAlternativeRef] {
        &self.entries
    }
    /// The next bounded traversal step, absent when the lineage has been fully visited.
    pub fn continuation(&self) -> Option<&WorthQueryCorrectionAlternativesContinuation> {
        self.continuation.as_ref()
    }
    /// Complete active-reversal classification, available only on the final page.
    pub fn resolution(&self) -> Option<&WorthQueryCorrectionAlternativeResolution> {
        self.resolution.as_ref()
    }
}

/// Disclosed historical alternative retaining exact native source and reversal ancestry.
///
/// The selected historical reversal and currently active reversal have distinct
/// identities. An absent active reversal still permits an exact command retry;
/// first-use reapplication requires an unconsumed active reversal. Preparing an
/// action freshly checks disclosure, current program support and source authority.
pub struct WorthQuerySelectedCorrectionAlternative<'runtime, Schema> {
    pub(super) source: WorthQuerySelectedCorrectionSource<'runtime, Schema>,
    pub(super) root: WorthQueryRecordedCorrectionSource,
    pub(super) selected: WorthQueryRecordedCorrectionSource,
    pub(super) active: Option<WorthQueryRecordedCorrectionSource>,
    _selected_ancestry: ProductBranchAncestorSelection,
    _active_ancestry: Option<ProductBranchAncestorSelection>,
}
impl<Schema> WorthQuerySelectedCorrectionAlternative<'_, Schema> {
    /// Reifies the retained source binding for publication's typed declaration dispatch.
    pub fn source_binding_type(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> std::any::TypeId {
        self.source.source_binding_type(access)
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    /// Reads a bounded lineage page with fresh disclosure and carried native continuation.
    pub fn correction_alternatives(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
        maximum_entries: NonZeroUsize,
        after: Option<&WorthQueryCorrectionAlternativesContinuation>,
    ) -> Result<WorthQueryCorrectionAlternativesPage, WorthQueryCorrectionDerivationDenial> {
        use WorthQueryCorrectionDerivationDenial as Denial;
        self.select_recorded_correction_source(
            phase,
            principal,
            request,
            &super::lineage::source_locator(source),
            maximum_history,
            maximum_results,
        )
        .map_err(|denial| Denial::Selection(Box::new(denial)))?;
        let root = super::lineage::reverse_identity(source)?.root().clone();
        let target =
            WorthQueryProductBranchReadIdentity::from_observation(self.product().observation());
        if after.is_some_and(|after| after.root != root || after.target != target) {
            return Err(Denial::ForeignAlternative);
        }
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        let provider = &self.application().primary_provider;
        let (facts, progress) = provider
            .correction_alternatives_progress_at_basis(
                &execution,
                self.product().relational_basis(),
                &root,
                maximum_entries.get(),
                after.map(|after| &after.progress),
            )
            .map_err(Denial::History)?;
        let entries = facts
            .into_iter()
            .map(|fact| WorthQueryCorrectionAlternativeRef { fact })
            .collect();
        let (continuation, resolution) = match progress {
            WorthQueryActiveReversalProgress::Complete(active) => (
                None,
                Some(WorthQueryCorrectionAlternativeResolution {
                    root,
                    target,
                    active,
                }),
            ),
            progress => (
                Some(WorthQueryCorrectionAlternativesContinuation {
                    root,
                    target,
                    progress,
                }),
                None,
            ),
        };
        Ok(WorthQueryCorrectionAlternativesPage {
            entries,
            continuation,
            resolution,
        })
    }

    /// Resolves a historical reference after complete traversal and pins its native ancestry.
    pub fn select_correction_alternative(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        selected: &WorthQueryCorrectionAlternativeRef,
        resolution: &WorthQueryCorrectionAlternativeResolution,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        WorthQuerySelectedCorrectionAlternative<'runtime, Schema>,
        WorthQueryCorrectionDerivationDenial,
    > {
        use WorthQueryCorrectionDerivationDenial as Denial;
        if resolution.target
            != WorthQueryProductBranchReadIdentity::from_observation(self.product().observation())
        {
            return Err(Denial::ForeignAlternative);
        }
        let provider = &self.application().primary_provider;
        let selected = provider
            .performed_correction_source(&selected.fact)
            .ok_or(Denial::SemanticMaterialUnavailable)?;
        let occurrence = provider
            .observe_completed_application(selected.relational())
            .ok_or(Denial::SemanticMaterialUnavailable)?;
        let fact = occurrence
            .aftermath_causality()
            .ok_or(Denial::ForeignAlternative)?;
        let lineage = fact
            .correction_lineage()
            .ok_or(Denial::ForeignAlternative)?;
        if lineage.role() != crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole::RecordedReverse
            || lineage.root() != &resolution.root { return Err(Denial::ForeignAlternative); }
        let operated = fact.recorded_source().ok_or(Denial::ForeignAlternative)?;
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        let canonical = provider
            .reversal_of_source_at_basis(&execution, self.product().relational_basis(), operated)
            .map_err(Denial::History)?
            .ok_or(Denial::ForeignAlternative)?;
        if canonical.child != *selected.relational() || canonical.record != *fact.fact_record() {
            return Err(Denial::ForeignAlternative);
        }
        let active = resolution
            .active
            .as_ref()
            .map(|fact| {
                provider
                    .performed_correction_source(fact)
                    .ok_or(Denial::SemanticMaterialUnavailable)
            })
            .transpose()?;
        let selected_ancestry =
            self.select_alternative_ancestry(phase, &selected, maximum_history)?;
        let active_ancestry = active
            .as_ref()
            .map(|active| self.select_alternative_ancestry(phase, active, maximum_history))
            .transpose()?;
        let source = self
            .select_recorded_correction_source(
                phase,
                principal,
                request,
                operated,
                maximum_history,
                maximum_results,
            )
            .map_err(|denial| Denial::Selection(Box::new(denial)))?;
        Ok(WorthQuerySelectedCorrectionAlternative {
            source,
            root: resolution.root.clone(),
            selected,
            active,
            _selected_ancestry: selected_ancestry,
            _active_ancestry: active_ancestry,
        })
    }

    pub(super) fn select_alternative_ancestry(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        source: &WorthQueryRecordedCorrectionSource,
        maximum_history: Option<NonZeroUsize>,
    ) -> Result<ProductBranchAncestorSelection, WorthQueryCorrectionDerivationDenial> {
        use WorthQueryCorrectionDerivationDenial as Denial;
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        self.application()
            .product_runtime
            .owner
            .observation_port()
            .select_product_branch_ancestor(
                &execution,
                self.product().observation(),
                source.world(),
                maximum_history,
            )
            .map_err(Denial::Ancestry)
    }
}
