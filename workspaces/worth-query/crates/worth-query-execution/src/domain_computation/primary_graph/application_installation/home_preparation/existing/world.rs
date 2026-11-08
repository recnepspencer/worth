//! Existing service consumes one actual selected root and its recovered meaning.
use super::super::world::{HomeWorldAttachmentCustody, QueryHomeWorldAttachmentRefused};
use super::*;
use worth_runtime_world::facade::*;
pub struct QueryExistingHomeWorldAttachmentRefused<Schema: ApplicationSchemaComposition, Program> {
    inner: QueryHomeWorldAttachmentRefused<Schema, Program>,
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryExistingHomeWorldAttachmentRefused<Schema, Program>
{
    pub fn detail(&self) -> String {
        self.inner.detail()
    }
}
impl<Schema, Program> PreparedExistingHomeQueryRuntime<Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    pub fn attach_existing_home_world(
        self,
        phase: &super::super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
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
        QueryExistingHomeWorldAttachmentRefused<Schema, Program>,
    > {
        let publication =
            match super::super::super::super::bootstrap::home_publication::readmission::readmit(
                &self.prepared.bootstrap,
                phase,
                &self.prepared.state.installed,
                &self.prepared.state.support.roster,
                &self.prepared.state.support.initial_revision,
                &root,
                self.prepared.state.transition_installation.as_ref(),
            ) {
                Ok(publication) => publication,
                Err(cause) => {
                    return Err(QueryExistingHomeWorldAttachmentRefused {
                        inner: QueryHomeWorldAttachmentRefused {
                            custody: HomeWorldAttachmentCustody::Prepared {
                                _prepared: self.prepared,
                                _root: root,
                                cause,
                            },
                        },
                    })
                }
            };
        self.prepared
            .attach_world(
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
            .map_err(|inner| QueryExistingHomeWorldAttachmentRefused { inner })
    }
}
