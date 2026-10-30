//! Installed resource journeys share the real capability application's owners.
#[path = "resources/action_authorization.rs"]
pub mod action_authorization;
#[path = "resources/definition.rs"]
pub mod definition;
#[path = "resources/derived_evidence.rs"]
pub mod derived_evidence;
#[path = "resources/grant_status.rs"]
pub mod grant_status;
#[path = "resources/journeys.rs"]
pub mod journeys;
#[path = "resources/lifecycle.rs"]
pub mod lifecycle;
#[path = "resources/metadata.rs"]
pub mod metadata;
#[path = "resources/provider.rs"]
pub mod provider;
pub use definition::{definition, Resource};
pub use metadata::ResourceDisclosureField;
pub use provider::{Clock, ClockSource, Producer, ProducerState};
pub fn declare(
    schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        super::schema::CapabilitySchema,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    super::schema::CapabilitySchema,
> {
    derived_evidence::declare(action_authorization::declare(lifecycle::declare(
        grant_status::declare(metadata::declare(schema)),
    )))
}
