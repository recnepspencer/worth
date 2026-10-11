mod support;

use worth_relational::facade::{history::BranchId, runtime::RelationalRuntimeApi};

fn main() {
    let host_request = support::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();

    let (seed, entity_id) = support::create_entity(execution, &runtime, "main-seed");
    let (_, fork_basis) = runtime
        .observe_fork_source(&BranchId("main".to_string()))
        .expect("observe main fork source");
    runtime
        .fork_branch(BranchId("feature".to_string()), fork_basis)
        .expect("create branch");

    let feature_commit = support::update_entity_on_branch(
        execution,
        &runtime,
        entity_id,
        "feature-name",
        Some(BranchId("feature".to_string())),
    );

    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(feature_commit.commit.commit_id)
        .expect("replay envelope");

    println!("commits={}", runtime.history().immutable_commit_count());
    println!(
        "seed_commit={} feature_commit={} authoritative_record_patches={}",
        seed.commit.commit_id.0,
        feature_commit.commit.commit_id.0,
        envelope.patch.authoritative_record_patches.len()
    );
}
