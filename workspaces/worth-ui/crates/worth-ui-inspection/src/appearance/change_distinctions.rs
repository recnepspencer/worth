/// Independent facts that can be observed on one appearance attempt.
///
/// These flags are evidence only. They do not form a priority or authorize a
/// subsequent runtime operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceInspectionChangeDistinctions {
    input_evidence_changed: bool,
    semantic_projection_changed: bool,
    resolved_aspect_value_changed: bool,
    mounted_mechanical_output_changed: bool,
    equal_output_suppressed: bool,
    denied_before_effects: bool,
}

impl UiAppearanceInspectionChangeDistinctions {
    pub const fn new(
        input_evidence_changed: bool,
        semantic_projection_changed: bool,
        resolved_aspect_value_changed: bool,
        mounted_mechanical_output_changed: bool,
        equal_output_suppressed: bool,
        denied_before_effects: bool,
    ) -> Self {
        Self {
            input_evidence_changed,
            semantic_projection_changed,
            resolved_aspect_value_changed,
            mounted_mechanical_output_changed,
            equal_output_suppressed,
            denied_before_effects,
        }
    }

    pub const fn input_evidence_changed(self) -> bool {
        self.input_evidence_changed
    }

    pub const fn semantic_projection_changed(self) -> bool {
        self.semantic_projection_changed
    }

    pub const fn resolved_aspect_value_changed(self) -> bool {
        self.resolved_aspect_value_changed
    }

    pub const fn mounted_mechanical_output_changed(self) -> bool {
        self.mounted_mechanical_output_changed
    }

    pub const fn equal_output_suppressed(self) -> bool {
        self.equal_output_suppressed
    }

    pub const fn denied_before_effects(self) -> bool {
        self.denied_before_effects
    }
}
