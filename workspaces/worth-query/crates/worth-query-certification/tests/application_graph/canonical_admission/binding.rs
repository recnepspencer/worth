//! Entry-owned canonical binding, including its explicit opaque admitted delegate.
use super::opaque::SwallowingValue;
use worth_query_decl::facade::worth_query_value_binding;
worth_query_value_binding! {
    pub NestedBinding for SwallowingValue {
        identity: "worth.query.certification.canonical-admission.swallowing.v1",
        scalar: String,
        canonical,
    }
}
