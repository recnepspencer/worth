//! Fresh source installation alone can cross the initialization durable step.
use super::*;
use crate::durability::source_initial_inputs::{
    image::InitialInputsImage, RelationalFreshInitialization,
};
use worth_execution::ExecutionRequest;
use worth_proof::{DurablyCommitted, FreshNativeInitialization};

#[derive(Debug)]
pub struct FreshRelationalInitialSchemaInstallation<'runtime> {
    installation: RelationalInitialSchemaInstallation<'runtime>,
    initialization: RelationalFreshInitialization,
}
impl RelationalRuntime {
    pub fn prepare_fresh_home_schema_installation(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        permission: FreshNativeInitialization<RelationalFreshInitialization>,
    ) -> Result<
        FreshRelationalInitialSchemaInstallation<'_>,
        RelationalInitialSchemaInstallationDenial,
    > {
        let initialization = permission.consume(|initialization| initialization);
        initialization
            .validate(self, request)
            .map_err(RelationalInitialSchemaInstallationDenial::from_initial_inputs)?;
        Ok(FreshRelationalInitialSchemaInstallation {
            installation: self.prepare_initial_schema_installation()?,
            initialization,
        })
    }
}
impl FreshRelationalInitialSchemaInstallation<'_> {
    pub fn install_with_managed_sources(
        self,
        request: &ExecutionRequest<'_, '_>,
        additions: RelationalSchemaRegistry,
        invariants: Vec<CustomInvariantRegistration>,
        sources: Vec<crate::validation::candidate_source::ManagedIndexSourceProgram>,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, RelationalInitialSchemaInstallationDenial>
    {
        self.installation.install_with_publication(
            additions,
            invariants,
            sources,
            Some((self.initialization, request)),
        )
    }
}

pub(crate) enum InitialSchemaCutover {
    Receiver,
    Durable(DurablyCommitted<crate::durability::medium::RelationalCommittedInitialInputs>),
}
impl InitialSchemaCutover {
    pub(crate) fn apply(self, cutover: impl FnOnce(Option<Arc<InitialInputsImage>>)) {
        match self {
            Self::Receiver => cutover(None),
            Self::Durable(committed) => {
                committed.consume(|publication| cutover(Some(publication.image)))
            }
        }
    }
}
