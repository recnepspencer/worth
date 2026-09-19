use std::sync::Arc;

use crate::data::graph::SignalGraph;

mod retained_charge;

/// Opaque identity for the one external authority allowed to allocate
/// semantic aspect correspondences in a Signal graph.
///
/// Clones identify the same owner. Independently minted values never do.
#[derive(Clone, Debug)]
pub struct SignalAspectLoweringOwner {
    identity: Arc<()>,
}

impl SignalAspectLoweringOwner {
    pub fn fresh() -> Self {
        Self {
            identity: Arc::new(()),
        }
    }

    pub(crate) fn is_same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalAspectLoweringOwnershipDenial {
    AlreadyOwned,
}

impl SignalGraph {
    pub fn claim_aspect_lowering_owner(
        &mut self,
        owner: &SignalAspectLoweringOwner,
    ) -> Result<(), SignalAspectLoweringOwnershipDenial> {
        match self.aspect_lowering_owner.as_ref() {
            Some(installed) if !installed.is_same_owner(owner) => {
                Err(SignalAspectLoweringOwnershipDenial::AlreadyOwned)
            }
            Some(_) => Ok(()),
            None => {
                self.aspect_lowering_owner = Some(owner.clone());
                Ok(())
            }
        }
    }
}
