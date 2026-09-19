use crate::application_schema::ApplicationStructuredValueBinding;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

/// Declaration-owned stable identities for every Rust marker carried by a query.
pub trait ApplicationQueryMarkerIdentity<Schema> {
    type ParameterBinding: ApplicationStructuredValueBinding;
    type ResultBinding: ApplicationStructuredValueBinding;
    type Scope;

    const IDENTIFIER: &'static str;
    const QUERY_TYPE_NAME: &'static str;
    const SCOPE_TYPE_NAME: &'static str;
    const QUERY_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared(Self::QUERY_TYPE_NAME);
    const PARAMETER_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        <Self::ParameterBinding as ApplicationStructuredValueBinding>::IDENTITY;
    const RESULT_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        <Self::ResultBinding as ApplicationStructuredValueBinding>::IDENTITY;
    const SCOPE_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared(Self::SCOPE_TYPE_NAME);
}
