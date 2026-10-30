//! Native changed-codec retirement retains an independent historical publication.
pub use super::{
    companion as old,
    successor::{self, companion as new},
    *,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_entry::*, application_installation as installation,
};
pub type Original = old::host::Host<old::program::SourceProgram>;
pub type Target = new::host::Host<new::program::TargetProgram>;
pub type OriginalSettlement =
    worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
        query::ValueQuery,
    >;
pub fn original(value: model::NestedValue) -> (Original, OriginalSettlement) {
    let host = old::host::install::<old::program::SourceProgram>(
        value,
        installation::WorthQueryApplicationProgramRoster::new()
            .support(old::host::program::<old::program::TargetProgram>()),
    )
    .unwrap();
    let _changed = old::reads::produce_changed(&host);
    let companion = old::host::produce(&host);
    (host, companion)
}
pub fn transition(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
    policy: installation::WorthQueryCheckpointCapturePolicy<'_, '_>,
) -> Result<Target, installation::WorthQueryInMemoryApplicationDenial> {
    transition_counted(checkpoint, policy, &mut 0)
}
pub fn transition_counted(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
    policy: installation::WorthQueryCheckpointCapturePolicy<'_, '_>,
    callbacks: &mut usize,
) -> Result<Target, installation::WorthQueryInMemoryApplicationDenial> {
    installation::in_memory_rostered_program_from_checkpoint_with_transition(
        new::host::program::<new::program::TargetProgram>(),
        installation::WorthQueryApplicationProgramRoster::new(),
        successor::schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        successor::host::limits(),
        checkpoint,
        installation::WorthQueryCheckpointProgramPredecessor::new(
            &old::host::program::<old::program::SourceProgram>()
                .revision()
                .to_string(),
        )
        .unwrap(),
        installation::WorthQueryCheckpointTransitionResources::bounded(65_536, 32, 65_536).unwrap(),
        policy,
        |_, _| {
            *callbacks += 1;
            Ok(())
        },
    )
}
/// The same transition inside a phase `funding` opens, with its capture
/// capped by the caller-named `capture` policy.
pub fn transition_funded(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
    funding: &worth_execution::ExecutionResourceLease<'_>,
    capture: installation::WorthQueryCheckpointCapturePolicy<'_, '_>,
    callbacks: &mut usize,
) -> Result<Target, installation::WorthQueryInMemoryApplicationDenial> {
    super::super::capture_child::funded_by(funding, |phase| {
        installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement(
            phase,
            new::host::program::<new::program::TargetProgram>(),
            installation::WorthQueryApplicationProgramRoster::new(),
            successor::schema::CanonicalSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            successor::host::limits(),
            checkpoint,
            installation::WorthQueryCheckpointProgramPredecessor::new(
                &old::host::program::<old::program::SourceProgram>()
                    .revision()
                    .to_string(),
            )
            .unwrap(),
            installation::WorthQueryCheckpointTransitionResources::bounded(65_536, 32, 65_536)
                .unwrap(),
            capture,
            |_, _| {
                *callbacks += 1;
                Ok(())
            },
        )
    })
}
pub fn companion_ids(
    settled: &OriginalSettlement,
) -> [worth_relational::facade::identity::EntityId; 2] {
    let roles = settled
        .outputs_of::<old::output::CompanionOutputs>()
        .unwrap();
    [
        roles
            .entity::<old::output::CreatedOutput>()
            .unwrap()
            .entity_id(),
        roles
            .entity::<old::output::CreatedLeft>()
            .unwrap()
            .entity_id(),
    ]
}
pub fn assert_target(host: &Target, ids: [worth_relational::facade::identity::EntityId; 2]) {
    assert_eq!(
        new::reads::read_companion(host, "companion-left"),
        "companion-left"
    );
    assert_eq!(
        new::reads::read_companion(host, "companion-right"),
        "companion-right"
    );
    // A historical locator is useful to the original installed producer but is
    // never a target Ready claim: this must perform the real Preserve operation.
    let settled = new::host::produce(host);
    let roles = settled
        .outputs_of::<new::output::PreservedOutputs>()
        .expect("the historical companion is genuinely preserved, not recreated");
    assert_eq!(
        [
            roles
                .entity::<new::output::PreservedOutput>()
                .unwrap()
                .entity_id(),
            roles
                .entity::<new::output::PreservedLeft>()
                .unwrap()
                .entity_id()
        ],
        ids
    );
    let _changed = new::reads::produce_changed(host);
    assert_eq!(
        successor::output::literals(&new::reads::read_derived(host)).0,
        "V2 ASSEMBLY Β"
    );
}

#[path = "companion_transition/capture.rs"]
pub mod capture;
#[path = "companion_transition/capture_lineage.rs"]
pub mod capture_lineage;

pub fn assert_cached_companion(
    original: &Original,
    performed: &worth_query_host::facade::product::WorthQueryApplicationCommitReceipt,
    ids: [worth_relational::facade::identity::EntityId; 2],
) {
    let scope = request_scope();
    let principal = authenticate_operator(original.installed_schema(), &scope);
    let request = original
        .runtime()
        .request(&principal, &scope)
        .on_branch(original.runtime().current_world());
    let mut demand = request
        .demand(old::producer::Demand)
        .start_in_program::<_, old::program::Root>(original)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(stable) =
        demand.settle(&request).unwrap()
    else {
        panic!("unchanged completed output must settle")
    };
    assert_eq!(stable.producer_contacts_in_this_demand(), 0);
    if let Some(receipt) = stable.application_commit_receipt() {
        assert!(receipt.is_same_authoritative_commit(performed));
    } else {
        assert!(matches!(
            stable.posture(),
            WorthQueryOutputSettlementPosture::StableReused
                | WorthQueryOutputSettlementPosture::RecoveredStableReused
        ));
    }
    let roles = stable
        .outputs_of::<old::output::PreservedOutputs>()
        .unwrap();
    assert_eq!(
        [
            roles
                .entity::<old::output::PreservedOutput>()
                .unwrap()
                .entity_id(),
            roles
                .entity::<old::output::PreservedLeft>()
                .unwrap()
                .entity_id()
        ],
        ids
    );
}
