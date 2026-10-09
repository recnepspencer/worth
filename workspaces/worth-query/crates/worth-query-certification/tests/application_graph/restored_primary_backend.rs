//! A Query runtime composed over a restored application gives the same answer
//! to the same write as one composed over the application before capture.
//!
//! The write reaches `WorthQueryPrimaryGraphBackendHandle::execute_mutation`,
//! the production owner of index maintenance. Its authority commits once on
//! main and then declines, so the runtime answers with that refusal exactly
//! when maintenance of the new commit succeeds. After a restore the durable
//! ledger carries the patch-stream position, so the first write resumes the
//! stream where the capture left it.

use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
#[path = "restored_primary_backend/adapters.rs"]
mod adapters;

use std::sync::Arc;

use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, CanonicalFieldPath, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType,
    StructAspectShape,
};
use worth_query::facade::runtime::{
    WorthQueryAspectMutationBuilder, WorthQueryAspectTouch, WorthQueryAuthoredAspectValue,
    WorthQueryPrimaryGraphSourceAdapter, WorthQueryRuntime, WorthQueryWriteCommand,
};
use worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation;
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
use worth_relational::facade::publication::PatchStreamPosition;

use crate::document_retention_model::host::{publish_on_first_program, restore_on_first_program};
use crate::document_retention_model::programs::validated_second_program;
use adapters::{CommitLog, CommitThenDecline, DECLINED_AFTER_COMMIT};

#[test]
fn the_first_production_write_after_restore_answers_as_before_the_capture() {
    let host = publish_on_first_program();
    let before = write_twice(&host.granular_invalidation_installation());
    let captured = ledger_position(&host.granular_invalidation_installation());
    let checkpoint = host
        .runtime()
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .expect("the host captures");
    drop(host);

    let restored = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    let installation = restored.granular_invalidation_installation();
    assert_eq!(
        ledger_position(&installation),
        captured,
        "a restore answers the ledger position the capture held"
    );

    let after = write_twice(&installation);
    assert_eq!(
        after, before,
        "the same write answers the same after restore"
    );
    assert_eq!(
        ledger_position(&installation),
        captured.map(|position| PatchStreamPosition(position.0 + 2)),
        "each production write committed exactly once after the restore"
    );
}

/// Composes a Query runtime over the installation and writes twice, returning
/// each answer. Every write must have committed exactly once.
fn write_twice(installation: &WorthQueryGranularInvalidationInstallation) -> Vec<String> {
    let commits = CommitLog::default();
    let mut runtime = production_runtime(installation, Arc::clone(&commits));
    let answers = (0..2)
        .map(|ordinal| {
            let answer = runtime
                .write(identity_insert(ordinal))
                .expect_err("the certification authority declines after its commit")
                .to_string();
            assert!(
                answer.contains(DECLINED_AFTER_COMMIT),
                "index maintenance of the committed write succeeded: {answer}"
            );
            answer
        })
        .collect();
    assert_eq!(
        commits.lock().unwrap().len(),
        2,
        "each write committed once"
    );
    answers
}

fn production_runtime(
    installation: &WorthQueryGranularInvalidationInstallation,
    commits: CommitLog,
) -> WorthQueryRuntime {
    let graph = installation.retain_primary_graph_integration_handle();
    let contract = identity_contract();
    let bridge = adapters::runtime_bridge(&graph, &contract, FieldKey::new("id").unwrap());
    WorthQueryRuntime::builder(
        worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
    )
    .primary_runtime_granular_invalidations(installation.clone())
    .conditional_execution_resources(
        worth_query::facade::runtime::WorthQueryConditionalExecutionResources::development(),
    )
    .aspect_contract(contract)
    .expect("the identity contract installs")
    .runtime_bridge(bridge)
    .schema_adapter(adapters::SchemaAdapter)
    .source_adapter(WorthQueryPrimaryGraphSourceAdapter::new(
        installation,
        adapters::EmptyProjection,
    ))
    .snapshot_identity(adapters::SnapshotAdapter(graph))
    .write_authority(CommitThenDecline(commits))
    .signal_sink(adapters::SignalSink)
    .subscription_activation(adapters::SubscriptionActivation)
    .preview_basis(adapters::PreviewBasis)
    .inspector_evidence(adapters::InspectorEvidence)
    .build_backend_from_parts()
    .build()
    .expect("the Query runtime composes over the primary graph")
}

fn ledger_position(
    installation: &WorthQueryGranularInvalidationInstallation,
) -> Option<PatchStreamPosition> {
    installation
        .retain_primary_graph_integration_handle()
        .with_runtime(|runtime| runtime.history().latest_patch_stream_position())
}

fn identity_insert(ordinal: usize) -> WorthQueryWriteCommand {
    let touch = WorthQueryAspectTouch::aspect_field_path(
        AspectKey::new("identity").unwrap(),
        CanonicalFieldPath::new([FieldKey::new("id").unwrap()]).unwrap(),
    );
    WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            touch,
            WorthQueryAuthoredAspectValue::string(format!("record-{ordinal}")),
        )
        .build_insert("Record")
        .expect("the certification insert builds")
}

fn identity_contract() -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new("id").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("identity").unwrap(),
        AspectIdentity(0x5752_5001),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    )
}
