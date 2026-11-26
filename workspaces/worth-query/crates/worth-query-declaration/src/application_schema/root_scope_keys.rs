//! A query declared through the consumer authoring surface and scoped on a
//! field of its own root selects that root by the field, so the field must be
//! a key: declared `unique`, or an immutable (`read_only`) identity the
//! application assigns. A mutable, non-unique field (a status, a label) is
//! never a key: the program would install, then refuse at run time as soon as
//! two roots share the value.
use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};
use crate::application_query::{
    ApplicationQueryBindingDescriptor, ApplicationQueryScopeResolutionMode,
    ErasedApplicationQueryDefinition,
};

/// Bindings declared by `bound_application_query`, checked at `build()` once
/// every field member is known.
#[derive(Clone, Debug, Default)]
pub(super) struct RootScopeKeyExpectations {
    bindings: Vec<ApplicationQueryBindingDescriptor>,
}

impl RootScopeKeyExpectations {
    pub(super) fn expect(&mut self, binding: ApplicationQueryBindingDescriptor) {
        self.bindings.push(binding);
    }

    pub(super) fn validate(
        self,
        members: &[ApplicationSchemaMember],
    ) -> Result<(), ApplicationSchemaDeclarationDenial> {
        for binding in self.bindings {
            let Some(query) = root_scoped_query(&binding, members) else {
                continue;
            };
            let locus = binding.scope().field().locus();
            let mutable_non_key = members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::Field {
                    entity, aspect, field, unique: false, writable: true, ..
                } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field())
            });
            if mutable_non_key {
                return Err(
                    ApplicationSchemaDeclarationDenial::RootScopedQueryRequiresKeyField {
                        binding: binding.identity().to_owned(),
                        query: query.name().to_owned(),
                        field: locus.clone(),
                    },
                );
            }
        }
        Ok(())
    }
}

/// The binding's query when an input field selects the query's own root
/// directly, with no root path from a separate scope entity. A missing query
/// is refused by binding validation, not here.
fn root_scoped_query<'members>(
    binding: &ApplicationQueryBindingDescriptor,
    members: &'members [ApplicationSchemaMember],
) -> Option<&'members ErasedApplicationQueryDefinition> {
    if binding.scope().mode() != ApplicationQueryScopeResolutionMode::InputField {
        return None;
    }
    members.iter().find_map(|member| match member {
        ApplicationSchemaMember::ApplicationQuery { definition }
            if definition.query_identity() == *binding.query_identity()
                && definition.root_entity() == definition.scope_entity()
                && definition.root_paths().is_empty() =>
        {
            Some(definition)
        }
        _ => None,
    })
}
