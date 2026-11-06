use worth_query_declaration::facade::application_query::{
    ApplicationQueryParameterRef, ApplicationQueryResultFieldRef,
};
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

use super::{
    ActivitySequence, ActivitySequenceResultSlot, CrossRootQuery, IdentityExecutionSchema,
    StatusParameter,
};

pub(crate) fn status_parameter<Query>(
) -> ApplicationQueryParameterRef<Query, StatusParameter, StringApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("status")
}

pub(super) fn activity_sequence_result_field() -> ApplicationQueryResultFieldRef<
    CrossRootQuery,
    ActivitySequenceResultSlot,
    IdentityExecutionSchema,
    ActivitySequence,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::at("ActivitySequence"))
}
