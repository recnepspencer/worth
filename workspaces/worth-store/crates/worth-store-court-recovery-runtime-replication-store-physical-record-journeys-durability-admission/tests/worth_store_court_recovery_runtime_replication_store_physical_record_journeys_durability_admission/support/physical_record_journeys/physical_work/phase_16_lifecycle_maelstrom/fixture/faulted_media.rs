use super::{
    CertificationMediaFaultAuthority, FilesystemMediaAdmission, MaelstromPauseGates,
    MediaFaultRule, Path, PhysicalWorkProfileDeclaration, ServingPhysicalRuntime,
};

pub(crate) fn open_with_maelstrom_faults(
    root: &Path,
    profile: PhysicalWorkProfileDeclaration,
    identified_reads_after_open: u64,
    identified_writes_after_open: u64,
) -> (ServingPhysicalRuntime, MaelstromPauseGates) {
    let (admission, gates) =
        faulted_admission(identified_reads_after_open, identified_writes_after_open);
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("Phase 16 faulted media should admit"),
    };
    let (format, _, access) = worth_store_testkit_store::physical_record_journeys::configuration();
    let serving = worth_store_testkit_store::physical_record_journeys::success(open_record_store!(
        media,
        |durability| {
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
                .with_physical_work_profile(profile)
        },
    ));
    (serving, gates)
}

pub(crate) fn faulted_admission(
    identified_reads_after_open: u64,
    identified_writes_after_open: u64,
) -> (FilesystemMediaAdmission, MaelstromPauseGates) {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let gates = MaelstromPauseGates {
        first_read: authority.pause_gate(),
        second_read: authority.pause_gate(),
        close_read: authority.pause_gate(),
        close_read_activation: authority.one_shot_activation(),
    };
    let mut rules = read_fault_rules(&authority, identified_reads_after_open, &gates).to_vec();
    rules.extend(write_fault_rules(&authority, identified_writes_after_open));
    let schedule = authority.schedule(rules).unwrap();
    (admission.with_fault_schedule(schedule), gates)
}

pub(crate) fn read_fault_rules(
    authority: &CertificationMediaFaultAuthority,
    after_open: u64,
    gates: &MaelstromPauseGates,
) -> [MediaFaultRule; 3] {
    [
        authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedRead,
                after_open + 1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(
                    gates.first_read.clone(),
                ),
            )
            .for_identified_operation_ordinal(),
        authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedRead,
                after_open + 2,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(
                    gates.second_read.clone(),
                ),
            )
            .for_identified_operation_ordinal(),
        authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedRead,
                1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(
                    gates.close_read.clone(),
                ),
            )
            .for_next_identified_operation_after_activation(gates.close_read_activation.clone()),
    ]
}

pub(crate) fn write_fault_rules(
    authority: &CertificationMediaFaultAuthority,
    after_open: u64,
) -> [MediaFaultRule; 1] {
    [authority
        .rule(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite,
            after_open + 1,
            worth_store_physical_backend::MediaFaultDirective::FailBefore {
                kind: std::io::ErrorKind::Other,
                raw_os_error: None,
            },
        )
        .for_identified_operation_ordinal()]
}
