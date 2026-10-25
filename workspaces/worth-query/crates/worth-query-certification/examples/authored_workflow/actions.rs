//! Source-completion actions declared by the installed retention program.
use super::document_retention_model::{
    programs::{DocumentRetentionFeature, RetentionCompletion},
    retention_entry::{OccurrenceSetRetentionBinding, ReviewedSetRetentionBinding},
    schema::DocumentRetentionSchema,
};
use worth_query_decl::facade::application_program::{
    ApplicationActionReference, ApplicationRootComposition,
};

pub(super) type OccurrenceAction = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    OccurrenceSetRetentionBinding,
    RetentionCompletion,
>;
pub(super) type ReviewedAction = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    ReviewedSetRetentionBinding,
    RetentionCompletion,
>;
