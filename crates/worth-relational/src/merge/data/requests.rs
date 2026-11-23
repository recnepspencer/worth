pub(crate) mod binding_denial;
#[path = "requests/denial.rs"]
pub(crate) mod denial;
#[path = "requests/digest.rs"]
pub(crate) mod digest;
#[path = "requests/foundational.rs"]
mod foundational;
#[path = "requests/normalized.rs"]
pub(crate) mod normalized;
#[path = "requests/owner_bound.rs"]
mod owner_bound;
#[path = "requests/raw.rs"]
pub(crate) mod raw;

pub use denial::RelationalMergeRequestNormalizationDenial;
pub use foundational::RelationalFoundationalMergeRequest;
pub use normalized::{
    NormalizedRelationalMergeRequest, RelationalMergeCorrespondencePosture,
    RelationalMergeRequestFamily, RelationalMergeSchemaReconciliationPosture, RelationalMergeScope,
    RelationalMergeTopologyIntent,
};
pub use owner_bound::{
    OwnerBoundMergeExecutionRequest, OwnerBoundMergePlanningRequest,
    RelationalMergeRequestBindingDenial,
};
pub use raw::{MergeExecutionRequest, MergeIntent, MergePlanningRequest};
