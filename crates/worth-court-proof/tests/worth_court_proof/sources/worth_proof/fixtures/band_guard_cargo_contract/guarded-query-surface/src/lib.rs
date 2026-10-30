//! Query-shaped macro surface used to prove the band guard travels with an export.

#[doc(hidden)]
pub use worth_proof::band_guard as __band_guard;

#[macro_export]
macro_rules! guarded_query_surface {
    () => {
        $crate::__band_guard!("worth-entry-", "worthy-entry-");
    };
}

#[macro_export]
macro_rules! guarded_cert_surface {
    () => {
        $crate::__band_guard!("worth-cert-", "worthy-cert-");
    };
}
