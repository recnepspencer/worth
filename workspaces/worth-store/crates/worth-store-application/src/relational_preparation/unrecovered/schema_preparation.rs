//! Fresh permission and existing Source DATA are different compiler inputs.
use super::*;

#[derive(Debug)]
pub enum ApplicationHomeNativeSchemaPreparationDenial {
    Initialization(worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial),
    Source(ReadmissionDenial),
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
                let (native, source, maximum_kind, maximum_aspect) = self.relational.prepare_home_existing_schema_source(request)
                    .map_err(Denial::Source)?;
                Ok(ApplicationHomeNativeSchemaPreparation::Existing { native, source, maximum_kind, maximum_aspect })
            }
            HomeInitializationOrigin::ConsumedNewHome => Err(Denial::Initialization(
                worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial::from_initial_inputs(
                    worth_relational::facade::durability::RelationalInitialInputsRefusal::Malformed),
            )),
        }
    }
}
