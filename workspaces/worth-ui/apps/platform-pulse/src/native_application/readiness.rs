use super::{
    intent, PlatformPulseApplicationRuntime, PlatformPulsePendingManagedRebind,
    PlatformPulseProjectionRebindDenial, PlatformPulseTerminalError,
};

mod physical_work_progress;

impl PlatformPulseApplicationRuntime {
    fn install_native_readiness(
        &mut self,
        readiness: Box<[worth_ui_native_platform::UiNativeApplicationReadinessPort]>,
    ) -> Result<worth_ui_native_platform::UiNativeApplicationReadinessPort, ()> {
        let readiness: [worth_ui_native_platform::UiNativeApplicationReadinessPort; 5] =
            readiness.into_vec().try_into().map_err(|_| ())?;
        let [startup, source, query, intent, visual] = readiness;
        self.source_watch.as_ref().ok_or(())?.install_readiness(
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(source),
        );
        self.query_watch.as_ref().ok_or(())?.install_readiness(
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(query),
        );
        self.intent_watch.as_ref().ok_or(())?.install_readiness(
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(intent),
        );
        self.visual_identity.install_readiness(
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(visual),
        );
        Ok(startup)
    }

    fn advance_native_product_turn(&mut self) {
        if self.pending_managed_rebind.is_some() || self.pending_frame_presentation.is_some() {
            return;
        }
        let mut shell = self.take_runtime_shell();
        self.advance_pending_intent_postures(&mut shell);
        self.shell = Some(shell);
        if self.terminal_error.is_some()
            || self.pending_managed_rebind.is_some()
            || self.pending_frame_presentation.is_some()
        {
            return;
        }
        self.poll_query();
        self.poll_intent_input();
        match self.drain_intent_product_cycle() {
            intent::PlatformPulseIntentProductCycleOutcome::Quiescent { .. }
            | intent::PlatformPulseIntentProductCycleOutcome::AwaitingExternal { .. } => {}
            intent::PlatformPulseIntentProductCycleOutcome::Interrupted { .. } => return,
        }
        self.poll_source();
        self.present();
        self.advance_visual_identity();
    }

    fn advance_after_visual_readiness(&mut self) {
        self.advance_visual_identity();
        if product_turn_admitted_after_visual_readiness(
            self.terminal_error.is_some(),
            self.pending_managed_rebind.is_some(),
            self.pending_frame_presentation.is_some(),
            self.visual_identity.retains_rebind_receipt(),
        ) {
            self.advance_native_product_turn();
        }
    }

    fn native_runtime_directive(
        &mut self,
    ) -> worth_ui_native_platform::UiNativeApplicationRuntimeDirective {
        if self.terminal_error.is_some() {
            self.report_terminal_error();
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective::Close
        } else {
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective::Continue
        }
    }

    fn take_runtime_shell(&mut self) -> worth_ui::facade::app::WorthUiNativeApplicationShell {
        self.shell
            .take()
            .expect("native application runtime retains the callback shell")
    }
}

impl worth_ui_native_platform::UiNativeApplicationRuntime for PlatformPulseApplicationRuntime {
    fn readiness_owner_count(
        &self,
    ) -> worth_ui_native_platform::UiNativeApplicationReadinessOwnerCount {
        worth_ui_native_platform::UiNativeApplicationReadinessOwnerCount::new(5)
            .expect("Pulse has startup, source, Query, intent, and visual readiness owners")
    }

    fn activate(
        &mut self,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
        readiness: Box<[worth_ui_native_platform::UiNativeApplicationReadinessPort]>,
    ) -> Result<
        worth_ui::facade::app::WorthUiNativeApplicationShell,
        worth_ui_native_platform::UiNativeApplicationRuntimeActivationStopped,
    > {
        let startup = match self.install_native_readiness(readiness) {
            Ok(startup) => startup,
            Err(()) => {
                return Err(
                    worth_ui_native_platform::UiNativeApplicationRuntimeActivationStopped::retain(
                        application,
                    ),
                );
            }
        };
        self.shell = Some(application);
        if startup.signal().is_err() {
            return Err(
                worth_ui_native_platform::UiNativeApplicationRuntimeActivationStopped::retain(
                    self.take_runtime_shell(),
                ),
            );
        }
        Ok(self.take_runtime_shell())
    }

    fn readiness_ready(
        &mut self,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
        owner_ordinal: u8,
        _generation: u64,
    ) -> Result<
        (
            worth_ui::facade::app::WorthUiNativeApplicationShell,
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective,
        ),
        worth_ui_native_platform::UiNativeApplicationRuntimeProgressStopped,
    > {
        if owner_ordinal >= 5 {
            return Err(
                worth_ui_native_platform::UiNativeApplicationRuntimeProgressStopped::retain(
                    application,
                ),
            );
        }
        self.shell = Some(application);
        if owner_ordinal == 0 {
            if self.terminal_error.is_none() {
                let copy = super::product_copy::install(
                    self.shell
                        .as_mut()
                        .expect("startup retains the activated application shell"),
                );
                if let Err(denial) = copy {
                    self.fail(
                        super::PlatformPulseTerminalError::ProductCopy(denial),
                        Ok(()),
                    );
                } else {
                    if let Some(sequence) = self
                        .initial_source
                        .as_ref()
                        .map(worth_ui::facade::source::WorthUiSourcePackageRevision::sequence)
                    {
                        let mut shell = self.take_runtime_shell();
                        let published = self.publish_source_story(&mut shell, sequence)
                            && self.refresh_product_story(&mut shell);
                        self.shell = Some(shell);
                        if !published {
                            let directive = self.native_runtime_directive();
                            return Ok((self.take_runtime_shell(), directive));
                        }
                    }
                    self.publish_initial_projection();
                    self.advance_visual_identity();
                }
            }
        } else if owner_ordinal == 4 {
            self.advance_after_visual_readiness();
        } else {
            self.advance_native_product_turn();
        }
        let directive = self.native_runtime_directive();
        Ok((self.take_runtime_shell(), directive))
    }

    fn native_observations_ready(
        &mut self,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
        progress: worth_ui_native_platform::UiNativeApplicationObservationProgress,
    ) -> Result<
        (
            worth_ui::facade::app::WorthUiNativeApplicationShell,
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective,
        ),
        worth_ui_native_platform::UiNativeApplicationRuntimeProgressStopped,
    > {
        let mut application = application;
        if let Err(denial) = self.native_input.observe_native(&progress, &self.publisher) {
            self.fail(
                super::PlatformPulseTerminalError::ObservationPublication,
                Err(denial),
            )
        }
        self.admit_worth_native_intent_input(&mut application, progress);
        self.shell = Some(application);
        self.advance_native_product_turn();
        let directive = self.native_runtime_directive();
        Ok((self.take_runtime_shell(), directive))
    }

    fn native_pointer_affordance_ready(
        &mut self,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
    ) -> Result<
        (
            worth_ui::facade::app::WorthUiNativeApplicationShell,
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective,
        ),
        worth_ui_native_platform::UiNativeApplicationRuntimeProgressStopped,
    > {
        self.shell = Some(application);
        self.present();
        self.advance_visual_identity();
        let directive = self.native_runtime_directive();
        Ok((self.take_runtime_shell(), directive))
    }

    fn native_viewport_ready(
        &mut self,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
    ) -> Result<
        (
            worth_ui::facade::app::WorthUiNativeApplicationShell,
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective,
        ),
        worth_ui_native_platform::UiNativeApplicationRuntimeProgressStopped,
    > {
        self.shell = Some(application);
        self.present();
        self.advance_visual_identity();
        let directive = self.native_runtime_directive();
        Ok((self.take_runtime_shell(), directive))
    }

    fn physical_work_progressed(
        &mut self,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
        progress: worth_ui_native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<
        (
            worth_ui::facade::app::WorthUiNativeApplicationShell,
            worth_ui_native_platform::UiNativeApplicationRuntimeDirective,
        ),
        worth_ui_native_platform::UiNativeApplicationRuntimeProgressStopped,
    > {
        self.progress_native_physical_work(application, progress)
            .map_err(|stopped| *stopped)
    }

    fn close(
        mut self: Box<Self>,
        application: worth_ui::facade::app::WorthUiNativeApplicationShell,
    ) -> Result<
        worth_ui_native_platform::UiNativeApplicationRuntimeClosed,
        worth_ui_native_platform::UiNativeApplicationRuntimeCloseIncomplete,
    > {
        self.shell = Some(application);
        let shutdown = self
            .shutdown_product()
            .expect("native Pulse close starts with the retained application shell");
        Ok(
            worth_ui_native_platform::UiNativeApplicationRuntimeClosed::from_application_shutdown(
                shutdown,
            ),
        )
    }
}

const fn product_turn_admitted_after_visual_readiness(
    terminal: bool,
    managed_rebind_pending: bool,
    frame_presentation_pending: bool,
    visual_rebind_receipt_retained: bool,
) -> bool {
    !terminal
        && !managed_rebind_pending
        && !frame_presentation_pending
        && !visual_rebind_receipt_retained
}

#[cfg(test)]
mod tests {
    #[test]
    fn visual_settlement_wakes_ordinary_product_progress_without_bypassing_blockers() {
        assert!(
            super::product_turn_admitted_after_visual_readiness(false, false, false, false),
            "visual retirement releases the receipt and wakes ordinary product progress"
        );
        assert!(!super::product_turn_admitted_after_visual_readiness(
            true, false, false, false
        ));
        assert!(!super::product_turn_admitted_after_visual_readiness(
            false, true, false, false
        ));
        assert!(!super::product_turn_admitted_after_visual_readiness(
            false, false, true, false
        ));
        assert!(
            !super::product_turn_admitted_after_visual_readiness(false, false, false, true),
            "successor capture and comparison retain the receipt and cannot wake product early"
        );
    }
}
