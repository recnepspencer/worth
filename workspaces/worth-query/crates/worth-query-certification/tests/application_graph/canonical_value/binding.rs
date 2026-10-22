//! Entry-owned canonical carrier binding for the Query-free authored value.
use super::model::NestedValue;
use worth_query_decl::facade::worth_query_value_binding;

worth_query_value_binding! {
    pub NestedBinding for NestedValue {
        identity: "worth.query.certification.canonical-nested.v1",
        scalar: String,
        canonical,
    }
}
