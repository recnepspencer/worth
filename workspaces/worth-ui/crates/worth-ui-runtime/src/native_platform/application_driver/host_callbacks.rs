use super::*;

impl UiNativeEventLoopClient for UiNativeApplicationDriver {
    fn install_observation_clock(
        &mut self,
        clock: worth_ui_host_native::UiNativeObservationClock,
    ) -> Result<(), UiNativeEventLoopClientFailure> {
        if self.observation_clock.is_some() {
            return Err(UiNativeEventLoopClientFailure::Rejected);
        }
        self.observation_clock = Some(clock);
        Ok(())
    }

    fn observation_time_ready(
        &mut self,
    ) -> Result<worth_ui_host_native::UiNativeObservationTimeProgress, UiNativeEventLoopClientFailure>
    {
        self.progress_observation_time()
            .map_err(|()| UiNativeEventLoopClientFailure::Rejected)
    }

    fn application_readiness_owner_count(
        &self,
    ) -> worth_ui_host_native::UiNativeApplicationReadinessOwnerCount {
        self.application_readiness_owner_count()
    }

    fn install_application_readiness(
        &mut self,
        ports: Vec<worth_ui_host_native::UiNativeApplicationReadinessPort>,
    ) -> Result<(), UiNativeEventLoopClientFailure> {
        self.install_application_readiness(ports.into_boxed_slice())
            .map_err(|()| UiNativeEventLoopClientFailure::Rejected)
    }

    fn application_readiness_ready(
        &mut self,
        grant: worth_ui_host_native::UiNativeApplicationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        self.progress_application_runtime(grant)
            .map_err(|()| UiNativeEventLoopClientFailure::Rejected)
    }

    fn native_surface_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        (|| -> Result<UiNativeEventLoopDirective, ()> {
            if grant.generation() != 0 || self.shell.is_some() {
                return Err(());
            }
            let application = self.application.take().ok_or(())?;
            self.shell = match application.launch_native_surface_at_scale_for(
                grant.scale_factor_milli(),
                self.native_surface_declaration.as_deref(),
            ) {
                Ok(shell) => Some(shell),
                Err(
                    crate::facade::WorthUiNativeApplicationShellLaunchDenial::RuntimeLaunchCleanup(
                        cleanup,
                    ),
                ) => {
                    self.pending_cleanup =
                        Some(UiNativeApplicationDriverCleanup::RuntimeLaunch(cleanup));
                    return Err(());
                }
                Err(
                    crate::facade::WorthUiNativeApplicationShellLaunchDenial::ApplicationCleanup(
                        cleanup,
                    ),
                ) => {
                    self.pending_cleanup = Some(UiNativeApplicationDriverCleanup::Application {
                        cleanup,
                        evidence: Box::new(UiNativeDriverShutdownEvidence::empty()),
                    });
                    return Err(());
                }
                Err(denial) => {
                    let _ = denial;
                    self.consumed_application_cleanup_complete = true;
                    return Err(());
                }
            };
            self.shell
                .as_mut()
                .ok_or(())?
                .install_native_observation_clock(self.observation_clock.clone().ok_or(())?)?;
            self.shell
                .as_mut()
                .ok_or(())?
                .observe_native_viewport_readiness(
                    grant.client_physical_size(),
                    grant.scale_factor_milli(),
                    false,
                );
            self.scale_factor_milli = Some(grant.scale_factor_milli());
            self.activate_application_runtime()?;
            Ok(UiNativeEventLoopDirective::Continue)
        })()
        .map_err(|()| UiNativeEventLoopClientFailure::Rejected)
    }

    fn redraw_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        (|| -> Result<UiNativeEventLoopDirective, ()> {
            if grant.generation() <= self.last_ready_generation {
                return Err(());
            }
            let shell = self.shell.as_mut().ok_or(())?;
            if self.scale_factor_milli != Some(grant.scale_factor_milli()) {
                if shell
                    .rebind_native_surface_scale(grant.scale_factor_milli())
                    .is_err()
                {
                    return Err(());
                }
                self.scale_factor_milli = Some(grant.scale_factor_milli());
            }
            shell.observe_native_viewport_readiness(
                grant.client_physical_size(),
                grant.scale_factor_milli(),
                true,
            );
            self.progress
                .observe_readiness(grant.generation(), grant.surface_basis_generation());
            if self.progress.advance(shell).is_err() {
                eprintln!("native-driver-diagnostic: redraw program progress denied");
                return Err(());
            }
            self.last_ready_generation = grant.generation();
            if self.application_runtime_active
                && self.shell.as_ref().is_some_and(
                    WorthUiNativeApplicationShell::native_viewport_presentation_pending,
                )
            {
                return self.progress_application_runtime_viewport();
            }
            Ok(self.next_directive())
        })()
        .map_err(|()| UiNativeEventLoopClientFailure::Rejected)
    }

    fn physical_work_progressed(
        &mut self,
        grant: worth_ui_host_native::UiNativePhysicalProgressGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        (|| -> Result<UiNativeEventLoopDirective, ()> {
            if self.progress_motion_physical(&grant)? {
                return Ok(UiNativeEventLoopDirective::Continue);
            }
            if self.application_runtime_active {
                return self
                    .progress_application_runtime_physical(grant)
                    .map_err(|()| {
                        eprintln!("native-driver-diagnostic: application physical progress denied");
                    });
            }
            let shell = self.shell.as_mut().ok_or(())?;
            self.progress.physical_work_progressed(shell, grant)?;
            Ok(self.next_directive())
        })()
        .map_err(|()| UiNativeEventLoopClientFailure::Rejected)
    }

    fn native_observations_ready(
        &mut self,
        grant: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        (|| -> Result<UiNativeEventLoopDirective, ()> {
            if grant.generation() <= self.last_observation_ready_generation {
                return Err(());
            }
            let shell = self.shell.as_mut().ok_or(())?;
            if !shell.native_observation_admission_ready() {
                return Ok(self.next_directive());
            }
            let settlement = shell.admit_native_observation_batches(grant.reachability());
            let (applied, duplicate, quarantined, denied) = settlement.counts();
            for (total, observed) in self.observation_ingress_counts[..4].iter_mut().zip([
                applied,
                duplicate,
                quarantined,
                denied,
            ]) {
                *total = total.saturating_add(observed as u64);
            }
            if settlement.drain_denial().is_some() {
                eprintln!("native-driver-diagnostic: observation drain denied");
                self.observation_ingress_counts[4] =
                    self.observation_ingress_counts[4].saturating_add(1);
                return Err(());
            }
            self.last_observation_ready_generation = grant.generation();
            let directive = self
                .progress_application_runtime_observations(settlement)
                .map_err(|()| {
                    eprintln!("native-driver-diagnostic: application observation progress denied");
                })?;
            if matches!(directive, UiNativeEventLoopDirective::Close) {
                Ok(directive)
            } else {
                Ok(self.next_directive())
            }
        })()
        .map_err(|()| UiNativeEventLoopClientFailure::Rejected)
    }

    fn external_close_requested(
        &mut self,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        self.progress.request_external_close();
        Ok(self.next_directive())
    }

    fn presentation_attribution(
        &self,
    ) -> Option<worth_ui_host_native::UiNativeClientPresentationAttribution> {
        self.shell
            .as_ref()
            .and_then(WorthUiNativeApplicationShell::current_presentation_attribution)
    }

    fn close(mut self) -> UiNativeEventLoopClientClose {
        let runtime_derived_state_reconstruction = self
            .application_runtime_shell()
            .and_then(WorthUiNativeApplicationShell::runtime_derived_state_reconstruction);
        let application_runtime_shutdown = match self.close_application_runtime() {
            Ok(shutdown) => shutdown,
            Err(()) => {
                return UiNativeEventLoopClientClose::Incomplete(Box::new(self));
            }
        };
        if self.pending_application_runtime_close.is_some() {
            return UiNativeEventLoopClientClose::Incomplete(Box::new(self));
        }
        if let Some(cleanup) = self.pending_cleanup.take() {
            match cleanup.retry() {
                Ok(completion) => return completion.into_client_close(),
                Err(cleanup) => return UiNativeEventLoopClientClose::Incomplete(Box::new(cleanup)),
            }
        }
        let shutdown = if let Some(shutdown) = application_runtime_shutdown {
            shutdown
        } else {
            let Some(shell) = self.shell.take() else {
                return if self.application.take().is_some()
                    || self.consumed_application_cleanup_complete
                {
                    UiNativeEventLoopClientClose::Complete
                } else {
                    UiNativeEventLoopClientClose::Incomplete(Box::new(
                        UiNativeApplicationDriverCleanup::UnresolvedApplication,
                    ))
                };
            };
            shell.shutdown()
        };
        let evidence = UiNativeDriverShutdownEvidence::captured(
            runtime_derived_state_reconstruction,
            self.observation_ingress_counts,
            self.progress.take_visual_snapshot(),
        );
        if shutdown.host_session_released()
            && shutdown.released_surface_count() == 1
            && shutdown.query_close_complete()
            && shutdown.intent_resources_empty()
        {
            let query_close = shutdown.into_query_close_observation();
            UiNativeEventLoopClientClose::CompleteWithObservation(evidence.finalize(&query_close))
        } else if let Some(cleanup) = shutdown.into_application_cleanup() {
            UiNativeEventLoopClientClose::Incomplete(Box::new(
                UiNativeApplicationDriverCleanup::Application {
                    cleanup: Box::new(cleanup),
                    evidence: Box::new(evidence),
                },
            ))
        } else {
            UiNativeEventLoopClientClose::Incomplete(Box::new(
                UiNativeApplicationDriverCleanup::UnresolvedApplication,
            ))
        }
    }
}
