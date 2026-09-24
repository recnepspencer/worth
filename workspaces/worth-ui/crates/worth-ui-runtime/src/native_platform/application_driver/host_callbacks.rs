use super::*;
use worth_ui_host_native::UiNativeEventLoopClientDenial as Denial;

impl UiNativeApplicationDriver {
    /// Take the application and launch its native surface at the granted
    /// scale, retaining for `close` whatever cleanup the launch denial
    /// carried. Every path out is `ApplicationLaunchDenied`; what differs is
    /// which cleanup the driver still owes, which is why the arms cannot
    /// collapse into one.
    fn launch_native_shell(&mut self, grant: &UiNativeReadinessGrant) -> Result<(), Denial> {
        let application = self
            .application
            .take()
            .ok_or(Denial::ApplicationUnavailable)?;
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
                return Err(Denial::ApplicationLaunchDenied);
            }
            Err(crate::facade::WorthUiNativeApplicationShellLaunchDenial::ApplicationCleanup(
                cleanup,
            )) => {
                self.pending_cleanup = Some(UiNativeApplicationDriverCleanup::Application {
                    cleanup,
                    evidence: Box::new(UiNativeDriverShutdownEvidence::empty()),
                });
                return Err(Denial::ApplicationLaunchDenied);
            }
            Err(denial) => {
                let _ = denial;
                self.consumed_application_cleanup_complete = true;
                return Err(Denial::ApplicationLaunchDenied);
            }
        };
        Ok(())
    }
}

impl UiNativeEventLoopClient for UiNativeApplicationDriver {
    fn install_observation_clock(
        &mut self,
        clock: worth_ui_host_native::UiNativeObservationClock,
    ) -> Result<(), Denial> {
        if self.observation_clock.is_some() {
            return Err(Denial::AlreadyInstalled);
        }
        self.observation_clock = Some(clock);
        Ok(())
    }

    fn observation_time_ready(
        &mut self,
    ) -> Result<worth_ui_host_native::UiNativeObservationTimeProgress, Denial> {
        self.progress_observation_time()
            .map_err(|()| Denial::Unattributed)
    }

    fn application_readiness_owner_count(
        &self,
    ) -> worth_ui_host_native::UiNativeApplicationReadinessOwnerCount {
        self.application_readiness_owner_count()
    }

    fn install_application_readiness(
        &mut self,
        ports: Vec<worth_ui_host_native::UiNativeApplicationReadinessPort>,
    ) -> Result<(), Denial> {
        self.install_application_readiness(ports.into_boxed_slice())
            .map_err(|()| Denial::Unattributed)
    }

    fn application_readiness_ready(
        &mut self,
        grant: worth_ui_host_native::UiNativeApplicationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        self.progress_application_runtime(grant)
            .map_err(|()| Denial::ApplicationProgressDenied)
    }

    fn native_surface_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        if grant.generation() != 0 {
            return Err(Denial::StaleGrant);
        }
        if self.shell.is_some() {
            return Err(Denial::AlreadyInstalled);
        }
        self.launch_native_shell(&grant)?;
        let clock = self.observation_clock.clone().ok_or(Denial::Unattributed)?;
        self.shell
            .as_mut()
            .ok_or(Denial::SurfaceUnbound)?
            .install_native_observation_clock(clock)
            .map_err(|()| Denial::Unattributed)?;
        self.shell
            .as_mut()
            .ok_or(Denial::SurfaceUnbound)?
            .observe_native_viewport_readiness(
                grant.client_physical_size(),
                grant.scale_factor_milli(),
                false,
            );
        self.progress
            .observe_readiness(grant.generation(), grant.surface_basis_generation());
        self.scale_factor_milli = Some(grant.scale_factor_milli());
        self.activate_application_runtime()
            .map_err(|()| Denial::ApplicationProgressDenied)?;
        Ok(UiNativeEventLoopDirective::Continue)
    }

    fn redraw_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        if grant.generation() <= self.last_ready_generation {
            return Err(Denial::StaleGrant);
        }
        let shell = self.shell.as_mut().ok_or(Denial::SurfaceUnbound)?;
        if self.scale_factor_milli != Some(grant.scale_factor_milli()) {
            if shell
                .rebind_native_surface_scale(grant.scale_factor_milli())
                .is_err()
            {
                return Err(Denial::SurfaceScaleRebindDenied);
            }
            self.scale_factor_milli = Some(grant.scale_factor_milli());
        }
        shell.observe_native_viewport_readiness(
            grant.client_physical_size(),
            grant.scale_factor_milli(),
            true,
        );
        let surface_basis_successor = self
            .progress
            .observe_readiness(grant.generation(), grant.surface_basis_generation());
        if self.progress.advance(shell).is_err() {
            return Err(Denial::ClientProgressDenied);
        }
        self.last_ready_generation = grant.generation();
        if self.application_runtime_active
            && (surface_basis_successor
                || self.shell.as_ref().is_some_and(
                    WorthUiNativeApplicationShell::native_viewport_presentation_pending,
                ))
        {
            return self
                .progress_application_runtime_viewport(surface_basis_successor)
                .map_err(|()| Denial::ApplicationProgressDenied);
        }
        Ok(self.next_directive())
    }

    fn physical_work_progressed(
        &mut self,
        grant: worth_ui_host_native::UiNativePhysicalProgressGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        if self
            .progress_motion_physical(&grant)
            .map_err(|()| Denial::Unattributed)?
        {
            return self
                .progress_application_runtime_motion_settlement(true)
                .map_err(|()| Denial::ApplicationProgressDenied);
        }
        if self.application_runtime_active {
            return self
                .progress_application_runtime_physical(grant)
                .map_err(|()| Denial::ApplicationProgressDenied);
        }
        let shell = self.shell.as_mut().ok_or(Denial::SurfaceUnbound)?;
        self.progress
            .physical_work_progressed(shell, grant)
            .map_err(|()| Denial::ClientProgressDenied)?;
        Ok(self.next_directive())
    }

    fn native_observations_ready(
        &mut self,
        grant: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        if grant.generation() <= self.last_observation_ready_generation {
            return Err(Denial::StaleGrant);
        }
        let shell = self.shell.as_mut().ok_or(Denial::SurfaceUnbound)?;
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
            self.observation_ingress_counts[4] =
                self.observation_ingress_counts[4].saturating_add(1);
            return Err(Denial::ObservationDrainDenied);
        }
        self.last_observation_ready_generation = grant.generation();
        let directive = self
            .progress_application_runtime_observations(settlement)
            .map_err(|()| Denial::ApplicationProgressDenied)?;
        if matches!(directive, UiNativeEventLoopDirective::Close) {
            Ok(directive)
        } else {
            Ok(self.next_directive())
        }
    }

    fn external_close_requested(&mut self) -> Result<UiNativeEventLoopDirective, Denial> {
        if let Some(runtime) = self.application_runtime.as_mut() {
            runtime.external_close_requested();
        }
        self.progress.request_external_close();
        Ok(self.next_directive())
    }

    fn native_input_retention_exhausted(
        &mut self,
        grant: worth_ui_host_native::UiNativeInputRecoveryGrant,
    ) -> Result<
        (
            worth_ui_host_native::UiNativeInputRecoveryAcknowledgement,
            UiNativeEventLoopDirective,
        ),
        Denial,
    > {
        let settlement = self
            .shell
            .as_mut()
            .ok_or(Denial::SurfaceUnbound)?
            .cancel_exhausted_native_input(&grant)
            .map_err(|()| Denial::ObservationDrainDenied)?;
        let directive = self
            .progress_application_runtime_observations(settlement)
            .map_err(|()| Denial::ApplicationProgressDenied)?;
        Ok((grant.acknowledge_cancellation(), directive))
    }

    fn presentation_attribution(
        &self,
        observed: &worth_ui_host_native::UiNativeRetainedFrameObservation,
    ) -> Option<worth_ui_host_native::UiNativeClientPresentationAttribution> {
        self.shell
            .as_ref()
            .and_then(|shell| shell.current_presentation_attribution(observed))
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
