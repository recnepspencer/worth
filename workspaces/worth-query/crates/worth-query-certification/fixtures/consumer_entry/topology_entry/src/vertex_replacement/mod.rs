mod binding;
mod declaration;
mod handler;
mod identity;

pub use binding::{VertexReplacementBinding, VertexReplacementOutputs};
pub(crate) use declaration::declare_vertex_replacement;
pub use handler::VertexReplacementHandler;

use super::{Body, BodyKey, Length, PlanarSuccessor, PositionX, PositionY, TopologySchemaBinding};
use worth_query_consumer_values::{
    PlanarReplacementDenial, PlanarVertexReplacement, PlanarVertexReplacementResult,
};
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_creates, worth_query_operation_deletes,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_unlinks,
    worth_query_operation_writes, worth_query_structured_value_binding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VertexReplacement {
    pub scope_key: String,
    pub replacement: PlanarVertexReplacement,
}

worth_query_structured_value_binding!(pub VertexReplacementInputBinding for VertexReplacement {
    identity: "worth.query.certification.vertex-replacement-input.v1"
});
worth_query_structured_value_binding!(pub VertexReplacementResultBinding for PlanarVertexReplacementResult {
    identity: "worth.query.certification.vertex-replacement-result.v1"
});
worth_query_structured_value_binding!(pub VertexReplacementDenialBinding for PlanarReplacementDenial {
    identity: "worth.query.certification.vertex-replacement-denial.v1"
});
worth_query_operation!(pub ReplacePlanarVertex for Schema: TopologySchemaBinding, input VertexReplacementInputBinding);
worth_query_operation_reads!(ReplacePlanarVertex => [Body, BodyKey, PositionX, PositionY, PlanarSuccessor]);
worth_query_operation_writes!(ReplacePlanarVertex => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_creates!(ReplacePlanarVertex => [Body]);
worth_query_operation_deletes!(ReplacePlanarVertex => [Body]);
worth_query_operation_links!(ReplacePlanarVertex => [PlanarSuccessor]);
worth_query_operation_unlinks!(ReplacePlanarVertex => [PlanarSuccessor]);
