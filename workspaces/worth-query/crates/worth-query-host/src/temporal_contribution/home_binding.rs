use worth_query_execution::facade::primary_graph::WorthQueryConditionalClockHandle;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryNamedClock};

use super::{clock::WorthQueryDesktopClockHandle, installer::WorthQueryInstalledDesktopTemporal};

/// Declarative hook implemented by an installed conditional contribution so
/// [`crate::application_home::WorthQueryApplicationHome::catch_up_temporal`]
/// can resolve the runtime clock handle and caller-pumped desktop clock handle
/// without requiring callers to re-specify runtime, branch, or clock arguments.
pub trait WorthQueryDesktopTemporalHomeBinding<Schema: ApplicationSchema> {
    type Node: 'static;
    type Clock: WorthQueryNamedClock;

    fn clock_handle(&self) -> &WorthQueryConditionalClockHandle<Schema, Self::Node, Self::Clock>;
    fn desktop_clock(&self) -> &WorthQueryDesktopClockHandle<Self::Clock>;
}

/// Ready-to-use `Installed` type pairing [`WorthQueryInstalledDesktopTemporal`]
/// with its caller-pumped [`WorthQueryDesktopClockHandle`].
pub struct WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock> {
    temporal: WorthQueryInstalledDesktopTemporal<Schema, Node, Clock>,
    desktop_clock: WorthQueryDesktopClockHandle<Clock>,
}

impl<Schema, Node, Clock> WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock> {
    pub(super) const fn new(
        temporal: WorthQueryInstalledDesktopTemporal<Schema, Node, Clock>,
        desktop_clock: WorthQueryDesktopClockHandle<Clock>,
    ) -> Self {
        Self {
            temporal,
            desktop_clock,
        }
    }

    pub const fn temporal(&self) -> &WorthQueryInstalledDesktopTemporal<Schema, Node, Clock> {
        &self.temporal
    }

    pub const fn desktop_clock(&self) -> &WorthQueryDesktopClockHandle<Clock> {
        &self.desktop_clock
    }
}

impl<Schema, Node, Clock> WorthQueryDesktopTemporalHomeBinding<Schema>
    for WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock>
where
    Schema: ApplicationSchema,
    Node: 'static,
    Clock: WorthQueryNamedClock,
{
    type Node = Node;
    type Clock = Clock;

    fn clock_handle(&self) -> &WorthQueryConditionalClockHandle<Schema, Node, Clock> {
        self.temporal.clock_handle()
    }

    fn desktop_clock(&self) -> &WorthQueryDesktopClockHandle<Clock> {
        &self.desktop_clock
    }
}
