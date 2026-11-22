//! The binding set of fragments whose fields are all native scalars.

crate::worth_query_native_scalar_bindings!(
    /// Stock codecs for every native scalar carrier and nothing else.
    ///
    /// Use it as `Fragment::Bindings` when no field needs a custom,
    /// unit-bearing, principal, content or structured codec. Otherwise
    /// declare the fragment's own set with `worth_query_native_scalar_bindings!`
    /// and add those `Scalar<T>` impls beside the native ones.
    pub NativeScalars
);
