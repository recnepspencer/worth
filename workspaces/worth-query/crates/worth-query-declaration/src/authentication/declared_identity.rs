//! Declared authentication vocabulary: audiences, methods and principal
//! issuers are application types, never strings at the call site.
//!
//! Declare each with its macro; the identity is the declared type name:
//!
//! ```
//! use worth_query_declaration::facade::authentication::*;
//! worth_query_declaration::worth_query_authentication_audience!(pub Board);
//! worth_query_declaration::worth_query_authentication_method!(pub LocalProcess);
//! worth_query_declaration::worth_query_principal_issuer!(pub BoardLocal);
//!
//! assert_eq!(Board::AUDIENCE, "Board");
//! assert_eq!(LocalProcess::METHOD, "LocalProcess");
//! let owner = WorthQueryExternalPrincipalIdentity::new(BoardLocal, "owner").unwrap();
//! assert_eq!(owner.issuer(), "BoardLocal");
//! ```
//!
//! An issuer string does not compile:
//!
//! ```compile_fail,E0277
//! use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
//! let _ = WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "owner");
//! ```

/// An accepted authentication audience declared with
/// [`crate::worth_query_authentication_audience!`].
pub trait WorthQueryAuthenticationAudienceIdentity: Copy + 'static {
    /// The declared audience: the audience type's name.
    const AUDIENCE: &'static str;
}

/// An accepted authentication method declared with
/// [`crate::worth_query_authentication_method!`].
pub trait WorthQueryAuthenticationMethodIdentity: Copy + 'static {
    /// The declared method: the method type's name.
    const METHOD: &'static str;
}

/// An external principal issuer declared with
/// [`crate::worth_query_principal_issuer!`].
pub trait WorthQueryPrincipalIssuerIdentity: Copy + 'static {
    /// The declared issuer: the issuer type's name.
    const ISSUER: &'static str;
}

/// Whether `value` is a non-empty declared vocabulary name of at most
/// `maximum_bytes` without surrounding whitespace or ASCII control bytes.
/// Typed constructors evaluate it at compile time, so a hand-written
/// identity with an unusable name fails to build.
pub const fn is_declared_vocabulary(value: &str, maximum_bytes: usize) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > maximum_bytes {
        return false;
    }
    if bytes[0].is_ascii_whitespace() || bytes[bytes.len() - 1].is_ascii_whitespace() {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_control() {
            return false;
        }
        index += 1;
    }
    true
}
