//! Accepted authentication audience and method values, built only from
//! declared vocabulary types.
use worth_query_declaration::facade::authentication::{
    is_declared_vocabulary, WorthQueryAuthenticationAudienceIdentity,
    WorthQueryAuthenticationMethodIdentity,
};

const MAX_AUDIENCE_BYTES: usize = 1_024;
const MAX_METHOD_BYTES: usize = 128;

/// The audience an adapter accepts and a validated principal was issued for.
///
/// ```compile_fail,E0277
/// use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticationAudience;
/// let _ = WorthQueryAuthenticationAudience::of("board");
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryAuthenticationAudience(&'static str);

impl WorthQueryAuthenticationAudience {
    /// The declared `Audience` ([`worth_query_authentication_audience!`]).
    ///
    /// [`worth_query_authentication_audience!`]: worth_query_declaration::worth_query_authentication_audience
    pub const fn of<Audience: WorthQueryAuthenticationAudienceIdentity>(_: Audience) -> Self {
        const {
            assert!(
                is_declared_vocabulary(Audience::AUDIENCE, MAX_AUDIENCE_BYTES),
                "a declared authentication audience names a non-empty bounded identity"
            )
        };
        Self(Audience::AUDIENCE)
    }

    pub const fn as_str(&self) -> &str {
        self.0
    }
}

/// The method an adapter accepts and a validated principal authenticated by.
///
/// ```compile_fail,E0277
/// use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticationMethod;
/// let _ = WorthQueryAuthenticationMethod::of("local-process");
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryAuthenticationMethod(&'static str);

impl WorthQueryAuthenticationMethod {
    /// The declared `Method` ([`worth_query_authentication_method!`]).
    ///
    /// [`worth_query_authentication_method!`]: worth_query_declaration::worth_query_authentication_method
    pub const fn of<Method: WorthQueryAuthenticationMethodIdentity>(_: Method) -> Self {
        const {
            assert!(
                is_declared_vocabulary(Method::METHOD, MAX_METHOD_BYTES),
                "a declared authentication method names a non-empty bounded identity"
            )
        };
        Self(Method::METHOD)
    }

    pub const fn as_str(&self) -> &str {
        self.0
    }
}
