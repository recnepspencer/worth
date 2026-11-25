//! Original dispatch source protection is independent of disposable correlation indexes.
use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;
use worth_runtime_world::facade::{
    RuntimeWorldPerformedPublicationProtection, WorldObservedHomePublication,
};

pub(super) enum OutstandingDispatchOriginal {
    Performed(WorthQueryCommittedProductPublication),
    Protected {
        description: WorthQueryCommittedProductPublication,
        _protection: RuntimeWorldPerformedPublicationProtection,
    },
    Readmitted(WorldObservedHomePublication),
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
                world.composite_commit() == original.composite_commit()
                    && world.publication_attempt() == original.publication_attempt()
                    && world.native_receipt() == original.relational_commit()
                    && world.product_branch() == original.product_branch()
                    && world.product_incarnation() == original.product_incarnation()
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
