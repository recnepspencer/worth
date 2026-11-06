use super::super::batch_digest_helpers::{
    batch_continuity_mutation_digest, batch_existing_truth_binding_digest,
    batch_naming_mutation_digest, batch_symbolic_target_reference_digest,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryContinuityMutationEvidence, WorthQueryContinuityMutationFamily,
    WorthQueryExistingTruthBindingEvidence, WorthQueryNamingMutationEvidence,
    WorthQuerySymbolicTargetReferenceEvidence,
};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};
use worth_runtime_bridge::facade::{
    BridgeContinuityAuthoritativeIdentity, BridgeContinuityMutationBundle,
    BridgeContinuityOutcomeClass, BridgeContinuityResolvedTargetIdentity,
    BridgeContinuityTargetCollection, BridgeExistingTruthBindingAuthoritativeIdentity,
    BridgeExistingTruthBindingBundle, BridgeExistingTruthBindingResolvedTargetIdentity,
    BridgeExistingTruthBindingTargetCollection, BridgeNamingAttachmentIdentity,
    BridgeNamingMutationBundle, BridgeNamingResolvedTargetIdentity, BridgeNamingTargetCollection,
    RelationalBridgeRecordIdentityParts,
};

mod existing_truth_mode;

#[test]
fn existing_truth_binding_batch_digest_changes_with_authoritative_identity(
) -> Result<(), IdentityDerivationDenial> {
    let left = WorthQueryExistingTruthBindingEvidence::from_bridge(
        &BridgeExistingTruthBindingBundle::direct_entity(
            bridge_existing_truth_authority("authority:left")?,
            bridge_existing_truth_target(RelationalBridgeRecordIdentityParts::entity(1, 1, 0)),
            Some(bridge_existing_truth_collection("Task")),
        ),
    )?;
    let right = WorthQueryExistingTruthBindingEvidence::from_bridge(
        &BridgeExistingTruthBindingBundle::direct_entity(
            bridge_existing_truth_authority("authority:right")?,
            bridge_existing_truth_target(RelationalBridgeRecordIdentityParts::entity(1, 1, 0)),
            Some(bridge_existing_truth_collection("Task")),
        ),
    )?;

    let left_digest =
        batch_existing_truth_binding_digest(&[Some(left)])?.expect("left digest should exist");
    let right_digest =
        batch_existing_truth_binding_digest(&[Some(right)])?.expect("right digest should exist");

    assert_ne!(left_digest, right_digest);

    Ok(())
}

#[test]
fn bridge_existing_truth_binding_preserves_relational_record_identity(
) -> Result<(), IdentityDerivationDenial> {
    let evidence = WorthQueryExistingTruthBindingEvidence::from_bridge(
        &BridgeExistingTruthBindingBundle::direct_entity(
            bridge_existing_truth_authority("authority:relational")?,
            bridge_existing_truth_target(RelationalBridgeRecordIdentityParts::entity(7, 42, 3)),
            Some(bridge_existing_truth_collection("Task")),
        ),
    )?;

    assert_eq!(
        evidence
            .resolved_target_identity()
            .relational_record_parts(),
        Some(RelationalBridgeRecordIdentityParts::entity(7, 42, 3))
    );

    Ok(())
}

#[test]
#[should_panic(
    expected = "existing-truth binding evidence must carry a relational record target identity"
)]
fn existing_truth_binding_evidence_rejects_non_relational_target_identity(
) -> Result<(), IdentityDerivationDenial> {
    let binding = crate::runtime::WorthQueryExistingTruthTargetBinding::direct_entity(
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:left")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity"),
        crate::memory_workspace::admit_authored_entity_label("Task:legacy")?,
    )
    .expect("binding constructor should still expose the boundary failure to evidence");

    let _ = WorthQueryExistingTruthBindingEvidence::from_binding(&binding)?;

    Ok(())
}

#[test]
fn symbolic_target_batch_digest_changes_with_symbol_identity(
) -> Result<(), IdentityDerivationDenial> {
    let resolved_identity = crate::memory_workspace::admit_authored_entity_label("entity:task")?;
    let left = WorthQuerySymbolicTargetReferenceEvidence::test_only(
        "draft:left",
        resolved_identity.clone(),
        Some("Task"),
    );
    let right = WorthQuerySymbolicTargetReferenceEvidence::test_only(
        "draft:right",
        resolved_identity,
        Some("Task"),
    );

    let left_digest =
        batch_symbolic_target_reference_digest(&[Some(left)]).expect("left digest should exist");
    let right_digest =
        batch_symbolic_target_reference_digest(&[Some(right)]).expect("right digest should exist");

    assert_ne!(left_digest, right_digest);

    Ok(())
}

#[test]
fn naming_batch_digest_changes_with_attachment_identity() -> Result<(), IdentityDerivationDenial> {
    let left = WorthQueryNamingMutationEvidence::from_bridge(
        &BridgeNamingMutationBundle::attach_new_target(
            bridge_naming_attachment("persistent-name:left")?,
            bridge_naming_target(RelationalBridgeRecordIdentityParts::entity(1, 2, 0)),
            Some(bridge_naming_collection("Task")),
        ),
    );
    let right = WorthQueryNamingMutationEvidence::from_bridge(
        &BridgeNamingMutationBundle::attach_new_target(
            bridge_naming_attachment("persistent-name:right")?,
            bridge_naming_target(RelationalBridgeRecordIdentityParts::entity(1, 2, 0)),
            Some(bridge_naming_collection("Task")),
        ),
    );

    let left_digest =
        batch_naming_mutation_digest(&[Some(left)])?.expect("left digest should exist");
    let right_digest =
        batch_naming_mutation_digest(&[Some(right)])?.expect("right digest should exist");

    assert_ne!(left_digest, right_digest);

    Ok(())
}

#[test]
fn split_successor_batch_digest_changes_with_successor_set() -> Result<(), IdentityDerivationDenial>
{
    let left = WorthQueryContinuityMutationEvidence::from_bridge(
        &BridgeContinuityMutationBundle::split_existing_target(
            BridgeContinuityOutcomeClass::ContinuesAsSplitSuccessors,
            continuity_identity("authority:task-1"),
            [
                continuity_identity("authority:task-1:a"),
                continuity_identity("authority:task-1:b"),
            ],
            Some(resolved_target("entity:task-1")),
            Some(target_collection("Task")),
        )
        .expect("left split continuity should build"),
    )?;
    let right = WorthQueryContinuityMutationEvidence::from_bridge(
        &BridgeContinuityMutationBundle::split_existing_target(
            BridgeContinuityOutcomeClass::ContinuesAsSplitSuccessors,
            continuity_identity("authority:task-1"),
            [
                continuity_identity("authority:task-1:a"),
                continuity_identity("authority:task-1:c"),
            ],
            Some(resolved_target("entity:task-1")),
            Some(target_collection("Task")),
        )
        .expect("right split continuity should build"),
    )?;

    let left_digest =
        batch_continuity_mutation_digest(&[Some(left)])?.expect("left digest should exist");
    let right_digest =
        batch_continuity_mutation_digest(&[Some(right)])?.expect("right digest should exist");

    assert_ne!(left_digest, right_digest);

    Ok(())
}

#[test]
fn split_successor_batch_digest_changes_with_resolved_target_identity(
) -> Result<(), IdentityDerivationDenial> {
    let left = WorthQueryContinuityMutationEvidence::from_bridge(
        &BridgeContinuityMutationBundle::split_existing_target(
            BridgeContinuityOutcomeClass::ContinuesAsSplitSuccessors,
            continuity_identity("authority:task-1"),
            [
                continuity_identity("authority:task-1:a"),
                continuity_identity("authority:task-1:b"),
            ],
            Some(resolved_target(
                &RelationalBridgeRecordIdentityParts::entity(1, 1, 0)
                    .terminal_projection_for_reporting(),
            )),
            Some(target_collection("Task")),
        )
        .expect("left split continuity should build"),
    )?;
    let right = WorthQueryContinuityMutationEvidence::from_bridge(
        &BridgeContinuityMutationBundle::split_existing_target(
            BridgeContinuityOutcomeClass::ContinuesAsSplitSuccessors,
            continuity_identity("authority:task-1"),
            [
                continuity_identity("authority:task-1:a"),
                continuity_identity("authority:task-1:b"),
            ],
            Some(resolved_target(
                &RelationalBridgeRecordIdentityParts::entity(2, 1, 0)
                    .terminal_projection_for_reporting(),
            )),
            Some(target_collection("Task")),
        )
        .expect("right split continuity should build"),
    )?;

    let left_digest =
        batch_continuity_mutation_digest(&[Some(left)])?.expect("left digest should exist");
    let right_digest =
        batch_continuity_mutation_digest(&[Some(right)])?.expect("right digest should exist");

    assert_ne!(left_digest, right_digest);

    Ok(())
}

#[test]
fn continuity_batch_digest_changes_with_family() -> Result<(), IdentityDerivationDenial> {
    let rebind = WorthQueryContinuityMutationEvidence::from_bridge(
        &BridgeContinuityMutationBundle::rebind_existing_target(
            BridgeContinuityOutcomeClass::ContinuesAsSingleSuccessor,
            continuity_identity("authority:task-1"),
            Some(continuity_identity("authority:task-1:successor")),
            Some(resolved_target("entity:task-1")),
            Some(target_collection("Task")),
        )
        .expect("rebind continuity should build"),
    )?;
    let split = rebind
        .clone()
        .with_test_family(WorthQueryContinuityMutationFamily::SplitExistingTarget);

    let rebind_digest =
        batch_continuity_mutation_digest(&[Some(rebind)])?.expect("rebind digest should exist");
    let split_digest =
        batch_continuity_mutation_digest(&[Some(split)])?.expect("split digest should exist");

    assert_ne!(rebind_digest, split_digest);

    Ok(())
}

fn continuity_identity(value: &str) -> BridgeContinuityAuthoritativeIdentity {
    BridgeContinuityAuthoritativeIdentity::new(value)
        .expect("test continuity identity should be native")
}

fn resolved_target(value: &str) -> BridgeContinuityResolvedTargetIdentity {
    BridgeContinuityResolvedTargetIdentity::new(value)
        .expect("test resolved target should be native")
}

fn target_collection(value: &str) -> BridgeContinuityTargetCollection {
    BridgeContinuityTargetCollection::new(value).expect("test target collection should be native")
}

fn bridge_existing_truth_authority(
    value: &str,
) -> Result<BridgeExistingTruthBindingAuthoritativeIdentity, IdentityDerivationDenial> {
    Ok(
        BridgeExistingTruthBindingAuthoritativeIdentity::from_bridge_evidence(
            &bridge_test_evidence("existing-truth-authority", value)?,
        ),
    )
}

fn bridge_existing_truth_target(
    parts: RelationalBridgeRecordIdentityParts,
) -> BridgeExistingTruthBindingResolvedTargetIdentity {
    BridgeExistingTruthBindingResolvedTargetIdentity::from_relational_record(parts)
}

fn bridge_existing_truth_collection(value: &str) -> BridgeExistingTruthBindingTargetCollection {
    BridgeExistingTruthBindingTargetCollection::new(value)
}

fn bridge_naming_attachment(
    value: &str,
) -> Result<BridgeNamingAttachmentIdentity, IdentityDerivationDenial> {
    Ok(BridgeNamingAttachmentIdentity::from_bridge_evidence(
        &bridge_test_evidence("naming-attachment", value)?,
    ))
}

fn bridge_test_evidence(
    role: &'static str,
    value: &str,
) -> Result<worth_runtime_bridge::facade::BridgeIdentityEvidence, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), role)
    .field_value(WorthQueryEvidenceTag::new("value"), value)
    .seal()?
    .bridge_external_identity_evidence())
}

fn bridge_naming_target(
    parts: RelationalBridgeRecordIdentityParts,
) -> BridgeNamingResolvedTargetIdentity {
    BridgeNamingResolvedTargetIdentity::from_relational_record(parts)
}

fn bridge_naming_collection(value: &str) -> BridgeNamingTargetCollection {
    BridgeNamingTargetCollection::new(value)
}

fn relational_entity(
    partition_id: u32,
    local_slot: u64,
    generation: u32,
) -> WorthQueryEntityIdentity {
    WorthQueryEntityIdentity::from_relational_record(RelationalBridgeRecordIdentityParts::entity(
        partition_id,
        local_slot,
        generation,
    ))
}

pub(super) fn retained_assertion_identity(
    label: &'static str,
) -> Result<crate::evidence_identity::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(crate::evidence_identity::worth_query_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceScope::RetainedExistingTruthAssertionEvidence,
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("role"),
        "retained-assertion-test",
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("label"),
        label,
    )
    .seal()?)
}
