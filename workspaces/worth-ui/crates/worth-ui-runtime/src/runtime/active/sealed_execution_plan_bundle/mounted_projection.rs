impl super::WorthUiSealedExecutionPlanBundle {
    pub(crate) fn mounted_projection_plan_index(&self, provenance: u64) -> Result<Option<u32>, ()> {
        self.execution_plan
            .mounted_projection_plan_index(provenance)
    }

    pub(crate) fn mounted_projection_ordinary_meaning(
        &self,
        plan_index: u32,
    ) -> Option<
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    > {
        self.execution_plan
            .mounted_projection_ordinary_meaning(plan_index)
    }

    pub(crate) fn mounted_projection_ordinary_meaning_with_identity(
        &self,
        plan_index: u32,
    ) -> Option<(
        String,
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        self.execution_plan
            .mounted_projection_ordinary_meaning_with_identity(plan_index)
    }

    pub(crate) fn mounted_projection_ordinary_meaning_for_identity(
        &self,
        identity: &str,
    ) -> Option<(
        u32,
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        self.execution_plan
            .mounted_projection_ordinary_meaning_for_identity(identity)
    }

    pub(crate) fn mounted_projection_theme_token(
        &self,
        token_id: &crate::capability::ThemeTokenId,
    ) -> Result<
        Option<(
            u32,
            std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
        )>,
        (),
    > {
        self.execution_plan.mounted_projection_theme_token(token_id)
    }
}
