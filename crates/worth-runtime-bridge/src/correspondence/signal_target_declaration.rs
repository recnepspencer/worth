use std::sync::Arc;

use worth_signal::facade::{
    InstalledSignalAspectCapability, InstalledSignalNodeCapability, NodeId, PartitionToken,
};

use crate::mapping::BridgeAspectRegistrationId;

use super::{BridgeCorrespondenceDenial, BridgeCorrespondenceDenialKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BridgeSignalSlotRequest {
    Allocate,
    Exact(Arc<InstalledSignalAspectCapability>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSignalAspectTargetDeclaration {
    pub(crate) aspect_registration_id: BridgeAspectRegistrationId,
    pub(crate) node_capability: Arc<InstalledSignalNodeCapability>,
    pub(crate) partition: PartitionToken,
    pub(crate) node: NodeId,
    pub(crate) slot: BridgeSignalSlotRequest,
}

impl BridgeSignalAspectTargetDeclaration {
    pub fn allocate(
        aspect_registration_id: BridgeAspectRegistrationId,
        partition: PartitionToken,
        node: InstalledSignalNodeCapability,
    ) -> Self {
        let node = Arc::new(node);
        Self {
            aspect_registration_id,
            partition,
            node: node.node(),
            node_capability: node,
            slot: BridgeSignalSlotRequest::Allocate,
        }
    }

    pub fn exact(
        aspect_registration_id: BridgeAspectRegistrationId,
        partition: PartitionToken,
        node: InstalledSignalNodeCapability,
        aspect: InstalledSignalAspectCapability,
    ) -> Result<Self, BridgeCorrespondenceDenial> {
        if node.graph_instance_id() != aspect.graph_instance_id() || node.node() != aspect.node() {
            return Err(BridgeCorrespondenceDenial::without_admission(
                BridgeCorrespondenceDenialKind::MixedGraphTargetSet,
            ));
        }
        let node = Arc::new(node);
        Ok(Self {
            aspect_registration_id,
            partition,
            node: node.node(),
            node_capability: node,
            slot: BridgeSignalSlotRequest::Exact(Arc::new(aspect)),
        })
    }

    /// Stable text projection of the registration identity retained by this
    /// target declaration. It grants no correspondence or Signal authority.
    #[doc(hidden)]
    pub fn aspect_registration_identity(&self) -> &str {
        self.aspect_registration_id.as_str()
    }

    /// Read-only Signal partition retained by this target declaration.
    /// This is descriptive installation evidence and grants no correspondence
    /// or Signal mutation authority.
    #[doc(hidden)]
    pub fn partition(&self) -> &PartitionToken {
        &self.partition
    }

    pub(crate) fn graph_instance_id(&self) -> u64 {
        self.node_capability.graph_instance_id()
    }

    pub(crate) fn canonical_registration_key(&self) -> String {
        let slot = match &self.slot {
            BridgeSignalSlotRequest::Allocate => "allocate".to_string(),
            BridgeSignalSlotRequest::Exact(aspect) => {
                format!("exact:{}", aspect.aspect().index())
            }
        };
        [
            self.aspect_registration_id.as_str().to_string(),
            self.graph_instance_id().to_string(),
            self.partition.0.clone(),
            self.node.index().to_string(),
            self.node.generation().to_string(),
            slot,
        ]
        .into_iter()
        .map(|field| format!("{}:{field}", field.len()))
        .collect()
    }
}
