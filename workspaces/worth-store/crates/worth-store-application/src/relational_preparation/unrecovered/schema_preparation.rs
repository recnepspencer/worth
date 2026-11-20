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
        identities: ApplicationHomeNativeIdentityBound,
        installation: Option<
            worth_proof::AdmittedHomeProgramTransitionSuccessor<
                worth_foundational::facade::HomeProgramTransitionInstallationBinding,
            >,
        >,
    },
}

/// The occupied identities a readmitted Native meaning continues from. Store
/// chooses the bound for the open it admitted; the compiler only consumes it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationHomeNativeIdentityBound {
    /// No transition is pending: every durable schema role stays reserved.
    Occupied {
        maximum_kind: Option<worth_relational::facade::identity::KindId>,
        maximum_aspect: Option<worth_foundational::facade::AspectIdentity>,
    },
    /// A pending transition resumes from its recorded planning predecessor:
    /// the identities occupied when that predecessor was durable, so the
    /// successor recompile reproduces the recorded plan.
    TransitionPlanning {
        maximum_kind: Option<worth_relational::facade::identity::KindId>,
        maximum_aspect: Option<worth_foundational::facade::AspectIdentity>,
    },
}
impl ApplicationHomeNativeIdentityBound {
    pub fn maximum_kind(&self) -> Option<worth_relational::facade::identity::KindId> {
        match self {
            Self::Occupied { maximum_kind, .. } | Self::TransitionPlanning { maximum_kind, .. } => {
                *maximum_kind
            }
        }
    }
    pub fn maximum_aspect(&self) -> Option<worth_foundational::facade::AspectIdentity> {
        match self {
            Self::Occupied { maximum_aspect, .. }
            | Self::TransitionPlanning { maximum_aspect, .. } => *maximum_aspect,
        }
    }
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
                let binding = &self.context.descriptor.binding;
                let installation = self.context.transition.installation_binding(binding)
                    .map_err(Denial::Transition)?;
                let target = installation.as_ref().map(|binding| binding.binding().target());
                // An open admitted with a pending intent is its transition's
                // resumption; ordinary opens of a pending Home refuse earlier.
                let planning = self.context.transition.committed_intent(binding)
                    .map_err(Denial::Transition)?
                    .map(|intent| intent.native_predecessor());
                let (native, source, maximum_kind, maximum_aspect) = self.relational
                    .prepare_home_existing_schema_source(request, target, planning)
                    .map_err(Denial::Source)?;
                let identities = match planning {
                    Some(_) => ApplicationHomeNativeIdentityBound::TransitionPlanning { maximum_kind, maximum_aspect },
                    None => ApplicationHomeNativeIdentityBound::Occupied { maximum_kind, maximum_aspect },
                };
                Ok(ApplicationHomeNativeSchemaPreparation::Existing { native, source, identities, installation })
            }
            HomeInitializationOrigin::ConsumedNewHome => Err(Denial::Initialization(
                worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial::from_initial_inputs(
                    worth_relational::facade::durability::RelationalInitialInputsRefusal::Malformed),
            )),
        }
    }
}
