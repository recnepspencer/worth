use super::UiMountedSemanticProjection;

impl UiMountedSemanticProjection {
    pub(in crate::mounting::projection) fn apply_projection_inputs(
        &mut self,
        content: &super::super::super::super::UiMountedSemanticContentInput,
    ) {
        use crate::mounting::semantic_content::UiMountedProjectionInputTransition as Transition;
        let replaces_table = content
            .projection_input_transition()
            .replaces_table(self.projection_input_capacity);

        let (capacity, inputs) = match content.projection_input_transition() {
            Transition::Retain => return,
            Transition::Merge { capacity, inputs } => {
                if replaces_table {
                    self.projection_input_capacity = *capacity;
                    self.projection_inputs = Default::default();
                }
                (*capacity, inputs)
            }
            Transition::Replace { capacity, inputs } => {
                self.projection_input_capacity = *capacity;
                self.projection_inputs = Default::default();
                (*capacity, inputs)
            }
        };
        for (slot, transition) in inputs {
            debug_assert!(slot.index() < capacity);
            let predecessor = self.projection_inputs.get(slot.index()).cloned();
            let input = transition.apply(predecessor.as_ref());
            self.projection_inputs.insert(slot.index(), input.clone());
        }
    }

    pub(in crate::mounting::projection) fn inherit_projection_inputs(
        &mut self,
        predecessor: Option<&Self>,
    ) {
        let Some(predecessor) = predecessor else {
            return;
        };
        self.projection_input_capacity = predecessor.projection_input_capacity;
        self.projection_inputs = predecessor.projection_inputs.clone();
    }

    pub(in crate::mounting) fn projection_input(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<&worth_ui_query_binding::UiProjectionInputFactReference> {
        self.projection_inputs.get(slot.index())
    }

    pub(in crate::mounting) fn projection_input_capacity(&self) -> usize {
        self.projection_input_capacity
    }
}
