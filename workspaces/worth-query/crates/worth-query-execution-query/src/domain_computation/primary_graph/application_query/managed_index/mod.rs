//! Controlled provider mechanics over a genuine completed source image.
mod adapter;
mod admitted_read;
pub mod certification;
pub use admitted_read::read_decision;
pub use admitted_read::read_source_admitted;
mod build;
mod candidates;
pub mod configuration;
mod control;
pub mod denial;
mod execution;
pub mod handle;
pub mod invariant_read;
pub mod invocation;

pub mod applied_read;
mod maintained_read;
pub mod realized_footprint;
pub use invariant_read::WorthQueryManagedIndexInvariantDenial;
mod node_backing;
mod nodes;
pub mod predicate;
pub use predicate::WatchSlot;
mod provider;
pub mod refinement;
pub use predicate::{
    WorthQueryManagedIndexPredicateProvider, WorthQueryManagedIndexPredicateSummary,
};
pub use refinement::WorthQueryManagedIndexRefinementProvider;
mod provider_failure;
pub mod receiver;
pub mod retention;
pub mod selection;
pub mod source;
pub use source::failure::source_preparation_denial;
pub mod source_acquisition;
mod source_completion;
mod source_content;
mod source_dependencies;
mod source_image;
mod source_order;
mod source_row;
mod source_rows;
mod source_text;
pub use source_content::WorthQueryManagedIndexContentReadDenial;
pub use source_row::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexTextReadDenial};
pub use source_rows::SourceImageView;
pub mod source_structure;
pub use source::{
    initialization::{
        initialization_execution_denial, SelectedSources, SourceInitializationIssuer,
        SourceInitializationRequest,
    },
    ManagedSourceInstallation,
};
pub use source_structure::SourceTree;
pub use source_structure::{
    WorthQueryManagedIndexNodeRegion, WorthQueryManagedIndexSummarySources,
};
mod stop;

pub use candidates::WorthQueryManagedIndexCandidateWriter;
pub use configuration::ManagedIndexProviderCatalog;
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
