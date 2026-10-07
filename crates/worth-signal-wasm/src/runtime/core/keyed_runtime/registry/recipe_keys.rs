use crate::boundary::errors::WorthSignalJsError;
use crate::expression::model::IdentitySpec;
use crate::recipe::model::{
    RecipeFamilyReadSpec, RecipeReadSignalSpec, RecipeReadSpec, RecipeSpec,
};

use super::super::super::keyed_families::{composite_keyed_id, rewrite_keyed_expr};
use super::super::super::state::KeyedEnsureStats;
use super::super::super::RuntimeCore;

impl RuntimeCore {
    pub fn ensure_recipe_key(
        &mut self,
        family_id: &str,
        key: &str,
    ) -> Result<String, WorthSignalJsError> {
        self.ensure_recipe_key_with_stats(family_id, key, &mut KeyedEnsureStats::default())
    }

    fn ensure_recipe_key_with_stats(
        &mut self,
        family_id: &str,
        key: &str,
        stats: &mut KeyedEnsureStats,
    ) -> Result<String, WorthSignalJsError> {
        let composite_id = composite_keyed_id(family_id, key);
        if self.catalog.contains_key(&composite_id) {
            stats.recipe_hits = stats.recipe_hits.saturating_add(1);
            return Ok(composite_id);
        }
        let family = {
            let store = self.lock_store()?;
            store
                .recipe_families
                .get(family_id)
                .cloned()
                .ok_or_else(|| {
                    WorthSignalJsError::invalid_input(format!(
                        "unknown recipe family `{family_id}`"
                    ))
                })?
        };
        family.scope_key_requirement.validate_key(key)?;
        let reads = family
            .spec
            .reads
            .iter()
            .map(|read| match read {
                RecipeFamilyReadSpec::Signal { id, scope, aspects } => {
                    Ok(RecipeReadSpec::Signal(RecipeReadSignalSpec {
                        id: id.clone(),
                        scope: scope.as_ref().map(|value| value.resolve(key)).transpose()?,
                        aspects: aspects.clone(),
                    }))
                }
                RecipeFamilyReadSpec::Keyed {
                    family_id,
                    scope,
                    aspects,
                } => Ok(RecipeReadSpec::Signal(RecipeReadSignalSpec {
                    id: composite_keyed_id(family_id, key),
                    scope: scope.as_ref().map(|value| value.resolve(key)).transpose()?,
                    aspects: aspects.clone(),
                })),
            })
            .collect::<Result<_, WorthSignalJsError>>()?;
        for read in &family.spec.reads {
            if let RecipeFamilyReadSpec::Keyed { family_id, .. } = read {
                let source_id = composite_keyed_id(family_id, key);
                if !self.catalog.contains_key(&source_id) {
                    if self.lock_store()?.source_families.contains_key(family_id) {
                        self.ensure_source_key_with_stats(family_id, key, None, stats)?;
                    } else {
                        self.ensure_recipe_key_with_stats(family_id, key, stats)?;
                    }
                }
            }
        }
        let recipe = RecipeSpec {
            id: composite_id.clone(),
            reads,
            expr: rewrite_keyed_expr(&family.spec.expr, &family.spec.reads, key),
            when: family.spec.when.as_ref().map(|condition| {
                crate::expression::model::ConditionSpec {
                    expr: rewrite_keyed_expr(&condition.expr, &family.spec.reads, key),
                }
            }),
            identity: family
                .spec
                .identity
                .as_ref()
                .map(|identity| match identity {
                    IdentitySpec::Exact => IdentitySpec::Exact,
                    IdentitySpec::Expr { expr } => IdentitySpec::Expr {
                        expr: rewrite_keyed_expr(expr, &family.spec.reads, key),
                    },
                }),
            produces_aspects: family.spec.produces_aspects.clone(),
        };
        self.define_recipe(recipe)?;
        stats.recipe_created = stats.recipe_created.saturating_add(1);
        Ok(composite_id)
    }

    pub(crate) fn ensure_keyed_entry(
        &mut self,
        family_id: &str,
        key: &str,
        stats: &mut KeyedEnsureStats,
    ) -> Result<String, WorthSignalJsError> {
        if self.lock_store()?.recipe_families.contains_key(family_id) {
            self.ensure_recipe_key_with_stats(family_id, key, stats)
        } else {
            self.ensure_source_key_with_stats(family_id, key, None, stats)
        }
    }
}
