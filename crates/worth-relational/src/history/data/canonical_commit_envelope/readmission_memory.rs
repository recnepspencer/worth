//! Concrete original-request admissions move with the decoded owners.
#[derive(Default)]
pub(crate) struct CanonicalEnvelopeReadmissionMemory {
    pub(crate) input: Option<worth_execution::ExecutionImmutableBytes>,
    pub(crate) envelope: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) semantic: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) index_delta: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) index_writes: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) association: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) merge: Option<crate::merge::data::MergeReadmissionMemory>,
    pub(crate) codec_declarations: Option<CodecDeclarationsReadmissionMemory>,
    pub(crate) validation: Option<crate::transactions::data::ValidationReadmissionMemory>,
    pub(crate) schema: Option<crate::schema::data::SchemaTransitionReadmissionMemory>,
    pub(crate) strategy: Option<StrategyReadmissionMemory>,
    pub(crate) origin: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) control: Option<std::sync::Arc<super::CanonicalEnvelopeControlAllocation>>,
}

/// Only the canonical codec declarations consumed by this decode.
pub(crate) struct CodecDeclarationsReadmissionMemory {
    pub(crate) declarations: Vec<crate::durability::data::DurableSchemaDeclarationMemory>,
    pub(crate) _tree_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl CodecDeclarationsReadmissionMemory {
    pub(crate) fn attach(
        &mut self,
        events: &[std::sync::Arc<super::CanonicalRequiredOutputCodecRetirement>],
    ) -> Result<(), &'static str> {
        let mut declarations = self.declarations.iter_mut();
        for event in events {
            for kind in event.kinds.iter() {
                declarations
                    .next()
                    .ok_or("missing source codec declaration admission")?
                    .attach(&kind.source)?;
                declarations
                    .next()
                    .ok_or("missing target codec declaration admission")?
                    .attach(&kind.target)?;
            }
        }
        if declarations.next().is_some() {
            return Err("unmatched codec declaration admission");
        }
        Ok(())
    }
}

/// The two escaping strategy allocations retain separate original leaf bills.
/// This fixed handoff value allocates no roster and carries no authority.
pub(crate) struct StrategyReadmissionMemory {
    pub(crate) input: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) targets: Option<worth_execution::ExecutionMemoryReservation>,
}
impl StrategyReadmissionMemory {
    pub(crate) fn attach(
        &mut self,
        strategy: &mut crate::commit_strategies::data::StrategyCommitArtifactBundle,
    ) -> Result<(), &'static str> {
        strategy.attach_readmission_memory(&mut self.input, &mut self.targets)
    }
}
