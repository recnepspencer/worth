use super::{
    configuration,
    initialization::InitializedWriter,
    markers,
    mutation_material::dirty_material,
    mutation_submission::{start_capacity_transition, start_dirty_checkpoint},
    Invocation,
};
use worth_store::physical_runtime::production::PhysicalMutationCheckpoint;
use worth_store::physical_runtime::PhysicalMutationOutcome;

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
    SelectedSegmentRewrite,
    MultiPageSegmentRewrite,
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
        "before-wal-append" => Ok(PhysicalMutationCheckpoint::BeforeWalAppend),
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
            "selected-segment-rewrite" => Ok(Self::SelectedSegmentRewrite),
            "multi-page-segment-rewrite" => Ok(Self::MultiPageSegmentRewrite),
            _ => Err(format!("unknown C8 mutation crash workload `{encoded}`")),
        }
    }

    fn payload_length(&self, writer: &InitializedWriter) -> usize {
        match self {
            Self::ExtentWriteback => configuration::dirty_checkpoint_payload_length(writer.format),
            Self::InlineRecord | Self::SelectedSegmentRewrite | Self::MultiPageSegmentRewrite => {
                INLINE_RECORD_PAYLOAD_BYTES
            }
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
    let material = dirty_material(seed);
    if matches!(
        crash.workload,
        MutationCrashWorkload::SelectedSegmentRewrite
    ) {
        finish_source_append(writer, material, INLINE_RECORD_PAYLOAD_BYTES)?;
    }
    if matches!(
        crash.workload,
        MutationCrashWorkload::MultiPageSegmentRewrite
    ) {
        finish_two_page_source(writer, material)?;
    }
    let gate = writer.serving.pause_physical_mutation_at(crash.checkpoint);
    let payload_length = crash.workload.payload_length(writer);
    let mutation = if matches!(
        crash.workload,
        MutationCrashWorkload::SelectedSegmentRewrite
            | MutationCrashWorkload::MultiPageSegmentRewrite
    ) {
        let mut rewrite_material = material;
        rewrite_material[0] ^= 0xA5;
        super::mutation_submission::start_selected_segment_rewrite(
            &writer.serving,
            writer.placement,
            rewrite_material,
        )?
    } else if crash.workload.changes_capacity() {
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

fn finish_two_page_source(writer: &InitializedWriter, material: [u8; 32]) -> Result<(), String> {
    // Two records of this size exceed the 90% inline page fill, so one
    // mutation publishes the second record on the next page.
    const SPANNED_PAGE_PAYLOAD_BYTES: usize = 7_500;
    let first =
        super::mutation_material::dirty_checkpoint_payload(material, SPANNED_PAGE_PAYLOAD_BYTES);
    let mut second_material = material;
    second_material[1] ^= 0x5A;
    let second = super::mutation_material::dirty_checkpoint_payload(
        second_material,
        SPANNED_PAGE_PAYLOAD_BYTES,
    );
    let tail = super::mutation_submission::start_dirty_checkpoint_batch(
        &writer.serving,
        writer.placement,
        material,
        [first, second],
    )?;
    match tail.wait() {
        PhysicalMutationOutcome::Completed(_) => Ok(()),
        PhysicalMutationOutcome::ProvenNoEffect(_) => {
            Err("C8 rewrite source append had no effect".to_owned())
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            Err("C8 rewrite source append did not finish".to_owned())
        }
    }
}

fn finish_source_append(
    writer: &InitializedWriter,
    material: [u8; 32],
    payload_length: usize,
) -> Result<(), String> {
    let tail = start_dirty_checkpoint(&writer.serving, writer.placement, material, payload_length)?;
    match tail.wait() {
        PhysicalMutationOutcome::Completed(_) => Ok(()),
        PhysicalMutationOutcome::ProvenNoEffect(_) => {
            Err("C8 rewrite source append had no effect".to_owned())
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            Err("C8 rewrite source append did not finish".to_owned())
        }
    }
}
