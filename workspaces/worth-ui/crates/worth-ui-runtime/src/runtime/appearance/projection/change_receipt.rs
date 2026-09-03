#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceChangeOutcome {
    InputEvidenceChanged,
    SemanticProjectionChanged,
    ResolvedAspectValueChanged,
    MountedMechanicalOutputChanged,
    EqualOutputSuppressed,
    DeniedBeforeEffects,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceChangeReceipt {
    input_evidence_changed: bool,
    semantic_projection_changed: bool,
    resolved_aspect_value_changed: bool,
    mounted_mechanical_output_changed: bool,
    equal_output_suppressed: bool,
    denied_before_effects: bool,
    mounting_result_available: bool,
}

impl UiAppearanceChangeReceipt {
    pub(crate) fn compare(
        predecessor: Option<&super::UiAppearanceProjection>,
        successor: Option<&super::UiAppearanceProjection>,
        mounting: &worth_ui_host_contract::UiMountedAppearanceWork,
    ) -> Self {
        let (Some(predecessor), Some(successor)) = (predecessor, successor) else {
            return Self {
                input_evidence_changed: predecessor.is_some() || successor.is_some(),
                semantic_projection_changed: predecessor.is_some() || successor.is_some(),
                resolved_aspect_value_changed: predecessor.is_some() || successor.is_some(),
                mounted_mechanical_output_changed: mounted_mechanical_output_changed(mounting),
                equal_output_suppressed: false,
                denied_before_effects: false,
                mounting_result_available: true,
            };
        };
        let input_evidence_changed = predecessor.state().basis() != successor.state().basis();
        let semantic_projection_changed =
            predecessor.semantic_digest() != successor.semantic_digest();
        let resolved_aspect_value_changed = predecessor.aspects().len()
            != successor.aspects().len()
            || predecessor.aspects().iter().any(|left| {
                successor
                    .aspects()
                    .iter()
                    .find(|right| right.aspect() == left.aspect())
                    .is_none_or(|right| right.value() != left.value())
            });
        let equal_output_suppressed = semantic_projection_changed
            && !resolved_aspect_value_changed
            && predecessor.physical_output_equivalent(successor)
            && physical_output_suppressed(mounting);
        Self {
            input_evidence_changed,
            semantic_projection_changed,
            resolved_aspect_value_changed,
            mounted_mechanical_output_changed: mounted_mechanical_output_changed(mounting),
            equal_output_suppressed,
            denied_before_effects: false,
            mounting_result_available: true,
        }
    }

    pub(crate) const fn denied() -> Self {
        Self {
            input_evidence_changed: false,
            semantic_projection_changed: false,
            resolved_aspect_value_changed: false,
            mounted_mechanical_output_changed: false,
            equal_output_suppressed: false,
            denied_before_effects: true,
            mounting_result_available: false,
        }
    }

    pub(crate) const fn input_evidence_changed(self) -> bool {
        self.input_evidence_changed
    }
    pub(crate) const fn semantic_projection_changed(self) -> bool {
        self.semantic_projection_changed
    }
    pub(crate) const fn resolved_aspect_value_changed(self) -> bool {
        self.resolved_aspect_value_changed
    }
    pub(crate) const fn mounted_mechanical_output_changed(self) -> bool {
        self.mounted_mechanical_output_changed
    }
    pub(crate) const fn equal_output_suppressed(self) -> bool {
        self.equal_output_suppressed
    }
    pub(crate) const fn denied_before_effects(self) -> bool {
        self.denied_before_effects
    }

    pub(crate) const fn mounting_result_available(self) -> bool {
        self.mounting_result_available
    }

    pub(crate) const fn outcome(self) -> Option<UiAppearanceChangeOutcome> {
        if self.denied_before_effects {
            Some(UiAppearanceChangeOutcome::DeniedBeforeEffects)
        } else if self.mounted_mechanical_output_changed {
            Some(UiAppearanceChangeOutcome::MountedMechanicalOutputChanged)
        } else if self.equal_output_suppressed {
            Some(UiAppearanceChangeOutcome::EqualOutputSuppressed)
        } else if self.resolved_aspect_value_changed {
            Some(UiAppearanceChangeOutcome::ResolvedAspectValueChanged)
        } else if self.semantic_projection_changed {
            Some(UiAppearanceChangeOutcome::SemanticProjectionChanged)
        } else if self.input_evidence_changed {
            Some(UiAppearanceChangeOutcome::InputEvidenceChanged)
        } else {
            None
        }
    }
}

fn mounted_mechanical_output_changed(
    mounting: &worth_ui_host_contract::UiMountedAppearanceWork,
) -> bool {
    !mounting.changes().is_empty() || mounting.order_changed()
}

fn physical_output_suppressed(mounting: &worth_ui_host_contract::UiMountedAppearanceWork) -> bool {
    mounting.posture() == worth_ui_host_contract::UiMountedAppearanceWorkPosture::Unchanged
}

#[cfg(test)]
#[path = "change_receipt_tests.rs"]
mod change_receipt_tests;
