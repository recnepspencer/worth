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
impl UnrecoveredApplicationHome {
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.context.descriptor.identity
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.context.descriptor.incarnation
    }
    pub fn native_for_schema_installation(&mut self) -> &mut RelationalRuntime {
        self.relational.home_schema_installation_runtime()
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
        let admitted = match &context.initialization {
            HomeInitializationOrigin::Available(RecoveryStorePresence::ProvenAbsent) => {
                if manifest.is_some() {
                    Err((relational, ReadmissionDenial::CompleteNativeRequired))
                } else {
                    relational
                        .admit_fresh_home_from_source(request)
                        .map(|relational| (relational, None))
                }
            }
            HomeInitializationOrigin::Available(RecoveryStorePresence::ExistingArtifacts) => {
                relational.readmit_home_from_read_without_schema(
                    manifest.as_ref().map(|manifest| manifest.relational_read()),
                    request,
                )
            }
            HomeInitializationOrigin::ConsumedNewHome => {
                Err((relational, ReadmissionDenial::MalformedSection))
            }
        };
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
