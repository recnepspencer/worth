//! Exact compiled inventory identity without retaining any live runtime port.
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

/// Independent identity custody: a receipt retains neither compiled meaning
/// nor runtime resources, and its allocation stays funded until its last share.
struct RelationalHomeProgramCatalogIdentity {
    _memory: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for RelationalHomeProgramCatalogIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RelationalHomeProgramCatalogIdentity")
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

/// Operations used by the Relational owners.
pub trait RelationalHomeProgramCatalogReceiptOwnerPort: Sized {
    fn allocation_layout() -> Result<std::alloc::Layout, std::alloc::LayoutError>;
    fn funded(memory: Option<ExecutionMemoryReservation>) -> Self;
}

impl RelationalHomeProgramCatalogReceiptOwnerPort for RelationalHomeProgramCatalogReceipt {
    fn allocation_layout() -> Result<std::alloc::Layout, std::alloc::LayoutError> {
        Ok(std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<
                RelationalHomeProgramCatalogIdentity,
            >())?
            .0
            .pad_to_align())
    }

    fn funded(memory: Option<ExecutionMemoryReservation>) -> Self {
        Self {
            identity: Arc::new(RelationalHomeProgramCatalogIdentity { _memory: memory }),
        }
    }
}
