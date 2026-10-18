//! Ordinary completion for the consumer's actions without required output roots.
use crate::ConsumerSchema;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{ApplicationActionDelivery, ApplicationSourceCompletion},
};

pub(crate) struct ConsumerSourceCompletion;
impl<B: ApplicationMutationBinding<ConsumerSchema>> ApplicationActionDelivery<ConsumerSchema, B>
    for ConsumerSourceCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "consumer.source-completion.v1";
}
