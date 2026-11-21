//! The Bridge successor one composite original planned, framed for World's
//! custody journal before Native's effect. It is DATA: it grants no publication.
use super::{
    fields::Reader,
    framing::{self, Section},
    provider_source, BridgeAuthoredSectionDenial as D,
};
use crate::conditional_execution::{
    BridgeAuthoredProviderDescriptions, BridgeConditionalContract, BridgeConditionalLocation,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_signal::facade::durability::SignalPlannedConditionalDefinition;

pub const BRIDGE_PLANNED_DEFINITION_LEG_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.bridge.planned-definition-leg");
pub const BRIDGE_PLANNED_DEFINITION_LEG_VERSION: BoundaryProtocolVersion =
    BoundaryProtocolVersion::new(1);
/// Bounded by the World custody row that carries it.
const MAXIMUM_BYTES: usize = 48 << 10;

/// Signal's planned definition at its recorded node, with the Bridge contract
/// row and authored provider sources that prepare the same successor again.
#[derive(Clone, Debug)]
pub struct BridgePlannedDefinitionLeg {
    bytes: ExecutionImmutableBytes,
}
impl PartialEq for BridgePlannedDefinitionLeg {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl Eq for BridgePlannedDefinitionLeg {}

/// One planned leg's decoded meaning, admitted by Bridge's own framing.
pub(super) struct PlannedLegParts {
    pub(super) signal: SignalPlannedConditionalDefinition,
    pub(super) contract: BridgeConditionalContract,
    pub(super) location: BridgeConditionalLocation,
    pub(super) providers: provider_source::DecodedProviders,
}

impl BridgePlannedDefinitionLeg {
    /// Admit recorded bytes: framing, version, checksum and every field.
    pub fn readmit_bytes(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, D> {
        if bytes.len() > MAXIMUM_BYTES {
            return Err(D::SizeLimit);
        }
        decode(execution, bytes)?;
        Ok(Self {
            bytes: framing::copy(execution, bytes)?,
        })
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
    pub(super) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        signal: &SignalPlannedConditionalDefinition,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        providers: &BridgeAuthoredProviderDescriptions,
    ) -> Result<Self, D> {
        let bytes = framing::encode_section(Section::PlannedDefinitionLeg, execution, &mut |out| {
            out.blob(signal.as_bytes())?;
            out.text(contract.identity())?;
            out.byte(u8::from(location.stage_identity().is_some()))?;
            if let Some(stage) = location.stage_identity() {
                out.text(stage)?;
            }
            out.number(contract.dependency_count() as u64)?;
            let ordinals = contract.condition_dependency_ordinals();
            out.number(ordinals.len() as u64)?;
            for ordinal in ordinals {
                out.number(*ordinal as u64)?;
            }
            out.record(&mut |out| provider_source::write(out, providers))
        })?;
        if bytes.bytes().len() > MAXIMUM_BYTES {
            return Err(D::SizeLimit);
        }
        Ok(Self { bytes })
    }
    pub(super) fn decode(&self, execution: &ExecutionRequest<'_, '_>) -> Result<PlannedLegParts, D> {
        decode(execution, self.as_bytes())
    }
}

fn decode(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<PlannedLegParts, D> {
    framing::work(execution, bytes.len().saturating_mul(2))?;
    let mut input = framing::payload_section(Section::PlannedDefinitionLeg, bytes)?;
    let signal = SignalPlannedConditionalDefinition::readmit_bytes(execution, input.blob()?)
        .map_err(D::SignalImage)?;
    let identity = input.text()?;
    let stage = match input.byte()? {
        0 => None,
        1 => Some(input.text()?),
        _ => return Err(D::Malformed),
    };
    let dependencies = usize::try_from(input.number()?).map_err(|_| D::SizeLimit)?;
    let ordinals = read_ordinals(&mut input, dependencies)?;
    if identity.trim().is_empty() || stage.is_some_and(|stage| stage.trim().is_empty()) {
        return Err(D::Malformed);
    }
    let providers = provider_source::decode(input.blob()?, execution)?;
    input.finish()?;
    Ok(PlannedLegParts {
        contract: super::definition::contract_from_signal(
            identity,
            dependencies,
            ordinals,
            signal.definition(),
        ),
        location: super::definition::location(identity, stage),
        signal,
        providers,
    })
}

/// Strictly increasing condition ordinals, each naming a declared dependency.
fn read_ordinals(input: &mut Reader<'_>, dependencies: usize) -> Result<Vec<usize>, D> {
    let count = usize::try_from(input.number()?).map_err(|_| D::SizeLimit)?;
    if count > dependencies {
        return Err(D::Malformed);
    }
    let mut ordinals = Vec::with_capacity(count);
    for _ in 0..count {
        let ordinal = usize::try_from(input.number()?).map_err(|_| D::SizeLimit)?;
        if ordinal >= dependencies || ordinals.last().is_some_and(|last| *last >= ordinal) {
            return Err(D::Malformed);
        }
        ordinals.push(ordinal);
    }
    Ok(ordinals)
}

