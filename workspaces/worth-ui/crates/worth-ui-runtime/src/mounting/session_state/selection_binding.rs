use super::WorthUiMountedSessionState;
use crate::runtime::selection::{UiSelectionAppearanceChange, UiSelectionProjectionMapping};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedSelectionBindingDenial {
    TargetNotCurrent,
    ForeignSurface,
    OwnerNotDeclared,
    CollectionUnavailable,
    OptionNotCurrent,
    ApplicationKeyUnavailable,
    ItemAlreadyBound,
    ConflictingOwnerIncarnation,
    BindingUnavailable,
    SelectionUnavailable,
    SelectionRejected,
}

impl WorthUiMountedSessionState {
    pub(crate) fn admit_selection_item_binding(
        &self,
        owner: UiMountedNodeReceiptIdentity,
        item: UiMountedNodeReceiptIdentity,
        option: &worth_ui_query_binding::UiProjectionOptionReference,
    ) -> Result<UiSelectionProjectionMapping, UiMountedSelectionBindingDenial> {
        self.validate_current_receipt(owner.mounted_instance(), owner)
            .and_then(|_| self.validate_current_receipt(item.mounted_instance(), item))
            .map_err(|_| UiMountedSelectionBindingDenial::TargetNotCurrent)?;
        let owner_basis = self
            .current_mounted_identity_basis(owner.mounted_instance())
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)?;
        let item_basis = self
            .current_mounted_identity_basis(item.mounted_instance())
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)?;
        if owner_basis.semantic_surface_identity() != item_basis.semantic_surface_identity() {
            return Err(UiMountedSelectionBindingDenial::ForeignSurface);
        }
        let Some(worth_ui_query_binding::UiProjectionInputFactReference::Collection(collection)) =
            self.current_projection_input(option.owner_revision().slot())
        else {
            return Err(UiMountedSelectionBindingDenial::CollectionUnavailable);
        };
        let mapping = UiSelectionProjectionMapping::from_current_option(&owner_basis, &collection, option)
            .map_err(|denial| match denial {
                crate::runtime::selection::UiSelectionProjectionMappingDenial::ApplicationKeyUnavailable => UiMountedSelectionBindingDenial::ApplicationKeyUnavailable,
                _ => UiMountedSelectionBindingDenial::OptionNotCurrent,
            })?;
        if self.selection_bindings.owner_conflicts(mapping) {
            return Err(UiMountedSelectionBindingDenial::ConflictingOwnerIncarnation);
        }
        if self
            .selection_bindings
            .get(item.mounted_instance())
            .is_some_and(|binding| {
                binding.owner_instance != owner.mounted_instance() || binding.mapping != mapping
            })
        {
            return Err(UiMountedSelectionBindingDenial::ItemAlreadyBound);
        }
        Ok(mapping)
    }

    pub(crate) fn install_selection_item_binding(
        &mut self,
        owner: UiMountedNodeReceiptIdentity,
        item: UiMountedNodeReceiptIdentity,
        option: worth_ui_query_binding::UiProjectionOptionReference,
        mapping: UiSelectionProjectionMapping,
    ) {
        let item_incarnation = self
            .current_mounted_identity_basis(item.mounted_instance())
            .expect("binding admission holds the current item")
            .mount_incarnation();
        self.selection_bindings.insert(
            item.mounted_instance(),
            crate::mounting::selection_binding::UiMountedSelectionItem {
                owner_instance: owner.mounted_instance(),
                item_incarnation,
                option,
                mapping,
            },
        );
    }

    pub(crate) fn selection_mapping_for_item(
        &self,
        item: UiMountedInstanceIdentity,
    ) -> Result<UiSelectionProjectionMapping, UiMountedSelectionBindingDenial> {
        let binding = self
            .selection_bindings
            .get(item)
            .ok_or(UiMountedSelectionBindingDenial::BindingUnavailable)?;
        let item_basis = self
            .current_mounted_identity_basis(item)
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)?;
        if item_basis.mount_incarnation() != binding.item_incarnation {
            return Err(UiMountedSelectionBindingDenial::TargetNotCurrent);
        }
        let owner = self
            .current_mounted_identity_basis(binding.owner_instance)
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)?;
        let Some(worth_ui_query_binding::UiProjectionInputFactReference::Collection(collection)) =
            self.current_projection_input(binding.option.owner_revision().slot())
        else {
            return Err(UiMountedSelectionBindingDenial::CollectionUnavailable);
        };
        let current =
            UiSelectionProjectionMapping::from_current_option(&owner, &collection, &binding.option)
                .map_err(|_| UiMountedSelectionBindingDenial::OptionNotCurrent)?;
        (current == binding.mapping)
            .then_some(current)
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)
    }

    pub(crate) fn selection_item_matches_option(
        &self,
        item: UiMountedInstanceIdentity,
        option: &worth_ui_query_binding::UiProjectionOptionReference,
    ) -> bool {
        self.selection_mapping_for_item(item).is_ok()
            && self
                .selection_bindings
                .get(item)
                .is_some_and(|binding| &binding.option == option)
    }

    pub(crate) fn selection_mapping_for_prepared_item(
        &self,
        item: UiMountedInstanceIdentity,
        frame: &crate::mounting::UiPreparedMountedFrame,
    ) -> Result<UiSelectionProjectionMapping, UiMountedSelectionBindingDenial> {
        let binding = self
            .selection_bindings
            .get(item)
            .ok_or(UiMountedSelectionBindingDenial::BindingUnavailable)?;
        let item_basis = self
            .current_mounted_identity_basis(item)
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)?;
        let owner = self
            .current_mounted_identity_basis(binding.owner_instance)
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)?;
        if item_basis.mount_incarnation() != binding.item_incarnation
            || item_basis.semantic_surface_identity() != owner.semantic_surface_identity()
        {
            return Err(UiMountedSelectionBindingDenial::TargetNotCurrent);
        }
        let Some(worth_ui_query_binding::UiProjectionInputFactReference::Collection(collection)) =
            frame.projection_input(binding.option.owner_revision().slot())
        else {
            return Err(UiMountedSelectionBindingDenial::CollectionUnavailable);
        };
        let option = collection
            .current_option(&binding.option.row_reference())
            .filter(|option| same_collection_item(option, &binding.option))
            .ok_or(UiMountedSelectionBindingDenial::OptionNotCurrent)?;
        let mapping =
            UiSelectionProjectionMapping::from_current_option(&owner, collection, &option)
                .map_err(|_| UiMountedSelectionBindingDenial::OptionNotCurrent)?;
        (mapping == binding.mapping)
            .then_some(mapping)
            .ok_or(UiMountedSelectionBindingDenial::TargetNotCurrent)
    }

    pub(crate) fn selection_changed_instances(
        &self,
        changes: &[UiSelectionAppearanceChange],
    ) -> Vec<UiMountedInstanceIdentity> {
        self.selection_bindings.affected(changes)
    }

    pub(crate) fn selection_projection_changed_instances(
        &self,
        content: &crate::mounting::UiMountedSemanticContentInput,
        predecessor: Option<&crate::mounting::UiPreparedMountedFrame>,
    ) -> Vec<UiMountedInstanceIdentity> {
        use crate::mounting::semantic_content::UiMountedProjectionInputTransition as Transition;
        let transition = content.projection_input_transition();
        let current_capacity = self.identity.current_projection().map_or(0, |frame| {
            frame.semantic_projection().projection_input_capacity()
        });
        // Full replacement may inherit an explicit candidate; delta assembly
        // may use the current publication. A reset on either path revokes all.
        let candidate_capacity = predecessor.map_or(current_capacity, |frame| {
            frame.semantic_projection().projection_input_capacity()
        });
        if transition.replaces_table(current_capacity)
            || transition.replaces_table(candidate_capacity)
        {
            return self.selection_bindings.all_items();
        }
        match transition {
            Transition::Retain => Vec::new(),
            Transition::Merge { inputs, .. } => inputs
                .keys()
                .flat_map(|slot| self.selection_bindings.for_slot(*slot))
                .collect(),
            Transition::Replace { .. } => unreachable!("replacement handled above"),
        }
    }

    pub(crate) fn take_selection_binding_changes(&mut self) -> Vec<UiMountedInstanceIdentity> {
        self.selection_bindings.take_changed()
    }

    pub(crate) fn refresh_selection_bindings(
        &mut self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) {
        let current = self.current_projection_input(slot);
        if current.as_ref().is_some_and(|input| {
            input.posture() == worth_ui_query_binding::UiProjectionInputPosture::Current
        }) && self.selection_bindings.slot_revision(slot)
            == current.as_ref().map(|input| input.revision())
        {
            return;
        }
        for item in self.selection_bindings.for_slot(slot) {
            let binding = self.selection_bindings.get(item).expect("indexed binding");
            let successor = match current.as_ref() {
                Some(worth_ui_query_binding::UiProjectionInputFactReference::Collection(
                    collection,
                )) => collection.current_option(&binding.option.row_reference()),
                _ => None,
            };
            match successor {
                Some(option) if same_collection_item(&option, &binding.option) => {
                    if option != binding.option {
                        let mut successor = binding.clone();
                        successor.option = option;
                        self.selection_bindings.insert(item, successor);
                    }
                }
                _ => self.selection_bindings.remove(item),
            }
        }
    }
}

fn same_collection_item(
    candidate: &worth_ui_query_binding::UiProjectionOptionReference,
    bound: &worth_ui_query_binding::UiProjectionOptionReference,
) -> bool {
    let candidate_revision = candidate.owner_revision();
    let bound_revision = bound.owner_revision();
    candidate.application_item_key() == bound.application_item_key()
        && candidate_revision.slot() == bound_revision.slot()
        && candidate_revision.projection_identity() == bound_revision.projection_identity()
        && candidate_revision.query_world_identity() == bound_revision.query_world_identity()
        && candidate_revision.binding_identity() == bound_revision.binding_identity()
}
