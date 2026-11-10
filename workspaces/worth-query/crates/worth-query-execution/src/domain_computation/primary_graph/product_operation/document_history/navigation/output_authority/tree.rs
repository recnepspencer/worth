//! A forest lends each original tree the same admitted C and candidate D.
use super::*;

impl<'phase, 'scope> WorthQueryHistoryOutputAuthority<'phase, 'scope> {
    /// The child borrows the parent's revocable callback and selection lifetimes.
    /// Its profile is taken only from the registered immutable forest.
    pub fn with_tree_authority<R>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        index: usize,
        callback: impl FnOnce(&WorthQueryHistoryOutputAuthority<'_, 'scope>) -> R,
    ) -> Result<R, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        if !std::ptr::eq(phase, self.phase) || !self.profile.is_output_forest() {
            return Err(Denial::IntentDrift);
        }
        let profile = self
            .profile
            .tree_profiles()
            .get(index)
            .ok_or(Denial::IntentDrift)?;
        self.selection
            .while_live(|| ())
            .map_err(|_| Denial::HeadChanged)?;
        self.callback
            .while_live(|| ())
            .map_err(|_| Denial::HeadChanged)?;
        if let Some(lapse) = self.request.lapse() {
            return Err(Denial::Authorization(
                WorthQueryOperationAuthorizationDenial::new(
                    lapse.denial_kind(),
                    "history output tree request",
                ),
            ));
        }
        let authority = WorthQueryHistoryOutputAuthority {
            phase: self.phase,
            runtime: self.runtime,
            binding: self.binding.clone(),
            profile: profile.clone(),
            current: self.current.retained_clone(),
            restored: self.restored.clone(),
            request: self.request.clone(),
            principal: self.principal,
            selection: self.selection.clone(),
            callback: self.callback.clone(),
        };
        Ok(callback(&authority))
    }
}
