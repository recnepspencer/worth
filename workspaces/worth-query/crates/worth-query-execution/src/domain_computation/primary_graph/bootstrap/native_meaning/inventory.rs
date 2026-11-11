//! Expected Native inventories derived from the actual compiled Query meaning.
use super::*;
use worth_relational::facade::runtime::{CustomInvariantRegistration, ManagedIndexSourceProgram};

pub(in crate::domain_computation::primary_graph) struct NativeSchemaInventory {
    digest: [u8; 32],
    sources: Vec<ManagedIndexSourceProgram>,
    public_sources: Vec<ManagedIndexSourceProgram>,
}
impl NativeSchemaInventory {
    pub(in crate::domain_computation::primary_graph) fn compile(
        layout: &WorthQueryPrimaryGraphLayout,
        registrations: &[CustomInvariantRegistration],
    ) -> Self {
        Self {
            digest: worth_relational::facade::runtime::custom_invariant_inventory_digest(
                registrations,
            ),
            sources: layout
                .required_managed_sources()
                .map(|source| source.program.clone())
                .collect(),
            public_sources: layout
                .required_managed_sources()
                .filter(|source| source.definition.maintenance().native_cold().is_some())
                .map(|source| source.program.clone())
                .collect(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(in crate::domain_computation::primary_graph) fn take_public_sources(
        &mut self,
    ) -> Vec<ManagedIndexSourceProgram> {
        std::mem::take(&mut self.public_sources)
    }
    pub(in crate::domain_computation::primary_graph) fn require_receipt(
        &self,
        receipt: &RelationalInitialSchemaInstallationReceipt,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if receipt.custom_invariant_inventory_digest() != &self.digest {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch,
                "Relational installed an invariant inventory outside the application catalog",
            ));
        }
        if !receipt.has_exact_managed_index_sources(&self.sources) {
            return Err(primary_graph_denial(WorthQueryPrimaryGraphInstallationDenialKind::ManagedSourceInstallationReceiptMismatch,
                "Relational installed a Source roster outside the compiled application catalog"));
        }
        Ok(())
    }
}
