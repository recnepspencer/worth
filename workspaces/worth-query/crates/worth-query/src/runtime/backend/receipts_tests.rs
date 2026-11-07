use super::*;
use crate::declarative_live::{DeclarativeLiveQueryRequest, DeclarativeLiveViewShape};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::{
    WorthQueryMutationDelta, WorthQueryMutationKind, WorthQueryMutationReceipt,
};
use crate::runtime::tests::support::test_bridge_with_writeback_authority;
use crate::runtime::{
    build_bridge_authority_bundle, WorthQueryAspectTouch, WorthQueryAuthoredAspectMutation,
    WorthQueryBackendAdmissibleMutation, WorthQueryWriteCommand,
};
use crate::WorthQueryEvidenceScope;
use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, CanonicalFieldPath, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType,
    StructAspectShape,
};
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;

#[test]
fn live_view_declaration_receipt_captures_request_shape() -> Result<(), IdentityDerivationDenial> {
    let request = DeclarativeLiveQueryRequest::new("Task", DeclarativeLiveViewShape::table());
    let receipt = LiveViewDeclarationAdmissionReceipt::from_request("tasks.table", &request)?;

    assert_eq!(receipt.view_name(), "tasks.table");
    assert_eq!(receipt.target_collection_for_reporting(), "Task");
    assert_eq!(receipt.view_shape(), &DeclarativeLiveViewShape::table());
    assert_eq!(receipt.view_shape_for_reporting(), "table");
    assert!(!receipt.receipt_for_reporting().is_empty());

    Ok(())
}

#[test]
fn signal_invalidation_routing_receipt_rejects_authority_less_receipt(
) -> Result<(), IdentityDerivationDenial> {
    let receipt = WorthQueryMutationReceipt::from_authoritative_parts(
        crate::memory_workspace::admit_external_commit_label("commit-1")?,
        crate::memory_workspace::admit_external_snapshot_label("snapshot-1")?,
        Vec::new(),
    );

    let error = SignalInvalidationRoutingReceipt::from_mutation_receipt(&receipt)
        .expect_err("authority-less receipt must not route signal invalidation");

    assert!(error
        .to_string()
        .contains("requires bridge-authored mutation authority"));

    Ok(())
}

#[test]
fn signal_invalidation_routing_receipt_summarizes_authoritative_delta(
) -> Result<(), IdentityDerivationDenial> {
    let command_entity_identity = relational_entity_identity(1);
    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: command_entity_identity.clone(),
        aspects: vec![
            WorthQueryAuthoredAspectMutation::new_set(
                title_value_touch(),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("Done"),
            )
            .expect("title aspect should build"),
            WorthQueryAuthoredAspectMutation::new_set(
                status_value_touch(),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("closed"),
            )
            .expect("status aspect should build"),
        ],
        metadata: Default::default(),
        naming_intent: None,
        continuity_intent: None,
    };
    let mutation = admit_test_mutation(command, [title_value_touch(), status_value_touch()]);
    let bridge = test_bridge_with_writeback_authority();
    let snapshot_identity =
        crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(1, 1),
        );
    let bridge_authority = build_bridge_authority_bundle(
        &bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            "Task",
            &command_entity_identity,
            WorthQueryMutationKind::Updated,
        ),
    )
    .expect("test bridge authority should build");
    let receipt = WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        crate::memory_workspace::WorthQueryCommitIdentity::from_relational_commit_id(1),
        snapshot_identity,
        vec![WorthQueryMutationDelta::from_touched_aspects(
            "Task",
            command_entity_identity,
            WorthQueryMutationKind::Updated,
            vec![title_value_touch(), status_value_touch()],
        )?],
        bridge_authority,
    )
    .admit_runtime_write_authority();

    let routed = SignalInvalidationRoutingReceipt::from_mutation_receipt(&receipt)
        .expect("bridge-authored receipt should route");

    assert!(!routed.causality_digest().is_empty());
    assert_eq!(routed.delta_count(), 1);
    assert_eq!(routed.routed_collection_count(), 1);
    assert_eq!(
        routed.receipt_identity().scope(),
        WorthQueryEvidenceScope::SignalInvalidationRoutingReceipt
    );
    assert!(!routed.receipt_identity().as_str().is_empty());

    Ok(())
}

#[test]
fn bridge_authority_cannot_admit_a_different_commit_or_snapshot_handoff(
) -> Result<(), IdentityDerivationDenial> {
    let authority = bridge_authority_for_title_value("Done");
    let entity_identity = crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 1, 0),
    );
    let delta = WorthQueryMutationDelta::from_touched_aspects(
        "Task",
        entity_identity,
        WorthQueryMutationKind::Updated,
        vec![title_value_touch()],
    )?;
    let wrong_commit = WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        crate::memory_workspace::WorthQueryCommitIdentity::from_relational_commit_id(2),
        crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(1, 1),
        ),
        vec![delta.clone()],
        authority.clone(),
    )
    .admit_runtime_write_authority();
    let wrong_snapshot = WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        crate::memory_workspace::WorthQueryCommitIdentity::from_relational_commit_id(1),
        crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(2, 1),
        ),
        vec![delta],
        authority,
    )
    .admit_runtime_write_authority();

    assert!(!wrong_commit.has_current_mutation_authority());
    assert!(!wrong_snapshot.has_current_mutation_authority());

    Ok(())
}

#[test]
fn bridge_authority_cannot_admit_a_different_mutation_subject(
) -> Result<(), IdentityDerivationDenial> {
    let authority = bridge_authority_for_title_value("Done");
    let mismatched_deltas = [
        WorthQueryMutationDelta::from_touched_aspects(
            "OtherTask",
            relational_entity_identity(1),
            WorthQueryMutationKind::Updated,
            vec![title_value_touch()],
        )?,
        WorthQueryMutationDelta::from_touched_aspects(
            "Task",
            relational_entity_identity(2),
            WorthQueryMutationKind::Updated,
            vec![title_value_touch()],
        )?,
        WorthQueryMutationDelta::from_touched_aspects(
            "Task",
            relational_entity_identity(1),
            WorthQueryMutationKind::Created,
            vec![title_value_touch()],
        )?,
        WorthQueryMutationDelta::from_touched_aspects(
            "Task",
            relational_entity_identity(1),
            WorthQueryMutationKind::Updated,
            vec![status_value_touch()],
        )?,
    ];

    Ok(for delta in mismatched_deltas {
        let receipt =
            projected_receipt_for_delta(delta, authority.clone()).admit_runtime_write_authority();
        assert!(!receipt.has_current_mutation_authority());
    })
}

#[test]
fn bridge_writeback_effect_intent_is_bound_to_admitted_aspect_patch() {
    let baseline = bridge_authority_for_title_value("Done");
    let changed = bridge_authority_for_title_value("Blocked");

    let baseline_basis = baseline.provenance().effect_intent_patch_canonical_basis();
    let changed_basis = changed.provenance().effect_intent_patch_canonical_basis();

    assert_ne!(baseline_basis, changed_basis);
    assert!(baseline_basis.contains("domain=authoritative-patch"));
    assert!(baseline_basis.contains("title.field.value.set"));
    assert!(
        baseline_basis.contains("exact-text:Done"),
        "{baseline_basis}"
    );
    assert!(!baseline_basis.contains("worth.query.writeback"));
}

#[test]
fn bridge_writeback_effect_intent_accepts_whole_entity_delete_empty_patch() {
    let entity_identity = relational_entity_identity(1);
    let command = WorthQueryWriteCommand::Delete {
        entity_identity: entity_identity.clone(),
    };
    let mutation = admit_test_mutation(command, []);
    let bridge = test_bridge_with_writeback_authority();
    let snapshot_identity =
        crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(1, 1),
        );

    let bridge_authority = build_bridge_authority_bundle(
        &bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            "Task",
            &entity_identity,
            WorthQueryMutationKind::Deleted,
        ),
    )
    .expect("whole-entity delete should lower through an empty native patch");
    let basis = bridge_authority
        .provenance()
        .effect_intent_patch_canonical_basis();

    assert!(basis.contains("domain=authoritative-patch"));
    assert!(!basis.contains(".set"));
    assert!(!basis.contains(".clear"));
}

fn bridge_authority_for_title_value(
    value: &str,
) -> worth_runtime_bridge::facade::BridgeMutationAuthorityBundle {
    let entity_identity = relational_entity_identity(1);
    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: entity_identity.clone(),
        aspects: vec![WorthQueryAuthoredAspectMutation::new_set(
            title_value_touch(),
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value),
        )
        .expect("title aspect should build")],
        metadata: Default::default(),
        naming_intent: None,
        continuity_intent: None,
    };
    let mutation = admit_test_mutation(command, [title_value_touch()]);
    let bridge = test_bridge_with_writeback_authority();
    let snapshot_identity =
        crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(1, 1),
        );
    build_bridge_authority_bundle(
        &bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            "Task",
            &entity_identity,
            WorthQueryMutationKind::Updated,
        ),
    )
    .expect("test bridge authority should build")
}

fn projected_receipt_for_delta(
    delta: WorthQueryMutationDelta,
    authority: worth_runtime_bridge::facade::BridgeMutationAuthorityBundle,
) -> WorthQueryMutationReceipt {
    WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        crate::memory_workspace::WorthQueryCommitIdentity::from_relational_commit_id(1),
        crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(1, 1),
        ),
        vec![delta],
        authority,
    )
}

fn relational_entity_identity(
    local_slot: u64,
) -> crate::memory_workspace::WorthQueryEntityIdentity {
    crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, local_slot, 0),
    )
}

fn title_value_touch() -> WorthQueryAspectTouch {
    aspect_field_touch("title", "value")
}

fn status_value_touch() -> WorthQueryAspectTouch {
    aspect_field_touch("status", "value")
}

fn aspect_field_touch(
    aspect_label: &'static str,
    field_label: &'static str,
) -> WorthQueryAspectTouch {
    let aspect_key =
        AspectKey::new(aspect_label).expect("receipt test static aspect key should admit");
    let field_key = FieldKey::new(field_label).expect("receipt test static field key should admit");
    let field_path =
        CanonicalFieldPath::new([field_key]).expect("receipt test static field path should admit");
    WorthQueryAspectTouch::aspect_field_path(aspect_key, field_path)
}

fn admit_test_mutation<const N: usize>(
    command: WorthQueryWriteCommand,
    touches: [WorthQueryAspectTouch; N],
) -> WorthQueryBackendAdmissibleMutation {
    let contracts =
        crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
            touches.into_iter().map(string_field_contract),
        )
        .expect("receipt test contracts should agree");
    WorthQueryBackendAdmissibleMutation::from_authored_command(command, &contracts)
        .expect("receipt test mutation should satisfy native contracts")
}

fn string_field_contract(touch: WorthQueryAspectTouch) -> AspectContract {
    let field = touch
        .native_field_path()
        .expect("receipt test field touch should contain a field")
        .fields()[0]
        .clone();
    let declaration = FieldDeclaration::new(
        field,
        ScalarAspectType::String,
        FieldRequirement::Optional,
        AbsenceLaw::Optional,
        AspectEvolutionPolicy::AdditiveFieldsAllowed,
    )
    .expect("receipt test field declaration should be coherent");
    AspectContract::struct_aspect(
        touch.native_aspect_key().clone(),
        AspectIdentity(1),
        AspectContractRevision(1),
        StructAspectShape::new([declaration]).expect("receipt test shape should be unique"),
    )
}
