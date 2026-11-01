//! One attached raw Native owner admits initial schema before its one recovery.
use super::*;
use worth_execution::ExecutionRequest;
use worth_proof::RelationalReadmitted;
use worth_relational::facade::runtime::{
    RelationalInitialSchemaInstallationReceipt, RelationalRuntime,
};

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
        if let Err(cause) = request.consult() {
            return Err(RelationalHomeReadmissionRefused {
                cause: crate::ApplicationHomeOpenRefusal::Execution(cause),
                home: Box::new(self),
            });
        }
        let manifest = match self.context.selected_home_manifest(request) {
            Ok(manifest) => manifest,
            Err(cause) => {
                return Err(RelationalHomeReadmissionRefused {
                    cause: crate::ApplicationHomeOpenRefusal::Checkpoint(cause),
                    home: Box::new(self),
                })
            }
        };
        let Self {
            relational,
            context,
        } = self;
        match relational.readmit_home_from_read(
            manifest.as_ref().map(|manifest| manifest.relational_read()),
            expected,
            request,
        ) {
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
