//! One attached raw Native owner admits initial schema before its one recovery.
use super::*;
use worth_execution::ExecutionRequest;
use worth_proof::RelationalReadmitted;
use worth_relational::facade::runtime::{
    RelationalInitialSchemaInstallationReceipt, RelationalRuntime,
};
use worth_store_relational::RelationalSectionReadmissionDenial as ReadmissionDenial;

pub struct UnrecoveredApplicationHome {
    pub(super) relational: worth_store_relational::UnrecoveredRelationalSection,
    pub(super) context: crate::home_context::HomeContext,
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
    },
}
impl UnrecoveredApplicationHome {
    pub fn prepare_native_schema_for_home(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ApplicationHomeNativeSchemaPreparation<'_>,
        worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial,
    > {
        use super::initialization::HomeInitializationOrigin;
        use worth_store::physical_runtime::RecoveryStorePresence;
        match &self.context.initialization {
            HomeInitializationOrigin::Available(RecoveryStorePresence::ProvenAbsent) => {
                let initialization = self.relational.home_fresh_schema_initialization(request)?;
                Ok(ApplicationHomeNativeSchemaPreparation::Fresh { native: self.relational.home_schema_installation_runtime(), initialization })
            },
            HomeInitializationOrigin::Available(RecoveryStorePresence::ExistingArtifacts) => Ok(ApplicationHomeNativeSchemaPreparation::Existing { native: self.relational.home_schema_installation_runtime() }),
            HomeInitializationOrigin::ConsumedNewHome => Err(worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial::from_initial_inputs(
                worth_relational::facade::durability::RelationalInitialInputsRefusal::Malformed)),
        }
    }
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.context.descriptor.identity
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.context.descriptor.incarnation
    }
    pub fn readmit(
        mut self,
        expected: &RelationalInitialSchemaInstallationReceipt,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        RelationalReadmitted<UnsharedReadmittedApplicationHome>,
        RelationalHomeReadmissionRefused,
    > {
        let manifest = match self.selected_source(request) {
            Ok(manifest) => manifest,
            Err(cause) => return Err(self.refused(cause)),
        };
        if expected.runtime_instance_id()
            != self
                .relational
                .home_schema_installation_runtime()
                .runtime_instance_id()
        {
            return Err(RelationalHomeReadmissionRefused {
                cause: crate::ApplicationHomeOpenRefusal::Relational(
                    ReadmissionDenial::InitialSchemaRuntimeMismatch,
                ),
                home: Box::new(self),
            });
        }
        self.readmit_selected_source(manifest, request)
    }

    pub(crate) fn readmit_source_for_home(
        mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        RelationalReadmitted<UnsharedReadmittedApplicationHome>,
        RelationalHomeReadmissionRefused,
    > {
        let manifest = match self.selected_source(request) {
            Ok(manifest) => manifest,
            Err(cause) => return Err(self.refused(cause)),
        };
        self.readmit_selected_source(manifest, request)
    }

    fn selected_source(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<crate::checkpoint::SelectedHomeManifest>, crate::ApplicationHomeOpenRefusal>
    {
        request
            .consult()
            .map_err(crate::ApplicationHomeOpenRefusal::Execution)?;
        self.context
            .selected_home_manifest(request)
            .map_err(crate::ApplicationHomeOpenRefusal::Checkpoint)
    }

    fn refused(self, cause: crate::ApplicationHomeOpenRefusal) -> RelationalHomeReadmissionRefused {
        RelationalHomeReadmissionRefused {
            cause,
            home: Box::new(self),
        }
    }

    fn readmit_selected_source(
        self,
        manifest: Option<crate::checkpoint::SelectedHomeManifest>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        RelationalReadmitted<UnsharedReadmittedApplicationHome>,
        RelationalHomeReadmissionRefused,
    > {
        let Self {
            relational,
            context,
        } = self;
        use super::initialization::HomeInitializationOrigin;
        use worth_store::physical_runtime::RecoveryStorePresence;
        let catalog =
            match crate::open::readmission::transition::admit_pending_catalog(&context, request) {
                Ok(catalog) => catalog,
                Err(cause) => {
                    return Err(Self {
                        relational,
                        context,
                    }
                    .refused(crate::ApplicationHomeOpenRefusal::Transition(cause)))
                }
            };
        let admitted = match &context.initialization {
            HomeInitializationOrigin::Available(RecoveryStorePresence::ProvenAbsent) => {
                if manifest.is_some() {
                    Err((relational, ReadmissionDenial::MalformedSection))
                } else {
                    relational
                        .admit_fresh_home_from_source(request)
                        .map(|relational| (relational, None))
                }
            }
            HomeInitializationOrigin::Available(RecoveryStorePresence::ExistingArtifacts) => {
                relational.readmit_prepared_home(request, catalog.as_ref())
            }
            HomeInitializationOrigin::ConsumedNewHome => {
                Err((relational, ReadmissionDenial::MalformedSection))
            }
        };
        drop(catalog);
        match admitted {
            Ok((relational, selected_coverage)) => {
                context
                    .checkpoint
                    .admit_selected_native_coverage(selected_coverage);
                Ok(RelationalReadmitted::from_owner(
                    UnsharedReadmittedApplicationHome {
                        relational,
                        context,
                    },
                ))
            }
            Err((relational, cause)) => Err(RelationalHomeReadmissionRefused {
                cause: crate::ApplicationHomeOpenRefusal::Relational(cause),
                home: Box::new(Self {
                    relational,
                    context,
                }),
            }),
        }
    }
}
