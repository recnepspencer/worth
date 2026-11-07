//! Exact typed identity first; name-derived column identity only across marker renames.
use super::{FragmentCatalog, RecordFieldDefinition};
use crate::application_schema::fragment::ColumnIdentity;
use crate::application_schema::ApplicationFieldBindingOwnerKind as Owner;
impl FragmentCatalog {
    pub(in crate::application_schema) fn declared_field_matches(
        &self,
        expected: &crate::application_schema::ApplicationFieldBindingRecipe,
    ) -> bool {
        self.definitions
            .values()
            .flat_map(|fragment| fragment.aspects.values())
            .flat_map(|aspect| &aspect.fields)
            .any(|field| {
                !field.internal_presence
                    && field.recipe.locus() == expected.locus()
                    && field.recipe.value_type() == expected.value_type()
                    && field.recipe.binding_type() == expected.binding_type()
            })
    }

    pub(in crate::application_schema::fragment) fn column(
        &self,
        identity: ColumnIdentity,
    ) -> Option<&RecordFieldDefinition> {
        let matches = |field: &&RecordFieldDefinition| {
            !field.internal_presence
                && field.recipe.locus().owner_kind() == identity.owner
                && field.recipe.locus().field() == identity.field
                && field.recipe.value_type() == identity.value
                && field.recipe.binding_type() == identity.binding
        };
        if let Some(fragment) = self.definitions.get(&identity.fragment) {
            return fragment
                .aspects
                .get(&(identity.record, identity.aspect))?
                .fields
                .iter()
                .find(matches);
        }
        if identity.owner != Owner::Entity {
            return None;
        }
        let mut fields = self
            .definitions
            .values()
            .filter(|fragment| {
                fragment.path == identity.fragment_name
                    || fragment
                        .path
                        .strip_suffix(identity.fragment_name)
                        .is_some_and(|prefix| prefix.ends_with('.'))
            })
            .flat_map(|fragment| fragment.aspects.values())
            .filter(|aspect| {
                aspect.entity == identity.record_name && aspect.meaning == identity.aspect_name
            })
            .flat_map(|aspect| &aspect.fields)
            .filter(matches);
        let field = fields.next()?;
        fields.next().is_none().then_some(field)
    }
}
