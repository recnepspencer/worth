//! Move-only authorization for one serialized application commit transition.

use std::marker::PhantomData;

use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitSerialization,
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
    _serialization: PhantomData<&'serialization ()>,
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
        _serialization: &'serialization WorthQueryApplicationCommitSerialization<'_>,
        admission: &'admission WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> Self {
        Self {
            admission,
            _serialization: PhantomData,
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
