use std::collections::HashSet;

use super::WorthUiNativeApplicationShell;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeComponentPresenceDenial {
    DuplicateComponent,
    UnknownComponent,
    AlreadyInRequestedState,
    RuntimeTransition,
    RollbackIndeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeComponentPresenceProgress {
    Applied,
    AwaitingPortalDismissal,
}

pub(super) enum UiNativePendingComponentPresence {
    AwaitingPortalDismissal(Box<[crate::facade::entry::UiNativeComponentPresenceChange]>),
    Applied(Box<[crate::facade::entry::UiNativeComponentPresenceChange]>),
}

impl WorthUiNativeApplicationShell {
    pub(crate) fn apply_component_presence(
        &mut self,
        changes: &[crate::facade::entry::UiNativeComponentPresenceChange],
    ) -> Result<UiNativeComponentPresenceProgress, UiNativeComponentPresenceDenial> {
        if let Some(pending) = self.pending_component_presence.as_ref() {
            let retained = match pending {
                UiNativePendingComponentPresence::AwaitingPortalDismissal(retained)
                | UiNativePendingComponentPresence::Applied(retained) => retained,
            };
            if retained.as_ref() != changes {
                return Err(UiNativeComponentPresenceDenial::RuntimeTransition);
            }
            let applied = matches!(pending, UiNativePendingComponentPresence::Applied(_));
            if applied {
                self.pending_component_presence = None;
                return Ok(UiNativeComponentPresenceProgress::Applied);
            }
            return Ok(UiNativeComponentPresenceProgress::AwaitingPortalDismissal);
        }
        let indices = self.validate_component_presence(changes)?;
        if self.next_anchor_loss_portal(&indices, changes).is_some() {
            self.pending_component_presence =
                Some(UiNativePendingComponentPresence::AwaitingPortalDismissal(
                    changes.to_vec().into_boxed_slice(),
                ));
            return self.resume_pending_component_presence(self.managed_rebind_completion_tick);
        }
        self.apply_validated_component_presence(&indices, changes)?;
        Ok(UiNativeComponentPresenceProgress::Applied)
    }

    pub(crate) const fn component_presence_awaits_portal_dismissal(&self) -> bool {
        matches!(
            self.pending_component_presence.as_ref(),
            Some(UiNativePendingComponentPresence::AwaitingPortalDismissal(_))
        )
    }

    pub(crate) fn resume_pending_component_presence(
        &mut self,
        now_tick: u64,
    ) -> Result<UiNativeComponentPresenceProgress, UiNativeComponentPresenceDenial> {
        let changes = match self.pending_component_presence.take() {
            Some(UiNativePendingComponentPresence::AwaitingPortalDismissal(changes)) => changes,
            Some(applied @ UiNativePendingComponentPresence::Applied(_)) => {
                self.pending_component_presence = Some(applied);
                return Ok(UiNativeComponentPresenceProgress::Applied);
            }
            None => return Ok(UiNativeComponentPresenceProgress::Applied),
        };
        loop {
            let indices = self.validate_component_presence(&changes)?;
            let Some(portal) = self.next_anchor_loss_portal(&indices, &changes) else {
                self.apply_validated_component_presence(&indices, &changes)?;
                self.pending_component_presence =
                    Some(UiNativePendingComponentPresence::Applied(changes));
                return Ok(UiNativeComponentPresenceProgress::Applied);
            };
            match self.begin_managed_anchor_loss_dismissal(portal, now_tick) {
                super::super::native_managed_rebind::WorthUiNativeManagedPortalDismissalOutcome::Published(_) => {}
                super::super::native_managed_rebind::WorthUiNativeManagedPortalDismissalOutcome::Pending => {
                    self.pending_component_presence = Some(
                        UiNativePendingComponentPresence::AwaitingPortalDismissal(changes),
                    );
                    return Ok(UiNativeComponentPresenceProgress::AwaitingPortalDismissal);
                }
                super::super::native_managed_rebind::WorthUiNativeManagedPortalDismissalOutcome::Ignored
                | super::super::native_managed_rebind::WorthUiNativeManagedPortalDismissalOutcome::Retained
                | super::super::native_managed_rebind::WorthUiNativeManagedPortalDismissalOutcome::Stopped(_) => {
                    self.pending_component_presence = None;
                    return Err(UiNativeComponentPresenceDenial::RuntimeTransition);
                }
            }
        }
    }

    fn next_anchor_loss_portal(
        &self,
        indices: &[usize],
        changes: &[crate::facade::entry::UiNativeComponentPresenceChange],
    ) -> Option<crate::runtime::portal::UiPortalIdentity> {
        let portal = self.session.portal.as_ref()?;
        indices
            .iter()
            .copied()
            .zip(changes)
            .filter(|(_, change)| !change.present())
            .map(|(index, _)| {
                let row = &self.mounted_rows[index];
                crate::runtime::portal::UiPortalIdentity::for_owner(
                    crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(
                        self.current_native_graph_node(row)
                            .expect("validated removal retains its graph node"),
                        self.current_native_mounted_instance(row)
                            .expect("validated removal retains mounted identity"),
                    ),
                )
            })
            .find(|identity| portal.anchor_requires_dismissal(*identity))
    }

    fn apply_validated_component_presence(
        &mut self,
        indices: &[usize],
        changes: &[crate::facade::entry::UiNativeComponentPresenceChange],
    ) -> Result<(), UiNativeComponentPresenceDenial> {
        let mut applied: Vec<(usize, bool)> = Vec::with_capacity(indices.len());
        for (index, change) in indices.iter().copied().zip(changes) {
            if self
                .set_component_presence(index, change.present())
                .is_err()
            {
                for (rollback_index, rollback_change) in applied.into_iter().rev() {
                    if self
                        .set_component_presence(rollback_index, !rollback_change)
                        .is_err()
                    {
                        return Err(UiNativeComponentPresenceDenial::RollbackIndeterminate);
                    }
                }
                return Err(UiNativeComponentPresenceDenial::RuntimeTransition);
            }
            applied.push((index, change.present()));
        }
        Ok(())
    }

    fn validate_component_presence(
        &self,
        changes: &[crate::facade::entry::UiNativeComponentPresenceChange],
    ) -> Result<Vec<usize>, UiNativeComponentPresenceDenial> {
        let mut seen = HashSet::with_capacity(changes.len());
        changes
            .iter()
            .map(|change| {
                if !seen.insert(change.authored_semantic_identity()) {
                    return Err(UiNativeComponentPresenceDenial::DuplicateComponent);
                }
                let index = self
                    .mounted_row_indices
                    .get(change.authored_semantic_identity())
                    .copied()
                    .ok_or(UiNativeComponentPresenceDenial::UnknownComponent)?;
                if self
                    .current_native_mounted_instance(&self.mounted_rows[index])
                    .is_some()
                    == change.present()
                {
                    return Err(UiNativeComponentPresenceDenial::AlreadyInRequestedState);
                }
                Ok(index)
            })
            .collect()
    }

    fn set_component_presence(&mut self, index: usize, present: bool) -> Result<(), ()> {
        let row = self.mounted_rows.get(index).ok_or(())?;
        let graph_node = self.current_native_graph_node(row).ok_or(())?;
        let current = self.current_native_mounted_instance(row);
        if present {
            let handle = self
                .session
                .mounted_graph_node(graph_node)
                .map_err(|_| ())?;
            let mounted = self
                .session
                .mount_instance(handle, self.surface)
                .map_err(|_| ())?;
            self.mounted_rows[index].latest_mounted = mounted;
        } else {
            let mounted = current.ok_or(())?;
            self.session.unmount_instance(mounted).map_err(|_| ())?;
        }
        Ok(())
    }
}
