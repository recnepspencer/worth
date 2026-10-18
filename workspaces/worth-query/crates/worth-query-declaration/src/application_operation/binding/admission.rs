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
pub trait ApplicationMutationAdmissionMode: sealed::Mode + 'static {
    /// Descriptive installed contract of the sealed admission choice; no permission.
    fn capability_identity() -> Option<(
        &'static str,
        crate::portable_identity::WorthQueryPortableTypeIdentity,
    )>;
}
impl ApplicationMutationAdmissionMode for ApplicationConventionalAdmission {
    fn capability_identity() -> Option<(
        &'static str,
        crate::portable_identity::WorthQueryPortableTypeIdentity,
    )> {
        None
    }
}
impl<C: crate::application_capability::ApplicationCapabilityMarkerIdentity + 'static>
    ApplicationMutationAdmissionMode for ApplicationCapabilityAdmission<C>
{
    fn capability_identity() -> Option<(
        &'static str,
        crate::portable_identity::WorthQueryPortableTypeIdentity,
    )> {
        Some((C::IDENTIFIER, C::PORTABLE_TYPE_IDENTITY))
    }
}
/// Associates a mutation binding with its native admission mode, without granting permission.
pub trait ApplicationMutationAdmissionBinding<S: ApplicationSchema>:
    ApplicationMutationBinding<S>
{
    type Admission: ApplicationMutationAdmissionMode;
}
