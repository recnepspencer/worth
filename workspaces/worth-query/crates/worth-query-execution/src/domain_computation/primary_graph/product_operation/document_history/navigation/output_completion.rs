//! Original SDK qualification preserves typed root/model provenance through adoption.
use super::*;
use crate::domain_computation::primary_graph::{
    application_installation::{WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner},
    WorthQueryAdvancementPhase, WorthQueryCompletedOriginalReadOutputs,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandSettlement,
};
use crate::publication_boundary::{
    discovery_currentness::WorthQueryRetainedQueryDependencies,
    original_read_outputs::WorthQueryOriginalReadOutputDenial,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_installation::facade::ApplicationSchema;

/// A complete original tree with the model correspondence of its actual root
/// admission. This is neither a selectable observation nor current permission.
pub struct WorthQueryCompletedHistoryOutputs {
    pub(super) profile: WorthQueryHistoryOutputProfile,
    pub(super) original: WorthQueryCompletedOriginalReadOutputs,
}

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn qualify_original_history_outputs<'s, R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        scope: &WorthQueryRequestScope,
        source: &WorthQueryHistoryOutputSource,
        root: &'s WorthQueryOutputDemandSettlement,
        dependents: impl Iterator<Item = &'s WorthQueryOutputDemandSettlement> + Clone,
        dependencies: Option<&WorthQueryRetainedQueryDependencies>,
    ) -> Result<WorthQueryCompletedHistoryOutputs, WorthQueryOriginalReadOutputDenial> {
        use WorthQueryOriginalReadOutputDenial as Denial;
        let foreign = || {
            Denial::Output(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "history output root/profile correspondence",
            ))
        };
        if source.profile.root != std::any::TypeId::of::<R>()
            || source.profile.program_type != std::any::TypeId::of::<P>()
            || source.profile.program != *self.owned_revision()
        {
            return Err(foreign());
        }
        source
            .root_completion
            .require_original(root)
            .map_err(Denial::Output)?;
        let original = self.qualify_original_read_outputs_with_header::<R>(
            phase,
            access,
            scope,
            &source.source,
            root,
            dependents,
            dependencies,
            std::mem::size_of::<WorthQueryCompletedHistoryOutputs>() as u64,
        )?;
        let selected = self
            .runtime()
            .select_application_read_observation(phase, original.original_endpoint())
            .map_err(Denial::Observation)?;
        let owner = self
            .selected_program_owner(&selected)
            .map_err(Denial::Program)?;
        if owner.owned_revision() != &source.profile.program {
            return Err(foreign());
        }
        drop(selected);
        Ok(WorthQueryCompletedHistoryOutputs {
            profile: source.profile.clone(),
            original,
        })
    }
}

impl<S: ApplicationSchema> NavigationProgress<'_, S> {
    pub(super) fn accept_outputs(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        outputs: WorthQueryCompletedHistoryOutputs,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let profile = self.output_profile.as_ref().ok_or(Denial::IntentDrift)?;
        let NavigationPhase::AwaitingOutputs { exact, .. } = &self.phase else {
            return Err(Denial::IntentDrift);
        };
        let source = outputs.original.original_source();
        let endpoint = outputs.original.original_endpoint();
        if !outputs.profile.same_custody(profile)
            || profile.model != self.model
            || source
                .product_observation()
                .compare(exact.observation())
                .is_err()
            || !endpoint.same_application_occurrence(source)
            || !endpoint.belongs_to_selected_occurrence(self.application, exact.observation())
        {
            return Err(Denial::IntentDrift);
        }
        let selected = self
            .application
            .select_application_read_observation(phase, endpoint)
            .map_err(Denial::ProductAdmission)?;
        let owner = self
            .application
            .selected_program_owner(&selected)
            .map_err(|_| {
                Denial::RequiresSemanticCorrection(
                    WorthQueryHistoryNavigationEligibilityDenial::ProgramUnavailable,
                )
            })?;
        if owner.owned_revision() != &profile.program
            || !owner.owns_output_root(crate::equality_type_id::EqualityTypeId::from(profile.root))
        {
            return Err(Denial::RequiresSemanticCorrection(
                WorthQueryHistoryNavigationEligibilityDenial::ProgramChanged,
            ));
        }
        let endpoint = endpoint.retained_product_lease();
        drop(selected);
        phase
            .check_live_for(&self.application.product_runtime)
            .map_err(Denial::Stopped)?;
        let NavigationPhase::AwaitingOutputs {
            candidate,
            exact,
            _publication,
        } = std::mem::replace(&mut self.phase, NavigationPhase::Discarded)
        else {
            unreachable!()
        };
        self.phase = NavigationPhase::Selecting {
            candidate,
            exact: endpoint,
            _publication,
            _outputs: Some(outputs),
        };
        drop(exact);
        Ok(())
    }
}
