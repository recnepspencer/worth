//! Native changed-codec retirement retains an independent historical publication.
use super::{
    companion as old,
    successor::{self, companion as new},
    *,
};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{application_entry::*, application_installation as installation};
type Original = old::host::Host<old::program::SourceProgram>;
type Target = new::host::Host<new::program::TargetProgram>;
type OriginalSettlement =
    worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
        query::ValueQuery,
    >;
fn original(value: model::NestedValue) -> (Original, OriginalSettlement) {
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
fn transition(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
    policy: installation::WorthQueryCheckpointCapturePolicy<'_, '_>,
) -> Result<Target, installation::WorthQueryInMemoryApplicationDenial> {
    transition_counted(checkpoint, policy, &mut 0)
}
fn transition_counted(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
    policy: installation::WorthQueryCheckpointCapturePolicy<'_, '_>,
    callbacks: &mut usize,
) -> Result<Target, installation::WorthQueryInMemoryApplicationDenial> {
    installation::in_memory_rostered_program_from_checkpoint_with_transition(
        new::host::program::<new::program::TargetProgram>(),
        installation::WorthQueryApplicationProgramRoster::new(),
        successor::schema::CanonicalSchema::declaration().unwrap(),
        ((),),
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
fn companion_ids(
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
fn assert_target(host: &Target, ids: [worth_relational::facade::identity::EntityId; 2]) {
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
#[test]
fn companion_historical_rows_survive_real_transition_production_preserve_and_reopen() {
    let (original, settlement) = original(model::authored_value());
    let ids = companion_ids(&settlement);
    let source_id = old::reads::source_id(&original);
    let first = old::reads::page(&original);
    journey::assert_old(&first, "derived-left");
    let generation = first.receipt().ordered_index_generation();
    let (rows, token, _) = first.into_parts();
    let mut before_capture = model::authored_value();
    before_capture.notes = Some("genuine source edit before checkpoint".into());
    let changed = old::host::edit_source(&original, before_capture.clone(), 0x2712900);
    assert!(changed.outputs_of::<output::PreservedOutputs>().is_ok());
    let preserved = old::host::produce(&original);
    let initial_receipt = settlement.application_commit_receipt().unwrap();
    let executed_receipt = preserved.application_commit_receipt().unwrap();
    assert_eq!(
        executed_receipt.changed_record_count(),
        1,
        "only the mandatory Query idempotency record changes; both producer members are unpatched"
    );
    assert_eq!(executed_receipt.emitted_effect_count(), 0);
    assert_ne!(
        executed_receipt.commit_reference(),
        initial_receipt.commit_reference()
    );
    assert_cached_companion(&original, executed_receipt, ids);

    let roles = preserved
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
        ids,
        "real V1 Preserve retains both original native companion identities"
    );
    let checkpoint = old::host::capture(&original);
    let held = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    successor::output::reset_decodes();
    let target = transition(
        checkpoint,
        installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
    )
    .expect("native survivor association plus target relation rule admits");
    assert_eq!(
        successor::output::decodes(),
        0,
        "V2 must never decode predecessor derived bytes"
    );
    assert_eq!(new::reads::source_id(&target), source_id);
    assert_eq!(new::reads::read(&target), before_capture);
    assert_target(&target, ids);
    let updated = new::host::edit_source(&target, journey::edited(), 0x2712901);
    assert!(updated
        .outputs_of::<successor::output::PreservedOutputs>()
        .is_ok());
    assert_eq!(
        successor::output::literals(&new::reads::read_derived(&target)).1,
        1037
    );
    // The historical token is still the original V1 native index/pin; no cast
    // or replacement token can provide the V1 changed derived literal.
    let scope = request_scope();
    let principal = authenticate_operator(original.installed_schema(), &scope);
    let second = original
        .runtime()
        .request(&principal, &scope)
        .on_branch(original.runtime().current_world())
        .query(page_query::ReadPage {
            identity: "derived-report".into(),
        })
        .resume(
            token.unwrap(),
            std::num::NonZeroUsize::MIN,
            std::num::NonZeroUsize::new(65_536).unwrap(),
        )
        .unwrap();
    journey::assert_old(&second, "derived-right");
    assert_eq!(second.receipt().ordered_index_generation(), generation);
    assert_eq!(output::literals(&rows[0].children[0].1).1, 12);
    assert_eq!(held.bytes(), bytes);
    let reopened = new::host::restore::<new::program::TargetProgram>(
        new::host::capture(&target),
        installation::WorthQueryApplicationProgramRoster::new(),
    );
    assert_eq!(new::reads::source_id(&reopened), source_id);
    assert_eq!(new::reads::read(&reopened), journey::edited());
    assert_eq!(
        successor::output::literals(&new::reads::read_derived(&reopened)).1,
        1037
    );
    assert_eq!(
        new::reads::read_companion(&reopened, "companion-right"),
        "companion-right"
    );
}
#[test]
fn companion_target_rule_rejects_actual_surviving_relation_before_publication() {
    let mut value = model::authored_value();
    value.title = "invalid-companion".into();
    let (original, _settlement) = original(value.clone());
    assert_eq!(
        old::reads::read_companion(&original, "companion-left"),
        "invalid"
    );
    let checkpoint = old::host::capture(&original);
    let held = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    successor::output::reset_decodes();
    let denial = transition(
        checkpoint,
        installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
    )
    .err()
    .expect("actual target rule must judge the surviving edge");
    let installation::WorthQueryInMemoryApplicationDenial::Graph(denial) = denial else {
        panic!("the target rule must reject native preparation before publication: {denial:?}")
    };
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected);
    assert_eq!(denial.subject(), "Relational rejected the application principal bootstrap transaction: custom invariant 'canonical-companion-link' reported a structural violation");
    assert_eq!(successor::output::decodes(), 0);
    assert_eq!(old::reads::read(&original), value);
    assert_eq!(held.bytes(), bytes);
    let reopened = old::host::restore::<old::program::SourceProgram>(
        held,
        installation::WorthQueryApplicationProgramRoster::new()
            .support(old::host::program::<old::program::TargetProgram>()),
    );
    assert_eq!(
        old::reads::read_companion(&reopened, "companion-left"),
        "invalid"
    );
}
#[test]
fn companion_mixed_native_output_is_not_partially_promoted_or_discarded() {
    let mut value = model::authored_value();
    value.title = "mixed-companion".into();
    let (original, _settlement) = original(value.clone());
    let checkpoint = old::host::capture(&original);
    let held = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    let denial = transition(
        checkpoint,
        installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
    )
    .err()
    .expect("one actual producer created both changed and stable output roles");
    let detail = format!("{denial:?}");
    assert!(
        detail.contains("partially retired output correspondence"),
        "{detail}"
    );
    assert_eq!(old::reads::read(&original), value);
    assert_eq!(held.bytes(), bytes);
}
#[path = "companion_transition/capture.rs"]
mod capture;

fn assert_cached_companion(
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
