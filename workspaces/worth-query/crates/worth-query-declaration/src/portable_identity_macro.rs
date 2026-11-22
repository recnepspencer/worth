/// Declares the stable semantic identity of a Rust type used in portable Query
/// meaning. Moving the type between modules does not alter this identity.
/// The identity is any `&'static str` constant expression, such as a
/// literal or `worth_query_identity!(NAMESPACE, slot "field")`.
#[macro_export]
macro_rules! worth_query_portable_type {
    ($Type:ty => $identity:expr $(,)?) => {
        impl $crate::facade::portable_identity::WorthQueryPortableType for $Type {
            const PORTABLE_TYPE_NAME: &'static str = $identity;
        }
    };
}
