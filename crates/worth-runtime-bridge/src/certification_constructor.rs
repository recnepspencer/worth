//! Public certification constructors whose consumers are sealed by the named
//! reference rule in tools/boundary-check.

macro_rules! certification_constructor {
    ($(#[$meta:meta])* fn $($item:tt)*) => {
        $(#[$meta])*
        pub fn $($item)*
    };
}

pub(crate) use certification_constructor;
