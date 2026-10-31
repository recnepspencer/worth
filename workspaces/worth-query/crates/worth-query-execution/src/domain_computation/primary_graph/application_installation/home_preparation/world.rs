//! Consuming attachment adopts the actual Home World without rebuilding it.
use super::super::super::application_runtime::installation::home_attachment;
use super::super::super::WorthQueryBootstrapAdvancementPhase;
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
use worth_runtime_world::facade::*;
#[cfg(test)]
#[path = "world/tests.rs"]
mod tests;

pub struct ReadmittedHomeQueryService<Schema, Program> {
    runtime: WorthQueryProgramApplicationRuntime<Schema, Program>,
}
impl<Schema: 'static, Program: 'static> worth_proof::AuthorityMarker
    for ReadmittedHomeQueryService<Schema, Program>
{
}
impl<Schema, Program> ReadmittedHomeQueryService<Schema, Program> {
    pub fn into_home_program_runtime(self) -> WorthQueryProgramApplicationRuntime<Schema, Program> {
        self.runtime
    }
}

pub struct QueryHomeWorldAttachmentRefused<Schema: ApplicationSchemaComposition, Program> {
    custody: HomeWorldAttachmentCustody<Schema, Program>,
}
enum HomeWorldAttachmentCustody<Schema: ApplicationSchemaComposition, Program> {
    Prepared {
        _prepared: PreparedHomeQueryRuntime<Schema, Program>,
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
        mut self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        publication_port: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: RuntimeWorldBranchPort,
        observation: RuntimeWorldObservationPort,
        inspection: RuntimeWorldInspectionPort,
        recovery: RuntimeWorldRecoveryPort,
        root: ProductBranchObservation,
        directory_support: fn(worth_foundational::facade::DurableStateFamily, u8) -> Option<u8>,
    ) -> Result<
        worth_proof::QueryReadmitted<ReadmittedHomeQueryService<Schema, Program>>,
        QueryHomeWorldAttachmentRefused<Schema, Program>,
    > {
        let prepared = (|| {
            let publication = self.publication.clone().ok_or_else(|| {
                home_world_denial("actual Home Query seed publication is required")
            })?;
            let (provider, participation, authorization, handlers) =
                home_attachment::preflight_home_application(
                    phase,
                    &self.bootstrap,
                    &self.state.runtime,
                    &self.state.authority,
                    &self.state.installed,
                )?;
            self.bootstrap
                .managed_indexes
                .validate_complete(&self.state.installed)?;
            let relational = self
                .bootstrap
                .graph
                .integration_handle()
                .prepare_product_source(
                    &phase.execution_request(),
                    &self
                        .bootstrap
                        .graph
                        .integration_handle()
                        .with_runtime(&phase.execution_request(), |native| {
                            native.main_branch_identity()
                        }),
                )
                .map_err(|cause| {
                    home_world_denial(format!("Home product source denied: {cause:?}"))
                })?;
            let product = WorthQueryProductRuntime::attach_home_world_ports(
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
            )
            .map_err(|cause| home_world_denial(cause.detail()))?;
            Ok((
                publication,
                provider,
                participation,
                authorization,
                handlers,
                product,
            ))
        })();
        let (publication, provider, participation, authorization, handlers, product) =
            match prepared {
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
        let PreparedHomeQueryRuntime {
            state,
            bootstrap,
            bridge,
            publication: _,
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
            crate::domain_computation::authorization::WorthQueryRuntimeClock::system(),
            handlers,
            bridge,
            product,
        );
        application.managed_indexes = managed_indexes;
        application.program_support = Some(
            super::super::super::program_occurrence::WorthQueryInstalledProgramSupport::installed(
                state.support.roster,
                state.activation,
            ),
        );
        application
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
            worth_proof::QueryReadmitted::from_owner(ReadmittedHomeQueryService { runtime })
        })
        .map_err(|refused| QueryHomeWorldAttachmentRefused {
            custody: HomeWorldAttachmentCustody::Program(refused),
        })
    }
}
fn home_world_denial(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        detail,
    )
}
