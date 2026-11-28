use super::*;

pub(in crate::harness::tests::pricing_shock) fn pricing_field_aspect_registration(
    component: &str,
) -> BridgeAspectRegistration {
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::admit_bridge_owned(format!("pricing-{component}-usd-field")),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact(format!("component:{component}")),
            worth_foundational::facade::AspectKey::new("cost").expect("valid native aspect key"),
            worth_foundational::facade::FieldKey::new("usd".to_owned())
                .expect("valid native field key"),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("cost").expect("valid native aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    )
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source(
) -> InMemoryRelationalBridgeSource {
    let scenario = generated_pricing_scenario();

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "steel",
    ));
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "rubber",
    ));
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-shock"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-shock"),
        ),
        "rubber",
    ));
    source.insert_snapshot(scenario.main_snapshot);
    source.insert_snapshot(scenario.speculative_snapshot);
    source
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_corrupted_shock_provenance(
    field: &str,
    content: impl Into<String>,
) -> InMemoryRelationalBridgeSource {
    let scenario = generated_pricing_scenario();
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "steel",
    ));
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "rubber",
    ));
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-shock"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-shock"),
        ),
        "rubber",
    ));
    source.insert_snapshot(scenario.main_snapshot);
    source.insert_snapshot(snapshot_with_corrupted_provenance_field(
        &scenario.speculative_snapshot,
        "rubber",
        field,
        content,
    ));
    source
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_conflicting_shock_snapshot(
) -> InMemoryRelationalBridgeSource {
    let scenario = generated_pricing_scenario();
    let source = pricing_reference_source();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-shock-conflicting"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "rubber",
    ));
    source.insert_snapshot(scenario.main_snapshot);
    source.insert_snapshot(scenario.speculative_snapshot);
    source
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_conflicting_route_commit_items(
    commit: &str,
    patch: &str,
    items: Vec<BridgeCommittedPatchItem>,
) -> InMemoryRelationalBridgeSource {
    let source = pricing_reference_source();
    source.insert_committed_patch(pricing_patch_items(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture(commit),
            crate::truth_identity_fixtures::truth_patch_fixture(patch),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        items,
    ));
    source
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_conflicting_commit_identity_for_route(
) -> InMemoryRelationalBridgeSource {
    pricing_reference_source_with_conflicting_route_commit_items(
        "commit:steel-main",
        "patch:steel-main-conflicting-meaning",
        vec![BridgeCommittedPatchItem::with_target(
            "component:rubber",
            crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    worth_foundational::facade::AspectKey::new("cost")
                        .expect("valid bridge patch aspect key"),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("usd".to_owned())
                        .expect("valid foundational field key"),
                ),
            ),
        )],
    )
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_branch_head_pointing_to(
    branch: &str,
    commit: &str,
) -> InMemoryRelationalBridgeSource {
    let source = pricing_reference_source();
    source.set_branch_head(
        &crate::truth_identity_fixtures::truth_branch_fixture(branch),
        &crate::truth_identity_fixtures::truth_commit_fixture(commit),
    );
    source
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_missing_branch_head_snapshot(
    branch: &str,
    commit: &str,
    snapshot: &str,
    component: &str,
) -> InMemoryRelationalBridgeSource {
    let source = pricing_reference_source();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture(branch),
            crate::truth_identity_fixtures::truth_commit_fixture(commit),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:missing-snapshot"),
            crate::truth_identity_fixtures::truth_snapshot_fixture(snapshot),
        ),
        component,
    ));
    source.set_branch_head(
        &crate::truth_identity_fixtures::truth_branch_fixture(branch),
        &crate::truth_identity_fixtures::truth_commit_fixture(commit),
    );
    source
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_missing_branch_head_commit(
    branch: &str,
    commit: &str,
) -> InMemoryRelationalBridgeSource {
    let source = pricing_reference_source();
    source.set_branch_head(
        &crate::truth_identity_fixtures::truth_branch_fixture(branch),
        &crate::truth_identity_fixtures::truth_commit_fixture(commit),
    );
    source
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_source_with_conflicting_snapshot_identity(
    snapshot: SnapshotFixture,
) -> InMemoryRelationalBridgeSource {
    let source = pricing_reference_source();
    source.insert_snapshot(snapshot);
    source
}
