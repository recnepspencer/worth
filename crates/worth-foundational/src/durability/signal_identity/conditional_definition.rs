//! Durable association of a conditional definition occurrence with its node.
use super::super::{identity_seal, DurableIdentity};
use super::{DurableSignalConditionalOccurrence, DurableSignalNodeIdentity};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalConditionalDefinitionOccurrence {
    node: DurableSignalNodeIdentity,
    occurrence: DurableSignalConditionalOccurrence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalConditionalDefinitionOwnerMismatch;
impl DurableSignalConditionalDefinitionOccurrence {
    pub fn from_recorded(
        node: DurableSignalNodeIdentity,
        occurrence: DurableSignalConditionalOccurrence,
    ) -> Result<Self, SignalConditionalDefinitionOwnerMismatch> {
        if node.branch().owner() != occurrence.owner() {
            return Err(SignalConditionalDefinitionOwnerMismatch);
        }
        Ok(Self { node, occurrence })
    }
    pub const fn node(self) -> DurableSignalNodeIdentity {
        self.node
    }
    pub const fn occurrence(self) -> DurableSignalConditionalOccurrence {
        self.occurrence
    }
}
impl identity_seal::Sealed for DurableSignalConditionalDefinitionOccurrence {}
impl DurableIdentity for DurableSignalConditionalDefinitionOccurrence {}
