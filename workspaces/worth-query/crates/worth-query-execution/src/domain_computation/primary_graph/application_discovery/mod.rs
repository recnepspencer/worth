//! Read-only projections of installed application declaration meaning.
mod description;
use super::WorthQueryPrimaryGraphApplicationRuntime;
pub use description::{
    WorthQueryApplicationCallablePosture, WorthQueryApplicationFieldDescription,
    WorthQueryApplicationMutationDescription, WorthQueryApplicationQueryDescription,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaMember,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

/// Descriptive application contracts for tooling. This view issues no authority.
pub struct WorthQueryApplicationDiscovery<'a, Schema> {
    schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn discovery(&self) -> WorthQueryApplicationDiscovery<'_, Schema> {
        WorthQueryApplicationDiscovery {
            schema: self.installed_schema(),
        }
    }
}

impl<'a, Schema: ApplicationSchema> WorthQueryApplicationDiscovery<'a, Schema> {
    /// Entry input and scope contracts for the installed typed query requests.
    pub fn query_requests(&self) -> impl ExactSizeIterator<Item = &'a worth_query_declaration::facade::application_query::ApplicationQueryBindingDescriptor> + 'a{
        self.schema.installed_query_binding_inventory()
    }

    pub fn mutations(
        &self,
    ) -> impl Iterator<Item = WorthQueryApplicationMutationDescription<'a>> + 'a {
        let schema = self.schema;
        let members = schema.installed_declaration().members();
        members.iter().filter_map(move |member| {
            let ApplicationSchemaMember::ApplicationMutation { description } = member else {
                return None;
            };
            let installed = schema
                .installed_mutation_binding_inventory()
                .any(|binding| binding.identity() == description.binding_identity().as_str());
            Some(WorthQueryApplicationMutationDescription {
                declaration: description,
                members,
                availability: posture(installed),
            })
        })
    }

    pub fn queries(&self) -> impl Iterator<Item = WorthQueryApplicationQueryDescription<'a>> + 'a {
        let schema = self.schema;
        schema
            .installed_declaration()
            .members()
            .iter()
            .filter_map(move |member| {
                let ApplicationSchemaMember::ApplicationQuery { definition } = member else {
                    return None;
                };
                let installed = schema.installed_query_binding_inventory().any(|binding| {
                    binding.query_name() == definition.name()
                        && binding.query_identity().as_str() == definition.query_type()
                });
                Some(WorthQueryApplicationQueryDescription {
                    definition,
                    availability: posture(installed),
                })
            })
    }

    pub fn fields(&self) -> impl Iterator<Item = WorthQueryApplicationFieldDescription<'a>> + 'a {
        self.schema
            .installed_declaration()
            .members()
            .iter()
            .filter_map(|member| {
                let ApplicationSchemaMember::Field {
                    entity,
                    aspect,
                    field,
                    value_type,
                    unit,
                    frame,
                    ..
                } = member
                else {
                    return None;
                };
                Some(WorthQueryApplicationFieldDescription {
                    entity,
                    aspect,
                    field,
                    value_type,
                    unit: unit.as_deref(),
                    frame: frame.as_deref(),
                })
            })
    }
}

const fn posture(installed: bool) -> WorthQueryApplicationCallablePosture {
    if installed {
        WorthQueryApplicationCallablePosture::InstalledRequestBinding
    } else {
        WorthQueryApplicationCallablePosture::Declared
    }
}
