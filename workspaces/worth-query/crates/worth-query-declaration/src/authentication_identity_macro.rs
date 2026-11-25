//! Declaration macros for typed authentication vocabulary.

/// Declares an authentication audience: `vis Audience` generates a unit
/// struct implementing `WorthQueryAuthenticationAudienceIdentity`, whose
/// audience is the type name.
#[macro_export]
macro_rules! worth_query_authentication_audience {
    ($(#[$meta:meta])* $vis:vis $Audience:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Audience;
        impl $crate::facade::authentication::WorthQueryAuthenticationAudienceIdentity for $Audience {
            const AUDIENCE: &'static str = ::core::stringify!($Audience);
        }
    };
}

/// Declares an authentication method: `vis Method` generates a unit struct
/// implementing `WorthQueryAuthenticationMethodIdentity`, whose method is the
/// type name.
#[macro_export]
macro_rules! worth_query_authentication_method {
    ($(#[$meta:meta])* $vis:vis $Method:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Method;
        impl $crate::facade::authentication::WorthQueryAuthenticationMethodIdentity for $Method {
            const METHOD: &'static str = ::core::stringify!($Method);
        }
    };
}

/// Declares an external principal issuer: `vis Issuer` generates a unit
/// struct implementing `WorthQueryPrincipalIssuerIdentity`, whose issuer is
/// the type name. Authentication adapters map a verified credential's issuer
/// onto one of these declared types.
#[macro_export]
macro_rules! worth_query_principal_issuer {
    ($(#[$meta:meta])* $vis:vis $Issuer:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Issuer;
        impl $crate::facade::authentication::WorthQueryPrincipalIssuerIdentity for $Issuer {
            const ISSUER: &'static str = ::core::stringify!($Issuer);
        }
    };
}
