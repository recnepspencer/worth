//! The Signal leg a composite original planned: one definition at one node.
use super::{codec, framing, AuthoredSectionKind, SignalAuthoredImageDenial as Denial};
use crate::data::conditional_execution::SignalConditionalContractDefinition;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::DurableSignalNodeIdentity;

/// The exact conditional definition a planned Signal leg publishes at its
/// recorded node, in the authored image's own versioned grammar. Descriptive
/// bytes: only the Signal owner's publication of the same original admits it.
#[derive(Clone, Debug)]
pub struct SignalPlannedConditionalDefinition {
    node: DurableSignalNodeIdentity,
    definition: SignalConditionalContractDefinition,
    bytes: ExecutionImmutableBytes,
}
impl SignalPlannedConditionalDefinition {
    pub fn capture(
        execution: &ExecutionRequest<'_, '_>,
        node: DurableSignalNodeIdentity,
        definition: &SignalConditionalContractDefinition,
    ) -> Result<Self, Denial> {
        let bytes = framing::encode(
            AuthoredSectionKind::PlannedDefinition,
            node.branch(),
            execution,
            &mut |out| codec::write_planned_definition(out, node, definition),
        )?;
        Ok(Self {
            node,
            definition: definition.clone(),
            bytes,
        })
    }
    pub fn readmit_bytes(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, Denial> {
        framing::admit_work(execution, bytes.len().saturating_mul(2))?;
        let (node, definition) = codec::read_planned_definition(framing::payload(
            AuthoredSectionKind::PlannedDefinition,
            bytes,
        )?)?;
        let mut copy = worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, bytes.len())
            .map_err(Denial::Backing)?;
        copy.extend_from_slice(bytes).map_err(Denial::Backing)?;
        Ok(Self {
            node,
            definition,
            bytes: copy.seal().map_err(Denial::Backing)?.into_immutable_bytes(),
        })
    }
    pub const fn node(&self) -> DurableSignalNodeIdentity {
        self.node
    }
    pub fn definition(&self) -> &SignalConditionalContractDefinition {
        &self.definition
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
}
