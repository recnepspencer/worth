//! Controlled provider mechanics over a genuine completed source image.
mod adapter;
mod admitted_read;
pub(in crate::domain_computation::primary_graph) use admitted_read::read_decision;
pub(in crate::domain_computation::primary_graph::application_query) use admitted_read::read_source_admitted;
mod build;
mod candidates;
pub(crate) mod configuration;
mod control;
mod denial;
mod execution;
mod handle;
pub(in crate::domain_computation::primary_graph) mod invariant_read;
pub(crate) mod invocation;

mod maintained_read;
pub(in crate::domain_computation::primary_graph) mod realized_footprint;
pub use invariant_read::WorthQueryManagedIndexInvariantDenial;
mod node_backing;
mod nodes;
pub(in crate::domain_computation::primary_graph::application_query) mod predicate;
pub(in crate::domain_computation::primary_graph) use predicate::WatchSlot;
mod provider;
pub(crate) mod refinement;
pub use predicate::{
    WorthQueryManagedIndexPredicateProvider, WorthQueryManagedIndexPredicateSummary,
};
pub use refinement::WorthQueryManagedIndexRefinementProvider;
mod provider_failure;
mod receiver;
pub(crate) mod retention;
pub(in crate::domain_computation::primary_graph::application_query) mod selection;
mod source;
pub(in crate::domain_computation::primary_graph::application_query) use source::failure::source_preparation_denial;
pub(in crate::domain_computation::primary_graph::application_query) mod source_acquisition;
mod source_completion;
mod source_dependencies;
mod source_image;
mod source_order;
mod source_row;
mod source_rows;
mod source_text;
pub use source_row::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexTextReadDenial};
pub(in crate::domain_computation::primary_graph) use source_rows::SourceImageView;
pub(crate) mod source_structure;
pub(in crate::domain_computation::primary_graph) use source::{
    initialization::{
        initialization_execution_denial, SelectedSources, SourceInitializationIssuer,
        SourceInitializationRequest,
    },
    ManagedSourceInstallation,
};
pub(in crate::domain_computation::primary_graph) use source_structure::SourceTree;
pub use source_structure::{
    WorthQueryManagedIndexNodeRegion, WorthQueryManagedIndexSummarySources,
};
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
pub use provider::{
    WorthQueryManagedIndexProbeValue, WorthQueryManagedIndexProvider,
    WorthQueryManagedIndexSourceChange,
};
pub use provider_failure::WorthQueryManagedIndexProviderFailure;
pub use receiver::{
    WorthQueryManagedIndexWatch, WorthQueryManagedIndexWatchOutcome,
    WorthQueryManagedIndexWatchUpdate,
};
pub use retention::WorthQueryManagedIndexLifecycle;
pub use source_rows::{WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexSourceSlot};
pub use stop::{WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexStop};
