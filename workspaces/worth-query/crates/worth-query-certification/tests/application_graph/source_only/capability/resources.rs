//! Installed resource journeys share the real capability application's owners.
#[path = "resources/action_authorization.rs"]
mod action_authorization;
#[path = "resources/definition.rs"]
mod definition;
#[path = "resources/grant_status.rs"]
pub(super) mod grant_status;
#[path = "resources/journeys.rs"]
mod journeys;
#[path = "resources/lifecycle.rs"]
pub(super) mod lifecycle;
#[path = "resources/metadata.rs"]
mod metadata;
#[path = "resources/provider.rs"]
mod provider;
pub(super) use definition::{definition, Resource};
pub(super) use metadata::ResourceDisclosureField;
pub(super) use provider::{Clock, ClockSource, Producer, ProducerState};
pub(super) fn declare(
    schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        super::schema::CapabilitySchema,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    super::schema::CapabilitySchema,
> {
    action_authorization::declare(lifecycle::declare(grant_status::declare(
        metadata::declare(schema),
    )))
}
