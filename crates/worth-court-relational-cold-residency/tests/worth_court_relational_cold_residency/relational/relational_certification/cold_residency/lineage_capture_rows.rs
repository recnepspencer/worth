//! Serialized event coverage is observed through the owner's validated recovery plan.
use super::lineage_sources::reopened;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{durability::*, identity::LineageId};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};

#[test]
fn fork_capture_preserves_every_original_published_event_in_its_admitted_directory() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let fixture = reopened(request);
    let fork = fixture.runtime.fork().unwrap();
    // Populate the shared event pages through actual public authored-history reads.
    for page in &fixture.directory.lineage {
        for id in page.first..=page.last {
            fixture
                .runtime
                .discover_authored_lineage_history(
                    request,
                    &fixture.runtime.main_branch_identity(),
                    LineageId(id),
                )
                .unwrap();
        }
    }
    let current_members = fixture.source.members.keys().copied().collect::<Vec<_>>();
    let native = fork
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current_members,
                source: Some(fixture.source.clone()),
            },
        )
        .unwrap();
    let source = fixture.source.with_published_members(native.members());
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        vec![],
        vec![],
        source,
    );
    let medium = TestDurableMedium::new();
    let receiver = durable_runtime(&medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let observed = plan
        .checkpoint
        .as_ref()
        .unwrap()
        .lineage
        .digest_basis()
        .canonical_published_event_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected = fixture
        .original_basis
        .canonical_published_event_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "the validated directory preserves every original selected event row"
    );
}
