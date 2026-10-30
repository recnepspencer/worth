//! A candidate read reborrows the actual observation and original request each time.
mod projection;
use super::{
    invocation::decoded_provider, source::projection::project_row,
    source::service::ManagedSourceRoot, WorthQueryManagedIndexProbeValue,
    WorthQueryManagedIndexWorkDenial,
};
use crate::domain_computation::primary_graph::application_query::{
    disclosure::WorthQueryApplicationQueryGovernance, WorthQueryApplicationProjection,
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
};
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding},
    application_schema::{
        ApplicationScalarValueBinding, ApplicationSchema, ApplicationStructuredValueBinding,
    },
};
use worth_relational::facade::runtime::{
    ManagedIndexSourcePreparationDenial as NativeDenial, ManagedIndexSourceRead,
};

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

pub(in crate::domain_computation::primary_graph) fn query<S, I>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &ManagedIndexSourceRead<'_>,
    program: &worth_relational::facade::runtime::ManagedIndexSourceProgram,
    posture: ReadPosture,
    probe: &WorthQueryManagedIndexProbeValue<S, I>,
) -> Result<Vec<ManagedSourceValue<S, I>>, WorthQueryManagedIndexInvariantDenial>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    ManagedSourceValue<S, I>:
        WorthQueryApplicationProjection<S, <I::SourceBinding as ApplicationQueryBinding<S>>::Query>,
{
    use WorthQueryManagedIndexInvariantDenial as Denial;
    let payload = match posture {
        ReadPosture::Committed => source.committed_root()?,
        ReadPosture::Proposed => source.proposed_root()?,
    };
    let root = payload
        .downcast_ref::<ManagedSourceRoot<S, I>>()
        .ok_or(NativeDenial::ForeignSource)?;
    let rows = match posture {
        ReadPosture::Committed => source.committed()?,
        ReadPosture::Proposed => source.proposed(),
    };
    let budget = source.read_budget();
    let probe = I::ProbeBinding::encode(probe).map_err(Denial::Probe)?;
    let slots = root
        .adapter
        .candidates(
            execution,
            root.rows.view(&root.bindings, Some(&budget)),
            root.nodes.as_ref(),
            &probe,
        )
        .map_err(Denial::Provider)?;
    projection::read::<S, I>(execution, source, program, root, &rows, slots.elements())
}
