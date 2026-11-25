//! Every root-scoped catalog shares its original admitted compiled owners.
use super::*;
use worth_relational_contracts::internal_port::schema::home_program_catalog_receipt::RelationalHomeProgramCatalogReceiptOwnerPort as _;
impl RelationalRuntime {
    pub(crate) fn retain_home_program_contracts(
        &self,
        request: &ExecutionRequest<'_, '_>,
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        contracts: Arc<SchemaContractRuntimeSubsystem>,
        digest: HomeProgramTransitionCatalogDigest,
        receiver_epoch: crate::runtime::RelationalRuntimeConfigurationSnapshot,
    ) -> Result<PreparedRelationalHomeProgramContracts, RelationalHomeProgramContractDenial> {
        use RelationalHomeProgramContractDenial as Denial;
        let binding = self.configuration_binding();
        let current = binding.operation();
        if !receiver_epoch.same_epoch(&current) {
            return Err(Denial::SourceContractChanged);
        }
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<RelationalRootExecutionContracts>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(Denial::Request)?;
        let identity_layout = RelationalHomeProgramCatalogReceipt::allocation_layout()
            .map_err(|_| Denial::CapacityOverflow)?;
        let identity_memory = request
            .reserve_retained_memory(identity_layout.size() as u64)
            .map_err(Denial::Request)?;
        request.consult().map_err(Denial::Request)?;
        let identity = RelationalHomeProgramCatalogReceipt::funded(identity_memory);
        request.consult().map_err(Denial::Request)?;
        Ok(PreparedRelationalHomeProgramContracts {
            meaning: Arc::new(RelationalRootExecutionContracts {
                identity,
                runtime: self.runtime_instance_id(),
                schema,
                contracts,
                digest,
                _memory: memory,
            }),
            receiver_epoch,
            index_definitions: None,
            configuration: None,
        })
    }
}
