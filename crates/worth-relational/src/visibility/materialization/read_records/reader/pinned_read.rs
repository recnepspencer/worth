use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::pinned::VisibilityProjectionViewOwnerPort as _;
use crate::history::retention::VisibilitySnapshotBasis;
use crate::indexes::access::selected_index::PinnedIndex;
#[cfg(test)]
use crate::indexes::data::DerivedIndexDefinition;
#[cfg(test)]
use crate::indexes::data::DerivedIndexGenerationId;
use crate::indexes::data::DerivedIndexId;
use crate::query::data::QueryPlanContextId;
use crate::snapshots::data::SnapshotHandle;
use crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis;
use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::Arc;
pub use crate::indexes::data::applied_candidate_qualification::RelationalAppliedCandidateReadQualification;
pub use candidate::RelationalAppliedCandidateReadIdentity;
pub use worth_relational_publication::internal_port::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedRead;
pub use super::pin_denial::RelationalPinnedReadAdmissionDenial;
pub(crate) use worth_relational_publication::internal_port::visibility::materialization::read_records::reader::pinned_read::candidate;
mod pinning;
