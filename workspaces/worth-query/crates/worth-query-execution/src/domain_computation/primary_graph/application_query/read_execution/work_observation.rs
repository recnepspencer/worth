//! Actual visited work of an admitted one-shot read, including denial.

use std::cell::Cell;

/// Root and tree observations include a refused read's completed prefix.
pub(in crate::domain_computation::primary_graph::application_query) struct OneShotReadWorkObservation<
    'charge,
> {
    charge: Option<
        &'charge dyn Fn(usize, &str) -> Result<(), super::WorthQueryApplicationReadExecutionDenial>,
    >,
    root: Cell<usize>,
    tree: Cell<usize>,
}

impl<'charge> OneShotReadWorkObservation<'charge> {
    pub(in crate::domain_computation::primary_graph::application_query) const fn new() -> Self {
        Self {
            charge: None,
            root: Cell::new(0),
            tree: Cell::new(0),
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

    /// Root and tree work: what the read examined.
    pub(in crate::domain_computation::primary_graph::application_query) fn read(
        &self,
    ) -> Option<usize> {
        self.root.get().checked_add(self.tree.get())
    }
}
