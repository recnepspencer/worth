use worth_ui_dsl::{UiAppearanceAxisClass as Class, UiAppearanceStateAxis as Axis};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceStateVector {
    basis: super::UiAppearanceCoherentBasis,
    binding: Option<super::UiAppearanceRoleBindingBasis>,
    classes: [Option<Class>; 6],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceStateVectorDenial {
    ForeignSession,
    ForeignGeneration,
    MissingAxis(Axis),
    AmbiguousSelection,
    RoleBinding(super::UiAppearanceNodeRoleBindingDenial),
}

impl UiAppearanceStateVector {
    pub(crate) fn seal(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        target: &super::UiAppearanceTarget,
    ) -> Result<Self, UiAppearanceStateVectorDenial> {
        Self::seal_with_demand(snapshot, target, snapshot.demand(), None)
    }

    pub(crate) fn seal_for_binding(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        graph: &crate::graph::UiGraphSnapshot,
        capabilities: &crate::capability::CapabilitySnapshot,
        binding: &super::UiAppearanceNodeRoleBinding,
    ) -> Result<Self, UiAppearanceStateVectorDenial> {
        binding
            .validate_current(graph, capabilities)
            .map_err(UiAppearanceStateVectorDenial::RoleBinding)?;
        let mut demand = super::UiAppearanceStateAxisDemand::default();
        for (_, partition) in binding.role().partitions() {
            for axis in partition.axes() {
                demand.include(axis.axis());
            }
        }
        Self::seal_with_demand(
            snapshot,
            binding.target(),
            demand,
            Some(binding.basis().clone()),
        )
    }

    fn seal_with_demand(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        target: &super::UiAppearanceTarget,
        demand: super::UiAppearanceStateAxisDemand,
        binding: Option<super::UiAppearanceRoleBindingBasis>,
    ) -> Result<Self, UiAppearanceStateVectorDenial> {
        if snapshot.session() != target.session() {
            return Err(UiAppearanceStateVectorDenial::ForeignSession);
        }
        if snapshot.generation().session_identity() != target.session() {
            return Err(UiAppearanceStateVectorDenial::ForeignGeneration);
        }
        let mut classes = [None; 6];
        let mut revisions = [0; 6];
        for axis in [
            Axis::Operability,
            Axis::Focus,
            Axis::Validation,
            Axis::Selection,
            Axis::Hover,
            Axis::Pressed,
        ] {
            if !demand.contains(axis) {
                continue;
            }
            if !snapshot.demand().contains(axis) {
                return Err(UiAppearanceStateVectorDenial::MissingAxis(axis));
            }
            let index = axis_index(axis);
            revisions[index] = owner_revision(snapshot, axis);
            classes[index] = Some(match axis {
                Axis::Operability => operability(snapshot, target)
                    .ok_or(UiAppearanceStateVectorDenial::MissingAxis(axis))?,
                Axis::Focus => focus(snapshot, target),
                Axis::Validation => validation(snapshot, target),
                Axis::Selection => selection(snapshot, target)?,
                Axis::Hover => hover(snapshot, target),
                Axis::Pressed => pressed(snapshot, target),
            });
        }
        Ok(Self {
            basis: super::UiAppearanceCoherentBasis::seal(snapshot, target, revisions),
            binding,
            classes,
        })
    }

    pub(crate) const fn basis(&self) -> &super::UiAppearanceCoherentBasis {
        &self.basis
    }

    pub(crate) const fn binding(&self) -> Option<&super::UiAppearanceRoleBindingBasis> {
        self.binding.as_ref()
    }

    pub(crate) const fn class(&self, axis: Axis) -> Option<Class> {
        self.classes[axis_index(axis)]
    }

    pub(crate) fn classes(&self) -> impl Iterator<Item = (Axis, Class)> + '_ {
        [
            Axis::Operability,
            Axis::Focus,
            Axis::Validation,
            Axis::Selection,
            Axis::Hover,
            Axis::Pressed,
        ]
        .into_iter()
        .filter_map(|axis| self.class(axis).map(|class| (axis, class)))
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let digest = self.binding.as_ref().map_or_else(
            || fold(self.basis.semantic_digest(), 0),
            |binding| {
                fold(
                    fold(self.basis.semantic_digest(), 1),
                    binding.semantic_digest(),
                )
            },
        );
        self.classes().fold(digest, |digest, (axis, class)| {
            fold(fold(digest, axis as u64 + 1), class as u64 + 1)
        })
    }

    pub(crate) fn evidence_digest(&self) -> u64 {
        fold(self.semantic_digest(), self.basis.evidence_digest())
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

fn owner_revision(snapshot: &super::UiAppearanceOwnerSnapshot, axis: Axis) -> u64 {
    match axis {
        Axis::Operability => snapshot
            .operability()
            .map_or(0, |value| value.owner_revision()),
        Axis::Focus => snapshot.focus().map_or(0, |value| value.owner_revision()),
        Axis::Validation => snapshot
            .validation()
            .map_or(0, |value| value.owner_revision()),
        Axis::Selection => snapshot
            .selection()
            .map_or(0, |value| value.owner_revision()),
        Axis::Hover => snapshot
            .pointer_presence()
            .map_or(0, |value| value.owner_revision()),
        Axis::Pressed => snapshot.pressed().map_or(0, |value| value.owner_revision()),
    }
}

fn operability(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    target: &super::UiAppearanceTarget,
) -> Option<Class> {
    snapshot.operability()?.facts().iter().find_map(|fact| {
        (fact.graph_node() == target.graph_node()
            && fact.mounted_instance() == target.mounted_instance()
            && fact.node_receipt() == target.node_receipt())
        .then(|| match fact.class() {
            crate::runtime::intent::UiIntentOperabilityAppearanceClass::Ready => {
                Class::OperabilityReady
            }
            crate::runtime::intent::UiIntentOperabilityAppearanceClass::Pending => {
                Class::OperabilityPending
            }
            crate::runtime::intent::UiIntentOperabilityAppearanceClass::Occupied => {
                Class::OperabilityOccupied
            }
            crate::runtime::intent::UiIntentOperabilityAppearanceClass::Denied => {
                Class::OperabilityDenied
            }
            crate::runtime::intent::UiIntentOperabilityAppearanceClass::Unsupported => {
                Class::OperabilityUnsupported
            }
            crate::runtime::intent::UiIntentOperabilityAppearanceClass::Stale => {
                Class::OperabilityStale
            }
        })
    })
}

fn focus(snapshot: &super::UiAppearanceOwnerSnapshot, target: &super::UiAppearanceTarget) -> Class {
    let posture = snapshot.focus().expect("demanded focus owner is sealed");
    let matches = posture.target().is_some_and(|focus| {
        focus.graph_node() == target.graph_node()
            && focus.mounted_instance() == target.mounted_instance()
            && focus.incarnation() == target.incarnation()
    });
    if !matches {
        return Class::FocusUnfocused;
    }
    match posture.class() {
        crate::runtime::focus::UiFocusAppearanceClass::Unfocused => Class::FocusUnfocused,
        crate::runtime::focus::UiFocusAppearanceClass::Focused => Class::FocusFocused,
        crate::runtime::focus::UiFocusAppearanceClass::FocusVisible => Class::FocusVisible,
        crate::runtime::focus::UiFocusAppearanceClass::FocusedWindowInactive => {
            Class::FocusedWindowInactive
        }
    }
}

fn validation(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    target: &super::UiAppearanceTarget,
) -> Class {
    snapshot
        .validation()
        .and_then(|facts| {
            facts.class_for(
                target.graph_node(),
                target.mounted_instance(),
                target.node_receipt(),
            )
        })
        .map_or(Class::ValidationUnspecified, |class| match class {
            crate::runtime::intent::UiValidationAppearanceClass::Valid => Class::ValidationValid,
            crate::runtime::intent::UiValidationAppearanceClass::Advisory => {
                Class::ValidationAdvisory
            }
            crate::runtime::intent::UiValidationAppearanceClass::Invalid => {
                Class::ValidationInvalid
            }
            crate::runtime::intent::UiValidationAppearanceClass::Pending => {
                Class::ValidationPending
            }
            crate::runtime::intent::UiValidationAppearanceClass::Stale => Class::ValidationStale,
        })
}

fn selection(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    target: &super::UiAppearanceTarget,
) -> Result<Class, UiAppearanceStateVectorDenial> {
    let Some(selection) = snapshot.selection() else {
        return Ok(Class::SelectionUnselected);
    };
    let candidates = selection
        .postures()
        .iter()
        .copied()
        .filter(|posture| {
            let owner = posture.owner();
            owner.semantic_surface() == target.surface()
                && owner.graph_node() == target.graph_node()
                && posture.incarnation()
                    == crate::runtime::selection::UiSelectionOwnerIncarnation::from_mount_incarnation(
                        target.incarnation(),
                    )
                && target
                    .selection_key()
                    .is_none_or(|key| posture.key().application_value() == key)
        })
        .collect::<Vec<_>>();
    let posture = match candidates.as_slice() {
        [] => return Ok(Class::SelectionUnselected),
        [posture] => *posture,
        _ => return Err(UiAppearanceStateVectorDenial::AmbiguousSelection),
    };
    Ok(match posture.class() {
        crate::runtime::selection::UiSelectionAppearanceClass::Unselected => {
            Class::SelectionUnselected
        }
        crate::runtime::selection::UiSelectionAppearanceClass::Selected => Class::SelectionSelected,
        crate::runtime::selection::UiSelectionAppearanceClass::Anchor => Class::SelectionAnchor,
        crate::runtime::selection::UiSelectionAppearanceClass::Cursor => Class::SelectionCursor,
        crate::runtime::selection::UiSelectionAppearanceClass::SelectedAnchorCursor => {
            Class::SelectedAnchorCursor
        }
    })
}

fn hover(snapshot: &super::UiAppearanceOwnerSnapshot, target: &super::UiAppearanceTarget) -> Class {
    let Some(pointer) = snapshot
        .pointer_presence()
        .and_then(|owner| owner.primary_pointer(target.surface()))
    else {
        return Class::HoverOutside;
    };
    snapshot
        .pointer_presence()
        .expect("primary pointer came from the same sealed owner")
        .postures()
        .iter()
        .find(|posture| posture.pointer() == pointer)
        .and_then(|posture| {
            (posture.target() == Some(target.mounted_instance())
                && posture.node_receipt() == Some(target.node_receipt()))
            .then_some(posture.class())
        })
        .map_or(Class::HoverOutside, |class| match class {
            crate::runtime::interaction::UiPointerPresenceClass::Outside => Class::HoverOutside,
            crate::runtime::interaction::UiPointerPresenceClass::Hovered => Class::Hovered,
        })
}

fn pressed(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    target: &super::UiAppearanceTarget,
) -> Class {
    snapshot
        .pressed()
        .expect("demanded pressed owner is sealed")
        .postures()
        .iter()
        .find(|posture| {
            posture.target() == target.mounted_instance()
                && posture.node_receipt() == target.node_receipt()
        })
        .map_or(Class::PressedIdle, |posture| match posture.class() {
            crate::runtime::interaction::gesture::UiPressedAppearanceClass::ArmedInside => {
                Class::PressedArmedInside
            }
            crate::runtime::interaction::gesture::UiPressedAppearanceClass::CapturedOutside => {
                Class::PressedCapturedOutside
            }
        })
}

const fn axis_index(axis: Axis) -> usize {
    match axis {
        Axis::Operability => 0,
        Axis::Focus => 1,
        Axis::Validation => 2,
        Axis::Selection => 3,
        Axis::Hover => 4,
        Axis::Pressed => 5,
    }
}
