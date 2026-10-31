//! Distinguish aspect and field absence within the selected field projection.
use super::*;
use worth_foundational::facade::AspectKey;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RelationalProjectedFieldObservation<'value> {
    AspectAbsent,
    FieldAbsent,
    Present(&'value AspectValue),
    UnsupportedAspectShape,
}
impl<'value> RelationalProjectedFieldObservation<'value> {
    pub const fn value(self) -> Option<&'value AspectValue> {
        match self {
            Self::Present(value) => Some(value),
            _ => None,
        }
    }
}
impl<'value> EntityProjectionRecord<'value> {
    /// `None` means the field is outside this projection. Aspect presence does
    /// not authorize any other field or expose the whole aspect payload.
    /// Scalar values require the selected kind's exact installed EntityField
    /// binding; the aspect name and an arbitrary selected field are insufficient.
    /// Scalar interpretation visits that kind's installed binding roster and
    /// borrows the value without allocating or acquiring another reader.
    pub fn observe_aspect_field(
        self,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> Option<RelationalProjectedFieldObservation<'value>> {
        use RelationalProjectedFieldObservation as Observation;
        if !self.projection_scope.contains_field(aspect, field) {
            return None;
        }
        let Some(value) = self
            .authoritative_aspect_state()
            .and_then(|state| state.get(aspect))
        else {
            return Some(Observation::AspectAbsent);
        };
        Some(match value.view() {
            ContractValidatedAspectValueView::Scalar(scalar) => {
                let installed = self.aspect_plan.is_some_and(|plan| {
                    plan.executable_bindings.iter().any(|binding| {
                        binding.aspect_key() == aspect
                            && binding.targets_entity_scalar_field(field)
                            && &binding.contract == value.contract()
                    })
                });
                if installed {
                    Observation::Present(scalar)
                } else {
                    Observation::UnsupportedAspectShape
                }
            }
            ContractValidatedAspectValueView::Struct(value) => value
                .get(field)
                .map_or(Observation::FieldAbsent, Observation::Present),
        })
    }
}
