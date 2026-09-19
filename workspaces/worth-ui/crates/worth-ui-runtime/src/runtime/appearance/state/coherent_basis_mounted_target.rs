use super::{UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput};

pub(super) fn validate_prepared(
    frame: &crate::mounting::UiAssembledMountedFrame,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    input: &UiAppearanceCoherentBasisInput,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    if frame
        .presented_receipt_basis()
        .receipt_for(input.mounted_instance)
        != Some(input.receipt_basis.successor_node_receipt())
        || mounted
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
    prepared: &crate::mounting::UiAssembledMountedFrame,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    use worth_ui_dsl::UiAppearanceStateAxis;
    if consumer.consumes(UiAppearanceStateAxis::Selection) && input.selection.is_none() {
        return Err(UiAppearanceCoherentBasisDenial::SelectionBindingUnavailable);
    }
    if let Some(selector) = input.selection {
        let mapping = mounted
            .selection_mapping_for_prepared_item(input.mounted_instance, prepared)
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
