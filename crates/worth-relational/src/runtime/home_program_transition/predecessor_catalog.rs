//! Preserve genuine declared A executors separately from current replay schema.
use super::*;
use crate::branch::RecordedRelationalComponentReference;

#[derive(Debug)]
pub(crate) struct RelationalHomeProgramPredecessorCatalog {
    pub(crate) source: RecordedRelationalComponentReference,
    pub(crate) contracts: PreparedRelationalHomeProgramContracts,
}
impl RelationalRuntime {
    pub(crate) fn retain_home_predecessor_catalog(
        &self,
        request: &ExecutionRequest<'_, '_>,
        source: &RecordedRelationalComponentReference,
        basis: &AdmittedRelationalBranchBasis,
        contracts: Arc<SchemaContractRuntimeSubsystem>,
    ) -> Result<RelationalHomeProgramPredecessorCatalog, RelationalHomeProgramContractDenial> {
        let schema = basis.inner.root.retained_schema_authority();
        let digest = catalog_digest::digest_frozen(
            schema.authority_digest(),
            &contracts.custom_invariant_registries,
            request,
        )?;
        let contracts = self.retain_home_program_contracts(request, schema, contracts, digest)?;
        request
            .consult()
            .map_err(RelationalHomeProgramContractDenial::Request)?;
        // Recovery is still exclusive here. The exact source root was admitted
        // against declared A; no current B registry or execution is substituted.
        basis
            .inner
            .root
            .execution_contracts
            .set(contracts.meaning.clone())
            .map_err(|_| RelationalHomeProgramContractDenial::SourceContractChanged)?;
        Ok(RelationalHomeProgramPredecessorCatalog {
            source: source.clone(),
            contracts,
        })
    }
}
