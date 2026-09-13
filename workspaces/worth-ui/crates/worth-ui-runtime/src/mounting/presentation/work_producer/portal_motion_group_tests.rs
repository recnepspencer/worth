use super::*;

#[test]
fn appearance_children_join_only_their_portal_and_leave_on_retirement() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let other_owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let card = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let icon = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let other_card = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let affinity = UiMountedPortalPresentationAffinity::from_runtime_mounting(owner, 801);
    let other_affinity =
        UiMountedPortalPresentationAffinity::from_runtime_mounting(other_owner, 802);
    let mut groups = UiMountedPortalMotionGroups::default();
    for instance in [card, icon] {
        groups.replace_instance(instance, None, Some(affinity), surface, true);
    }
    groups.replace_instance(other_card, None, Some(other_affinity), surface, true);
    let commands = |groups: &UiMountedPortalMotionGroups| {
        groups
            .group(target(surface, owner, 801))
            .unwrap()
            .commands()
            .collect::<std::collections::HashSet<_>>()
    };
    let expected = [card, icon]
        .map(UiMountedPaintCommandIdentity::appearance_surface)
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(commands(&groups), expected);
    let predecessor = groups.clone();
    groups.replace_instance(card, None, Some(affinity), surface, true);
    assert_eq!(
        commands(&groups),
        expected,
        "rebind cannot duplicate group membership"
    );
    groups.replace_instance(icon, None, None, surface, false);
    assert_eq!(
        commands(&groups),
        [UiMountedPaintCommandIdentity::appearance_surface(card)]
            .into_iter()
            .collect()
    );
    assert_eq!(
        commands(&predecessor),
        expected,
        "prepared membership preserves its predecessor"
    );
    groups.replace_instance(card, None, None, surface, false);
    assert!(groups.group(target(surface, owner, 801)).is_none());
    assert_eq!(
        groups
            .group(target(surface, other_owner, 802))
            .unwrap()
            .commands()
            .collect::<Vec<_>>(),
        vec![UiMountedPaintCommandIdentity::appearance_surface(
            other_card
        )]
    );
}
