use super::{UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput};

pub(super) fn validate_current(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    input: &UiAppearanceCoherentBasisInput,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    validate_prepared(mounted, input)?;
    input
        .receipt_basis
        .owner_node_receipt()
        .is_some()
        .then_some(())
        .ok_or(UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent)
}

pub(super) fn validate_prepared(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    input: &UiAppearanceCoherentBasisInput,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    if mounted
        .current_mounted_identity_basis(input.mounted_instance)
        .as_ref()
        != Some(&input.mounted_identity)
        || input.receipt_basis.mounted_instance() != input.mounted_instance
        || input.receipt_basis.incarnation() != input.mounted_identity.mount_incarnation()
        || mounted
            .validate_appearance_receipt_basis(input.receipt_basis)
            .is_err()
    {
        return Err(UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent);
    }
    Ok(())
}

pub(super) fn validate_selection(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    consumer: &super::super::UiAppearanceStateConsumer,
    input: &UiAppearanceCoherentBasisInput,
    prepared: Option<&crate::mounting::UiPreparedMountedFrame>,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    use worth_ui_dsl::UiAppearanceStateAxis;
    if consumer.consumes(UiAppearanceStateAxis::Selection) && input.selection.is_none() {
        return Err(UiAppearanceCoherentBasisDenial::SelectionBindingUnavailable);
    }
    if let Some(selector) = input.selection {
        let mapping = match prepared {
            Some(frame) => {
                mounted.selection_mapping_for_prepared_item(input.mounted_instance, frame)
            }
            None => mounted.selection_mapping_for_item(input.mounted_instance),
        }
        .map_err(|_| UiAppearanceCoherentBasisDenial::SelectionBindingUnavailable)?;
        if selector.owner() != mapping.owner
            || selector.key() != mapping.key
            || selector.incarnation() != mapping.incarnation
        {
            return Err(UiAppearanceCoherentBasisDenial::SelectionBindingUnavailable);
        }
    }
    Ok(())
}
