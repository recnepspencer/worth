//! Crate facade for primary-graph application attempts, provider custody,
//! authorization, workflow semantics, aftermath, and live delivery.

pub mod domain_computation {
    pub use crate::domain_computation::{
        application_aftermath, authorization, primary_graph, provider_session, runtime_ports,
    };
}
pub use crate::domain_computation::{application_aftermath, authorization, primary_graph};
pub use crate::lineage_ports::{
    CommitAuthorityBindingExt, PrimaryGraphHandleWorkflowExt, RetainedPreImageExt,
    WorkflowRetentionStore,
};
pub mod ports {
    pub use crate::ports::*;
}
