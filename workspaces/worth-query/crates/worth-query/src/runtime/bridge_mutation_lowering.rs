use worth_runtime_bridge::facade::{
    BridgeContinuityAuthoritativeIdentity, BridgeContinuityMutationBundle,
    BridgeContinuityOutcomeClass, BridgeContinuityResolvedTargetIdentity,
    BridgeContinuityTargetCollection, BridgeNamingAttachmentIdentity,
    BridgeNamingAuthoritativeIdentity, BridgeNamingMutationBundle,
    BridgeNamingResolvedTargetIdentity, BridgeNamingTargetCollection,
    RelationalBridgeRecordIdentityParts,
};

use crate::memory_workspace::WorthQueryEntityIdentity;

use super::{
    WorthQueryContinuityMutationFamily, WorthQueryContinuityMutationIntent,
    WorthQueryContinuityMutationOutcomeClass, WorthQueryMutationTargetCollectionIdentity,
    WorthQueryNamingMutationFamily, WorthQueryNamingMutationIntent,
};

pub(super) fn bridge_naming_mutation_bundle(
    intent: &WorthQueryNamingMutationIntent,
    resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
    target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
) -> Option<BridgeNamingMutationBundle> {
    let resolved_target_entity_identity =
        resolved_target_entity_identity.and_then(bridge_naming_resolved_target_identity);
    let target_collection =
        target_collection.map(|collection| BridgeNamingTargetCollection::new(collection.as_str()));
    let attachment_identity = bridge_naming_attachment_identity(intent.attachment_identity());
    match intent.family() {
        WorthQueryNamingMutationFamily::AttachNewTarget => {
            resolved_target_entity_identity.map(|resolved_target_entity_identity| {
                BridgeNamingMutationBundle::attach_new_target(
                    attachment_identity.clone(),
                    resolved_target_entity_identity,
                    target_collection.clone(),
                )
            })
        }
        WorthQueryNamingMutationFamily::AttachExistingTarget => resolved_target_entity_identity
            .and_then(|resolved_target_entity_identity| {
                intent
                    .target_authoritative_identity()
                    .map(|target_authoritative_identity| {
                        BridgeNamingMutationBundle::attach_existing_target(
                            attachment_identity.clone(),
                            bridge_naming_authoritative_identity(target_authoritative_identity),
                            resolved_target_entity_identity,
                            target_collection.clone(),
                        )
                    })
            }),
        WorthQueryNamingMutationFamily::RebindTarget => {
            resolved_target_entity_identity.and_then(|resolved_target_entity_identity| {
                intent
                    .prior_authoritative_identity()
                    .and_then(|prior_authoritative_identity| {
                        intent.target_authoritative_identity().map(
                            |target_authoritative_identity| {
                                BridgeNamingMutationBundle::rebind_target(
                                    attachment_identity.clone(),
                                    bridge_naming_authoritative_identity(
                                        prior_authoritative_identity,
                                    ),
                                    bridge_naming_authoritative_identity(
                                        target_authoritative_identity,
                                    ),
                                    resolved_target_entity_identity,
                                    target_collection.clone(),
                                )
                            },
                        )
                    })
            })
        }
        WorthQueryNamingMutationFamily::Remove => {
            intent
                .prior_authoritative_identity()
                .map(|prior_authoritative_identity| {
                    BridgeNamingMutationBundle::remove(
                        attachment_identity.clone(),
                        bridge_naming_authoritative_identity(prior_authoritative_identity),
                        resolved_target_entity_identity,
                        target_collection.clone(),
                    )
                })
        }
    }
}

pub(super) fn bridge_continuity_mutation_bundle(
    intent: &WorthQueryContinuityMutationIntent,
    _basis_binding_digest: Option<&str>,
    resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
    target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
) -> Option<BridgeContinuityMutationBundle> {
    let outcome_class = match intent.outcome_class() {
        WorthQueryContinuityMutationOutcomeClass::ContinuesAsSingleSuccessor => {
            BridgeContinuityOutcomeClass::ContinuesAsSingleSuccessor
        }
        WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors => {
            BridgeContinuityOutcomeClass::ContinuesAsSplitSuccessors
        }
        WorthQueryContinuityMutationOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor => {
            BridgeContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
        }
    };
    let prior_authoritative_identity =
        bridge_continuity_authoritative_identity(intent.prior_authoritative_identity());
    let resolved_target_entity_identity =
        resolved_target_entity_identity.and_then(bridge_resolved_target_identity);
    let resolved_target_entity_identity = match resolved_target_entity_identity {
        Some(identity) => Some(continuity_resolved_target_identity(identity)?),
        None => None,
    };
    let target_collection = match target_collection {
        Some(collection) => Some(continuity_target_collection(collection.as_str())?),
        None => None,
    };

    match intent.family() {
        WorthQueryContinuityMutationFamily::RebindExistingTarget => {
            BridgeContinuityMutationBundle::rebind_existing_target(
                outcome_class,
                prior_authoritative_identity,
                Some(bridge_continuity_authoritative_identity(
                    intent.successor_authoritative_identity(),
                )),
                resolved_target_entity_identity,
                target_collection,
            )
            .ok()
        }
        WorthQueryContinuityMutationFamily::SplitExistingTarget => Some(
            BridgeContinuityMutationBundle::split_existing_target(
                outcome_class,
                prior_authoritative_identity,
                intent
                    .successor_authoritative_identities()
                    .iter()
                    .map(bridge_continuity_authoritative_identity)
                    .collect::<Vec<_>>(),
                resolved_target_entity_identity,
                target_collection,
            )
            .expect("validated split continuity intent should lower into bridge bundle"),
        ),
    }
}

fn bridge_continuity_authoritative_identity(
    identity: &crate::runtime::WorthQueryMutationAuthorityIdentity,
) -> BridgeContinuityAuthoritativeIdentity {
    BridgeContinuityAuthoritativeIdentity::from_bridge_evidence(
        &identity.evidence_identity().bridge_evidence_identity(),
    )
}

fn bridge_naming_attachment_identity(
    identity: &crate::runtime::WorthQueryMutationAuthorityIdentity,
) -> BridgeNamingAttachmentIdentity {
    BridgeNamingAttachmentIdentity::from_bridge_evidence(
        &identity.evidence_identity().bridge_evidence_identity(),
    )
}

fn bridge_naming_authoritative_identity(
    identity: &crate::runtime::WorthQueryMutationAuthorityIdentity,
) -> BridgeNamingAuthoritativeIdentity {
    BridgeNamingAuthoritativeIdentity::from_bridge_evidence(
        &identity.evidence_identity().bridge_evidence_identity(),
    )
}

fn bridge_naming_resolved_target_identity(
    identity: &WorthQueryEntityIdentity,
) -> Option<BridgeNamingResolvedTargetIdentity> {
    identity
        .relational_record_parts()
        .map(BridgeNamingResolvedTargetIdentity::from_relational_record)
}

fn bridge_resolved_target_identity(
    identity: &WorthQueryEntityIdentity,
) -> Option<RelationalBridgeRecordIdentityParts> {
    identity.relational_record_parts()
}

fn continuity_resolved_target_identity(
    parts: RelationalBridgeRecordIdentityParts,
) -> Option<BridgeContinuityResolvedTargetIdentity> {
    BridgeContinuityResolvedTargetIdentity::new(parts.terminal_projection_for_reporting()).ok()
}

fn continuity_target_collection(value: &str) -> Option<BridgeContinuityTargetCollection> {
    BridgeContinuityTargetCollection::new(value).ok()
}

#[cfg(test)]
mod tests {
    use super::{bridge_continuity_mutation_bundle, bridge_naming_mutation_bundle};
    use crate::memory_workspace::WorthQueryEntityIdentity;
    use crate::runtime::{
        WorthQueryContinuityMutationEvidence, WorthQueryContinuityMutationIntent,
        WorthQueryMutationTargetCollectionIdentity, WorthQueryNamingMutationIntent,
    };
    use worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts;

    #[test]
    fn bridge_lowered_continuity_uses_bridge_native_digest_basis() -> Result<(), crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let intent = WorthQueryContinuityMutationIntent::rebind_merge_successor(
            crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(
                crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-1")
                    .expect("continuity prior authority label"),
            )
            .expect("continuity prior authority identity"),
            crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
                crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                    "authority:task-1-successor",
                )
                .expect("continuity successor authority label"),
            )
            .expect("continuity successor authority identity"),
        )
        .expect("continuity intent should build");

        let entity_identity = WorthQueryEntityIdentity::from_relational_record(
            RelationalBridgeRecordIdentityParts::entity(1, 42, 1),
        );
        let target_collection =
            WorthQueryMutationTargetCollectionIdentity::new("test-target", "Task")?;
        let lowered = bridge_continuity_mutation_bundle(
            &intent,
            None,
            Some(&entity_identity),
            Some(&target_collection),
        )
        .expect("bridge continuity bundle should lower");

        let bridge_evidence = WorthQueryContinuityMutationEvidence::from_bridge(&lowered)?;

        assert_eq!(bridge_evidence.family(), intent.family());
        assert_eq!(
            intent.prior_authoritative_identity().as_str(),
            "authority:task-1",
        );
        assert!(bridge_evidence
            .prior_authoritative_identity()
            .as_str()
            .starts_with("bridge-continuity-authoritative:"),);
        assert_eq!(
            intent.successor_authoritative_identity().as_str(),
            "authority:task-1-successor",
        );
        assert!(bridge_evidence
            .successor_authoritative_identity()
            .expect("successor authoritative identity")
            .as_str()
            .starts_with("bridge-continuity-authoritative:"),);
        assert_eq!(
            bridge_evidence.resolved_target_entity_identity(),
            Some(&entity_identity)
        );
        assert_eq!(
            bridge_evidence
                .target_collection()
                .map(|collection| collection.as_str()),
            Some("Task")
        );
        assert!(!bridge_evidence
            .lineage_digest()
            .starts_with("bridge-continuity-mutation-lineage:"));
        assert!(bridge_evidence
            .lineage_digest()
            .as_str()
            .starts_with("worth.query.evidence-identity.v1:"));
        assert!(!bridge_evidence
            .continuity_resolution_digest()
            .starts_with("bridge-continuity-mutation-resolution:"));
        assert!(bridge_evidence
            .continuity_resolution_digest()
            .as_str()
            .starts_with("worth.query.evidence-identity.v1:"));
    
Ok(())
}

    #[test]
    fn bridge_lowering_rejects_authored_query_identity_for_native_target() -> Result<(), crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let intent = WorthQueryNamingMutationIntent::attach_new_target(
            crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
                crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new("attachment:task-1")
                    .expect("naming attachment authority label"),
            )
            .expect("naming attachment identity"),
        );
        let authored_identity =
            crate::memory_workspace::admit_authored_entity_label("entity:task-1");
        let target_collection =
            WorthQueryMutationTargetCollectionIdentity::new("test-target", "Task")?;

        assert!(
            bridge_naming_mutation_bundle(
                &intent,
                Some(&authored_identity),
                Some(&target_collection)
            )
            .is_none(),
            "bridge mutation lowering must not smuggle authored Query evidence into native bridge target identity"
        );
    
Ok(())
}
}
