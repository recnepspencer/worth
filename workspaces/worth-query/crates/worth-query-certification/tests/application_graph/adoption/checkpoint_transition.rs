//! Target laws must judge all recovered rows before a World exists.
use crate::document_retention_model::{
    host::{host_limits, publish_on_first_program, restore_on_first_program},
    presented_request::set_retention,
    programs::{validated_first_program, validated_second_program},
    readback::read_retention,
    schema::{Document, DocumentIdentityField, DocumentRetentionField, DocumentRetentionSchema},
    settled_verdict::{settle, RetentionVerdict},
};
use worth_query_host::facade::{application_installation as installation, primary_graph};

#[test]
fn checkpoint_transition_target_rule_rejects_retained_predecessor_only_value() {
    let host = publish_on_first_program();
    let predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &host.installed_program().revision().to_string(),
    )
    .unwrap();
    assert_eq!(
        settle(set_retention(&host, host.current_world(), 3, 0x61_0001)),
        RetentionVerdict::Performed(3)
    );
    let (source, sections) = host
        .runtime()
        .capture_application_checkpoint_with_sections()
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 0);
    drop(host);
    let mut authored = false;
    let denial = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        validated_second_program(),
        installation::WorthQueryApplicationProgramRoster::new().support(validated_first_program()),
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        host_limits(),
        source.clone(),
        predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(512, 32, 8192).unwrap(),
        |writer, _| {
            authored = true;
            writer.bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    Document::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new("migration-probe").unwrap(),
                )
                .field(
                    DocumentIdentityField::reference(),
                    "migration-probe".to_owned(),
                )
                .field(DocumentRetentionField::reference(), 7_u64),
            )
        },
    )
    .err()
    .expect("target law must reject the retained P0-only value, with no World or migrated effects");
    assert!(
        authored,
        "candidate validation follows source preflight and authoring"
    );
    assert!(
        format!("{denial:?}").contains("document-retention-v2"),
        "target rule identity must survive denial: {denial:?}"
    );
    let restored = restore_on_first_program(
        source,
        installation::WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap();
    assert_eq!(
        read_retention(restored.runtime(), restored.current_world()),
        3,
        "denied transition leaves the original checkpoint lawfully restorable"
    );
}

#[test]
fn checkpoint_transition_target_rule_accepts_complete_retained_and_created_state() {
    let host = publish_on_first_program();
    let predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &host.installed_program().revision().to_string(),
    )
    .unwrap();
    let source = host.runtime().capture_application_checkpoint().unwrap();
    drop(host);
    let migrated = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        validated_second_program(),
        installation::WorthQueryApplicationProgramRoster::new().support(validated_first_program()),
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        host_limits(),
        source,
        predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(512, 32, 8192).unwrap(),
        |writer, _| {
            writer.bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    Document::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new("migration-positive")
                        .unwrap(),
                )
                .field(
                    DocumentIdentityField::reference(),
                    "migration-positive".to_owned(),
                )
                .field(DocumentRetentionField::reference(), 7_u64),
            )
        },
    )
    .expect("target v2 rule accepts both retained and created values in the activation candidate");
    let selected = migrated
        .runtime()
        .on_branch(migrated.current_world())
        .select()
        .unwrap();
    assert_eq!(
        selected.inspect_selected_program().unwrap().revision(),
        validated_second_program().revision()
    );
    assert!(selected
        .resolve_entity(
            DocumentIdentityField::reference(),
            "migration-positive".to_owned(),
            &crate::document_retention_model::operator_identity::request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary
        )
        .is_ok());
}
