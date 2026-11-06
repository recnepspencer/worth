//! Single-field handles retain name-derived identity across Rust marker renames.
use super::{Aspect, Entity, Fragment, FragmentCatalog, RecordFieldDefinition};
use std::any::TypeId;

impl FragmentCatalog {
    pub(in crate::application_schema::fragment) fn named_field<
        F: Fragment,
        E: Entity,
        A: Aspect,
        T: 'static,
    >(
        &self,
        path: &str,
        binding: TypeId,
    ) -> Option<&RecordFieldDefinition> {
        let mut fields = self
            .definitions
            .values()
            .filter(|fragment| {
                fragment.path == F::NAME
                    || fragment
                        .path
                        .strip_suffix(F::NAME)
                        .is_some_and(|prefix| prefix.ends_with('.'))
            })
            .flat_map(|fragment| fragment.aspects.values())
            .filter(|aspect| aspect.entity == E::NAME && aspect.meaning == A::NAME)
            .flat_map(|aspect| &aspect.fields)
            .filter(|field| {
                !field.internal_presence
                    && field.recipe.locus().field() == path
                    && field.recipe.value_type() == TypeId::of::<T>()
                    && field.recipe.binding_type() == binding
            });
        let field = fields.next()?;
        fields.next().is_none().then_some(field)
    }
}
