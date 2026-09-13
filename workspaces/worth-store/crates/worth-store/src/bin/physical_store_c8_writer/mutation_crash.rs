use super::{
    configuration,
    initialization::InitializedWriter,
    markers,
    mutation_material::dirty_material,
    mutation_submission::{start_capacity_transition, start_dirty_checkpoint},
    Invocation,
};
use worth_store::physical_runtime::production::PhysicalMutationCheckpoint;

const INLINE_RECORD_PAYLOAD_BYTES: usize = 128;

#[derive(Clone, Copy)]
pub(super) struct MutationCrashInvocation {
    checkpoint: PhysicalMutationCheckpoint,
    workload: MutationCrashWorkload,
}

#[derive(Clone, Copy)]
enum MutationCrashWorkload {
    ExtentWriteback,
    InlineRecord,
    CapacityTransition,
}

pub(super) fn admit(
    stage: Option<String>,
    workload: Option<String>,
) -> Result<Option<MutationCrashInvocation>, String> {
    match (stage, workload) {
        (None, None) => Ok(None),
        (Some(stage), Some(workload)) => Ok(Some(MutationCrashInvocation {
            checkpoint: parse_stage(&stage)?,
            workload: MutationCrashWorkload::parse(&workload)?,
        })),
        (Some(_), None) => Err("--mutation-crash-workload is required with the crash stage".into()),
        (None, Some(_)) => Err("--mutation-crash-stage is required with the workload".into()),
    }
}

fn parse_stage(encoded: &str) -> Result<PhysicalMutationCheckpoint, String> {
    match encoded {
        "before-effect-cutover" => Ok(PhysicalMutationCheckpoint::BeforeEffectCutover),
        "after-group-seal" => Ok(PhysicalMutationCheckpoint::AfterGroupSeal),
        "after-wal-durability" => Ok(PhysicalMutationCheckpoint::AfterWalDurability),
        "after-writeback-admission-before-effect" => {
            Ok(PhysicalMutationCheckpoint::AfterWritebackAdmissionBeforeEffect)
        }
        "during-data-settlement" => Ok(PhysicalMutationCheckpoint::DuringDataSettlement),
        "after-data-settlement" => Ok(PhysicalMutationCheckpoint::AfterDataSettlement),
        "during-root-publication" => Ok(PhysicalMutationCheckpoint::DuringRootPublication),
        "before-terminal-finalization" => {
            Ok(PhysicalMutationCheckpoint::BeforeTerminalFinalization)
        }
        _ => Err(format!("unknown C8 mutation crash stage `{encoded}`")),
    }
}

impl MutationCrashWorkload {
    fn parse(encoded: &str) -> Result<Self, String> {
        match encoded {
            "extent-writeback" => Ok(Self::ExtentWriteback),
            "inline-record" => Ok(Self::InlineRecord),
            "capacity-transition" => Ok(Self::CapacityTransition),
            _ => Err(format!("unknown C8 mutation crash workload `{encoded}`")),
        }
    }

    fn payload_length(&self, writer: &InitializedWriter) -> usize {
        match self {
            Self::ExtentWriteback => configuration::dirty_checkpoint_payload_length(writer.format),
            Self::InlineRecord => INLINE_RECORD_PAYLOAD_BYTES,
            Self::CapacityTransition => INLINE_RECORD_PAYLOAD_BYTES,
        }
    }

    const fn changes_capacity(self) -> bool {
        matches!(self, Self::CapacityTransition)
    }
}

pub(super) fn hold_for_process_death(
    writer: &InitializedWriter,
    crash: MutationCrashInvocation,
    invocation: &Invocation,
) -> Result<(), String> {
    let seed = invocation.stage.perturbation_seed;
    let gate = writer.serving.pause_physical_mutation_at(crash.checkpoint);
    let material = dirty_material(seed);
    let payload_length = crash.workload.payload_length(writer);
    let mutation = if crash.workload.changes_capacity() {
        start_capacity_transition(
            &writer.serving,
            configuration::capacity_transition_placement(writer.format),
            material,
            payload_length,
        )?
    } else {
        start_dirty_checkpoint(&writer.serving, writer.placement, material, payload_length)?
    };
    if !gate.await_arrival() {
        gate.release();
        return Err(format!(
            "ordinary C8 mutation did not reach process crash stage {:?}",
            crash.checkpoint
        ));
    }
    markers::write_ready(
        &invocation.start_marker,
        "write C8 mutation-crash ready marker",
    )?;
    markers::wait_for_parent(&invocation.start_marker);
    markers::write_reached(
        &invocation.reached_marker,
        format!("{:?}", crash.checkpoint).as_bytes(),
        "write C8 mutation-crash reached marker",
    )?;
    let _paused_mutation = mutation;
    let _pause_gate = gate;
    markers::park_forever()
}
