//! Controlled provider mechanics over a genuine completed source image.
mod adapter;
mod build;
mod candidates;
mod configuration;
mod control;
mod denial;
mod execution;
mod handle;
mod invocation;
mod node_backing;
mod nodes;
mod provider;
mod provider_failure;
mod receiver;
pub(in crate::domain_computation::primary_graph::application_query) mod retention;
pub(in crate::domain_computation::primary_graph::application_query) mod selection;
mod source_acquisition;
mod source_completion;
mod source_dependencies;
mod source_image;
mod source_order;
mod source_rows;
mod stop;

pub use candidates::WorthQueryManagedIndexCandidateWriter;
pub(in crate::domain_computation::primary_graph) use configuration::ManagedIndexProviderCatalog;
pub use control::WorthQueryManagedIndexControl;
pub use denial::{
    WorthQueryManagedIndexDenial, WorthQueryManagedIndexDenialCause, WorthQueryManagedIndexRefusal,
};
pub use handle::WorthQueryManagedIndex;
pub use invocation::WorthQueryManagedIndexWorkDenial;
pub use nodes::{
    WorthQueryManagedIndexNodeId, WorthQueryManagedIndexNodeWriter, WorthQueryManagedIndexNodes,
};
pub use provider::{WorthQueryManagedIndexProbeValue, WorthQueryManagedIndexProvider};
pub use provider_failure::WorthQueryManagedIndexProviderFailure;
pub use retention::WorthQueryManagedIndexLifecycle;
pub use source_rows::{WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexSourceSlot};
pub use stop::{WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexStop};
