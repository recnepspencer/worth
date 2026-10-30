//! Canonical no-output action occurrence and source-completion delivery.
use super::{AmendTemporalBinding, TemporalExampleFeature, TemporalHostSchema};
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRootComposition,
    ApplicationSourceCompletion,
};

pub struct TemporalCompletion;
impl ApplicationActionDelivery<TemporalHostSchema, AmendTemporalBinding> for TemporalCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.example.temporal-completion.v1";
}
pub type AmendTemporalAction = ApplicationActionReference<
    TemporalHostSchema,
    ApplicationRootComposition,
    TemporalExampleFeature,
    AmendTemporalBinding,
    TemporalCompletion,
>;
