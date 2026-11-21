pub use worth_query_execution_lineage::facade::primary_graph::invariant_projection::*;

pub mod operation_reader {
    use std::marker::PhantomData;
    use std::sync::Arc;
    use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
    use worth_query_installation::facade::{
        ApplicationReadableScalarValueBinding, ApplicationRelationRef, ApplicationSchema,
        OperationReads,
    };
    use crate::domain_computation::primary_graph::{
        WorthQueryInvariantEntityIdentity, WorthQueryInvariantProjectionDenial,
        WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantRelation,
    };
    pub use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::*;
    pub mod managed_index;
    pub mod original;
    pub mod output_availability;
    pub mod predecessor;
    pub use predecessor::{BeforeDecisionReader, BeforeEntity, BeforeRelation};
}
pub use operation_reader::predecessor::{BeforeDecisionReader, BeforeEntity, BeforeRelation};
