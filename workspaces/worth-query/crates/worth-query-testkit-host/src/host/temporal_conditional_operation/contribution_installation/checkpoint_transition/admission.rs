//! Real native performance and linear repair custody under caller byte policy.
pub use super::*;
pub use application_installation::WorthQueryCheckpointCaptureDenial as CaptureDenial;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::runtime::LeaseDenial;

pub fn author_record(
    writer: &mut application_installation::WorthQueryCheckpointMigrationWriter<
        '_,
        TemporalHostSchema,
    >,
    key: &str,
    value: u64,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    writer.bind_entity(
        primary_graph::WorthQueryApplicationEntitySeed::new(
            UnrelatedRecord::reference(),
            primary_graph::WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(UnrelatedValueField::reference(), value),
    )
}

pub fn assert_target_record(
    checkpoint: application_installation::WorthQueryApplicationCheckpoint,
    value: u64,
) {
    let reopened = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint::checkpoint_limits(),
        checkpoint,
    )
    .expect("ordinary target admission authenticates the repaired native successor");
    let selected = reopened
        .with_host_advancement(|host_phase| {
            reopened
                .on_branch(reopened.current_world())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let record = reopened
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    UnrelatedValueField::reference(),
                    value,
                    &request_scope(),
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the actual typed migrated value survives native recovery")
        })
        .expect("installed host call opens");
    assert_eq!(record.examined_candidate_count(), 1);
}
