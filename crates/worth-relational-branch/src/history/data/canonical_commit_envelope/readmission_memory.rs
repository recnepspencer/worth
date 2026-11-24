//! Concrete original-request admissions move with the decoded owners.
use worth_relational_contracts::internal_port::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundleOwnerPort as _;
#[derive(Default)]
pub struct CanonicalEnvelopeReadmissionMemory {
    pub input: Option<worth_execution::ExecutionImmutableBytes>,
    pub envelope: Option<worth_execution::ExecutionMemoryReservation>,
    pub semantic: Option<worth_execution::ExecutionMemoryReservation>,
    pub index_delta: Option<worth_execution::ExecutionMemoryReservation>,
    pub index_writes: Option<worth_execution::ExecutionMemoryReservation>,
    pub association: Option<worth_execution::ExecutionMemoryReservation>,
    pub merge: Option<crate::merge::data::readmission_memory::MergeReadmissionMemory>,
    pub codec_declarations: Option<CodecDeclarationsReadmissionMemory>,
    pub validation: Option<crate::transactions::data::outcomes::commit_semantic_result::validation::ValidationReadmissionMemory>,
    pub schema: Option<crate::schema::data::SchemaTransitionReadmissionMemory>,
    pub strategy: Option<StrategyReadmissionMemory>,
    pub cascades: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
    pub selection: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
    pub origin: Option<worth_execution::ExecutionMemoryReservation>,
    pub control: Option<std::sync::Arc<super::CanonicalEnvelopeControlAllocation>>,
}

/// Only the canonical codec declarations consumed by this decode.
pub struct CodecDeclarationsReadmissionMemory {
    pub declarations: Vec<super::schema_carrier::DurableSchemaDeclarationMemory>,
    pub _tree_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl CodecDeclarationsReadmissionMemory {
    pub fn attach(
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
pub struct StrategyReadmissionMemory {
    pub input: Option<worth_execution::ExecutionMemoryReservation>,
    pub targets: Option<worth_execution::ExecutionMemoryReservation>,
}
impl StrategyReadmissionMemory {
    pub fn attach(
        &mut self,
        strategy: &mut crate::commit_strategies::data::StrategyCommitArtifactBundle,
    ) -> Result<(), &'static str> {
        strategy.attach_readmission_memory(&mut self.input, &mut self.targets)
    }
}
