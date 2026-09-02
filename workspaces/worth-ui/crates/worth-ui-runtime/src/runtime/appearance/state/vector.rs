use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceStateVectorDenial {
    SnapshotChanged,
    PresentationChanged,
    Adapter(UiAppearanceStateAdapterDenial),
    ForeignSession,
    ForeignGeneration,
    MissingAxis(UiAppearanceStateAxis),
    AmbiguousSelection,
    RoleBinding(UiAppearanceNodeRoleBindingDenial),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceStateVector {
    basis: super::UiAppearanceCoherentBasis,
    binding: Option<super::UiAppearanceRoleBindingBasis>,
    operability: Option<super::UiOperabilityAppearanceState>,
    focus: Option<super::UiFocusAppearanceState>,
    validation: Option<super::UiValidationAppearanceState>,
    selection: Option<super::UiSelectionAppearanceState>,
    hover: Option<super::UiHoverAppearanceState>,
    pressed: Option<super::UiPressedAppearanceState>,
}

pub(crate) trait UiAppearanceStateVectorSealInput {
    fn seal_vector(
        &self,
        snapshot: &super::UiAppearanceOwnerSnapshot,
    ) -> Result<UiAppearanceStateVector, UiAppearanceStateVectorDenial>;
}

impl UiAppearanceStateVectorSealInput for super::UiAppearanceCoherentBasis {
    fn seal_vector(
        &self,
        snapshot: &super::UiAppearanceOwnerSnapshot,
    ) -> Result<UiAppearanceStateVector, UiAppearanceStateVectorDenial> {
        UiAppearanceStateVector::seal_from_basis(snapshot, self, None)
    }
}

impl UiAppearanceStateVector {
    pub(crate) fn seal<Input: UiAppearanceStateVectorSealInput>(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        input: &Input,
    ) -> Result<Self, UiAppearanceStateVectorDenial> {
        input.seal_vector(snapshot)
    }

    fn seal_from_basis(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        basis: &super::UiAppearanceCoherentBasis,
        binding: Option<super::UiAppearanceRoleBindingBasis>,
    ) -> Result<Self, UiAppearanceStateVectorDenial> {
        if !basis.matches_snapshot(snapshot) {
            return Err(UiAppearanceStateVectorDenial::SnapshotChanged);
        }
        if !basis.presentation_matches_owner_snapshot(snapshot) {
            return Err(UiAppearanceStateVectorDenial::PresentationChanged);
        }
        let consumer = basis.consumer();
        let operability = consumer
            .consumes(UiAppearanceStateAxis::Operability)
            .then(|| super::operability::adapt(snapshot, basis))
            .transpose()
            .map_err(UiAppearanceStateVectorDenial::Adapter)?;
        let focus = consumer
            .consumes(UiAppearanceStateAxis::Focus)
            .then(|| super::focus::adapt(snapshot, basis))
            .transpose()
            .map_err(UiAppearanceStateVectorDenial::Adapter)?;
        let validation = consumer
            .consumes(UiAppearanceStateAxis::Validation)
            .then(|| super::validation::adapt(snapshot, basis))
            .transpose()
            .map_err(UiAppearanceStateVectorDenial::Adapter)?;
        let selection = consumer
            .consumes(UiAppearanceStateAxis::Selection)
            .then(|| super::selection::adapt(snapshot, basis))
            .transpose()
            .map_err(UiAppearanceStateVectorDenial::Adapter)?;
        let hover = consumer
            .consumes(UiAppearanceStateAxis::Hover)
            .then(|| super::hover::adapt(snapshot, basis))
            .transpose()
            .map_err(UiAppearanceStateVectorDenial::Adapter)?;
        let pressed = consumer
            .consumes(UiAppearanceStateAxis::Pressed)
            .then(|| super::pressed::adapt(snapshot, basis))
            .transpose()
            .map_err(UiAppearanceStateVectorDenial::Adapter)?;
        Ok(Self {
            basis: basis.clone(),
            binding,
            operability,
            focus,
            validation,
            selection,
            hover,
            pressed,
        })
    }

    #[cfg(test)]
    pub(crate) fn seal_for_binding(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        graph: &crate::graph::UiGraphSnapshot,
        capabilities: &crate::capability::CapabilitySnapshot,
        binding: &super::UiAppearanceNodeRoleBinding,
    ) -> Result<Self, UiAppearanceStateVectorDenial> {
        binding
            .validate_current(graph, capabilities)
            .map_err(UiAppearanceStateVectorDenial::RoleBinding)?;
        let consumer = super::UiAppearanceStateConsumer::from_role(
            binding.basis().graph_node(),
            binding.role(),
        );
        let basis = super::UiAppearanceCoherentBasis::for_test(
            snapshot,
            consumer,
            binding.target().mounted_instance(),
            binding.target().incarnation(),
            binding.target().node_receipt(),
            binding.target().surface(),
            presentation_for_target(snapshot, binding.target()),
            selection_for_target(snapshot, binding.target()),
            operability_route_for_target(snapshot, binding.target()),
        );
        Self::seal_from_basis(snapshot, &basis, Some(binding.basis().clone()))
    }

    pub(crate) fn basis(&self) -> &super::UiAppearanceCoherentBasis {
        &self.basis
    }

    pub(crate) const fn binding(&self) -> Option<&super::UiAppearanceRoleBindingBasis> {
        self.binding.as_ref()
    }

    pub(crate) fn operability(&self) -> Option<&super::UiOperabilityAppearanceState> {
        self.operability.as_ref()
    }

    pub(crate) fn focus(&self) -> Option<&super::UiFocusAppearanceState> {
        self.focus.as_ref()
    }

    pub(crate) fn validation(&self) -> Option<&super::UiValidationAppearanceState> {
        self.validation.as_ref()
    }

    pub(crate) fn selection(&self) -> Option<&super::UiSelectionAppearanceState> {
        self.selection.as_ref()
    }

    pub(crate) fn hover(&self) -> Option<&super::UiHoverAppearanceState> {
        self.hover.as_ref()
    }

    pub(crate) fn pressed(&self) -> Option<&super::UiPressedAppearanceState> {
        self.pressed.as_ref()
    }

    pub(crate) fn class(&self, axis: UiAppearanceStateAxis) -> Option<UiAppearanceAxisClass> {
        match axis {
            UiAppearanceStateAxis::Operability => {
                self.operability.as_ref().map(|state| state.class())
            }
            UiAppearanceStateAxis::Focus => self.focus.as_ref().map(|state| state.class()),
            UiAppearanceStateAxis::Validation => {
                self.validation.as_ref().map(|state| state.class())
            }
            UiAppearanceStateAxis::Selection => self.selection.as_ref().map(|state| state.class()),
            UiAppearanceStateAxis::Hover => self.hover.as_ref().map(|state| state.class()),
            UiAppearanceStateAxis::Pressed => self.pressed.as_ref().map(|state| state.class()),
        }
    }

    pub(crate) fn classes(
        &self,
    ) -> impl Iterator<Item = (UiAppearanceStateAxis, UiAppearanceAxisClass)> + '_ {
        [
            UiAppearanceStateAxis::Operability,
            UiAppearanceStateAxis::Focus,
            UiAppearanceStateAxis::Validation,
            UiAppearanceStateAxis::Selection,
            UiAppearanceStateAxis::Hover,
            UiAppearanceStateAxis::Pressed,
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

#[cfg(test)]
impl UiAppearanceStateVectorSealInput for super::UiAppearanceTarget {
    fn seal_vector(
        &self,
        snapshot: &super::UiAppearanceOwnerSnapshot,
    ) -> Result<UiAppearanceStateVector, UiAppearanceStateVectorDenial> {
        let consumer = super::UiAppearanceStateConsumer::for_demand_for_test(
            self.graph_node(),
            snapshot.demand(),
        );
        let basis = super::UiAppearanceCoherentBasis::for_test(
            snapshot,
            consumer,
            self.mounted_instance(),
            self.incarnation(),
            self.node_receipt(),
            self.surface(),
            presentation_for_target(snapshot, self),
            selection_for_target(snapshot, self),
            operability_route_for_target(snapshot, self),
        );
        UiAppearanceStateVector::seal_from_basis(snapshot, &basis, None)
    }
}

#[cfg(test)]
fn presentation_for_target(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    target: &super::UiAppearanceTarget,
) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
    let pointer = snapshot
        .pointer_presence()
        .and_then(|owner| owner.primary_pointer(target.surface()))?;
    snapshot
        .pointer_presence()?
        .postures()
        .iter()
        .find(|posture| posture.pointer() == pointer)
        .map(|posture| posture.presentation())
}

#[cfg(test)]
fn selection_for_target(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    target: &super::UiAppearanceTarget,
) -> Option<super::UiAppearanceSelectionSelector> {
    snapshot.selection()?.postures().iter().find_map(|posture| {
        let owner = posture.owner();
        (owner.semantic_surface() == target.surface()
            && owner.graph_node() == target.graph_node()
            && target
                .selection_key()
                .is_none_or(|key| posture.key().application_value() == key))
        .then(|| {
            super::UiAppearanceSelectionSelector::new(owner, posture.key(), posture.incarnation())
        })
    })
}

#[cfg(test)]
fn operability_route_for_target(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    target: &super::UiAppearanceTarget,
) -> Option<Box<str>> {
    snapshot.operability()?.facts().iter().find_map(|fact| {
        (fact.graph_node() == target.graph_node()
            && fact.mounted_instance() == target.mounted_instance()
            && fact.node_receipt() == target.node_receipt())
        .then(|| fact.route().into())
    })
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

use super::{UiAppearanceNodeRoleBindingDenial, UiAppearanceStateAdapterDenial};
