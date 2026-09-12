use super::PhysicalPublicationDenial;
use crate::{PhysicalOrderingContract, PhysicalOrderingSite};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootSwapOrderingContract {
    ordering: PhysicalOrderingContract,
}

impl RootSwapOrderingContract {
    pub fn acquire_release_or_stronger() -> Self {
        Self {
            ordering: PhysicalOrderingContract::root_swap_acquire_release(),
        }
    }

    pub fn from_ordering(
        ordering: PhysicalOrderingContract,
    ) -> Result<Self, PhysicalPublicationDenial> {
        Ok(Self {
            ordering: ordering.require_site(PhysicalOrderingSite::RootSwap)?,
        })
    }

    pub const fn ordering(self) -> PhysicalOrderingContract {
        self.ordering
    }
}
