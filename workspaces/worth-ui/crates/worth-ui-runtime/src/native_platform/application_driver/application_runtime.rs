use super::UiNativeApplicationDriver;
use crate::native_platform::{
    UiNativeApplicationReadinessPort, UiNativeApplicationRuntimeDirective,
};

impl UiNativeApplicationDriver {
    pub(super) fn application_readiness_owner_count(
        &self,
    ) -> worth_ui_host_native::UiNativeApplicationReadinessOwnerCount {
        let public = self
            .application_runtime
            .as_ref()
            .map_or(0, |runtime| runtime.readiness_owner_count().get());
        total_readiness_owner_count(public, self.motion_support_installed)
    }

    pub(super) fn install_application_readiness(
        &mut self,
        ports: Box<[worth_ui_host_native::UiNativeApplicationReadinessPort]>,
    ) -> Result<(), ()> {
        let expected = usize::from(self.application_readiness_owner_count().get());
        if self.application_runtime_ports.is_some()
            || self.motion_readiness.is_some()
            || ports.len() != expected
        {
            return Err(());
        }
        let mut ports = ports.into_vec();
        if self.motion_support_installed {
            let port = ports.pop().ok_or(())?;
            self.motion_readiness = Some(super::UiNativeMotionReadinessLane::start(port)?);
        }
        self.application_runtime_ports = Some(
            ports
                .into_iter()
                .map(UiNativeApplicationReadinessPort::from_host)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        Ok(())
    }

    pub(super) fn activate_application_runtime(&mut self) -> Result<(), ()> {
        let Some(runtime) = self.application_runtime.as_mut() else {
            let activated = self
                .application_runtime_ports
                .take()
                .filter(|ports| ports.is_empty())
                .map(|_| ())
                .ok_or(());
            if activated.is_ok() {
                self.arm_motion_readiness_now();
            }
            return activated;
        };
        let ports = self.application_runtime_ports.take().ok_or(())?;
        let shell = self.shell.take().ok_or(())?;
        self.application_runtime_active = true;
        match runtime.activate(shell, ports) {
            Ok(shell) => {
                self.shell = Some(shell);
                self.arm_motion_readiness_now();
                Ok(())
            }
            Err(stopped) => {
                self.shell = Some(stopped.into_application());
                Err(())
            }
        }
    }

    pub(super) fn progress_application_runtime(
        &mut self,
        grant: worth_ui_host_native::UiNativeApplicationReadinessGrant,
    ) -> Result<worth_ui_host_native::UiNativeEventLoopDirective, ()> {
        if !self.application_runtime_active && !self.motion_support_installed {
            return Err(());
        }
        let public_owner_count = self
            .application_runtime
            .as_ref()
            .map_or(0, |runtime| runtime.readiness_owner_count().get());
        if self.motion_support_installed && grant.owner_ordinal() == public_owner_count {
            return self.progress_motion_readiness(grant);
        }
        if !self.application_runtime_active {
            return Err(());
        }
        let runtime = self.application_runtime.as_mut().ok_or(())?;
        let shell = self.shell.take().ok_or(())?;
        match runtime.readiness_ready(shell, grant.owner_ordinal(), grant.generation()) {
            Ok((shell, directive)) => {
                self.shell = Some(shell);
                self.arm_motion_readiness_now();
                Ok(map_directive(directive))
            }
            Err(stopped) => {
                self.shell = Some(stopped.into_application());
                Err(())
            }
        }
    }

    pub(super) fn progress_application_runtime_observations(
        &mut self,
        settlement: crate::facade::entry::UiNativeObservationIngressSettlement,
    ) -> Result<worth_ui_host_native::UiNativeEventLoopDirective, ()> {
        if !self.application_runtime_active {
            return Ok(worth_ui_host_native::UiNativeEventLoopDirective::Continue);
        }
        let runtime = self.application_runtime.as_mut().ok_or(())?;
        let shell = self.shell.take().ok_or(())?;
        let progress =
            crate::native_platform::UiNativeApplicationObservationProgress::from_settlement(
                settlement,
            );
        match runtime.native_observations_ready(shell, progress) {
            Ok((shell, directive)) => {
                self.shell = Some(shell);
                self.arm_motion_readiness_now();
                Ok(map_directive(directive))
            }
            Err(stopped) => {
                self.shell = Some(stopped.into_application());
                Err(())
            }
        }
    }

    pub(super) fn progress_application_runtime_pointer(
        &mut self,
    ) -> Result<worth_ui_host_native::UiNativeEventLoopDirective, ()> {
        if !self.application_runtime_active {
            return Err(());
        }
        let runtime = self.application_runtime.as_mut().ok_or(())?;
        let shell = self.shell.take().ok_or(())?;
        match runtime.native_pointer_affordance_ready(shell) {
            Ok((shell, directive)) => {
                self.shell = Some(shell);
                self.arm_motion_readiness_now();
                Ok(map_directive(directive))
            }
            Err(stopped) => {
                self.shell = Some(stopped.into_application());
                Err(())
            }
        }
    }

    pub(super) fn progress_application_runtime_viewport(
        &mut self,
    ) -> Result<worth_ui_host_native::UiNativeEventLoopDirective, ()> {
        if !self.application_runtime_active {
            return Err(());
        }
        let runtime = self.application_runtime.as_mut().ok_or(())?;
        let shell = self.shell.take().ok_or(())?;
        match runtime.native_viewport_ready(shell) {
            Ok((shell, directive)) => {
                self.shell = Some(shell);
                self.arm_motion_readiness_now();
                Ok(map_directive(directive))
            }
            Err(stopped) => {
                self.shell = Some(stopped.into_application());
                Err(())
            }
        }
    }

    fn progress_motion_readiness(
        &mut self,
        grant: worth_ui_host_native::UiNativeApplicationReadinessGrant,
    ) -> Result<worth_ui_host_native::UiNativeEventLoopDirective, ()> {
        if self.motion_readiness.is_none()
            || grant.generation() <= self.last_motion_readiness_generation
        {
            return Err(());
        }
        let schedules_next_frame = self
            .shell
            .as_mut()
            .ok_or(())?
            .admit_native_motion_tick(grant.physical_tick(), grant.reduced_motion())?
            .schedules_next_frame();
        self.last_motion_readiness_generation = grant.generation();
        if schedules_next_frame {
            self.arm_motion_readiness_next_frame();
        }
        Ok(worth_ui_host_native::UiNativeEventLoopDirective::Continue)
    }

    pub(super) fn progress_motion_physical(
        &mut self,
        grant: &worth_ui_host_native::UiNativePhysicalProgressGrant,
    ) -> Result<bool, ()> {
        if !self.shell.as_ref().is_some_and(|shell| {
            shell.owns_pending_native_motion_physical(grant.class(), grant.presentation())
        }) {
            return Ok(false);
        }
        let schedules_next_frame = self
            .shell
            .as_mut()
            .ok_or(())?
            .complete_pending_native_motion_physical(grant.class(), grant.presentation())
            .schedules_next_frame();
        if schedules_next_frame {
            self.arm_motion_readiness_next_frame();
        }
        Ok(true)
    }

    pub(super) fn arm_motion_readiness_now(&self) {
        if self.motion_can_request_readiness() {
            if let Some(readiness) = self.motion_readiness.as_ref() {
                readiness.arm_now();
            }
        }
    }

    fn arm_motion_readiness_next_frame(&self) {
        if self.motion_can_request_readiness() {
            if let Some(readiness) = self.motion_readiness.as_ref() {
                readiness.arm_next_frame();
            }
        }
    }

    pub(super) fn motion_can_request_readiness(&self) -> bool {
        self.shell.as_ref().is_some_and(|shell| {
            shell.native_motion_sampling_active()
                && !shell.native_motion_sample_presentation_pending()
        })
    }

    pub(super) fn progress_application_runtime_physical(
        &mut self,
        grant: worth_ui_host_native::UiNativePhysicalProgressGrant,
    ) -> Result<worth_ui_host_native::UiNativeEventLoopDirective, ()> {
        if !self.application_runtime_active {
            return Err(());
        }
        let runtime = self.application_runtime.as_mut().ok_or(())?;
        let shell = self.shell.take().ok_or(())?;
        let progress =
            crate::native_platform::UiNativeApplicationPhysicalProgress::from_host(grant);
        match runtime.physical_work_progressed(shell, progress) {
            Ok((shell, directive)) => {
                self.shell = Some(shell);
                Ok(map_directive(directive))
            }
            Err(stopped) => {
                self.shell = Some(stopped.into_application());
                Err(())
            }
        }
    }

    pub(super) fn application_runtime_shell(
        &self,
    ) -> Option<&crate::facade::WorthUiNativeApplicationShell> {
        self.shell.as_ref().or_else(|| {
            self.pending_application_runtime_close
                .as_ref()
                .map(crate::native_platform::UiNativeApplicationRuntimeCloseIncomplete::application)
        })
    }

    pub(super) fn close_application_runtime(
        &mut self,
    ) -> Result<Option<crate::facade::WorthUiNativeApplicationShutdownReceipt>, ()> {
        if let Some(incomplete) = self.pending_application_runtime_close.take() {
            return match incomplete.retry() {
                Ok(closed) => {
                    self.application_runtime_active = false;
                    Ok(Some(closed.into_application_shutdown()))
                }
                Err(incomplete) => {
                    self.pending_application_runtime_close = Some(incomplete);
                    Err(())
                }
            };
        }
        if !self.application_runtime_active {
            self.application_runtime.take();
            self.application_runtime_ports.take();
            if let Some(mut readiness) = self.motion_readiness.take() {
                readiness.shutdown();
            }
            return Ok(None);
        }
        let runtime = self.application_runtime.take().ok_or(())?;
        let shell = self.shell.take().ok_or(())?;
        self.application_runtime_ports.take();
        if let Some(mut readiness) = self.motion_readiness.take() {
            readiness.shutdown();
        }
        match runtime.close(shell) {
            Ok(closed) => {
                self.application_runtime_active = false;
                Ok(Some(closed.into_application_shutdown()))
            }
            Err(incomplete) => {
                self.pending_application_runtime_close = Some(incomplete);
                Err(())
            }
        }
    }
}

fn total_readiness_owner_count(
    public: u8,
    motion_support_installed: bool,
) -> worth_ui_host_native::UiNativeApplicationReadinessOwnerCount {
    worth_ui_host_native::UiNativeApplicationReadinessOwnerCount::new(
        public.saturating_add(u8::from(motion_support_installed)),
    )
    .expect("five public readiness owners plus one internal Motion owner fit the host")
}

fn map_directive(
    directive: UiNativeApplicationRuntimeDirective,
) -> worth_ui_host_native::UiNativeEventLoopDirective {
    match directive {
        UiNativeApplicationRuntimeDirective::Continue => {
            worth_ui_host_native::UiNativeEventLoopDirective::Continue
        }
        UiNativeApplicationRuntimeDirective::Close => {
            worth_ui_host_native::UiNativeEventLoopDirective::Close
        }
    }
}

#[cfg(test)]
mod tests {
    use super::total_readiness_owner_count;

    #[test]
    fn motion_conditionally_reserves_one_internal_slot_after_all_public_slots() {
        assert_eq!(total_readiness_owner_count(5, false).get(), 5);
        assert_eq!(total_readiness_owner_count(5, true).get(), 6);
        assert_eq!(total_readiness_owner_count(0, true).get(), 1);
        assert_eq!(total_readiness_owner_count(0, false).get(), 0);
    }
}
