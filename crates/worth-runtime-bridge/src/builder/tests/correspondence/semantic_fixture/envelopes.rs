use super::*;

pub(in super::super) fn field_change_envelope() -> BridgeCommittedPatchEnvelope {
    field_change_envelope_with_precision(BridgeAspectChangePrecision::Exact)
}

pub(in super::super) fn field_change_envelope_with_precision(
    precision: BridgeAspectChangePrecision,
) -> BridgeCommittedPatchEnvelope {
    field_change_envelope_with_metadata(precision, BridgeProducerMetadata::bridge_harness_fixture())
}

pub(in super::super) fn whole_aspect_change_envelope(
    kind: AuthoritativeAspectChangeKind,
) -> BridgeCommittedPatchEnvelope {
    assert!(matches!(
        kind,
        AuthoritativeAspectChangeKind::WholeAspectSet
            | AuthoritativeAspectChangeKind::WholeAspectClear
    ));
    let semantic = BridgeSemanticAspectChange::from_authoritative_publication(
        aspect_key(),
        AspectIdentity(31),
        AspectContractRevision(4),
        AspectBinding::EntityField {
            field: FieldKey::new("profile").unwrap(),
        },
        kind,
        None,
    );
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            truth_commit(2),
            truth_patch(2),
            truth_snapshot(2, 2),
            truth_branch("main"),
        ),
        vec![BridgeCommittedPatchItem::with_relational_semantic_change(
            RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            BridgeCommittedPatchTarget::authoritative_aspect(AspectLocator::new(
                LocatorAuthority::Authoritative,
                aspect_key(),
            )),
            semantic,
        )],
    )
    .expect("whole-aspect fixture envelope is valid")
}

pub(in super::super) fn unidentified_whole_aspect_envelope() -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            truth_commit(3),
            truth_patch(3),
            truth_snapshot(3, 3),
            truth_branch("main"),
        ),
        vec![BridgeCommittedPatchItem::with_target(
            "copied-entity-label",
            BridgeCommittedPatchTarget::authoritative_aspect(AspectLocator::new(
                LocatorAuthority::Authoritative,
                aspect_key(),
            )),
        )],
    )
    .expect("unidentified descriptive fixture envelope is valid")
}

pub(in super::super) fn field_change_envelope_for_source_role(
    graph_role: &str,
) -> BridgeCommittedPatchEnvelope {
    field_change_envelope_for_source(99, graph_role, "relational-adapter:99")
}

pub(in super::super) fn field_change_envelope_for_source(
    runtime_instance_id: u64,
    graph_role: &str,
    adapter_identity: &str,
) -> BridgeCommittedPatchEnvelope {
    let source = BridgeAuthoritativeSourceProvenance::from_owner_publication(
        runtime_instance_id,
        graph_role,
        adapter_identity,
        "commit:1",
    );
    field_change_envelope_with_metadata(
        BridgeAspectChangePrecision::Exact,
        BridgeProducerMetadata::registered_authoritative_source().with_authoritative_source(source),
    )
}

fn field_change_envelope_with_metadata(
    precision: BridgeAspectChangePrecision,
    producer_metadata: BridgeProducerMetadata,
) -> BridgeCommittedPatchEnvelope {
    field_change_envelope_with_metadata_and_scope(precision, producer_metadata, None)
}

pub(in super::super) fn field_change_envelope_with_scope(
    scope: worth_signal::facade::ChangedRegion,
) -> BridgeCommittedPatchEnvelope {
    field_change_envelope_with_metadata_and_scope(
        BridgeAspectChangePrecision::Exact,
        BridgeProducerMetadata::bridge_harness_fixture(),
        Some(scope),
    )
}

fn field_change_envelope_with_metadata_and_scope(
    precision: BridgeAspectChangePrecision,
    producer_metadata: BridgeProducerMetadata,
    scope: Option<worth_signal::facade::ChangedRegion>,
) -> BridgeCommittedPatchEnvelope {
    let binding = AspectBinding::EntityField {
        field: FieldKey::new("profile".to_string()).unwrap(),
    };
    let semantic = match precision {
        BridgeAspectChangePrecision::Exact => {
            BridgeSemanticAspectChange::from_authoritative_publication(
                aspect_key(),
                AspectIdentity(31),
                AspectContractRevision(4),
                binding,
                AuthoritativeAspectChangeKind::FieldSet,
                Some(field_path()),
            )
        }
        BridgeAspectChangePrecision::DeclaredWidening => {
            BridgeSemanticAspectChange::from_declared_authoritative_widening(
                aspect_key(),
                AspectIdentity(31),
                AspectContractRevision(4),
                binding,
                AuthoritativeAspectChangeKind::FieldSet,
                Some(field_path()),
                BridgeAspectChangeWideningCause::FieldToWholeAspect,
            )
        }
    };
    let semantic = match scope {
        Some(scope) => semantic.with_scope_change(scope),
        None => semantic,
    };
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            producer_metadata,
            truth_commit(1),
            truth_patch(1),
            truth_snapshot(1, 1),
            truth_branch("main"),
        ),
        vec![BridgeCommittedPatchItem::with_relational_semantic_change(
            RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(LocatorAuthority::Authoritative, aspect_key()),
                field_path(),
            ),
            semantic,
        )],
    )
    .expect("valid semantic envelope")
}
