//! SourceOnly delivery for document business actions without required outputs.
use super::DocumentRetentionSchema;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{ApplicationActionDelivery, ApplicationSourceCompletion},
};

pub struct RetentionCompletion;
impl<B: ApplicationMutationBinding<DocumentRetentionSchema>>
    ApplicationActionDelivery<DocumentRetentionSchema, B> for RetentionCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "document-retention.source-completion.v1";
}
