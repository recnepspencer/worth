//! Visibility for constructors that certification code outside the Bridge
//! may call.
//!
//! A constructor wrapped in `certification_constructor!` is `pub` when the
//! `certification-construction` feature is on and `pub(crate)` otherwise. The
//! Bridge's own callers need no feature; a downstream crate reaches the
//! constructor only by enabling the feature, which Query does through a
//! dev-dependency.

macro_rules! certification_constructor {
    ($(#[$meta:meta])* fn $($item:tt)*) => {
        #[cfg(feature = "certification-construction")]
        $(#[$meta])*
        pub fn $($item)*

        #[cfg(not(feature = "certification-construction"))]
        $(#[$meta])*
        pub(crate) fn $($item)*
    };
}

pub(crate) use certification_constructor;
