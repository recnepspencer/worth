//! Restricted native entrances for the discovered and workflow lanes whose owners land later.
use crate::application_entry::mutation::request::WorthQueryMutationSourceUnprepared;
use crate::application_entry::mutation::WorthQueryApplicationMutationRequestWithIdempotency;
use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

/// Borrowed-key construction for the still-native discovered and workflow entrances.
/// It cannot execute or recover a fixed/source-only action. Those lanes require
/// `idempotency_owned` and an exact declared action through the owning builder.
#[must_use = "select the declared discovered or workflow lane; fixed and source-only actions need an owned key"]
pub struct WorthQueryPendingApplicationLane<
    'application,
    'principal,
    'scope,
    'key,
    Schema,
    Intent,
    SourcePreparation = WorthQueryMutationSourceUnprepared,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) native: WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >,
}
