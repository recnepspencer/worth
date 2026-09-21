impl super::UiScrollRuntimeState {
    pub(crate) fn owner_anchor(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
    ) -> Result<
        Option<crate::runtime::scroll::UiScrollAnchor>,
        crate::runtime::scroll::UiScrollRouteDenial,
    > {
        Ok(self.exact_owner(owner, incarnation)?.anchor)
    }
}
