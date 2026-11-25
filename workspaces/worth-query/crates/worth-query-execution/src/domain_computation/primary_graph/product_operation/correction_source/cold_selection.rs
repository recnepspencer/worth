//! Home selection always consumes semantic material, including during a warm open.
use super::occurrence::{HomeCorrectionOccurrence, OriginalCorrectionLineage};
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionLineageBinding,
    WorthQueryCorrectionSourceCoordinates, WorthQueryRecordedCorrectionSource,
};
use crate::domain_computation::primary_graph::retained_correction::RetainedCorrectionMaterial;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_runtime_world::facade::WorldProgramActivationHistory;

/// An original already resolved by a walk of this target's exact history.
///
/// Only the in-crate history listing constructs it, from the very page whose
/// coordinates it accompanies, so selection need not walk the history again.
pub(super) struct WalkedOriginal {
    pub(super) identity: worth_runtime_world::facade::CompositeCommitIdentity,
    pub(super) parent_links_bound: NonZeroUsize,
}

impl<'runtime, S: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, S> {
    pub(super) fn select_home_correction_source(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        coordinates: &WorthQueryCorrectionSourceCoordinates,
        known: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        maximum: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
        walked: Option<WalkedOriginal>,
    ) -> Result<WorthQuerySelectedCorrectionSource<'runtime, S>, WorthQueryCorrectionSelectionDenial>
    {
        use WorthQueryCorrectionSelectionDenial as D;
        let application = self.application();
        let execution = phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|_| D::ForeignPhase)?;
        let owner = &application.inner.runtime.product_runtime_ref().owner_ref();
        if owner
            .owner_identity()
            .durable()
            .map(|owner| worth_foundational::facade::WorldOwnerCoordinate::Home { owner })
            != Some(coordinates.world_owner())
        {
            return Err(D::SourcePublicationMismatch);
        }
        let material = application
            .inner
            .runtime
            .primary_provider_ref()
            .observe_correction_material(coordinates.relational(), coordinates.member())
            .map_err(|cause| match cause {
                crate::facade::correction_material::QueryCorrectionMaterialDenial::MemberUnavailable => {
                    D::SourceMemberUnavailable
                }
                cause => D::Material(cause),
            })?;
        let originals = owner
            .inspection_port()
            .observe_program_activation_history(&execution)
            .map_err(D::HistorySource)?;
        let original = originals
            .original_state(material.capture.original)
            .ok_or(D::SourceUnavailable)?;
        if original.reserved_commit().ordinal() != coordinates.world_commit()
            || original.reserved_commit().owner() != material.capture.original.owner()
        {
            return Err(D::SourcePublicationMismatch);
        }
        let (observed, ancestor_bound) = match (walked, known) {
            (Some(walked), _) => (walked.identity, Some(walked.parent_links_bound)),
            (None, Some(known)) => (known.clone(), maximum),
            (None, None) => (
                exact_original::resolve(
                    execution,
                    &owner.observation_port(),
                    self.product().observation(),
                    coordinates,
                    maximum,
                )?,
                maximum,
            ),
        };
        let ancestry = owner
            .observation_port()
            .select_retained_product_branch_ancestor(
                &execution,
                self.product().observation(),
                &observed,
                ancestor_bound,
            )
            .map_err(D::Ancestry)?;
        if ancestry
            .source()
            .basis()
            .relational_basis()
            .observation()
            .commit_receipt()
            != Some(coordinates.relational())
        {
            return Err(D::SourcePublicationMismatch);
        }
        let source_type = application
            .installed_schema()
            .installed_mutation_binding_descriptor(&material.capture.mutation)
            .ok_or(D::SourceNotCorrectable)?
            .binding_type();
        let fact = application
            .inner
            .runtime
            .primary_provider_ref()
            .observe_correction_original_fact(&execution, &original, &material.ready.commit)
            .map_err(D::Material)?;
        let causality = fact
            .map(|fact| read_lineage(self, execution, &fact, maximum))
            .transpose()?;
        S::visit_correction(
            source_type,
            ReadHomeSelection {
                target: self,
                phase,
                principal,
                request,
                material,
                originals,
                ancestry,
                causality,
                maximum,
                maximum_results,
            },
        )
    }
}
struct ReadHomeSelection<'a, 'runtime, 'phase, S> {
    target: &'a WorthQuerySelectedProductOperation<'runtime, S>,
    phase: &'a WorthQueryAdvancementPhase<'phase>,
    principal: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &'a WorthQueryRequestScope,
    material: RetainedCorrectionMaterial,
    originals: WorldProgramActivationHistory,
    ancestry: ProductBranchAncestorSelection,
    causality: Option<OriginalCorrectionLineage>,
    maximum: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
}
impl<'runtime, S: ApplicationSchema> ApplicationCorrectionVisitor<S>
    for ReadHomeSelection<'_, 'runtime, '_, S>
{
    type Output = Result<
        WorthQuerySelectedCorrectionSource<'runtime, S>,
        WorthQueryCorrectionSelectionDenial,
    >;
    fn unavailable(self) -> Self::Output {
        Err(WorthQueryCorrectionSelectionDenial::SourceNotCorrectable)
    }
    fn declared<C: ApplicationCorrectionBinding<S>>(self) -> Self::Output {
        let application = self.target.application();
        let execution = self
            .phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|_| WorthQueryCorrectionSelectionDenial::ForeignPhase)?;
        let original = self
            .originals
            .original_state(self.material.capture.original)
            .ok_or(WorthQueryCorrectionSelectionDenial::SourceUnavailable)?;
        let occurrence = HomeCorrectionOccurrence::readmit::<S, C>(
            application,
            self.material,
            &original,
            self.causality,
            &execution,
        )
        .map_err(WorthQueryCorrectionSelectionDenial::Disclosure)?;
        let source = WorthQuerySelectedCorrectionSource {
            application,
            ancestry: self.ancestry,
            occurrence: CorrectionOccurrence::Home(occurrence),
        };
        disclosure::disclose_correction_source::<S, C>(
            self.target,
            self.phase,
            self.principal,
            self.request,
            &source,
            self.maximum,
            self.maximum_results,
        )
        .map_err(WorthQueryCorrectionSelectionDenial::Disclosure)?;
        Ok(source)
    }
}
pub(super) fn read_lineage<S: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, S>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    fact: &crate::domain_computation::primary_graph::provider::WorthQueryCanonicalCorrectionFact,
    maximum: Option<NonZeroUsize>,
) -> Result<OriginalCorrectionLineage, WorthQueryCorrectionSelectionDenial> {
    use WorthQueryCorrectionSelectionDenial as D;
    let observation = target
        .application()
        .inner
        .runtime
        .product_runtime_ref()
        .owner_ref()
        .observation_port();
    let resolve = |coordinate: &WorthQueryCorrectionSourceCoordinates| {
        exact_original::resolve(
            execution,
            &observation,
            target.product().observation(),
            coordinate,
            maximum,
        )
        .map(|identity| {
            WorthQueryRecordedCorrectionSource::from_untrusted_parts(
                ARCHIVE_DECODE,
                identity,
                coordinate.relational().clone(),
                coordinate.member(),
            )
        })
    };
    // The Native row's coordinate grammar is checked by the sole original-row
    // parser. Each actual identity is then observed in this selected lineage.
    let _source = resolve(&fact.source)?;
    let root = resolve(&fact.root)?;
    let lineage = match fact.role {
        WorthQueryAftermathCausalRole::RecordedReverse => {
            WorthQueryCorrectionLineageBinding::reverse(
                root,
                fact.prior.as_ref().map(resolve).transpose()?,
            )
        }
        WorthQueryAftermathCausalRole::RecordedReapply => {
            WorthQueryCorrectionLineageBinding::reapply(
                root,
                resolve(fact.selected.as_ref().ok_or(D::SourcePublicationMismatch)?)?,
                resolve(fact.active.as_ref().ok_or(D::SourcePublicationMismatch)?)?,
            )
        }
    };
    Ok(OriginalCorrectionLineage {
        role: fact.role,
        lineage,
    })
}
