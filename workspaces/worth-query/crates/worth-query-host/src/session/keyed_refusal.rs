//! The missing or already-held entity key behind a refused session run.
use worth_query_execution::facade::primary_graph::WorthQueryKeyedEntityRefusal;
use worth_query_publication::facade::application_entry::WorthQueryApplicationRequestMutationDenial as Request;

use super::WorthQuerySessionRunDenial;

impl<DomainDenial> WorthQuerySessionRunDenial<DomainDenial> {
    /// The entity type, field and key a `NotFound` or `AlreadyExists` refusal
    /// names: a handler's `resolve_entity` miss, an unresolved scope key, or a
    /// unique-key collision. `None` for every other refusal.
    ///
    /// For a message, `field_label()` names the field in words with its entity
    /// once, even when the declared name repeats the entity:
    ///
    /// ```
    /// use worth_query_host::facade::primary_graph::WorthQueryKeyedEntityRefusal;
    /// let order = WorthQueryKeyedEntityRefusal::not_found("Order", "OrderOrderId", None);
    /// assert_eq!(order.field(), "OrderOrderId");
    /// assert_eq!(order.field_label(), "order id");
    /// let sku = WorthQueryKeyedEntityRefusal::already_exists("Item", "SKUCode", None);
    /// assert_eq!(sku.field_label(), "item sku code");
    /// ```
    pub fn keyed_entity(&self) -> Option<&WorthQueryKeyedEntityRefusal> {
        match self {
            Self::Request(Request::Handler(denial)) => denial.keyed_entity(),
            Self::Request(Request::ScopeResolution(denial)) => denial.keyed_entity(),
            Self::Uncommitted(
                worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted::Denied(
                    denial,
                ),
            ) => denial.keyed_entity(),
            _ => None,
        }
    }
}
