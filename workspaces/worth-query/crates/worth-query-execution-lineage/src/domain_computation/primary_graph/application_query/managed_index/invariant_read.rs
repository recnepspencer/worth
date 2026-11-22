//! Pure invariant-index read denial and posture types.

use super::WorthQueryManagedIndexWorkDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjectionDenial;
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding},
    application_schema::ApplicationStructuredValueBinding,
};
use worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial as NativeDenial;

pub type ManagedSourceValue<S, I> = <<<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

#[derive(Clone, Copy)]
pub enum ReadPosture {
    Committed,
    Proposed,
}

#[derive(Debug)]
pub enum WorthQueryManagedIndexInvariantDenial {
    Native(NativeDenial),
    Probe(worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial),
    Provider(WorthQueryManagedIndexWorkDenial),
    Projection(WorthQueryApplicationProjectionDenial),
}

impl From<NativeDenial> for WorthQueryManagedIndexInvariantDenial {
    fn from(cause: NativeDenial) -> Self {
        Self::Native(cause)
    }
}

impl std::fmt::Display for WorthQueryManagedIndexInvariantDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for WorthQueryManagedIndexInvariantDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provider(cause) => Some(cause),
            Self::Native(NativeDenial::Provider(cause)) => Some(cause),
            _ => None,
        }
    }
}

impl WorthQueryManagedIndexInvariantDenial {
    pub fn source_unavailable(&self) -> bool {
        matches!(self, Self::Native(cause) if cause.source_unavailable())
    }
}
