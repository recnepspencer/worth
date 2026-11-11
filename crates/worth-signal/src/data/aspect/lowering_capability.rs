use std::collections::BTreeSet;
mod backing;
use worth_proof::{CapabilityMarker, CapabilityWitness, TransitionOutcome};

use crate::data::{graph::SignalGraph, handle::NodeId, node::NodeContract};

use super::Aspect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAspectCapabilityDenial {
    MissingOrStaleNode,
}

pub struct InstalledSignalAspectLoweringAuthority {
    _private: (),
}

impl CapabilityMarker for InstalledSignalAspectLoweringAuthority {}

#[derive(Debug, PartialEq, Eq)]
pub struct InstalledSignalGraphCapability {
    graph_instance_id: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstalledSignalNodeCapability {
    graph_instance_id: u64,
    node: NodeId,
    contract: NodeContract,
}

impl InstalledSignalNodeCapability {
    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub const fn node(&self) -> NodeId {
        self.node
    }

    pub fn contract(&self) -> &NodeContract {
        &self.contract
    }

    pub(crate) fn lowering_witness(
        &self,
    ) -> CapabilityWitness<InstalledSignalAspectLoweringAuthority> {
        CapabilityWitness::from_capability_marker(InstalledSignalAspectLoweringAuthority {
            _private: (),
        })
    }
}

impl InstalledSignalGraphCapability {
    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstalledSignalAspectCapability {
    graph_instance_id: u64,
    node: NodeId,
    aspect: Aspect,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstalledSignalAspectSetCapability {
    graph_instance_id: u64,
    aspects: Vec<InstalledSignalAspectCapability>,
}

impl InstalledSignalAspectSetCapability {
    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub fn aspects(&self) -> impl ExactSizeIterator<Item = (NodeId, Aspect)> + '_ {
        self.aspects
            .iter()
            .map(|capability| (capability.node, capability.aspect))
    }

    pub fn lowering_witness(&self) -> CapabilityWitness<InstalledSignalAspectLoweringAuthority> {
        CapabilityWitness::from_capability_marker(InstalledSignalAspectLoweringAuthority {
            _private: (),
        })
    }
}

impl InstalledSignalAspectCapability {
    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub const fn node(&self) -> NodeId {
        self.node
    }

    pub const fn aspect(&self) -> Aspect {
        self.aspect
    }

    pub fn lowering_witness(&self) -> CapabilityWitness<InstalledSignalAspectLoweringAuthority> {
        CapabilityWitness::from_capability_marker(InstalledSignalAspectLoweringAuthority {
            _private: (),
        })
    }
}

impl SignalGraph {
    pub fn installed_graph_capability(&self) -> InstalledSignalGraphCapability {
        InstalledSignalGraphCapability {
            graph_instance_id: self.runtime_instance_id(),
        }
    }

    pub fn admit_installed_aspect(
        &self,
        node: NodeId,
        aspect: Aspect,
    ) -> TransitionOutcome<InstalledSignalAspectCapability, SignalAspectCapabilityDenial> {
        match self.node_aspect_version(node) {
            Ok(_) => TransitionOutcome::Success(InstalledSignalAspectCapability {
                graph_instance_id: self.runtime_instance_id(),
                node,
                aspect,
            }),
            Err(_) => TransitionOutcome::Denied(SignalAspectCapabilityDenial::MissingOrStaleNode),
        }
    }

    pub fn admit_installed_node(
        &self,
        node: NodeId,
    ) -> TransitionOutcome<InstalledSignalNodeCapability, SignalAspectCapabilityDenial> {
        match self.get_contract(node) {
            Ok(contract) => TransitionOutcome::Success(InstalledSignalNodeCapability {
                graph_instance_id: self.runtime_instance_id(),
                node,
                contract: contract.clone(),
            }),
            Err(_) => TransitionOutcome::Denied(SignalAspectCapabilityDenial::MissingOrStaleNode),
        }
    }

    pub fn admit_installed_aspects(
        &self,
        aspects: impl IntoIterator<Item = (NodeId, Aspect)>,
    ) -> TransitionOutcome<InstalledSignalAspectSetCapability, SignalAspectCapabilityDenial> {
        let mut unique = BTreeSet::new();
        let mut capabilities = Vec::new();
        for (node, aspect) in aspects {
            if !unique.insert((node, aspect)) {
                return TransitionOutcome::Denied(SignalAspectCapabilityDenial::MissingOrStaleNode);
            }
            let TransitionOutcome::Success(capability) = self.admit_installed_aspect(node, aspect)
            else {
                return TransitionOutcome::Denied(SignalAspectCapabilityDenial::MissingOrStaleNode);
            };
            capabilities.push(capability);
        }
        if capabilities.is_empty() {
            return TransitionOutcome::Denied(SignalAspectCapabilityDenial::MissingOrStaleNode);
        }
        TransitionOutcome::Success(InstalledSignalAspectSetCapability {
            graph_instance_id: self.runtime_instance_id(),
            aspects: capabilities,
        })
    }
}
