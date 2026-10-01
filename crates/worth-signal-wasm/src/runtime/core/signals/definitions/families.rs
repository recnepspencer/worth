use crate::boundary::errors::WorthSignalJsError;
use crate::recipe::model::{
    KeyedRecipeFamilySpec, KeyedSourceFamilySpec, RecipeFamilyReadSpec,
    RecipeFamilyScopeKeyRequirement,
};

use super::super::super::state::{StoredRecipeFamily, StoredSourceFamily};
use super::super::super::RuntimeCore;

impl RuntimeCore {
    pub fn define_source_family(
        &mut self,
        spec: KeyedSourceFamilySpec,
    ) -> Result<(), WorthSignalJsError> {
        let mut store = self.lock_store()?;
        if store.source_families.contains_key(&spec.family_id)
            || store.recipe_families.contains_key(&spec.family_id)
        {
            return Err(WorthSignalJsError::invalid_input(format!(
                "family `{}` already exists",
                spec.family_id
            )));
        }
        store
            .source_families
            .insert(spec.family_id.clone(), StoredSourceFamily { spec });
        Ok(())
    }

    pub fn define_keyed_recipe_family(
        &mut self,
        spec: KeyedRecipeFamilySpec,
    ) -> Result<(), WorthSignalJsError> {
        let mut store = self.lock_store()?;
        if store.recipe_families.contains_key(&spec.family_id)
            || store.source_families.contains_key(&spec.family_id)
        {
            return Err(WorthSignalJsError::invalid_input(format!(
                "family `{}` already exists",
                spec.family_id
            )));
        }
        let mut scope_key_requirement = RecipeFamilyScopeKeyRequirement::Unrestricted;
        for read in &spec.reads {
            let scope = match read {
                RecipeFamilyReadSpec::Signal { scope, .. }
                | RecipeFamilyReadSpec::Keyed { scope, .. } => scope,
            };
            if let Some(scope) = scope {
                scope_key_requirement =
                    scope_key_requirement.including(scope.compile_key_requirement()?);
            }
            match read {
                RecipeFamilyReadSpec::Signal { id, .. } => {
                    if !self.catalog.contains_key(id) {
                        return Err(WorthSignalJsError::invalid_input(format!(
                            "keyed family `{}` reads unknown signal `{id}`",
                            spec.family_id
                        )));
                    }
                }
                RecipeFamilyReadSpec::Keyed { family_id, .. } => {
                    if !store.source_families.contains_key(family_id)
                        && !store.recipe_families.contains_key(family_id)
                    {
                        return Err(WorthSignalJsError::invalid_input(format!(
                            "keyed family `{}` reads unknown family `{family_id}`",
                            spec.family_id
                        )));
                    }
                    if let Some(child) = store.recipe_families.get(family_id) {
                        scope_key_requirement =
                            scope_key_requirement.including(child.scope_key_requirement);
                    }
                }
            }
        }
        store.recipe_families.insert(
            spec.family_id.clone(),
            StoredRecipeFamily {
                spec,
                scope_key_requirement,
            },
        );
        Ok(())
    }
}
