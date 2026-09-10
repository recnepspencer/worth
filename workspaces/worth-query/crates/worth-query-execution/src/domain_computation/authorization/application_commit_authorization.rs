//! Move-only authorization for one exact product-branch commit transition.

use std::marker::PhantomData;

use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationBranchCommitCoordination,
};

pub(in crate::domain_computation) struct WorthQueryApplicationCommitAuthorization<
    'serialization,
    'admission,
    Schema,
    Operation,
    Input,
    Scope,
> {
    admission: &'admission WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    _coordination: PhantomData<&'serialization ()>,
}

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
    pub(super) fn mint(
        _coordination: &'serialization WorthQueryApplicationBranchCommitCoordination<'_>,
        admission: &'admission WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> Self {
        Self {
            admission,
            _coordination: PhantomData,
        }
    }

    pub(in crate::domain_computation) fn govern<Subject, Outcome>(
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
        if let Err(denial) = self.admission.validate_current_authority() {
            return Err((subject, denial));
        }
        Ok(transition(subject))
    }
}
