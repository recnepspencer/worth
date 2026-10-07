//! Request-local Work spent by an admitted one-shot read, including denial.

use std::cell::Cell;

/// The existing root and tree meters own the read's own Work, which is what
/// the receipt reports against the Query's declared limit. A successful
/// finalization adds its performed custody copies (scope and descriptor), which
/// only the carried admission pays. Separate cells keep a denial's partial work
/// without a second budget.
pub(in crate::domain_computation::primary_graph::application_query) struct OneShotReadWorkObservation<
    'charge,
> {
    charge: Option<
        &'charge dyn Fn(usize, &str) -> Result<(), super::WorthQueryApplicationReadExecutionDenial>,
    >,
    root: Cell<usize>,
    tree: Cell<usize>,
    scope: Cell<usize>,
    descriptor: Cell<usize>,
}

impl<'charge> OneShotReadWorkObservation<'charge> {
    pub(in crate::domain_computation::primary_graph::application_query) const fn new() -> Self {
        Self {
            charge: None,
            root: Cell::new(0),
            tree: Cell::new(0),
            scope: Cell::new(0),
            descriptor: Cell::new(0),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn with_charge(
        charge: &'charge dyn Fn(
            usize,
            &str,
        )
            -> Result<(), super::WorthQueryApplicationReadExecutionDenial>,
    ) -> Self {
        Self {
            charge: Some(charge),
            ..Self::new()
        }
    }

    pub(super) fn charge(
        &self,
        units: usize,
        subject: &str,
    ) -> Result<(), super::WorthQueryApplicationReadExecutionDenial> {
        match self.charge {
            Some(charge) => charge(units, subject),
            None => Ok(()),
        }
    }

    pub(super) fn root(&self, work: usize) {
        self.root.set(work);
    }

    pub(super) fn tree(&self, work: usize) {
        self.tree.set(work);
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn scope(
        &self,
        work: usize,
    ) {
        self.scope.set(work);
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn descriptor(
        &self,
        work: usize,
    ) {
        self.descriptor.set(work);
    }

    /// Root and tree work: what the read examined.
    pub(in crate::domain_computation::primary_graph::application_query) fn read(
        &self,
    ) -> Option<usize> {
        self.root.get().checked_add(self.tree.get())
    }

    /// Read work plus the custody copies the carried admission settles.
    pub(in crate::domain_computation::primary_graph::application_query) fn total(
        &self,
    ) -> Option<usize> {
        self.read()?
            .checked_add(self.scope.get())?
            .checked_add(self.descriptor.get())
    }
}
