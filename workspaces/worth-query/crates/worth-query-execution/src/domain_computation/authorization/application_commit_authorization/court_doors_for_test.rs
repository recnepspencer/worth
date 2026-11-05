//! Sealed test doors over this module's private members.
use super::*;

impl<'serialization, 'admission, Schema, Operation, Input, Scope>
    WorthQueryApplicationCommitAuthorization<
        'serialization,
        'admission,
        Schema,
        Operation,
        Input,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn govern_for_test<Subject, Outcome>(
        self,
        subject: Subject,
        transition: impl FnOnce(Subject) -> Outcome,
    ) -> Result<
        Outcome,
        (
            Subject,
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
        ),
    > {
        Self::govern(self, subject, transition)
    }
}
