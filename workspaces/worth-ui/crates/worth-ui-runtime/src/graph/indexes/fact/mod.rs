mod appearance_relation;
mod authored_declaration_lookup;
mod basis;
mod consumer;
mod consumer_key;
mod denial;
mod entry;
mod index;
mod intent_posture;
mod lookup;

pub(crate) use appearance_relation::UiGraphFactConsumptionRelation;
pub(crate) use authored_declaration_lookup::UiAuthoredDeclarationLookup;
pub use basis::UiGraphFactIndexBasis;
pub use consumer::UiGraphFactConsumerIdentity;
pub use consumer_key::{UiGraphFactConsumerKey, UiGraphFactConsumerKind};
pub use denial::UiGraphFactLookupDenial;
pub use entry::UiGraphFactIndexEntry;
pub use index::UiGraphConsumedFactIndex;
#[cfg(any(test, feature = "certification-support"))]
pub use lookup::UiGraphFactLookupCost;
pub use lookup::UiGraphFactLookupReceipt;

#[cfg(test)]
mod projection_routing_tests;
#[cfg(test)]
mod tests;
