use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use crate::application_query::WorthQueryInstalledApplicationQuery;
use crate::domain_operation::{
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentBounds,
    WorthQueryTemporalIntentCandidate, WorthQueryTemporalIntentProjectionFailure,
    WorthQueryTemporalIntentProjector,
};

use super::{
    WorthQueryConditionalApplicationOperationDenial,
    WorthQueryConditionalApplicationOperationDenialKind,
    WorthQueryInstalledNamedClockConditionalNode,
};

/// Complete move-only temporal installation contract for one exact application
/// operation and conditional node.
pub struct WorthQueryInstalledTemporalConditionalOperation<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    N,
    Provider,
    Clock,
    Source,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Projector,
> {
    clocked_node: WorthQueryInstalledNamedClockConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
        Clock,
        Source,
    >,
    query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
    projector: Projector,
    bounds: WorthQueryTemporalIntentBounds,
}

impl<Schema, ApplicationOperation, Input, D, O, F, N, Provider, Clock, Source>
    WorthQueryInstalledNamedClockConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
        Clock,
        Source,
    >
where
    Provider: crate::domain_operation::WorthQueryHostConditionalPredicateProvider<N>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
{
    pub fn bind_temporal_intent_projection<Query, Parameters, QueryResult, Scope, Projector>(
        self,
        query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        parameters: ApplicationQueryParameterSet<Query>,
        projector: Projector,
        bounds: WorthQueryTemporalIntentBounds,
    ) -> Result<
        WorthQueryInstalledTemporalConditionalOperation<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            N,
            Provider,
            Clock,
            Source,
            Query,
            Parameters,
            QueryResult,
            Scope,
            Projector,
        >,
        WorthQueryConditionalApplicationOperationDenial,
    >
    where
        Projector: WorthQueryTemporalIntentProjector<N, Clock, QueryResult, Input>,
    {
        validate_query_binding(self.provider().node(), &query)?;
        validate_projector_identity::<N, Clock, QueryResult, Input, Projector>()?;
        Ok(WorthQueryInstalledTemporalConditionalOperation {
            clocked_node: self,
            query,
            parameters,
            projector,
            bounds,
        })
    }
}

impl<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >
    WorthQueryInstalledTemporalConditionalOperation<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >
where
    Provider: crate::domain_operation::WorthQueryHostConditionalPredicateProvider<N>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Projector: WorthQueryTemporalIntentProjector<N, Clock, QueryResult, Input>,
{
    pub fn clocked_node(
        &self,
    ) -> &WorthQueryInstalledNamedClockConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
        Clock,
        Source,
    > {
        &self.clocked_node
    }

    pub fn query(
        &self,
    ) -> &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope> {
        &self.query
    }

    pub fn parameters(&self) -> &ApplicationQueryParameterSet<Query> {
        &self.parameters
    }

    pub fn bounds(&self) -> WorthQueryTemporalIntentBounds {
        self.bounds
    }

    pub fn projector_semantic_identity(&self) -> &'static str {
        Projector::SEMANTIC_IDENTITY
    }

    #[doc(hidden)]
    pub fn observe_clock_for_runtime(
        &self,
    ) -> Result<
        crate::domain_operation::WorthQueryNamedClockObservation<Clock>,
        crate::domain_operation::WorthQueryNamedClockFailure,
    > {
        self.clocked_node.observe_for_runtime()
    }

    #[doc(hidden)]
    pub fn project_for_runtime(
        &self,
        row: &QueryResult,
    ) -> Result<
        WorthQueryTemporalIntentCandidate<Clock, Input>,
        WorthQueryTemporalIntentProjectionFailure,
    > {
        self.projector.project(row)
    }
}

fn validate_query_binding<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    N,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    node: &super::WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
    >,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
    if node.operation().application_operation().binding_identity() != query.binding_identity() {
        Err(conditional_denial(
            WorthQueryConditionalApplicationOperationDenialKind::TemporalIntentQueryForeign,
            query.name(),
        ))
    } else {
        Ok(())
    }
}

fn validate_projector_identity<Node, Clock, QueryResult, Input, Projector>(
) -> Result<(), WorthQueryConditionalApplicationOperationDenial>
where
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
{
    let identity = Projector::SEMANTIC_IDENTITY;
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(conditional_denial(
            WorthQueryConditionalApplicationOperationDenialKind::TemporalIntentProjectorInvalid,
            identity,
        ))
    } else {
        Ok(())
    }
}

fn conditional_denial(
    kind: WorthQueryConditionalApplicationOperationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryConditionalApplicationOperationDenial {
    WorthQueryConditionalApplicationOperationDenial::new(kind, subject)
}
