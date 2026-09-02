impl super::UiPointerGestureRuntimeState {
    pub(crate) fn retest_committed_presentation(
        &mut self,
        trigger: &crate::runtime::interaction::pointer_presence::UiPointerPresencePresentationTrigger,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> usize {
        if !self.appearance_enabled
            || self.active.is_empty()
            || mounted
                .validate_current_frame(trigger.presentation().frame())
                .is_err()
            || mounted
                .validate_binding(trigger.presentation().binding())
                .is_err()
        {
            return 0;
        }
        let pointers = self.active.keys().copied().collect::<Vec<_>>();
        let mut changed = 0;
        for pointer in pointers {
            let Some(active) = self.active.get(&pointer) else {
                continue;
            };
            if !trigger.affects_position(
                active.position,
                Some(active.target.mounted_instance()),
            ) {
                continue;
            }
            let target = (
                active.target.surface(),
                active.target.binding(),
                active.target.mounted_instance(),
                active.target.node_receipt(),
            );
            let inside = match crate::runtime::interaction::targeting::resolve_presented_target(
                mounted,
                trigger.presentation(),
                active.position,
            ) {
                Ok(current) => (
                    current.surface(),
                    current.binding(),
                    current.mounted_instance(),
                    current.node_receipt(),
                ) == target,
                Err(crate::runtime::interaction::targeting::UiInteractionTargetingDenial::NoTarget { .. }) => false,
                Err(_) => continue,
            };
            let active = self
                .active
                .get_mut(&pointer)
                .expect("active gesture remains present during bounded retest");
            if active.inside != inside {
                active.inside = inside;
                changed += 1;
                self.bump_appearance_revision();
            }
        }
        changed
    }
}
