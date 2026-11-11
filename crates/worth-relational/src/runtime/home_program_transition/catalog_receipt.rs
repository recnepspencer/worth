//! Exact compiled inventory identity without retaining any live runtime port.
use super::{PreparedRelationalHomeProgramContracts, RelationalRootExecutionContracts};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

/// Independent identity custody: a receipt retains neither compiled meaning
/// nor runtime resources, and its allocation stays funded until its last share.
pub(super) struct RelationalHomeProgramCatalogIdentity {
    _memory: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for RelationalHomeProgramCatalogIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RelationalHomeProgramCatalogIdentity")
    }
}
impl RelationalHomeProgramCatalogIdentity {
    pub(super) fn issue(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, super::RelationalHomeProgramContractDenial> {
        use super::RelationalHomeProgramContractDenial as Denial;
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(Denial::Request)?;
        request.consult().map_err(Denial::Request)?;
        Ok(Arc::new(Self { _memory: memory }))
    }
}

/// A receiver compares the exact compiled inventory it installed. This receipt
/// conveys no read, Source, publication or migration permission.
#[derive(Clone, Debug)]
pub struct RelationalHomeProgramCatalogReceipt {
    identity: Arc<RelationalHomeProgramCatalogIdentity>,
}
impl RelationalHomeProgramCatalogReceipt {
    pub fn same_inventory(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
}
impl RelationalRootExecutionContracts {
    pub(crate) fn catalog_receipt(self: &Arc<Self>) -> RelationalHomeProgramCatalogReceipt {
        RelationalHomeProgramCatalogReceipt {
            identity: self.identity.clone(),
        }
    }
}
impl PreparedRelationalHomeProgramContracts {
    pub fn catalog_receipt(&self) -> RelationalHomeProgramCatalogReceipt {
        self.meaning.catalog_receipt()
    }
}
