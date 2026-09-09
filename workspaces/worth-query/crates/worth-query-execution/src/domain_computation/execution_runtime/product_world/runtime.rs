use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;
use worth_runtime_world::facade::{ProductBranchIdentity, RuntimeWorldOwner};

use std::sync::Arc;

use super::{activation::WorthQueryProductActivationRegistry, WorthQueryProductWorldClock};

#[derive(Debug)]
pub(crate) struct WorthQueryProductRootIdentity {
    _private: (),
}

/// One live Query composition of World and its exact source-observation custody.
///
/// Application and installed-domain entry points select through this owner. The
/// default branch is a caller convenience; World alone resolves its current head.
#[derive(Clone)]
pub struct WorthQueryProductRuntime {
    pub(crate) owner: Arc<RuntimeWorldOwner<(), (), (), (), ()>>,
    pub(crate) source: RuntimeBridgeRelationalSource,
    pub(crate) activations: Arc<WorthQueryProductActivationRegistry>,
    pub(crate) clock: WorthQueryProductWorldClock,
    pub(crate) default_branch: ProductBranchIdentity,
    pub(crate) root_identity: Arc<WorthQueryProductRootIdentity>,
}

impl WorthQueryProductRuntime {
    pub(crate) fn from_parts(
        owner: RuntimeWorldOwner<(), (), (), (), ()>,
        source: RuntimeBridgeRelationalSource,
        activations: WorthQueryProductActivationRegistry,
        clock: WorthQueryProductWorldClock,
        default_branch: ProductBranchIdentity,
    ) -> Self {
        Self {
            owner: Arc::new(owner),
            source,
            activations: Arc::new(activations),
            clock,
            default_branch,
            root_identity: Arc::new(WorthQueryProductRootIdentity { _private: () }),
        }
    }

    pub fn default_branch(&self) -> &ProductBranchIdentity {
        &self.default_branch
    }

    pub(crate) fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Arc::clone(&self.root_identity)
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(crate) fn operation_control(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOperationControl {
        self.owner.operation_control()
    }
}
