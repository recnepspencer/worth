//! The keyed-entity evidence a handler stop carries, found through its
//! complete error chain so an application's own wrapper keeps it visible.
use super::HandlerExecutionDenial;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationAttemptDenial, WorthQueryEntityResolutionDenial,
    WorthQueryKeyedEntityRefusal,
};

impl HandlerExecutionDenial {
    /// The missing or already-held entity key this stop names, from a
    /// `resolve_entity` miss, a unique-key collision, or an application error
    /// whose `source()` chain reaches either.
    pub fn keyed_entity(&self) -> Option<&WorthQueryKeyedEntityRefusal> {
        let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(self.source.as_ref());
        while let Some(current) = cause {
            if let Some(keyed) = current.downcast_ref::<WorthQueryKeyedEntityRefusal>() {
                return Some(keyed);
            }
            let named = current
                .downcast_ref::<WorthQueryEntityResolutionDenial>()
                .and_then(WorthQueryEntityResolutionDenial::keyed_entity)
                .or_else(|| {
                    current
                        .downcast_ref::<WorthQueryApplicationAttemptDenial>()
                        .and_then(WorthQueryApplicationAttemptDenial::keyed_entity)
                });
            if named.is_some() {
                return named;
            }
            cause = current.source();
        }
        None
    }
}

impl super::MutationHandlerExecutionDenial {
    /// The missing or already-held entity key a handler or attempt stop names.
    pub fn keyed_entity(&self) -> Option<&WorthQueryKeyedEntityRefusal> {
        match self {
            Self::Handler(denial) => denial.keyed_entity(),
            Self::Attempt(denial) => denial.keyed_entity(),
            _ => None,
        }
    }
}
