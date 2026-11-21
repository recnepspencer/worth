use std::collections::BTreeMap;
use worth_query_execution_query::facade::ports::ConditionalRuntimeInstallationDenialInternalPort as _;

use worth_query_installation::facade::{
    WorthQueryHostConditionalPredicateProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentCandidate,
    WorthQueryTemporalIntentProjector,
};

use super::installation::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};

pub(super) fn project_unique_candidates<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
    Provider,
    Clock,
    Source,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Projector,
>(
    binding: &WorthQueryInstalledTemporalConditionalOperation<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >,
    rows: Vec<QueryResult>,
) -> Result<
    BTreeMap<String, WorthQueryTemporalIntentCandidate<Clock, Input>>,
    WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
{
    let mut candidates = BTreeMap::new();
    for row in &rows {
        let candidate = isolate_projector(|| binding.project_for_runtime(row))??;
        let identity = candidate.identity().as_str().to_string();
        if candidates.insert(identity.clone(), candidate).is_some() {
            return Err(projection_denial(format!(
                "duplicate temporal intent identity: {identity}"
            )));
        }
    }
    Ok(candidates)
}

fn isolate_projector<Output, Failure>(
    project: impl FnOnce() -> Result<Output, Failure>,
) -> Result<
    Result<Output, WorthQueryConditionalRuntimeInstallationDenial>,
    WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Failure: TemporalProjectionFailure,
{
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(project))
        .map(|result| result.map_err(|failure| projection_denial(failure.detail())))
        .map_err(|_| projection_denial("installed temporal-intent projector panicked"))
}

trait TemporalProjectionFailure {
    fn detail(&self) -> String;
}

impl TemporalProjectionFailure
    for worth_query_installation::facade::WorthQueryTemporalIntentProjectionFailure
{
    fn detail(&self) -> String {
        format!("{:?}: {}", self.kind(), self.detail())
    }
}

fn projection_denial(detail: impl Into<String>) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::from_kind_and_subject(
        WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionProjection,
        detail,
    )
}
