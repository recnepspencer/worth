//! Closed native provenance encodings; supplier codecs cannot forge an authentication initializer.
use super::{
    ApplicationSigningGraphSubjectProvenanceBinding, ApplicationSigningProvenanceBinding,
    ApplicationSigningSubjectProvenanceBinding,
};
use crate::{
    application_schema::ApplicationScalarValueBinding,
    portable_identity::WorthQueryPortableTypeIdentity,
};

/// The authenticated record either predates or carries durable certified dependency correspondence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationSigningProvenanceEncoding {
    AuthenticationV1,
    CertifiedSubjectV2,
    CertifiedGraphSubjectV3,
}
impl ApplicationSigningProvenanceEncoding {
    pub fn for_binding(binding: &WorthQueryPortableTypeIdentity) -> Option<Self> {
        if *binding == ApplicationSigningProvenanceBinding::IDENTITY {
            Some(Self::AuthenticationV1)
        } else if *binding == ApplicationSigningSubjectProvenanceBinding::IDENTITY {
            Some(Self::CertifiedSubjectV2)
        } else if *binding == ApplicationSigningGraphSubjectProvenanceBinding::IDENTITY {
            Some(Self::CertifiedGraphSubjectV3)
        } else {
            None
        }
    }
}
mod sealed {
    pub trait Native {}
}
impl sealed::Native for ApplicationSigningProvenanceBinding {}
impl sealed::Native for ApplicationSigningSubjectProvenanceBinding {}
impl sealed::Native for ApplicationSigningGraphSubjectProvenanceBinding {}

/// Only the native, immutable provenance codecs can occupy a `signed_by` slot.
///
/// ```
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationSigningProvenanceValueBinding, ApplicationSigningProvenanceBinding,
///     ApplicationSigningSubjectProvenanceBinding,
/// };
/// fn native<B: ApplicationSigningProvenanceValueBinding>() {}
/// native::<ApplicationSigningProvenanceBinding>();
/// native::<ApplicationSigningSubjectProvenanceBinding>();
/// ```
///
/// An ordinary supplier scalar cannot replace the native deferred initializer:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationSigningProvenanceValueBinding, StringApplicationValueBinding,
/// };
/// fn native<B: ApplicationSigningProvenanceValueBinding>() {}
/// native::<StringApplicationValueBinding>();
/// ```
pub trait ApplicationSigningProvenanceValueBinding:
    ApplicationScalarValueBinding<Unit = ()> + sealed::Native
{
}
impl ApplicationSigningProvenanceValueBinding for ApplicationSigningProvenanceBinding {}
impl ApplicationSigningProvenanceValueBinding for ApplicationSigningSubjectProvenanceBinding {}
impl ApplicationSigningProvenanceValueBinding for ApplicationSigningGraphSubjectProvenanceBinding {}
