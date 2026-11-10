//! Home selection always consumes semantic material, including during a warm open.
use super::occurrence::{HomeCorrectionOccurrence, OriginalCorrectionLineage};
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionLineageBinding,
    WorthQueryCorrectionSourceCoordinates, WorthQueryRecordedCorrectionSource,
};
use crate::domain_computation::primary_graph::retained_correction::RetainedCorrectionMaterial;
use worth_runtime_world::facade::WorldProgramActivationHistory;

impl<'runtime, S: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, S> {
    pub(super) fn select_home_correction_source(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        coordinates: &WorthQueryCorrectionSourceCoordinates,
        maximum: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<WorthQuerySelectedCorrectionSource<'runtime, S>, WorthQueryCorrectionSelectionDenial>
    {
        use WorthQueryCorrectionSelectionDenial as D;
        let application = self.application();
        let execution = phase
            .execution_request_for(&application.runtime.product_runtime)
            .map_err(|_| D::ForeignPhase)?;
        let owner = &application.runtime.product_runtime.owner;
        if owner
            .owner_identity()
            .durable()
            .map(|owner| worth_foundational::facade::WorldOwnerCoordinate::Home { owner })
            != Some(coordinates.world_owner())
        {
            return Err(D::SourcePublicationMismatch);
        }
        let material = application
            .runtime
            .primary_provider
            .observe_correction_material(coordinates.relational())
            .map_err(D::Material)?;
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
        let observed = exact_original::resolve(
            execution,
            &owner.observation_port(),
            self.product().observation(),
            coordinates,
            maximum,
        )?;
        let ancestry = owner
            .observation_port()
            .select_retained_product_branch_ancestor(
                &execution,
                self.product().observation(),
                &observed,
                maximum,
            )
            .map_err(D::Ancestry)?;
        let source_type = application
            .installed_schema()
            .installed_mutation_binding_descriptor(&material.capture.mutation)
            .ok_or(D::SourceNotCorrectable)?
            .binding_type();
        let fact = application
            .runtime
            .primary_provider
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
            .execution_request_for(&application.runtime.product_runtime)
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
        .runtime
        .product_runtime
        .owner
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
                identity,
                coordinate.relational().clone(),
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
