//! Original bounded graph and retention inspection requests used by these courts.
use worth_relational::facade::inspection::{
    ConnectivityInspectionBudget, GraphInspectionBudget, RetentionInspectionRequest,
};
pub(super) fn graph_budget() -> GraphInspectionBudget {
    GraphInspectionBudget {
        max_entities: 1024,
        max_relations: 1024,
        max_work_units: 4096,
    }
}
pub(super) fn connectivity_budget() -> ConnectivityInspectionBudget {
    ConnectivityInspectionBudget {
        max_entities: 1024,
        max_relations: 1024,
        max_frontier: 1024,
        max_components: 1024,
        max_work_units: 8192,
    }
}
pub(super) fn retention_request() -> RetentionInspectionRequest {
    RetentionInspectionRequest {
        max_entity_slots_scanned: 1024,
        max_relation_slots_scanned: 1024,
        max_work_units: 4096,
    }
}
