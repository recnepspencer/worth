use super::PlatformPulseApplicationRuntime;

impl PlatformPulseApplicationRuntime {
    pub(super) fn progress_native_physical_work(
        &mut self,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
        progress: worth_ui_native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<
        (
            worth_ui::facade::app::WorthUiNativeApplicationShell,
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective,
        ),
        Box<worth_ui_native_platform::UiNativeApplicationRuntimeProgressStopped>,
    > {
        self.shell = Some(application);
        if self.progress_pending_frame_presentation(&progress) {
            if self.terminal_error.is_none() && self.pending_frame_presentation.is_none() {
                self.advance_native_product_turn();
            }
            let directive = self.native_runtime_directive();
            return Ok((self.take_runtime_shell(), directive));
        }
        let mut shell = self.take_runtime_shell();
        let managed = shell.progress_managed_rebind(&progress);
        if matches!(
            &self.pending_managed_rebind,
            Some(super::PlatformPulsePendingManagedRebind::ThemeSwitch(_))
        ) {
            let Some(super::PlatformPulsePendingManagedRebind::ThemeSwitch(preference)) =
                self.pending_managed_rebind.take()
            else {
                unreachable!()
            };
            match managed {
                Ok(progress) => self.settle_theme_progress(&mut shell, preference, progress),
                Err(denial) => self.fail(
                    super::PlatformPulseTerminalError::NativeManagedProgress(denial),
                    Ok(()),
                ),
            }
            self.shell = Some(shell);
            self.advance_native_product_turn();
            let directive = self.native_runtime_directive();
            return Ok((self.take_runtime_shell(), directive));
        }
        match managed {
            Ok(worth_ui::facade::app::WorthUiNativeManagedRebindProgress::Published(receipt)) => {
                let continue_retained_dismissal = match self.pending_managed_rebind.take() {
                    Some(super::PlatformPulsePendingManagedRebind::ThemeSwitch(_)) => unreachable!("theme progress is handled before other publication families"),
                    Some(super::PlatformPulsePendingManagedRebind::Projection(pending)) => {
                        self.settle_pending_projection(&mut shell, pending, receipt);
                        false
                    }
                    Some(super::PlatformPulsePendingManagedRebind::Source(source)) => {
                        self.settle_pending_source_rebind(&mut shell, source, receipt);
                        false
                    }
                    Some(super::PlatformPulsePendingManagedRebind::IntentPosture(pending)) => {
                        self.settle_intent_posture_publication(&mut shell, pending, receipt);
                        false
                    }
                    Some(super::PlatformPulsePendingManagedRebind::IntentConsequence(pending)) => {
                        let _ = pending;
                        self.fail(
                            super::PlatformPulseTerminalError::NativeManagedAttribution(
                                "intent consequence settled as an ordinary rebind",
                            ),
                            Ok(()),
                        );
                        false
                    }
                    Some(super::PlatformPulsePendingManagedRebind::PortalDismissal) => {
                        self.fail(
                            super::PlatformPulseTerminalError::NativeManagedAttribution(
                                "portal dismissal settled as an ordinary rebind",
                            ),
                            Ok(()),
                        );
                        false
                    }
                    None => {
                        self.fail(
                            super::PlatformPulseTerminalError::NativeManagedAttribution(
                                "physical publication had no product attribution",
                            ),
                            Ok(()),
                        );
                        false
                    }
                };
                if continue_retained_dismissal && self.terminal_error.is_none() {
                    self.continue_retained_portal_dismissal(&mut shell);
                }
                self.advance_pending_native_publications(&mut shell);
                self.shell = Some(shell);
                self.advance_native_product_turn();
            }
            Ok(
                worth_ui::facade::app::WorthUiNativeManagedRebindProgress::
                    IntentConsequencePublished(receipt),
            ) => {
                let continue_retained_dismissal = match self.pending_managed_rebind.take() {
                    Some(super::PlatformPulsePendingManagedRebind::IntentConsequence(pending)) => {
                        self.settle_pending_intent_consequence(&mut shell, pending, receipt);
                        true
                    }
                    _ => {
                        self.fail(
                            super::PlatformPulseTerminalError::NativeManagedAttribution(
                                "intent consequence publication had no product attribution",
                            ),
                            Ok(()),
                        );
                        false
                    }
                };
                if continue_retained_dismissal && self.terminal_error.is_none() {
                    self.continue_retained_portal_dismissal(&mut shell);
                }
                self.advance_pending_native_publications(&mut shell);
                self.shell = Some(shell);
                self.advance_native_product_turn();
            }
            Ok(worth_ui::facade::app::WorthUiNativeManagedRebindProgress::PortalDismissed(
                receipt,
            )) => {
                match self.pending_managed_rebind.take() {
                    Some(super::PlatformPulsePendingManagedRebind::ThemeSwitch(_)) => unreachable!("theme progress is handled before other publication families"),
                    Some(super::PlatformPulsePendingManagedRebind::PortalDismissal) => {
                        self.settle_portal_dismissal(&mut shell, receipt);
                    }
                    _ => self.fail(
                        super::PlatformPulseTerminalError::NativeManagedAttribution(
                            "portal dismissal had no product attribution",
                        ),
                        Ok(()),
                    ),
                }
                self.advance_pending_native_publications(&mut shell);
                self.shell = Some(shell);
                self.advance_native_product_turn();
            }
            Ok(worth_ui::facade::app::WorthUiNativeManagedRebindProgress::Unrelated) => {
                self.shell = Some(shell);
                if self.pending_managed_rebind.is_some() {
                    self.fail(
                        super::PlatformPulseTerminalError::NativeManagedAttribution(
                            "product attribution outlived the shell-managed publication",
                        ),
                        Ok(()),
                    );
                } else {
                    self.advance_visual_identity();
                }
            }
            Ok(worth_ui::facade::app::WorthUiNativeManagedRebindProgress::AwaitingProgress) => {
                self.shell = Some(shell);
                self.advance_visual_identity();
            }
            Ok(worth_ui::facade::app::WorthUiNativeManagedRebindProgress::RecoveryBlocked(_)) => {
                self.shell = Some(shell);
            }
            Ok(worth_ui::facade::app::WorthUiNativeManagedRebindProgress::RebindRecovered(_)) => {
                // Recovery reinstalls predecessor presentation; no successor receipt exists.
                let attribution = self.pending_managed_rebind.take();
                self.shell = Some(shell);
                let error = match attribution {
                    Some(super::PlatformPulsePendingManagedRebind::Source(_)
                        | super::PlatformPulsePendingManagedRebind::Projection(_)
                        | super::PlatformPulsePendingManagedRebind::IntentPosture(_)) =>
                        super::PlatformPulseTerminalError::NativeRecoveredWithoutPublication,
                    _ => super::PlatformPulseTerminalError::NativeManagedAttribution(
                        "rebind recovery did not match its product attribution",
                    ),
                };
                self.fail(error, Ok(()));
            }
            Ok(
                worth_ui::facade::app::WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(
                    recovery,
                ),
            ) => {
                let attribution = self.pending_managed_rebind.take();
                let attribution_matches = matches!(
                    (&recovery, &attribution),
                    (
                        worth_ui::facade::app::WorthUiNativePredecessorRecovery::IntentConsequence,
                        Some(super::PlatformPulsePendingManagedRebind::IntentConsequence(_)),
                    ) | (
                        worth_ui::facade::app::WorthUiNativePredecessorRecovery::PortalDismissal,
                        Some(super::PlatformPulsePendingManagedRebind::PortalDismissal),
                    )
                );
                if attribution_matches {
                    if matches!(
                        recovery,
                        worth_ui::facade::app::WorthUiNativePredecessorRecovery::IntentConsequence
                    ) {
                        self.continue_retained_portal_dismissal(&mut shell);
                    }
                    self.shell = Some(shell);
                    self.advance_native_product_turn();
                } else {
                    self.shell = Some(shell);
                    self.fail(
                        super::PlatformPulseTerminalError::NativeManagedAttribution(
                            "predecessor recovery did not match its product attribution",
                        ),
                        Ok(()),
                    );
                }
            }
            Ok(worth_ui::facade::app::WorthUiNativeManagedRebindProgress::Stopped(stop)) => {
                self.shell = Some(shell);
                match self.pending_managed_rebind.take() {
                    Some(super::PlatformPulsePendingManagedRebind::ThemeSwitch(_)) => unreachable!("theme progress is handled before other publication families"),
                    Some(super::PlatformPulsePendingManagedRebind::Source(_)) => self.fail(
                        super::PlatformPulseTerminalError::NativeManagedSourceRebind(stop),
                        Ok(()),
                    ),
                    Some(super::PlatformPulsePendingManagedRebind::Projection(_)) => self.fail(
                        super::PlatformPulseTerminalError::NativeProjection(
                            super::PlatformPulseProjectionRebindDenial::Nonpublication(stop),
                        ),
                        Ok(()),
                    ),
                    Some(super::PlatformPulsePendingManagedRebind::IntentPosture(_)) => self.fail(
                        super::PlatformPulseTerminalError::IntentPosturePublication(
                            super::intent::PlatformPulseIntentPosturePublicationDenial::Stopped(
                                stop,
                            ),
                        ),
                        self.publisher.intent_preparation_failure(),
                    ),
                    Some(super::PlatformPulsePendingManagedRebind::IntentConsequence(_)) => self
                        .fail(
                            super::PlatformPulseTerminalError::IntentExecution(format!(
                                "intent consequence managed publication stopped: {stop:?}"
                            )),
                            self.publisher.intent_preparation_failure(),
                        ),
                    Some(super::PlatformPulsePendingManagedRebind::PortalDismissal) => self.fail(
                        super::PlatformPulseTerminalError::IntentExecution(format!(
                            "portal dismissal managed publication stopped: {stop:?}"
                        )),
                        self.publisher.intent_preparation_failure(),
                    ),
                    None => self.fail(
                        super::PlatformPulseTerminalError::NativeManagedAttribution(
                            "managed stop had no product attribution",
                        ),
                        Ok(()),
                    ),
                }
            }
            Err(denial) => {
                self.shell = Some(shell);
                self.pending_managed_rebind = None;
                self.fail(
                    super::PlatformPulseTerminalError::NativeManagedProgress(denial),
                    Ok(()),
                );
            }
        }
        let directive = self.native_runtime_directive();
        Ok((self.take_runtime_shell(), directive))
    }
}
