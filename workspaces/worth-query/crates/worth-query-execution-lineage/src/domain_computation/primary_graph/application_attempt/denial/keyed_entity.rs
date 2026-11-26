//! A unique-key collision keeps the entity type, field and key it names.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use crate::domain_computation::primary_graph::WorthQueryKeyedEntityRefusal;
impl Denial {
    /// A write names a unique value another entity holds, or names it twice.
    pub fn unique_value_taken(subject: String, keyed: WorthQueryKeyedEntityRefusal) -> Self {
        Self::from_cause(
            Kind::UniqueValueTaken,
            subject,
            AttemptDenialCause::KeyedEntity(Box::new(keyed)),
        )
    }
    /// The entity type, field and key a keyed refusal names, when known.
    pub fn keyed_entity(&self) -> Option<&WorthQueryKeyedEntityRefusal> {
        match self.cause() {
            Some(AttemptDenialCause::KeyedEntity(keyed)) => Some(keyed),
            _ => None,
        }
    }
}
