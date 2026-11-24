use std::collections::BTreeMap;
use worth_query_host::facade::declaration::application_program::*;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::{seed_host, RelationalRetentionContribution},
    model_lifecycle::validated_model_lifecycle_program,
    programs::{
        validate, CommitBoundaryRule, DocumentRetentionV1, DocumentRetentionV2, RetentionProgramP0,
        RetentionProgramP1,
    },
};
pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::checkpoint_transition::*;

/// D16 Home program transitions are Relational-only (no authored Signal programme).
struct RelationalPredecessorProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for RelationalPredecessorProgram {
    type Contributions = (RelationalRetentionContribution,);
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.relational-p0.v1",
    );
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        RetentionProgramP0::feature_specs()
    }
}

struct RelationalSuccessorProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for RelationalSuccessorProgram {
    type Contributions = (RelationalRetentionContribution,);
    type Outputs =
        <RetentionProgramP1 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules = CommitBoundaryRule<DocumentRetentionV2>;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.relational-p1.v1",
    );
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        RetentionProgramP1::feature_specs()
    }
}

fn open_relational_predecessor(
    directory: std::path::PathBuf,
) -> DirectoryHomeGuard<DocumentRetentionSchema, RelationalPredecessorProgram> {
    let home = with_directory_bootstrap(|phase| {
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            validate::<RelationalPredecessorProgram>(),
            installation::WorthQueryApplicationProgramRoster::new()
                .support(validate::<RelationalSuccessorProgram>())
                .support(validated_actor_program())
                .support(validated_model_lifecycle_program()),
            DocumentRetentionSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            host_limits(
                worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
            ),
            seed_host,
        )
        .open_in_advancement(phase)
    })
    .unwrap_or_else(|refused| panic!("relational predecessor Home open refused: {refused:?}"));
    DirectoryHomeGuard::wrap(home, directory)
}

fn transition_resources() -> home_program_transition::DerivedIndexMaintenanceBudget {
    home_program_transition::DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1_000_000),
        maximum_cold_record_slots: Some(100_000),
        maximum_derived_rows: Some(100_000),
    }
}

fn create_document_step(
    target: &home_program_transition::WorthQueryHomeProgramTransitionTarget<'_>,
    key: &str,
    retention_days: u64,
) -> WorthQueryHomeProgramTransitionStep {
    use home_program_transition::*;
    let id_field = target
        .planned_field(
            "Document",
            DocumentIdentityField::at("DocumentIdentityField").aspect(),
            "DocumentIdentityField",
        )
        .unwrap();
    let retention_field = target
        .planned_field(
            "Document",
            DocumentRetentionField::at("DocumentRetentionField").aspect(),
            "DocumentRetentionField",
        )
        .unwrap();
    let mut fields = BTreeMap::new();
    fields.insert(
        id_field,
        AspectValue::String(worth_foundational::InternedString::Raw(key.to_owned())),
    );
    fields.insert(retention_field, AspectValue::UInt64(retention_days));
    WorthQueryHomeProgramTransitionStep {
        batches: vec![
            WorkerIntentBatch::new("migration-document").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: target.entity_kind("Document").unwrap(),
                    client_key: ClientKey::raw(key),
                    fields: AspectFieldPatch::new(fields),
                }),
            )),
        ],
    }
}

fn predecessor_request(
    directory: std::path::PathBuf,
) -> WorthQueryApplicationHomeRequest<
    'static,
    DocumentRetentionSchema,
    RelationalPredecessorProgram,
> {
    let roster = installation::WorthQueryApplicationProgramRoster::new()
        .support(validate::<RelationalSuccessorProgram>())
        .support(validated_actor_program())
        .support(validated_model_lifecycle_program());
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(directory),
        validate::<RelationalPredecessorProgram>(),
        roster,
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, _, _| panic!("existing predecessor Home cannot initialize during transition"),
    )
}

fn successor_request(
    directory: std::path::PathBuf,
) -> WorthQueryApplicationHomeRequest<
    'static,
    DocumentRetentionSchema,
    RelationalSuccessorProgram,
> {
    let roster = installation::WorthQueryApplicationProgramRoster::new()
        .support(validate::<RelationalPredecessorProgram>())
        .support(validated_actor_program())
        .support(validated_model_lifecycle_program());
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(directory),
        validate::<RelationalSuccessorProgram>(),
        roster,
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, _, _| panic!("existing successor Home cannot initialize during transition"),
    )
}

#[test]
fn checkpoint_transition_target_rule_rejects_retained_predecessor_only_value() {
    let directory = unique_directory("program-transition-reject");
    let host = open_relational_predecessor(directory);
    assert_eq!(
        settle(set_retention(&host, host.current_world(), 3, 0x61_0001)),
        RetentionVerdict::Performed(3)
    );
    let directory = host.checkpoint_and_close_for_reopen();
    let mut authored = false;
    let request = successor_request(directory.clone()).with_program_transition(
        predecessor_request(directory.clone()),
        transition_resources(),
        |_source, target| {
            authored = true;
            Ok(WorthQueryHomeProgramTransitionPlan {
                target: 0,
                schema_transition: None,
                steps: vec![create_document_step(target, "migration-probe", 7)],
            })
        },
    );
    let denial = with_directory_bootstrap(|phase| request.open_in_advancement(phase))
        .err()
        .expect("target law must reject the retained P0-only value, with no migrated effects");
    assert!(
        authored,
        "candidate validation follows source preflight and authoring: {denial:?}"
    );
    assert!(
        format!("{denial:?}").contains("document-retention-v2"),
        "target rule identity must survive denial: {denial:?}"
    );
    assert!(
        matches!(
            denial.phase(),
            worth_query_host::facade::WorthQueryHomeProgramTransitionOpenPhase::Transition {
                cause: worth_query_host::facade::WorthQueryHomeProgramTransitionDenial::Commit(_),
                ..
            }
        ),
        "target rule refusal occurs during candidate commit validation: {denial:?}"
    );
    drop(denial);
    let reopened = with_directory_bootstrap(|phase| {
        predecessor_request(directory.clone()).open_in_advancement(phase)
    })
    .err()
    .expect("once a D16 transition intent is published, ordinary predecessor open must be refused");
    assert!(
        format!("{reopened:?}").contains("TransitionPending"),
        "once a D16 transition intent is published, ordinary predecessor open is refused with TransitionPending: {reopened:?}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn checkpoint_transition_target_rule_accepts_complete_retained_and_created_state() {
    let directory = unique_directory("program-transition-accept");
    let host = open_relational_predecessor(directory);
    let directory = host.checkpoint_and_close_for_reopen();
    let request = successor_request(directory.clone()).with_program_transition(
        predecessor_request(directory.clone()),
        transition_resources(),
        |_source, target| {
            Ok(WorthQueryHomeProgramTransitionPlan {
                target: 0,
                schema_transition: None,
                steps: vec![create_document_step(target, "migration-positive", 7)],
            })
        },
    );
    let home = with_directory_bootstrap(|phase| request.open_in_advancement(phase))
        .expect("target v2 rule accepts both retained and created values in the activation candidate");
    let migrated = DirectoryHomeGuard::wrap(home, directory);
    let selected = migrated
        .with_host_advancement(|host_phase| {
            migrated
                .runtime()
                .on_branch(migrated.current_world())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(
        selected.inspect_selected_program().unwrap().revision(),
        validate::<RelationalSuccessorProgram>().revision()
    );
    assert!(migrated
        .with_host_advancement(|host_phase| selected
            .resolve_entity(
                &host_phase,
                DocumentIdentityField::at("DocumentIdentityField"),
                "migration-positive".to_owned(),
                &worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::request_scope(),
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary
            )
            .is_ok())
        .expect("installed host call opens"));
}
