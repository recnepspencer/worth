use std::sync::{Arc, OnceLock};
use worth_signal::facade::specialist::EvaluationOutput;
use worth_signal::facade::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteDefinition(pub u8);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CargoKey(pub u64);
pub struct CargoEvent;
pub struct RoutingContext {
    pub records: Arc<OnceLock<super::observation::SupplyChainObservation>>,
}
pub type CourtSignal = SignalRuntime<RouteDefinition, CargoKey, CargoEvent, RoutingContext, u64>;
pub const INPUT: Aspect = Aspect::new(0);
pub const ROUTED: Aspect = Aspect::new(1);
#[derive(Clone, Copy)]
pub struct RouteGraph {
    pub source: NodeId,
    pub route: NodeId,
}
impl RouteGraph {
    pub fn install(graph: &mut SignalGraph) -> Self {
        let source = graph.node().build();
        let route = graph.node().on_demand().build();
        graph
            .set_dependencies(route, [DependencyEdge::new(source, INPUT)])
            .expect("court declaration: routing dependency");
        Self { source, route }
    }
    pub fn evaluate(
        &self,
        view: &mut EvaluationContext<'_, RoutingContext>,
    ) -> Result<EvaluationOutput, SignalError> {
        let data = view
            .domain()
            .records
            .get()
            .expect("routing requires one exact Relational input snapshot");
        if view.node() == self.source {
            let load = data
                .links
                .iter()
                .filter(|(source, _)| source == "voyage")
                .flat_map(|(_, manifest)| {
                    data.links
                        .iter()
                        .filter(move |(source, _)| source == manifest)
                })
                .filter_map(|(_, cargo)| data.records[cargo].parse::<u64>().ok())
                .sum::<u64>();
            return Ok(view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(INPUT, load)]),
            )));
        }
        let load = view.read_aspect_version(self.source, INPUT)?.get(INPUT);
        let data = view
            .domain()
            .records
            .get()
            .expect("routing requires one exact Relational input snapshot");
        let destination_open = data
            .links
            .iter()
            .filter(|(source, _)| source == "voyage")
            .any(|(_, manifest)| {
                data.links.iter().any(|(source, target)| {
                    source == manifest
                        && data.records[manifest] == *target
                        && data.records[target] == "open"
                })
            });
        let total = if destination_open && load <= data.records["voyage"].parse::<u64>().unwrap() {
            load
        } else {
            0
        };
        Ok(view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(ROUTED, total)]),
        )))
    }
}
