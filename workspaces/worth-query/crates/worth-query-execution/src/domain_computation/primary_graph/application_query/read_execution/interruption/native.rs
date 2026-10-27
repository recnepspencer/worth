//! Preserve the first concrete Query stop across Native's narrower callback cause.
use super::*;
use std::cell::RefCell;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
pub(in crate::domain_computation::primary_graph::application_query::read_execution) struct NativeReadInterruption<
    'a,
    'subject,
> {
    interruption: ReadInterruption<'a>,
    subject: &'subject str,
    first: RefCell<Option<WorthQueryApplicationReadExecutionDenial>>,
}
impl<'a, 'subject> NativeReadInterruption<'a, 'subject> {
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn new(
        interruption: ReadInterruption<'a>,
        subject: &'subject str,
    ) -> Self {
        Self {
            interruption,
            subject,
            first: RefCell::new(None),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn checkpoint(
        &self,
    ) -> Result<(), Cause> {
        match self.interruption.checkpoint(self.subject) {
            Ok(()) => Ok(()),
            Err(denial) => {
                self.remember(&denial);
                Err(Cause::NestedStopped)
            }
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn remember(
        &self,
        denial: &WorthQueryApplicationReadExecutionDenial,
    ) {
        self.first
            .borrow_mut()
            .get_or_insert_with(|| denial.clone());
    }
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn denial(
        &self,
    ) -> Option<WorthQueryApplicationReadExecutionDenial> {
        self.first.borrow().clone()
    }
}
