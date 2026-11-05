//! Moved whole from `domain_computation::primary_graph::application_query::managed_index::invariant_read`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial;
use crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenial;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial as NativeDenial;

pub type ManagedSourceValue<S, I> = <<<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum ReadPosture {
    Committed,
    Proposed,
}

#[derive(Debug)]
/// An invariant index read failed at its Native loan, probe, provider or projection boundary.
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
    /// No material was available for this particular read posture. This is
    /// diagnostic only; an unavailable Source never means empty membership.
    pub fn source_unavailable(&self) -> bool {
        matches!(self, Self::Native(cause) if cause.source_unavailable())
    }
}
