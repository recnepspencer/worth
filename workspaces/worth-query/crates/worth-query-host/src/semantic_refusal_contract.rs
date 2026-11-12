//! Original semantic preparation failures retain a typed record subject through Host.
//!
//! ```
//! use worth_query_host::facade::primary_graph::{
//!     CommitPreparationReason, ExistingRecordTarget,
//!     RelationalCommitSemanticResultRefusal as Semantic,
//! };
//!
//! fn record_subject(reason: CommitPreparationReason) -> Option<ExistingRecordTarget> {
//!     if let CommitPreparationReason::SemanticResult(cause) = reason {
//!         return match cause {
//!             Semantic::DuplicateAssignedCreation { record }
//!             | Semantic::CreatedPatchMissing { record } => Some(record),
//!             Semantic::MissingClientKeyText
//!             | Semantic::ConflictingCreateCorrespondence
//!             | Semantic::CreateCardinalityOverflow
//!             | Semantic::RelationBirthMismatch { .. }
//!             | Semantic::OriginalEntityKeyMissing { .. }
//!             | Semantic::OriginalRelationKeyMissing { .. }
//!             | Semantic::BulkRelationArityMismatch { .. }
//!             | Semantic::CreatePartitionMismatch
//!             | Semantic::CreatedEndpointMissing
//!             | Semantic::NonCanonicalCorrespondence
//!             | Semantic::ReceiptMismatch
//!             | Semantic::SchemaMismatch
//!             | Semantic::ChangedRecordsMismatch
//!             | Semantic::PublicationMismatch
//!             | Semantic::ValidationMismatch
//!             | Semantic::MissingOriginalSummary => None,
//!         };
//!     }
//!     None
//! }
//!
//! fn subject_kind(record: ExistingRecordTarget) -> &'static str {
//!     match record {
//!         ExistingRecordTarget::Entity(_) => "entity",
//!         ExistingRecordTarget::Relation(_) => "relation",
//!     }
//! }
//! let _ = (record_subject, subject_kind);
//! ```
