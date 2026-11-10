//! Consuming attachment adopts the actual Home World without rebuilding it.
use super::super::super::application_runtime::installation::home_attachment;
use super::super::super::WorthQueryBootstrapAdvancementPhase;
use super::source::assembly::HomeQueryPreparation;
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
use worth_runtime_world::facade::*;
mod custody;
mod dispatch_custody;
pub use crate::domain_computation::primary_graph::bootstrap::branch_activation_history::QueryHomeProgramActivationDenial;
pub use custody::{QueryHomeCustodyReadmissionDenial, QueryHomeCustodyReadmissionRefused};
#[cfg(test)]
#[path = "world/tests.rs"]
mod tests;

pub struct ReadmittedHomeQueryService<Schema, Program> {
    runtime: WorthQueryProgramApplicationRuntime<Schema, Program>,
    program_support_admission: std::sync::OnceLock<worth_proof::AdmittedHomeQueryObligations>,
    correction_material_admission: std::sync::OnceLock<worth_proof::AdmittedHomeQueryCorrections>,
    dispatch_custody_admission: std::sync::OnceLock<worth_proof::AdmittedHomeQueryDispatch>,
}
impl<Schema: 'static, Program: 'static> worth_proof::AuthorityMarker
    for ReadmittedHomeQueryService<Schema, Program>
{
}
impl<Schema, Program> ReadmittedHomeQueryService<Schema, Program> {
    pub fn correction_material_admission(
        &self,
    ) -> Option<worth_proof::AdmittedHomeQueryCorrections> {
        self.correction_material_admission.get().cloned()
    }
    pub fn program_support_admission(&self) -> Option<worth_proof::AdmittedHomeQueryObligations> {
        self.program_support_admission.get().cloned()
    }
    pub fn dispatch_custody_admission(&self) -> Option<worth_proof::AdmittedHomeQueryDispatch> {
        self.dispatch_custody_admission.get().cloned()
    }
    pub fn into_home_program_runtime(self) -> WorthQueryProgramApplicationRuntime<Schema, Program> {
        self.runtime
    }
}

pub struct QueryHomeWorldAttachmentRefused<Schema: ApplicationSchemaComposition, Program> {
    pub(super) custody: HomeWorldAttachmentCustody<Schema, Program>,
}
pub(super) enum HomeWorldAttachmentCustody<Schema: ApplicationSchemaComposition, Program> {
    Prepared {
        _prepared: HomeQueryPreparation<Schema, Program>,
        _root: ProductBranchObservation,
        cause: WorthQueryPrimaryGraphInstallationDenial,
    },
    Program(super::super::program::assembly::ProgramAssemblyRefused<Schema, Program>),
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryHomeWorldAttachmentRefused<Schema, Program>
{
    pub fn detail(&self) -> String {
        match &self.custody {
            HomeWorldAttachmentCustody::Prepared { cause, .. } => cause.to_string(),
            HomeWorldAttachmentCustody::Program(refused) => refused.cause.to_string(),
        }
    }
}

impl<Schema, Program> PreparedHomeQueryRuntime<Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    pub fn attach_home_world(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        publication_port: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: RuntimeWorldBranchPort,
        observation: RuntimeWorldObservationPort,
        inspection: RuntimeWorldInspectionPort,
        recovery: RuntimeWorldRecoveryPort,
        root: ProductBranchObservation,
        directory_support: fn(worth_foundational::facade::DurableStateFamily, u8) -> Option<u8>,
        maintenance: std::sync::Arc<dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>,
    ) -> Result<
        worth_proof::QueryReadmitted<ReadmittedHomeQueryService<Schema, Program>>,
        QueryHomeWorldAttachmentRefused<Schema, Program>,
    > {
        let Some(publication) = self.publication else {
            return Err(QueryHomeWorldAttachmentRefused {
                custody: HomeWorldAttachmentCustody::Prepared {
                    _prepared: self.prepared,
                    _root: root,
                    cause: home_world_denial("actual Home Query seed publication is required"),
                },
            });
        };
        self.prepared.attach_world(
            phase,
            publication,
            publication_port,
            branch,
            observation,
            inspection,
            recovery,
            root,
            directory_support,
            maintenance,
        )
    }
}
impl<Schema, Program> HomeQueryPreparation<Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn attach_world(
        mut self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        publication: super::super::super::WorthQueryPrimaryGraphPublication,
        publication_port: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: RuntimeWorldBranchPort,
        observation: RuntimeWorldObservationPort,
        inspection: RuntimeWorldInspectionPort,
        recovery: RuntimeWorldRecoveryPort,
        root: ProductBranchObservation,
        directory_support: fn(worth_foundational::facade::DurableStateFamily, u8) -> Option<u8>,
        maintenance: std::sync::Arc<dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>,
    ) -> Result<
        worth_proof::QueryReadmitted<ReadmittedHomeQueryService<Schema, Program>>,
        QueryHomeWorldAttachmentRefused<Schema, Program>,
    > {
        let prepared = (|| {
            let (provider, participation, authorization, handlers, resources, mutation_partitions) =
                home_attachment::preflight_home_application(
                    phase,
                    &mut self.bootstrap,
                    &self.state.runtime,
                    &self.state.authority,
                    &self.state.installed,
                    root.basis().relational_basis(),
                )?;
            self.bootstrap
                .readmit_home_activation_record(phase, &root)?;
            let relational = self
                .bootstrap
                .graph
                .integration_handle()
                .prepare_product_source_at_basis(
                    &phase.execution_request(),
                    root.basis().relational_basis(),
                )
                .map_err(|cause| {
                    home_world_denial(format!("Home product source denied: {cause:?}"))
                })?;
            let product = WorthQueryProductRuntime::attach_home_world_ports(
                &phase.execution_request(),
                relational,
                &mut self.bridge.conditional_lifecycle(),
                self.state.limits.world.clone(),
                publication_port,
                branch,
                observation,
                inspection,
                recovery,
                root.clone(),
                directory_support,
                maintenance,
            )
            .map_err(|cause| home_world_denial(cause.detail()))?;
            Ok((
                publication,
                provider,
                participation,
                authorization,
                handlers,
                resources,
                product,
                mutation_partitions,
            ))
        })();
        let (
            publication,
            provider,
            participation,
            authorization,
            handlers,
            resources,
            product,
            mutation_partitions,
        ) = match prepared {
            Ok(prepared) => prepared,
            Err(cause) => {
                return Err(QueryHomeWorldAttachmentRefused {
                    custody: HomeWorldAttachmentCustody::Prepared {
                        _prepared: self,
                        _root: root,
                        cause,
                    },
                })
            }
        };
        // The actual configured providers remain in the refused preparation
        // through every fallible attachment check. Success transfers them once.
        let managed_indexes = std::mem::take(&mut self.bootstrap.managed_indexes);
        let HomeQueryPreparation {
            state,
            bootstrap,
            bridge,
        } = self;
        let mut application = home_attachment::finish_home_application(
            phase,
            bootstrap,
            state.runtime,
            state.installed,
            publication,
            provider,
            participation,
            authorization,
            match state.authorization_time_source {
                Some(source) => {
                    crate::domain_computation::authorization::WorthQueryRuntimeClock::from_source(
                        source,
                    )
                }
                None => crate::domain_computation::authorization::WorthQueryRuntimeClock::system(),
            },
            handlers,
            resources,
            managed_indexes,
            bridge,
            product,
            mutation_partitions,
        );
        application.program_support = Some(
            super::super::super::program_occurrence::WorthQueryInstalledProgramSupport::installed(
                state.support.roster,
                state.activation,
            ),
        );
        application
            .runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .bind_implementation_build(state.build_identity);
        super::super::program::assembly::assemble_owning(
            application,
            Some((state.program, state.supported)),
        )
        .map(|runtime| {
            worth_proof::QueryReadmitted::from_owner(ReadmittedHomeQueryService {
                runtime,
                program_support_admission: std::sync::OnceLock::new(),
                correction_material_admission: std::sync::OnceLock::new(),
                dispatch_custody_admission: std::sync::OnceLock::new(),
            })
        })
        .map_err(|refused| QueryHomeWorldAttachmentRefused {
            custody: HomeWorldAttachmentCustody::Program(refused),
        })
    }
}
pub(super) fn home_world_denial(
    detail: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        detail,
    )
}
