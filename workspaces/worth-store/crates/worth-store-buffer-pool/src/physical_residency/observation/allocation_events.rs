use std::sync::{Arc, Mutex};

use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::super::{
    PhysicalOperationAllocationScope, PhysicalResidencyDimension, PhysicalResidencyIncarnation,
};

#[path = "allocation_events/actualization.rs"]
mod actualization;
#[path = "allocation_events/boundary_facts.rs"]
mod boundary_facts;
#[path = "allocation_events/boundary_trace.rs"]
mod boundary_trace;
#[path = "allocation_events/trace_storage.rs"]
mod trace_storage;
pub(crate) use trace_storage::PreparedTracePage;
#[path = "allocation_events/release.rs"]
mod release;
pub(crate) use actualization::{
    PhysicalResidencyActualAllocationUnits, PhysicalResidencyAllocationActualization,
    PhysicalResidencyRequestedAllocationUnits,
};
use boundary_facts::PhysicalResidencyAllocationBoundaryFacts;
pub use boundary_trace::{
    PhysicalResidencyAllocationBoundaryEvent, PhysicalResidencyAllocationBoundaryKind,
    PhysicalResidencyAllocationOperation, PhysicalResidencyAllocationTrace,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalResidencyAllocationEventCounters {
    attempts: u64,
    admissions: u64,
    releases: u64,
    denials: u64,
    allocator_failures: u64,
    admitted_units: u64,
    released_units: u64,
    denied_units: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalResidencyAllocationEventSnapshot {
    store: StableStoreIdentity,
    pool: PhysicalResidencyIncarnation,
    dimensions: [PhysicalResidencyAllocationEventCounters; PhysicalResidencyDimension::COUNT],
}

#[derive(Debug, Clone)]
pub struct PhysicalResidencyAllocationEventObserver {
    store: StableStoreIdentity,
    pool: PhysicalResidencyIncarnation,
    cells: Arc<Mutex<PhysicalResidencyAllocationEventCells>>,
}

#[derive(Debug, Clone)]
pub(crate) struct PhysicalResidencyAllocationEventRecorder {
    cells: Arc<Mutex<PhysicalResidencyAllocationEventCells>>,
}

#[derive(Debug)]
struct PhysicalResidencyAllocationEventCells {
    dimensions: [PhysicalResidencyAllocationEventCounters; PhysicalResidencyDimension::COUNT],
    #[cfg(feature = "certification-test-authority")]
    next_sequence: u64,
    #[cfg(feature = "certification-test-authority")]
    events: trace_storage::EventTrace,
}

impl PhysicalResidencyAllocationEventRecorder {
    pub(crate) fn new(
        store: StableStoreIdentity,
        pool: PhysicalResidencyIncarnation,
    ) -> (Self, PhysicalResidencyAllocationEventObserver) {
        let cells = Arc::new(Mutex::new(PhysicalResidencyAllocationEventCells {
            dimensions: [PhysicalResidencyAllocationEventCounters::default();
                PhysicalResidencyDimension::COUNT],
            #[cfg(feature = "certification-test-authority")]
            next_sequence: 1,
            #[cfg(feature = "certification-test-authority")]
            events: trace_storage::EventTrace::default(),
        }));
        (
            Self {
                cells: Arc::clone(&cells),
            },
            PhysicalResidencyAllocationEventObserver { store, pool, cells },
        )
    }

    pub(crate) fn admit(&self, dimension: PhysicalResidencyDimension, units: u64) {
        self.admit_with_scope(dimension, None, units);
    }

    pub(crate) fn admit_scoped(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: PhysicalOperationAllocationScope,
        units: u64,
    ) {
        self.admit_with_scope(dimension, Some(scope), units);
    }

    fn admit_with_scope(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: Option<PhysicalOperationAllocationScope>,
        units: u64,
    ) {
        let mut cells = self.lock();
        let counters = &mut cells.dimensions[dimension.index()];
        counters.attempts += 1;
        counters.admissions += 1;
        counters.admitted_units = counters.admitted_units.saturating_add(units);
        Self::record(
            &mut cells,
            PhysicalResidencyAllocationBoundaryFacts::admission(dimension, scope, units),
        );
    }

    pub(crate) fn release(&self, dimension: PhysicalResidencyDimension, units: u64) {
        self.release_with_scope(dimension, None, units);
    }

    pub(crate) fn release_scoped(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: PhysicalOperationAllocationScope,
        units: u64,
    ) {
        self.release_with_scope(dimension, Some(scope), units);
    }

    fn release_with_scope(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: Option<PhysicalOperationAllocationScope>,
        units: u64,
    ) {
        if units == 0 {
            return;
        }
        let mut cells = self.lock();
        let counters = &mut cells.dimensions[dimension.index()];
        counters.releases += 1;
        counters.released_units = counters.released_units.saturating_add(units);
        Self::record(
            &mut cells,
            PhysicalResidencyAllocationBoundaryFacts::release(dimension, scope, units),
        );
    }

    pub(crate) fn deny_scoped(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: PhysicalOperationAllocationScope,
        units: u64,
    ) {
        self.deny_with_scope(dimension, Some(scope), units);
    }

    fn deny_with_scope(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: Option<PhysicalOperationAllocationScope>,
        units: u64,
    ) {
        let mut cells = self.lock();
        let counters = &mut cells.dimensions[dimension.index()];
        counters.attempts += 1;
        counters.denials += 1;
        counters.denied_units = counters.denied_units.saturating_add(units);
        Self::record(
            &mut cells,
            PhysicalResidencyAllocationBoundaryFacts::denial(dimension, scope, units),
        );
    }

    pub(crate) fn allocator_failure_scoped(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: PhysicalOperationAllocationScope,
        units: u64,
    ) {
        self.allocator_failure_with_scope(dimension, Some(scope), units);
    }

    fn allocator_failure_with_scope(
        &self,
        dimension: PhysicalResidencyDimension,
        scope: Option<PhysicalOperationAllocationScope>,
        units: u64,
    ) {
        let mut cells = self.lock();
        let counters = &mut cells.dimensions[dimension.index()];
        counters.allocator_failures += 1;
        counters.releases += 1;
        counters.released_units = counters.released_units.saturating_add(units);
        Self::record(
            &mut cells,
            PhysicalResidencyAllocationBoundaryFacts::allocator_failure(dimension, scope, units),
        );
    }

    pub(crate) fn actualize_unscoped(
        &self,
        dimension: PhysicalResidencyDimension,
        requested_units: u64,
        actual_units: u64,
    ) {
        let mut cells = self.lock();
        Self::record(
            &mut cells,
            PhysicalResidencyAllocationBoundaryFacts {
                kind: PhysicalResidencyAllocationBoundaryKind::Actualization,
                dimension,
                scope: None,
                requested_units,
                actual_units,
                operation: None,
            },
        );
    }

    pub(crate) fn actualize(&self, actualization: PhysicalResidencyAllocationActualization) {
        let mut cells = self.lock();
        Self::record(
            &mut cells,
            PhysicalResidencyAllocationBoundaryFacts::actualization(actualization),
        );
    }

    #[cfg(feature = "certification-test-authority")]
    fn record(
        cells: &mut PhysicalResidencyAllocationEventCells,
        facts: PhysicalResidencyAllocationBoundaryFacts,
    ) {
        let sequence = cells.next_sequence;
        cells.next_sequence = cells
            .next_sequence
            .checked_add(1)
            .expect("allocation evidence sequence exhausted");
        cells
            .events
            .push(PhysicalResidencyAllocationBoundaryEvent::new(
                sequence, facts,
            ));
    }

    #[cfg(not(feature = "certification-test-authority"))]
    fn record(
        _: &mut PhysicalResidencyAllocationEventCells,
        _: PhysicalResidencyAllocationBoundaryFacts,
    ) {
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PhysicalResidencyAllocationEventCells> {
        self.cells
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl PhysicalResidencyAllocationEventObserver {
    pub fn snapshot(&self) -> PhysicalResidencyAllocationEventSnapshot {
        let cells = self
            .cells
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        PhysicalResidencyAllocationEventSnapshot {
            store: self.store,
            pool: self.pool,
            dimensions: cells.dimensions,
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn trace(&self) -> PhysicalResidencyAllocationTrace {
        let cells = self
            .cells
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        PhysicalResidencyAllocationTrace::new(self.store, self.pool, cells.events.snapshot())
    }
}

impl PhysicalResidencyAllocationEventSnapshot {
    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn pool(self) -> PhysicalResidencyIncarnation {
        self.pool
    }

    pub const fn for_dimension(
        self,
        dimension: PhysicalResidencyDimension,
    ) -> PhysicalResidencyAllocationEventCounters {
        self.dimensions[dimension.index()]
    }
}

impl PhysicalResidencyAllocationEventCounters {
    pub const fn attempts(self) -> u64 {
        self.attempts
    }

    pub const fn admissions(self) -> u64 {
        self.admissions
    }

    pub const fn releases(self) -> u64 {
        self.releases
    }

    pub const fn denials(self) -> u64 {
        self.denials
    }

    pub const fn allocator_failures(self) -> u64 {
        self.allocator_failures
    }

    pub const fn admitted_units(self) -> u64 {
        self.admitted_units
    }

    pub const fn released_units(self) -> u64 {
        self.released_units
    }

    pub const fn denied_units(self) -> u64 {
        self.denied_units
    }

    pub const fn active_units(self) -> u64 {
        self.admitted_units.saturating_sub(self.released_units)
    }
}
