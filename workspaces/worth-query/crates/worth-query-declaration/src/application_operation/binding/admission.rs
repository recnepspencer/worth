//! Binding-selected native admission; these markers carry no permission.
use super::ApplicationMutationBinding;
use crate::application_schema::ApplicationSchema;
use std::marker::PhantomData;
/// Selects native conventional admission; this marker grants no authority.
pub struct ApplicationConventionalAdmission;
/// Selects native admission through capability `C`; each attempt requires fresh authority.
pub struct ApplicationCapabilityAdmission<C>(PhantomData<fn() -> C>);
mod sealed {
    pub trait Mode {}
}
impl sealed::Mode for ApplicationConventionalAdmission {}
impl<C> sealed::Mode for ApplicationCapabilityAdmission<C> {}
/// Sealed choices of the existing native admission engine.
pub trait ApplicationMutationAdmissionMode: sealed::Mode + 'static {}
impl ApplicationMutationAdmissionMode for ApplicationConventionalAdmission {}
impl<C: 'static> ApplicationMutationAdmissionMode for ApplicationCapabilityAdmission<C> {}
/// Associates a mutation binding with its native admission mode, without granting permission.
pub trait ApplicationMutationAdmissionBinding<S: ApplicationSchema>:
    ApplicationMutationBinding<S>
{
    type Admission: ApplicationMutationAdmissionMode;
}
