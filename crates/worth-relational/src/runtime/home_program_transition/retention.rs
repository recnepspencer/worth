//! Every root-scoped catalog shares its original admitted compiled owners.
use super::*;
impl RelationalRuntime {
    pub(crate) fn retain_home_program_contracts(
        &self,
        request: &ExecutionRequest<'_, '_>,
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        contracts: Arc<SchemaContractRuntimeSubsystem>,
        digest: HomeProgramTransitionCatalogDigest,
    ) -> Result<PreparedRelationalHomeProgramContracts, RelationalHomeProgramContractDenial> {
        use RelationalHomeProgramContractDenial as Denial;
        let snapshot = self.configuration_binding().snapshot();
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<RelationalRootExecutionContracts>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(Denial::Request)?;
        let identity = catalog_receipt::RelationalHomeProgramCatalogIdentity::issue(request)?;
        request.consult().map_err(Denial::Request)?;
        Ok(PreparedRelationalHomeProgramContracts {
            meaning: Arc::new(RelationalRootExecutionContracts {
                identity,
                runtime: self.runtime_instance_id(),
                configuration: snapshot.config,
                configuration_contracts: snapshot.schema_contract_runtime,
                schema,
                contracts,
                digest,
                _memory: memory,
            }),
        })
    }
}
