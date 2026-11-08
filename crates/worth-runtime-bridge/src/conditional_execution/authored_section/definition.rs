//! Bridge meaning refers to Signal's full authored condition instead of duplicating it.
use super::{
    fields::{Reader, Writer},
    BridgeAuthoredSectionDenial as D,
};
use crate::conditional_execution::{
    BridgeConditionalCondition, BridgeConditionalContract, BridgeConditionalContractParts,
    BridgeConditionalLocation, BridgeInstalledConditionalLowering,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::DurableSignalConditionalDefinitionOccurrence;
use worth_signal::facade::durability::SignalReadmittedConditionalContract;
use worth_signal::facade::{SignalConditionalCondition, SignalConditionalContractDefinition};

pub(in crate::conditional_execution) struct BridgeReadmittedDefinitionSource {
    pub(in crate::conditional_execution) contract: Arc<BridgeConditionalContract>,
    pub(in crate::conditional_execution) location: Arc<BridgeConditionalLocation>,
    _memory: Option<ExecutionMemoryReservation>,
    _projection_backing: Arc<super::readmission::backing::Backing>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Kind {
    Always,
    AspectFiltered,
    DeltaThreshold,
    OnDemand,
    RuntimePredicate,
    TemporalWake,
}
impl Kind {
    fn bridge(condition: &BridgeConditionalCondition) -> Self {
        match condition {
            BridgeConditionalCondition::Always => Self::Always,
            BridgeConditionalCondition::AspectFiltered => Self::AspectFiltered,
            BridgeConditionalCondition::DeltaThreshold(_) => Self::DeltaThreshold,
            BridgeConditionalCondition::OnDemand => Self::OnDemand,
            BridgeConditionalCondition::RuntimePredicate => Self::RuntimePredicate,
            BridgeConditionalCondition::TemporalWake => Self::TemporalWake,
        }
    }
    fn signal(condition: &SignalConditionalCondition) -> Self {
        match condition {
            SignalConditionalCondition::Always => Self::Always,
            SignalConditionalCondition::AspectFilter(_) => Self::AspectFiltered,
            SignalConditionalCondition::DeltaThreshold(_) => Self::DeltaThreshold,
            SignalConditionalCondition::OnDemand => Self::OnDemand,
            SignalConditionalCondition::RuntimePredicate => Self::RuntimePredicate,
            SignalConditionalCondition::TemporalWake => Self::TemporalWake,
        }
    }
    fn code(self) -> u8 {
        match self {
            Self::Always => 0,
            Self::AspectFiltered => 1,
            Self::DeltaThreshold => 2,
            Self::OnDemand => 3,
            Self::RuntimePredicate => 4,
            Self::TemporalWake => 5,
        }
    }
    fn read(value: u8) -> Result<Self, D> {
        match value {
            0 => Ok(Self::Always),
            1 => Ok(Self::AspectFiltered),
            2 => Ok(Self::DeltaThreshold),
            3 => Ok(Self::OnDemand),
            4 => Ok(Self::RuntimePredicate),
            5 => Ok(Self::TemporalWake),
            _ => Err(D::Malformed),
        }
    }
}
pub(super) fn write(
    out: &mut Writer<'_>,
    occurrence: DurableSignalConditionalDefinitionOccurrence,
    lowering: &BridgeInstalledConditionalLowering,
    claims: &[u8],
) -> Result<(), D> {
    let node = lowering.signal_node();
    if occurrence.node().index() != node.index()
        || occurrence.node().generation() != node.generation()
        || lowering.location().node_identity() != lowering.contract().identity()
    {
        return Err(D::SignalDefinitionMismatch);
    }
    out.occurrence(occurrence)?;
    out.text(lowering.contract().identity())?;
    out.byte(u8::from(lowering.location().stage_identity().is_some()))?;
    if let Some(stage) = lowering.location().stage_identity() {
        out.text(stage)?;
    }
    out.number(lowering.contract().dependency_count() as u64)?;
    let ordinals = lowering.contract().condition_dependency_ordinals();
    out.number((ordinals.len() as u64).checked_mul(8).ok_or(D::SizeLimit)?)?;
    for ordinal in ordinals {
        out.number(*ordinal as u64)?;
    }
    out.byte(Kind::bridge(lowering.contract().condition()).code())?;
    out.blob(claims)
}
pub(super) struct DefinitionView<'a> {
    pub(super) occurrence: DurableSignalConditionalDefinitionOccurrence,
    identity: &'a str,
    stage: Option<&'a str>,
    dependencies: usize,
    ordinals: &'a [u8],
    kind: Kind,
    pub(super) claims: &'a [u8],
}
impl<'a> DefinitionView<'a> {
    pub(super) fn read(input: &mut Reader<'a>) -> Result<Self, D> {
        let occurrence = input.occurrence()?;
        let identity = input.text()?;
        let stage = match input.byte()? {
            0 => None,
            1 => Some(input.text()?),
            _ => return Err(D::Malformed),
        };
        let dependencies = input.number()?.try_into().map_err(|_| D::SizeLimit)?;
        let ordinals = input.blob()?;
        if ordinals.len() % 8 != 0
            || identity.trim().is_empty()
            || stage.is_some_and(|s| s.trim().is_empty())
        {
            return Err(D::Malformed);
        }
        let mut previous = None;
        for bytes in ordinals.chunks_exact(8) {
            let ordinal = usize::try_from(u64::from_le_bytes(bytes.try_into().unwrap()))
                .map_err(|_| D::SizeLimit)?;
            if ordinal >= dependencies || previous.is_some_and(|old| old >= ordinal) {
                return Err(D::Malformed);
            }
            previous = Some(ordinal);
        }
        let kind = Kind::read(input.byte()?)?;
        let claims = input.blob()?;
        Ok(Self {
            occurrence,
            identity,
            stage,
            dependencies,
            ordinals,
            kind,
            claims,
        })
    }
    pub(super) fn readmit_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &SignalReadmittedConditionalContract,
        backing: Arc<super::readmission::backing::Backing>,
    ) -> Result<Arc<BridgeReadmittedDefinitionSource>, D> {
        let signal = installed.installed();
        let definition = signal.authored_definition();
        if installed.occurrence() != self.occurrence
            || self.kind != Kind::signal(&definition.condition)
        {
            return Err(D::SignalDefinitionMismatch);
        }
        let bytes = self.source_storage_bytes(definition)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(D::Request)?;
        let condition = match &definition.condition {
            SignalConditionalCondition::Always => BridgeConditionalCondition::Always,
            SignalConditionalCondition::AspectFilter(_) => {
                BridgeConditionalCondition::AspectFiltered
            }
            SignalConditionalCondition::DeltaThreshold(value) => {
                BridgeConditionalCondition::DeltaThreshold(value.clone())
            }
            SignalConditionalCondition::OnDemand => BridgeConditionalCondition::OnDemand,
            SignalConditionalCondition::RuntimePredicate => {
                BridgeConditionalCondition::RuntimePredicate
            }
            SignalConditionalCondition::TemporalWake => BridgeConditionalCondition::TemporalWake,
        };
        let contract = Arc::new(BridgeConditionalContract::new(
            BridgeConditionalContractParts {
                identity: Arc::from(self.identity),
                dependency_count: self.dependencies,
                condition_dependency_ordinals: self
                    .ordinals
                    .chunks_exact(8)
                    .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()) as usize)
                    .collect(),
                condition,
                dependency_comparator: definition.dependency_comparator,
                output_comparator: definition.output_comparator,
                artifact_reuse: definition.artifact_reuse,
            },
        ));
        let location = Arc::new(match self.stage {
            Some(stage) => BridgeConditionalLocation::workflow_stage(stage, self.identity),
            None => BridgeConditionalLocation::operation(self.identity),
        });
        Ok(Arc::new(BridgeReadmittedDefinitionSource {
            contract,
            location,
            _memory: memory,
            _projection_backing: backing,
        }))
    }
    fn source_storage_bytes(
        &self,
        definition: &SignalConditionalContractDefinition,
    ) -> Result<u64, D> {
        use crate::conditional_execution::retention::{
            arc_charge, arc_slice_charge, array_charge, sum,
        };
        let threshold = match &definition.condition {
            SignalConditionalCondition::DeltaThreshold(value) => value
                .threshold()
                .owned_allocation_capacity_bytes()
                .checked_add(value.unit_identity().len())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(D::SizeLimit)?,
            SignalConditionalCondition::Always
            | SignalConditionalCondition::AspectFilter(_)
            | SignalConditionalCondition::OnDemand
            | SignalConditionalCondition::RuntimePredicate
            | SignalConditionalCondition::TemporalWake => 0,
        };
        let quote = || {
            sum(&[
                arc_charge::<BridgeReadmittedDefinitionSource>()?,
                arc_charge::<BridgeConditionalContract>()?,
                arc_charge::<BridgeConditionalLocation>()?,
                arc_slice_charge::<u8>(self.identity.len())?,
                arc_slice_charge::<u8>(self.identity.len())?,
                array_charge::<usize>(self.ordinals.len() / 8)?,
                self.stage
                    .map(|s| arc_slice_charge::<u8>(s.len()))
                    .transpose()?
                    .unwrap_or(0),
                threshold,
            ])
        };
        quote().map_err(|_| D::SizeLimit)
    }
}
