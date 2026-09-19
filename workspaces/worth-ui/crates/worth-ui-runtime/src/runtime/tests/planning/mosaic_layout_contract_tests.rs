use crate::capability::*;
use crate::runtime::planning::execution_plan_input::{
    WorthUiLayoutPlanMeaning as Layout, WorthUiPlanOrdinaryMeaning as Meaning,
};
use crate::runtime::planning::plan_topology::WorthUiPlanRegionStore;
use worth_ui_dsl::{WorthUiArtifactInputBodyAtom as Atom, WorthUiRustAuthoredArtifactInputModule};

#[test]
fn sealed_mosaic_occurrences_retain_admitted_sizing_and_placement() {
    let store = store(240, MosaicPlacementConflictBehavior::reject_conflict());
    let mut regions = Vec::new();
    let mut mounts = Vec::new();
    for identity in store.canonical_identities() {
        let Some(meaning) = store
            .executable_for(&identity)
            .unwrap()
            .ordinary_meaning_reference()
        else {
            continue;
        };
        match meaning.as_ref() {
            Meaning::Layout(Layout::Region {
                descriptor,
                sizing_contract,
                ..
            }) if descriptor.id().as_str() == "region.leaf" => {
                let sizing = sizing_contract
                    .as_ref()
                    .expect("each leaf keeps its explicit contract");
                regions.push((identity.exact_basis().to_owned(), sizing.clone()));
            }
            Meaning::Layout(Layout::Surface {
                descriptor,
                placement_policy,
                ..
            }) => {
                assert_eq!(descriptor.id().as_str(), "surface.content");
                mounts.push((identity.exact_basis().to_owned(), placement_policy.clone()));
            }
            Meaning::Layout(Layout::Region {
                sizing_contract, ..
            }) => assert!(sizing_contract.is_none()),
            _ => {}
        }
    }
    regions.sort_by(|left, right| left.0.cmp(&right.0));
    mounts.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(regions.len(), 2);
    assert_eq!(mounts.len(), 2);
    assert!(regions[0].0.ends_with("::region::region.leaf#0"));
    assert!(regions[1].0.ends_with("::region::region.leaf#1"));
    assert_eq!(regions[0].1, sizing("sizing.first", 240));
    assert_eq!(regions[1].1, sizing("sizing.second", 480));
    assert!(mounts[0].0.contains("::region::region.leaf#0::mount::"));
    assert!(mounts[1].0.contains("::region::region.leaf#1::mount::"));
    assert_eq!(
        mounts[0].1,
        Some(placement(MosaicPlacementConflictBehavior::reject_conflict()))
    );
    assert!(
        mounts[1].1.is_none(),
        "absence must not acquire a default placement"
    );
}

#[test]
fn same_identifier_contract_changes_reach_exact_executable_meaning_and_digest() {
    let previous = store(240, MosaicPlacementConflictBehavior::reject_conflict());
    for changed in [
        store(320, MosaicPlacementConflictBehavior::reject_conflict()),
        store(240, MosaicPlacementConflictBehavior::replace_existing()),
    ] {
        assert!(!previous.semantically_matches(&changed).0);
        assert_ne!(previous.semantic_digest(), changed.semantic_digest());
        let changed_layouts = previous
            .canonical_identities()
            .into_iter()
            .filter(|identity| {
                let before = previous.executable_for(identity).unwrap();
                let after = changed.executable_for(identity).unwrap();
                if before.ordinary_meaning_reference() != after.ordinary_meaning_reference() {
                    assert_ne!(
                        before.ordinary_semantic_digest(),
                        after.ordinary_semantic_digest()
                    );
                    true
                } else {
                    false
                }
            })
            .count();
        assert_eq!(
            changed_layouts, 1,
            "only the edited occurrence's contract changes"
        );
    }
}

fn store(first_width: u16, conflict: MosaicPlacementConflictBehavior) -> WorthUiPlanRegionStore {
    store_with_descriptors(
        first_width,
        conflict,
        region("region.leaf", MosaicChildRule::accepts_surfaces()),
        surface(SurfaceStateClass::restorable()),
    )
}

#[test]
fn same_id_region_and_surface_changes_invalidate_each_executed_occurrence() {
    let leaf = region("region.leaf", MosaicChildRule::accepts_surfaces());
    let previous = store(240, MosaicPlacementConflictBehavior::reject_conflict());
    for (region, surface) in [
        (
            leaf.clone()
                .with_clipping(MosaicClippingPosture::allow_overlay_escape()),
            surface(SurfaceStateClass::restorable()),
        ),
        (
            leaf.clone()
                .with_scroll_ownership(MosaicScrollOwnership::no_scrolling()),
            surface(SurfaceStateClass::restorable()),
        ),
        (leaf, surface(SurfaceStateClass::persistent())),
    ] {
        let changed = store_with_descriptors(
            240,
            MosaicPlacementConflictBehavior::reject_conflict(),
            region,
            surface,
        );
        let mut changed_occurrences = Vec::new();
        for identity in previous.canonical_identities() {
            let before = previous.executable_for(&identity).unwrap();
            let after = changed.executable_for(&identity).unwrap();
            if before.ordinary_meaning_reference() != after.ordinary_meaning_reference() {
                assert_ne!(
                    before.ordinary_semantic_digest(),
                    after.ordinary_semantic_digest(),
                    "changed descriptor at {} must invalidate executable geometry meaning",
                    identity.exact_basis()
                );
                changed_occurrences.push(identity);
            }
        }
        assert_eq!(
            changed_occurrences.len(),
            2,
            "both copies use the same changed descriptor"
        );
        assert_ne!(previous.semantic_digest(), changed.semantic_digest());
        assert!(!previous.semantically_matches(&changed).0);
    }
}

fn store_with_descriptors(
    first_width: u16,
    conflict: MosaicPlacementConflictBehavior,
    leaf: MosaicRegionKindDescriptor,
    surface: SurfaceDescriptor,
) -> WorthUiPlanRegionStore {
    let app = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            super::source_ingress_boundary_test_support::source_backed_package_component(
                "component.root",
            ),
        )
        .register_component(
            super::source_ingress_boundary_test_support::source_backed_package_component(
                "component.content",
            ),
        )
        .register_surface(surface)
        .register_mosaic_region_kind(region("region.outer", MosaicChildRule::accepts_regions()))
        .register_mosaic_region_kind(leaf)
        .register_mosaic_sizing_contract(sizing("sizing.first", first_width))
        .register_mosaic_sizing_contract(sizing("sizing.second", 480))
        .register_mosaic_placement_policy(placement(conflict))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .unwrap();
    let ident = |value: &str| Atom::Identifier(value.to_owned());
    let mut body = vec![ident("region"), ident("region.outer"), Atom::LeftBrace];
    for (sizing, placed) in [("sizing.first", true), ("sizing.second", false)] {
        body.extend([
            ident("region"),
            ident("region.leaf"),
            Atom::LeftBrace,
            ident("sizing"),
            ident(sizing),
            Atom::Semicolon,
            ident("mount"),
            ident("surface.content"),
        ]);
        if placed {
            body.extend([ident("placement"), ident("placement.content")]);
        }
        body.extend([Atom::Semicolon, Atom::RightBrace]);
    }
    body.push(Atom::RightBrace);
    let artifact = super::replacement_impact_test_support::artifact_from_modules(
        &app,
        [
            WorthUiRustAuthoredArtifactInputModule::new("mosaic/contracts.wui")
                .with_component_body_atoms("component.root", body),
        ],
    );
    let runtime = super::replacement_impact_test_support::launch_runtime(&app, artifact);
    runtime
        .active
        .active_plan()
        .exact_plan()
        .region_store()
        .clone()
}

fn region(id: &str, children: MosaicChildRule) -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new(id).unwrap(),
        MosaicRegionRole::primary(),
    )
    .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
    .with_scroll_ownership(MosaicScrollOwnership::region_owned())
    .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
    .with_child_rule(children)
    .with_allowed_surface_class(SurfacePlacementClass::primary_region())
    .with_persistence(MosaicRegionPersistence::restorable())
    .with_clipping(MosaicClippingPosture::clip_to_region())
    .with_hit_test(MosaicHitTestPosture::participates())
}

fn surface(state: SurfaceStateClass) -> SurfaceDescriptor {
    SurfaceDescriptor::new(
        SurfaceId::new("surface.content").unwrap(),
        SurfaceKind::primary_content(),
        ComponentId::new("component.content").unwrap(),
        SurfacePlacementClass::primary_region(),
        state,
    )
}

fn sizing(id: &str, width: u16) -> MosaicSizingContractDescriptor {
    MosaicSizingContractDescriptor::new(
        MosaicSizingContractId::new(id).unwrap(),
        MosaicSizingKind::fill(),
    )
    .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
    .with_resize_permission(MosaicResizePermission::user_resizable())
    .with_persistence(MosaicSizingPersistence::restorable())
    .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
    .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
    .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
    .with_named_measurement(NamedMeasurementDefinition::new(
        NamedMeasurementToken::new(format!("{id}.width")).unwrap(),
        MeasurementValue::logical_pixels(width.into()),
        MeasurementConstraint::between(
            MeasurementValue::logical_pixels(100),
            MeasurementValue::logical_pixels(600),
        ),
    ))
}

fn placement(conflict: MosaicPlacementConflictBehavior) -> MosaicPlacementPolicyDescriptor {
    MosaicPlacementPolicyDescriptor::new(
        MosaicPlacementPolicyId::new("placement.content").unwrap(),
        MosaicPlacementAction::dock(),
    )
    .with_source(MosaicPlacementSource::surface_class(
        SurfacePlacementClass::primary_region(),
    ))
    .with_target(MosaicPlacementTarget::region_role(
        MosaicRegionRole::primary(),
    ))
    .with_persistence(MosaicPlacementPersistence::restorable())
    .with_stable_identity_behavior(MosaicStableIdentityBehavior::preserve_surface_identity())
    .with_conflict_behavior(conflict)
    .with_reload_reconciliation(MosaicPlacementReloadReconciliation::restore_when_possible())
}
