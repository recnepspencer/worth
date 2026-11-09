//! Forward controls to the actual journal receiver; no parallel recipe state.
use super::*;
impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
{
    #[doc(hidden)]
    pub fn clear_history_discovery_recipes_for_test(
        &mut self,
        step: WorthQueryHistoryNodeId,
    ) -> bool {
        self.owner.clear_history_discovery_recipes_for_test(step)
    }
    #[doc(hidden)]
    pub fn corrupt_history_discovery_recipe_for_test(
        &mut self,
        step: WorthQueryHistoryNodeId,
    ) -> bool {
        self.owner.corrupt_history_discovery_recipe_for_test(step)
    }
    #[doc(hidden)]
    pub fn replace_history_discovery_recipes_from_for_test<
        OtherPayload: WorthQueryHistoryPayloadMode<S, H>,
        OtherOutputs,
    >(
        &mut self,
        step: WorthQueryHistoryNodeId,
        source: &WorthQueryApplicationHistoryDocument<'_, S, H, OtherPayload, OtherOutputs>,
        source_step: WorthQueryHistoryNodeId,
    ) -> bool {
        self.owner
            .replace_history_discovery_recipes_from_for_test(step, &source.owner, source_step)
    }
}
