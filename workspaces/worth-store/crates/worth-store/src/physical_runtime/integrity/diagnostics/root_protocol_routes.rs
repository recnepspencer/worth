use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootProtocolRoute {
    OrdinaryOpen,
    Initialization,
    ScheduledReopen,
    CleanupFreshness,
    CleanupRemoval,
}

const ROUTE_COUNT: usize = 5;

impl PhysicalRootProtocolRoute {
    const fn index(self) -> usize {
        match self {
            Self::OrdinaryOpen => 0,
            Self::Initialization => 1,
            Self::ScheduledReopen => 2,
            Self::CleanupFreshness => 3,
            Self::CleanupRemoval => 4,
        }
    }
}

#[derive(Default)]
pub(in crate::physical_runtime) struct RootProtocolRouteCounterCells {
    selector_entries: [AtomicU64; ROUTE_COUNT],
    root_entries: [AtomicU64; ROUTE_COUNT],
    publications: [AtomicU64; ROUTE_COUNT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootProtocolRouteCounters {
    selector_entries: [u64; ROUTE_COUNT],
    root_entries: [u64; ROUTE_COUNT],
    publications: [u64; ROUTE_COUNT],
}

impl RootProtocolRouteCounterCells {
    pub(in crate::physical_runtime) fn observe_selector(&self, route: PhysicalRootProtocolRoute) {
        self.selector_entries[route.index()].fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn observe_root(&self, route: PhysicalRootProtocolRoute) {
        self.root_entries[route.index()].fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn observe_publication(
        &self,
        route: PhysicalRootProtocolRoute,
    ) {
        self.publications[route.index()].fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn snapshot(&self) -> RootProtocolRouteCounters {
        RootProtocolRouteCounters {
            selector_entries: std::array::from_fn(|index| {
                self.selector_entries[index].load(Ordering::Relaxed)
            }),
            root_entries: std::array::from_fn(|index| {
                self.root_entries[index].load(Ordering::Relaxed)
            }),
            publications: std::array::from_fn(|index| {
                self.publications[index].load(Ordering::Relaxed)
            }),
        }
    }
}

impl RootProtocolRouteCounters {
    pub const fn selector_entries(self, route: PhysicalRootProtocolRoute) -> u64 {
        self.selector_entries[route.index()]
    }

    pub const fn root_entries(self, route: PhysicalRootProtocolRoute) -> u64 {
        self.root_entries[route.index()]
    }

    pub const fn publications(self, route: PhysicalRootProtocolRoute) -> u64 {
        self.publications[route.index()]
    }
}
