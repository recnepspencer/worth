//! Preserve the first concrete Query stop across Native's narrower callback cause.
use super::*;
use std::cell::RefCell;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
pub struct NativeReadInterruption<
    'a,
    'subject,
> {
    interruption: ReadInterruption<'a>,
    subject: &'subject str,
    first: RefCell<Option<WorthQueryApplicationReadExecutionDenial>>,
}
impl<'a, 'subject> NativeReadInterruption<'a, 'subject> {
    pub fn new(
        interruption: ReadInterruption<'a>,
        subject: &'subject str,
    ) -> Self {
        Self {
            interruption,
            subject,
            first: RefCell::new(None),
        }
    }
    pub fn checkpoint(
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
    pub fn remember(
        &self,
        denial: &WorthQueryApplicationReadExecutionDenial,
    ) {
        self.first
            .borrow_mut()
            .get_or_insert_with(|| denial.clone());
    }
    pub fn denial(
        &self,
    ) -> Option<WorthQueryApplicationReadExecutionDenial> {
        self.first.borrow().clone()
    }
}
