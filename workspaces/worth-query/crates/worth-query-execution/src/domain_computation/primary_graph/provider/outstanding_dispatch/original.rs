//! Original dispatch source protection is independent of disposable correlation indexes.
use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;
use worth_runtime_world::facade::{
    RuntimeWorldPerformedPublicationProtection, WorldReadmittedPublication,
};

pub(super) enum OutstandingDispatchOriginal {
    Performed(WorthQueryCommittedProductPublication),
    Protected {
        description: WorthQueryCommittedProductPublication,
        _protection: RuntimeWorldPerformedPublicationProtection,
    },
    Readmitted(WorldReadmittedPublication),
}

impl OutstandingDispatchOriginal {
    pub(super) fn matches(&self, original: &WorthQueryCommittedProductPublication) -> bool {
        match self {
            Self::Performed(performed)
            | Self::Protected {
                description: performed,
                ..
            } => performed == original,
            Self::Readmitted(world) => {
                world.after().selected_commit() == original.composite_commit()
                    && world.attempt_identity() == original.publication_attempt()
                    && world.native_receipt() == original.relational_commit()
                    && world.after().branch_identity() == original.product_branch()
                    && world.after().lifecycle_incarnation() == original.product_incarnation()
            }
        }
    }
    pub(super) fn has_live_basis(&self) -> bool {
        match self {
            Self::Performed(_) | Self::Protected { .. } => true,
            Self::Readmitted(_) => false,
        }
    }
}
