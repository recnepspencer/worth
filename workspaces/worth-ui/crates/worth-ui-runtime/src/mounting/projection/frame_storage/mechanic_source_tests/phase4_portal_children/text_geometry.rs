use super::*;
use worth_ui_host_contract::{
    UiAppearanceTextDamageTransition, UiMountedAppearanceMechanicChange,
    UiMountedAppearanceWorkPosture,
};

#[test]
fn partial_clip_changes_foreground_without_color_change_or_foreign_work() {
    let mut world = GeometryWorld::new();
    let [child, foreign] = world.children;
    adopt_value(&mut world, child);
    adopt_value(&mut world, foreign);
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(-10_000, -10_000, 110_000, 70_000).unwrap()),
    );
    let initial = world.frame(&[0, 1], None);
    let (session, binding, _, vector, theme) = foreground_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let mut sidecar = UiMountedAppearanceSidecar::default();
    let first = sidecar
        .mount(lower(&context(&initial, child), &projection))
        .unwrap();
    let mut foreign_sidecar = UiMountedAppearanceSidecar::default();
    foreign_sidecar
        .mount(lower(&context(&initial, foreign), &projection))
        .unwrap();
    let original = initial.appearance_text_candidates(child).unwrap();
    assert_eq!(original.len(), 2);
    assert_eq!(original[0].clip_bounds().width(), 80.0);

    // Update only completed ancestry input. Retained qualified text is inherited
    // by the real frame completion path; no focus or theme change drives this.
    let mut candidate = initial.clone();
    let mut node = candidate.semantic.node(child).unwrap().clone();
    node.appearance_clip =
        Clip::Ancestor(UiAppearanceClip::new(-10_000, -10_000, 80_000, 60_000).unwrap());
    candidate.semantic.insert_node(node);
    let changed = world.frame(&[0, 1], Some(&candidate));
    let current = changed.appearance_text_candidates(child).unwrap();
    assert_eq!(current.len(), 2);
    for (old, new) in original.iter().zip(&current) {
        assert_eq!(new.clip_bounds().width(), 50.0);
        assert_eq!(new.clip_bounds().height(), 30.0);
        assert_eq!(
            new.qualified_layout_identity(),
            old.qualified_layout_identity()
        );
        assert_eq!(new.foregrounds(), old.foregrounds());
        assert_eq!(new.performed_layout_cost(), None);
    }
    let change = sidecar
        .mount(lower(&context(&changed, child), &projection))
        .unwrap();
    assert_eq!(change.posture(), UiMountedAppearanceWorkPosture::Delta);
    let [UiMountedAppearanceMechanicChange::Replace {
        predecessor,
        successor,
    }] = change.changes()
    else {
        panic!("partial geometry change must replace exactly the retained foreground");
    };
    assert_eq!(predecessor, &first.successor().mechanics()[0].identity());
    assert_eq!(successor.identity(), *predecessor);
    let [requirement] = change.text_damage_requirements().collect::<Vec<_>>()[..] else {
        panic!("replacement must retain its unresolved physical damage requirement");
    };
    assert_eq!(
        requirement.transition(),
        UiAppearanceTextDamageTransition::Replace
    );
    assert_eq!(requirement.target(), child);
    assert!(change.damage().is_empty());
    let foreign_work = foreign_sidecar
        .mount(lower(&context(&changed, foreign), &projection))
        .unwrap();
    assert_eq!(
        foreign_work.posture(),
        UiMountedAppearanceWorkPosture::Unchanged
    );
    assert!(foreign_work.changes().is_empty());

    let unchanged = sidecar
        .mount(lower(&context(&changed, child), &projection))
        .unwrap();
    assert_eq!(
        unchanged.posture(),
        UiMountedAppearanceWorkPosture::Unchanged
    );
    assert_eq!(unchanged.text_damage_requirements().count(), 0);
    // Reconstruction must compare the same physical basis, even with equal paint.
    let mut recovery = UiMountedAppearanceSidecar::default();
    recovery
        .mount(lower(&context(&initial, child), &projection))
        .unwrap();
    let rebuilt = recovery
        .reconstruct(lower(&context(&changed, child), &projection))
        .unwrap();
    assert_eq!(
        rebuilt.posture(),
        UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert_eq!(rebuilt.changes(), change.changes());
    assert_eq!(
        rebuilt.successor().mechanics(),
        change.successor().mechanics()
    );
}
