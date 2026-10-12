//! certification intent mutation receipt.
use super::*;

pub(super) fn certification_intent_mutation_receipt(
    bridge: &RuntimeBridge,
    collection: &str,
) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
    let entity_identity = certification_entity_identity("certification-intent-entity-1");
    let snapshot_identity = WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(1, 1),
    );
    let touch = title_value_touch();
    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: entity_identity.clone(),
        aspects: vec![WorthQueryAuthoredAspectMutation::new_set(
            touch.clone(),
            WorthQueryAuthoredAspectMutation::native_string_value("Certification title"),
        )
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?],
        metadata: Default::default(),
        naming_intent: None,
        continuity_intent: None,
    };
    let contracts = crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
        certification_aspect_contracts(),
    )
    .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    let mutation = WorthQueryBackendAdmissibleMutation::from_authored_command(command, &contracts)
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    let bridge_authority = crate::runtime::build_bridge_authority_bundle(
        bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            collection,
            &entity_identity,
            WorthQueryMutationKind::Updated,
        ),
    )?;
    Ok(WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        crate::memory_workspace::WorthQueryCommitIdentity::from_relational_commit_id(1),
        snapshot_identity,
        vec![WorthQueryMutationDelta::from_touched_aspects(
            collection,
            entity_identity,
            WorthQueryMutationKind::Updated,
            vec![touch],
        )],
        bridge_authority,
    ))
}
