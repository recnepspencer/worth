//! Fresh permission and existing Source DATA are different compiler inputs.
use super::*;

#[derive(Debug)]
pub enum ApplicationHomeNativeSchemaPreparationDenial {
    Initialization(worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial),
    Source(ReadmissionDenial),
    Transition(crate::ApplicationHomeTransitionIntentDenial),
}

/// Source-origin routing precedes schema effects. Existing receivers receive
/// configuration preparation only; they cannot obtain initialization permission.
pub enum ApplicationHomeNativeSchemaPreparation<'runtime> {
    Fresh {
        native: &'runtime mut RelationalRuntime,
        initialization: worth_proof::FreshNativeInitialization<
            worth_relational::facade::durability::RelationalFreshInitialization,
        >,
    },
    Existing {
        native: &'runtime mut RelationalRuntime,
        source: &'runtime worth_relational::facade::schema::RelationalSchemaRegistry,
        maximum_kind: Option<worth_relational::facade::identity::KindId>,
        maximum_aspect: Option<worth_foundational::facade::AspectIdentity>,
        installation: Option<
            worth_proof::AdmittedHomeProgramTransitionSuccessor<
                worth_foundational::facade::HomeProgramTransitionInstallationBinding,
            >,
        >,
    },
}
impl UnrecoveredApplicationHome {
    pub fn prepare_native_schema_for_home(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ApplicationHomeNativeSchemaPreparation<'_>,
        ApplicationHomeNativeSchemaPreparationDenial,
    > {
        use crate::relational_preparation::initialization::HomeInitializationOrigin;
        use worth_store::physical_runtime::RecoveryStorePresence;
        use ApplicationHomeNativeSchemaPreparationDenial as Denial;
        match &self.context.initialization {
            HomeInitializationOrigin::Available(RecoveryStorePresence::ProvenAbsent) => {
                let initialization = self.relational.home_fresh_schema_initialization(request)
                    .map_err(Denial::Initialization)?;
                Ok(ApplicationHomeNativeSchemaPreparation::Fresh {
                    native: self.relational.home_schema_installation_runtime(), initialization,
                })
            }
            HomeInitializationOrigin::Available(RecoveryStorePresence::ExistingArtifacts) => {
                let installation = self.context.transition.installation_binding(&self.context.descriptor.binding)
                    .map_err(Denial::Transition)?;
                let target = installation.as_ref().map(|binding| binding.binding().target());
                let (native, source, maximum_kind, maximum_aspect) = self.relational.prepare_home_existing_schema_source(request, target)
                    .map_err(Denial::Source)?;
                Ok(ApplicationHomeNativeSchemaPreparation::Existing { native, source, maximum_kind, maximum_aspect, installation })
            }
            HomeInitializationOrigin::ConsumedNewHome => Err(Denial::Initialization(
                worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial::from_initial_inputs(
                    worth_relational::facade::durability::RelationalInitialInputsRefusal::Malformed),
            )),
        }
    }
}

/// The identities occupied at a pending transition's recorded planning
/// predecessor. Distinct from the prepared bound, which keeps reporting every
/// schema role; only the resumed successor recompile consumes this.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHomeTransitionPlanningMaxima {
    maximum_kind: Option<worth_relational::facade::identity::KindId>,
    maximum_aspect: Option<worth_foundational::facade::AspectIdentity>,
}
impl ApplicationHomeTransitionPlanningMaxima {
    pub fn maximum_kind(&self) -> Option<worth_relational::facade::identity::KindId> {
        self.maximum_kind
    }
    pub fn maximum_aspect(&self) -> Option<worth_foundational::facade::AspectIdentity> {
        self.maximum_aspect
    }
}

impl UnrecoveredApplicationHome {
    /// `None` when no transition intent is pending.
    pub fn home_program_transition_planning_maxima(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<ApplicationHomeTransitionPlanningMaxima>,
        ApplicationHomeNativeSchemaPreparationDenial,
    > {
        use ApplicationHomeNativeSchemaPreparationDenial as Denial;
        let Some(intent) = self
            .context
            .transition
            .committed_intent(&self.context.descriptor.binding)
            .map_err(Denial::Transition)?
        else {
            return Ok(None);
        };
        let (maximum_kind, maximum_aspect) = self
            .relational
            .home_transition_planning_maxima(request, intent.native_predecessor())
            .map_err(Denial::Source)?;
        Ok(Some(ApplicationHomeTransitionPlanningMaxima {
            maximum_kind,
            maximum_aspect,
        }))
    }
}
