use worth_ui_dsl::UiAppearanceStateAxis;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceStateVectorDenial {
    SnapshotChanged,
    Adapter(UiAppearanceStateAdapterDenial),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceStateVector {
    basis: super::UiAppearanceCoherentBasis,
    operability: Option<super::UiOperabilityAppearanceState>,
    focus: Option<super::UiFocusAppearanceState>,
    validation: Option<super::UiValidationAppearanceState>,
    selection: Option<super::UiSelectionAppearanceState>,
    hover: Option<super::UiHoverAppearanceState>,
    pressed: Option<super::UiPressedAppearanceState>,
}

impl UiAppearanceStateVector {
    pub(crate) fn seal(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        basis: &super::UiAppearanceCoherentBasis,
    ) -> Result<Self, UiAppearanceStateVectorDenial> {
        if !basis.matches_snapshot(snapshot) {
            return Err(UiAppearanceStateVectorDenial::SnapshotChanged);
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
            operability,
            focus,
            validation,
            selection,
            hover,
            pressed,
        })
    }

    pub(crate) fn basis(&self) -> &super::UiAppearanceCoherentBasis {
        &self.basis
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
}

use super::UiAppearanceStateAdapterDenial;
