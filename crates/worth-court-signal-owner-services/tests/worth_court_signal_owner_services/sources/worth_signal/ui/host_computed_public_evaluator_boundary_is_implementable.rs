use worth_signal::facade::{
    Aspect, AspectVersion, DependencyEdge, HostComputedEvaluationRequest,
    HostComputedEvaluationResponse, HostComputedEvaluator, HostComputedFailureClass,
    HostComputedPreparedResponse, NodeEvaluationResult, NodeId, SignalError,
};

struct ExternalEvaluator;

impl HostComputedEvaluator for ExternalEvaluator {
    type Context = ();

    fn evaluate(
        &mut self,
        _request: &HostComputedEvaluationRequest,
        _ctx: &mut Self::Context,
    ) -> Result<HostComputedEvaluationResponse, SignalError> {
        let prepared = HostComputedPreparedResponse::from_result(
            NodeEvaluationResult::from_version(AspectVersion::zero()),
            [DependencyEdge::new(NodeId::new(7, 0), Aspect::new(0))],
        );
        let _typed_failure = HostComputedEvaluationResponse::failed(
            HostComputedFailureClass::HostAdapterRejected,
            "simulated callback failure",
        );
        Ok(HostComputedEvaluationResponse::prepared(prepared))
    }
}

fn main() {}
