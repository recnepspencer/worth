use worth_ui_host_contract::UiMountedInstanceIdentity;

pub(super) fn assert_invalid_bindings_and_retire_unbound(
    session: &mut crate::facade::entry::WorthUiActiveApplicationSession,
    conflicting_owner: UiMountedInstanceIdentity,
    bound_item: UiMountedInstanceIdentity,
    bound_option: worth_ui_query_binding::UiProjectionOptionReference,
    unbound_item: UiMountedInstanceIdentity,
    unbound_option: worth_ui_query_binding::UiProjectionOptionReference,
) {
    assert_eq!(
        session.bind_selection_item(
            super::fixture::receipt(session, conflicting_owner),
            super::fixture::receipt(session, bound_item),
            bound_option,
        ),
        Err(crate::mounting::UiMountedSelectionBindingDenial::ConflictingOwnerIncarnation)
    );
    assert_eq!(
        session.bind_selection_item(
            super::fixture::receipt(session, unbound_item),
            super::fixture::receipt(session, unbound_item),
            unbound_option,
        ),
        Err(crate::mounting::UiMountedSelectionBindingDenial::OwnerNotDeclared)
    );
    assert!(session
        .mounted
        .selection_mapping_for_item(unbound_item)
        .is_err());
    session.unmount_instance(unbound_item).unwrap();
}

pub(super) fn assert_surface_local_retirement(
    session: &mut crate::facade::entry::WorthUiActiveApplicationSession,
    owner: UiMountedInstanceIdentity,
    first: UiMountedInstanceIdentity,
    second: UiMountedInstanceIdentity,
    other_first: UiMountedInstanceIdentity,
    other_second: UiMountedInstanceIdentity,
    option: worth_ui_query_binding::UiProjectionOptionReference,
) {
    let foreign = session.bind_selection_item(
        super::fixture::receipt(session, owner),
        super::fixture::receipt(session, other_first),
        option,
    );
    assert_eq!(
        foreign,
        Err(crate::mounting::UiMountedSelectionBindingDenial::ForeignSurface)
    );
    session.unmount_instance(owner).unwrap();
    assert!(session.mounted.selection_mapping_for_item(first).is_err());
    assert!(session.mounted.selection_mapping_for_item(second).is_err());
    assert!(session
        .mounted
        .selection_mapping_for_item(other_first)
        .is_ok());
    session.unmount_instance(other_second).unwrap();
    assert!(session
        .mounted
        .selection_mapping_for_item(other_second)
        .is_err());
    assert!(session
        .mounted
        .selection_mapping_for_item(other_first)
        .is_ok());
}
