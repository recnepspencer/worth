//! `G7` through the public facade: a long commit history reopens without
//! reading any commit-index page, lookups fault only the pages they touch and
//! evict under one tight historical budget at `1x` and `10x` history, and
//! successive captures carry the unchanged index prefix.
use worth_execution::ExecutionRequest;
use worth_relational::facade::config::{RelationalRuntimeProfile, ResidencyBudget};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCanonicalCommitArtifact, RelationalCheckpointMember,
    RelationalDurableMemberLocator, RelationalDurableRecoverySource,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, recover, source_with_history, update_entity,
};
use worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource;
use worth_testkit_relational::durable_publication::{durable_runtime, schema};

/// Commit-index pages are chunk-member frames with kind tag 4.
fn commit_pages(source: &ImmutableCheckpointMemberSource) -> Vec<RelationalDurableMemberLocator> {
    source
        .members
        .iter()
        .filter(|(_, bytes)| {
            bytes.bytes().starts_with(b"WORTH-REL-CHUNK1\n") && bytes.bytes().get(17) == Some(&4)
        })
        .map(|(locator, _)| *locator)
        .collect()
}

fn page_reads(source: &ImmutableCheckpointMemberSource) -> usize {
    source.commit_page_reads.lock().unwrap().len()
}

fn reopen(
    request: &ExecutionRequest<'_, '_>,
    fixture: &worth_testkit_relational::cold_recovery::ColdRecoverySourceFixture,
    historical: u64,
) -> RelationalRuntime {
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: u64::MAX / 4,
            historical_working_set_bytes: historical,
            max_in_flight_fill_bytes: u64::MAX / 4,
            streaming_window_bytes: 64 * 1024,
        })
        .build_durable(fixture.medium.attachment())
        .unwrap();
    let input = RelationalDurableRecoverySource::new(
        fixture.input.checkpoint().cloned(),
        Vec::new(),
        Vec::new(),
        fixture.source.clone(),
    );
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    runtime
}

/// The oracle commit: the producer's own canonical artifact at `position`.
fn oracle(source: &ImmutableCheckpointMemberSource, position: u64) -> CommitId {
    let bytes = source.commits[&PatchStreamPosition(position)].clone();
    let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes).unwrap();
    assert_eq!(artifact.position(), PatchStreamPosition(position));
    artifact.commit_id()
}

/// Position, commit and version lookups agree with each other and the oracle.
fn exact(runtime: &RelationalRuntime, source: &ImmutableCheckpointMemberSource, position: u64) {
    let receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(position))
        .unwrap();
    assert_eq!(receipt.commit_id, oracle(source, position));
    assert_eq!(
        runtime
            .history()
            .immutable_commit_receipt(receipt.commit_id),
        Some(receipt.clone())
    );
    assert_eq!(
        runtime
            .history()
            .immutable_commit_receipt_for_version(receipt.version_id),
        Some(receipt)
    );
}

#[test]
fn long_history_reopens_without_commit_pages_and_faults_only_touched_pages() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    // About two resident commit pages: the same pool at either history length.
    let historical = 128 * 1024;
    let mut reopened_resident = Vec::new();
    for updates in [260_u64, 2600] {
        let fixture = source_with_history(request, updates as usize, false);
        let source = fixture.source.clone();
        let commits = updates + 1;
        let pages = commit_pages(&source);
        assert_eq!(pages.len() as u64, commits.div_ceil(128));
        let runtime = reopen(request, &fixture, historical);
        assert_eq!(page_reads(&source), 0, "reopen reads no commit-index page");
        let pressure = runtime.residency_pressure().unwrap();
        reopened_resident.push(pressure.historical_resident_bytes);
        assert_eq!(
            runtime.history().latest_patch_stream_position(),
            Some(PatchStreamPosition(commits))
        );
        // One middle lookup faults one page; its commit and version resolve
        // from the same resident page.
        let middle = commits / 2;
        exact(&runtime, &source, middle);
        assert_eq!(
            page_reads(&source),
            1,
            "touched lookups fault only their page"
        );
        // A whole patch-stream traversal returns the oracle order and stays
        // within the historical pool by evicting.
        for position in 1..=commits {
            exact(&runtime, &source, position);
        }
        let pressure = runtime.residency_pressure().unwrap();
        assert!(pressure.historical_resident_bytes <= historical);
        let reads = page_reads(&source);
        assert!(reads >= pages.len(), "every page faulted once");
        if pages.len() > 4 {
            assert!(
                pressure.evictions_completed > 0,
                "pages evict under the budget"
            );
            // A second traversal re-faults evicted pages: nothing is pinned.
            for position in 1..=commits {
                runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(
                        position,
                    ))
                    .unwrap();
            }
            assert!(page_reads(&source) > reads);
        }
    }
    assert_eq!(
        reopened_resident[0], reopened_resident[1],
        "reopen residency is history-independent"
    );
}

#[test]
fn successive_captures_carry_unchanged_commit_pages_and_retire_superseded_ones() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let fixture = source_with_history(request, 300, false);
    let first = fixture.input.checkpoint().unwrap().checkpoint().clone();
    let first_source = fixture.source.clone();
    let first_pages = commit_pages(&first_source);
    assert_eq!(first_pages.len(), 3);
    // Reopen C1, commit a short tail and capture C2 carrying C1's members.
    let runtime = recover(
        request,
        first.clone(),
        first_source.clone(),
        durable_runtime(&fixture.medium),
    );
    let entity = fixture.entity;
    for update in 0..20 {
        update_entity(request, &runtime, entity, &format!("tail-{update}"));
    }
    let canonical_before = first_source.canonical_member_reads.lock().unwrap().len();
    let (second, second_source) = capture_external_carrying(request, &runtime, &first_source);
    assert!(
        page_reads(&first_source) <= 1,
        "capture reads only C1's last commit page"
    );
    assert_eq!(
        first_source.canonical_member_reads.lock().unwrap().len(),
        canonical_before,
        "covered canonical members are carried, not re-read"
    );
    let directory = super::selected_directory::read_native(&second);
    let second_pages = directory
        .commit_index
        .pages
        .iter()
        .map(|page| page.member)
        .collect::<Vec<_>>();
    assert_eq!(
        second_pages[..2],
        first_pages_in_order(&super::selected_directory::read_native(&first))[..2]
    );
    let carried =
        |checkpoint: &worth_relational::facade::durability::RelationalNativeCheckpoint,
         locator: &RelationalDurableMemberLocator| {
            checkpoint.members().iter().any(|member| {
            matches!(member, RelationalCheckpointMember::Carried { locator: carried } if carried == locator)
        })
        };
    assert!(second_pages[..2].iter().all(|page| carried(&second, page)));
    let superseded = first_pages_in_order(&super::selected_directory::read_native(&first))[2];
    assert!(
        !second_pages.contains(&superseded),
        "C1's partial tail page is superseded"
    );
    assert_eq!(directory.commit_index.count, 321);
    // C3 without new commits carries every C2 page unchanged.
    let (third, _) = capture_external_carrying(request, &runtime, &second_source);
    let third_pages = super::selected_directory::read_native(&third)
        .commit_index
        .pages
        .iter()
        .map(|page| page.member)
        .collect::<Vec<_>>();
    assert_eq!(third_pages, second_pages);
    assert!(third_pages.iter().all(|page| carried(&third, page)));
}

fn first_pages_in_order(
    directory: &super::selected_directory::Directory,
) -> Vec<RelationalDurableMemberLocator> {
    directory
        .commit_index
        .pages
        .iter()
        .map(|page| page.member)
        .collect()
}
