#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeApplicationReadinessOwnerCount {
    count: u8,
}

#[derive(Clone)]
pub struct UiNativeApplicationReadinessPort {
    host: worth_ui_host_native::UiNativeApplicationReadinessPort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeApplicationReadinessOwnerCountDenial {
    CapacityExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeApplicationReadinessSignalDisposition {
    WakeRequested,
    Coalesced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeApplicationReadinessSignalDenial {
    OwnerClosed,
    EventLoopClosed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeApplicationRuntimeDirective {
    Continue,
    Close,
}

#[must_use]
pub struct UiNativeApplicationPhysicalProgress {
    host: worth_ui_host_native::UiNativePhysicalProgressGrant,
}

pub struct UiNativeApplicationObservationProgress {
    settlement: crate::facade::entry::UiNativeObservationIngressSettlement,
}

pub struct UiNativeApplicationRuntimeActivationStopped {
    application: Box<crate::facade::WorthUiNativeApplicationShell>,
}

pub struct UiNativeApplicationRuntimeProgressStopped {
    application: Box<crate::facade::WorthUiNativeApplicationShell>,
}

pub struct UiNativeApplicationRuntimeClosed {
    application: crate::facade::WorthUiNativeApplicationShutdownReceipt,
}

pub struct UiNativeApplicationRuntimeCloseIncomplete {
    runtime: Box<dyn UiNativeApplicationRuntime>,
    application: Box<crate::facade::WorthUiNativeApplicationShell>,
}

pub trait UiNativeApplicationRuntime: 'static {
    fn readiness_owner_count(&self) -> UiNativeApplicationReadinessOwnerCount;

    fn activate(
        &mut self,
        application: crate::facade::WorthUiNativeApplicationShell,
        readiness: Box<[UiNativeApplicationReadinessPort]>,
    ) -> Result<
        crate::facade::WorthUiNativeApplicationShell,
        UiNativeApplicationRuntimeActivationStopped,
    >;

    fn readiness_ready(
        &mut self,
        application: crate::facade::WorthUiNativeApplicationShell,
        owner_ordinal: u8,
        generation: u64,
    ) -> Result<
        (
            crate::facade::WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    >;

    fn native_observations_ready(
        &mut self,
        application: crate::facade::WorthUiNativeApplicationShell,
        _progress: UiNativeApplicationObservationProgress,
    ) -> Result<
        (
            crate::facade::WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    > {
        Ok((application, UiNativeApplicationRuntimeDirective::Continue))
    }

    /// Present current pointer meaning through the application's existing frame lifecycle.
    fn native_pointer_affordance_ready(
        &mut self,
        application: crate::facade::WorthUiNativeApplicationShell,
    ) -> Result<
        (
            crate::facade::WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    >;

    /// Progress one host-owned viewport successor after the native driver has
    /// installed its exact physical basis in the runtime shell. The callback
    /// carries no copied extent and is not measurement authority.
    fn native_viewport_ready(
        &mut self,
        application: crate::facade::WorthUiNativeApplicationShell,
    ) -> Result<
        (
            crate::facade::WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    > {
        Ok((application, UiNativeApplicationRuntimeDirective::Continue))
    }

    fn physical_work_progressed(
        &mut self,
        application: crate::facade::WorthUiNativeApplicationShell,
        _progress: UiNativeApplicationPhysicalProgress,
    ) -> Result<
        (
            crate::facade::WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    > {
        Ok((application, UiNativeApplicationRuntimeDirective::Continue))
    }

    fn close(
        self: Box<Self>,
        application: crate::facade::WorthUiNativeApplicationShell,
    ) -> Result<UiNativeApplicationRuntimeClosed, UiNativeApplicationRuntimeCloseIncomplete>;
}

impl UiNativeApplicationPhysicalProgress {
    pub(crate) fn from_host(host: worth_ui_host_native::UiNativePhysicalProgressGrant) -> Self {
        Self { host }
    }

    pub(crate) fn into_host(self) -> worth_ui_host_native::UiNativePhysicalProgressGrant {
        self.host
    }

    #[cfg(any(test, feature = "certification-support"))]
    #[doc(hidden)]
    pub fn from_certification(host: worth_ui_host_native::UiNativePhysicalProgressGrant) -> Self {
        Self::from_host(host)
    }

    pub(crate) fn class(&self) -> worth_ui_host_native::UiNativePhysicalProgressClass {
        self.host.class()
    }

    pub(crate) fn presentation(
        &self,
    ) -> Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation> {
        self.host.presentation()
    }

    pub(crate) fn recovery_presentation(
        &self,
    ) -> Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation> {
        self.host
            .presentation()
            .or_else(|| self.host.originating_presentation())
    }
}

impl UiNativeApplicationObservationProgress {
    pub(crate) fn from_settlement(
        settlement: crate::facade::entry::UiNativeObservationIngressSettlement,
    ) -> Self {
        Self { settlement }
    }

    pub fn event_count(&self) -> u64 {
        self.settlement.reachability().event_count()
    }

    pub fn pointer_button_events(&self) -> u64 {
        self.settlement.reachability().pointer_button_events()
    }

    pub fn keyboard_events(&self) -> u64 {
        self.settlement.reachability().keyboard_events()
    }

    pub fn text_events(&self) -> u64 {
        self.settlement.reachability().text_events()
    }

    pub fn ime_preedit_events(&self) -> u64 {
        self.settlement.reachability().ime_preedit_events()
    }

    pub fn ime_commit_events(&self) -> u64 {
        self.settlement.reachability().ime_commit_events()
    }

    pub fn ime_cancel_events(&self) -> u64 {
        self.settlement.reachability().ime_cancel_events()
    }

    pub fn retained_batch_count(&self) -> usize {
        self.settlement.retained_batch_count()
    }

    pub(crate) fn into_settlement(
        self,
    ) -> crate::facade::entry::UiNativeObservationIngressSettlement {
        self.settlement
    }
}

impl UiNativeApplicationRuntimeActivationStopped {
    pub fn retain(application: crate::facade::WorthUiNativeApplicationShell) -> Self {
        Self {
            application: Box::new(application),
        }
    }

    pub(crate) fn into_application(self) -> crate::facade::WorthUiNativeApplicationShell {
        *self.application
    }
}

impl UiNativeApplicationRuntimeProgressStopped {
    pub fn retain(application: crate::facade::WorthUiNativeApplicationShell) -> Self {
        Self {
            application: Box::new(application),
        }
    }

    pub(crate) fn into_application(self) -> crate::facade::WorthUiNativeApplicationShell {
        *self.application
    }
}

impl UiNativeApplicationRuntimeClosed {
    pub fn from_application_shutdown(
        application: crate::facade::WorthUiNativeApplicationShutdownReceipt,
    ) -> Self {
        Self { application }
    }

    pub(crate) fn into_application_shutdown(
        self,
    ) -> crate::facade::WorthUiNativeApplicationShutdownReceipt {
        self.application
    }
}

impl UiNativeApplicationRuntimeCloseIncomplete {
    pub fn retain<Runtime>(
        runtime: Box<Runtime>,
        application: crate::facade::WorthUiNativeApplicationShell,
    ) -> Self
    where
        Runtime: UiNativeApplicationRuntime,
    {
        Self {
            runtime,
            application: Box::new(application),
        }
    }

    pub(crate) fn application(&self) -> &crate::facade::WorthUiNativeApplicationShell {
        &self.application
    }

    pub(crate) fn retry(
        self,
    ) -> Result<UiNativeApplicationRuntimeClosed, UiNativeApplicationRuntimeCloseIncomplete> {
        self.runtime.close(*self.application)
    }
}

impl UiNativeApplicationReadinessOwnerCount {
    pub const MAXIMUM: u8 = 5;

    pub const fn new(count: u8) -> Result<Self, UiNativeApplicationReadinessOwnerCountDenial> {
        if count <= Self::MAXIMUM {
            Ok(Self { count })
        } else {
            Err(UiNativeApplicationReadinessOwnerCountDenial::CapacityExceeded)
        }
    }

    pub const fn none() -> Self {
        Self { count: 0 }
    }

    pub const fn get(self) -> u8 {
        self.count
    }
}

impl UiNativeApplicationReadinessPort {
    pub(crate) fn from_host(host: worth_ui_host_native::UiNativeApplicationReadinessPort) -> Self {
        Self { host }
    }

    pub fn signal(
        &self,
    ) -> Result<
        UiNativeApplicationReadinessSignalDisposition,
        UiNativeApplicationReadinessSignalDenial,
    > {
        self.host
            .signal()
            .map(|disposition| {
                match disposition {
            worth_ui_host_native::UiNativeApplicationReadinessSignalDisposition::WakeRequested => {
                UiNativeApplicationReadinessSignalDisposition::WakeRequested
            }
            worth_ui_host_native::UiNativeApplicationReadinessSignalDisposition::Coalesced => {
                UiNativeApplicationReadinessSignalDisposition::Coalesced
            }
            }
            })
            .map_err(|denial| match denial {
                worth_ui_host_native::UiNativeApplicationReadinessSignalDenial::OwnerClosed => {
                    UiNativeApplicationReadinessSignalDenial::OwnerClosed
                }
                worth_ui_host_native::UiNativeApplicationReadinessSignalDenial::EventLoopClosed => {
                    UiNativeApplicationReadinessSignalDenial::EventLoopClosed
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiNativeApplicationReadinessOwnerCount, UiNativeApplicationReadinessOwnerCountDenial,
    };

    #[test]
    fn public_runtime_owner_count_preserves_five_slots_beneath_host_internal_capacity() {
        assert_eq!(UiNativeApplicationReadinessOwnerCount::none().get(), 0);
        assert_eq!(
            UiNativeApplicationReadinessOwnerCount::new(5)
                .expect("five application readiness owners fit")
                .get(),
            5
        );
        assert_eq!(
            UiNativeApplicationReadinessOwnerCount::new(6),
            Err(UiNativeApplicationReadinessOwnerCountDenial::CapacityExceeded)
        );
        assert_eq!(
            worth_ui_host_native::UiNativeApplicationReadinessOwnerCount::MAXIMUM,
            6
        );
    }
}
