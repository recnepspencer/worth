pub use crate::contracts::merge::data::requests::raw;
#[path = "requests/owner_bound.rs"]
pub mod owner_bound;

pub use crate::contracts::merge::data::requests::denial::RelationalMergeRequestNormalizationDenial;
pub use crate::contracts::merge::data::requests::foundational::RelationalFoundationalMergeRequest;
pub use crate::contracts::merge::data::requests::normalized::{
    NormalizedRelationalMergeRequest, RelationalMergeCorrespondencePosture,
    RelationalMergeRequestFamily, RelationalMergeSchemaReconciliationPosture, RelationalMergeScope,
    RelationalMergeTopologyIntent,
};
pub use crate::contracts::merge::data::requests::raw::{
    MergeExecutionRequest, MergeIntent, MergePlanningRequest,
};
pub use owner_bound::{
    OwnerBoundMergeExecutionRequest, OwnerBoundMergePlanningRequest,
    RelationalMergeRequestBindingDenial,
};
