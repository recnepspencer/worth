//! Author a whole-parent removal from the complete same-program decision closure.
use super::*;
use worth_foundational::facade::AspectKey;
use worth_query_declaration::facade::application_schema::OperationRemovesAspects;
use worth_relational::facade::identity::EntityId;
impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub fn remove_aspect<F, Entity, Aspect>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        aspect: worth_query_declaration::facade::application_schema::fragment::AspectSelection<
            F,
            Entity,
            Aspect,
        >,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        F: worth_query_declaration::facade::application_schema::fragment::Fragment,
        Entity: worth_query_declaration::facade::application_schema::fragment::Entity,
        Aspect: worth_query_declaration::facade::application_schema::fragment::Aspect
            + OperationRemovesAspects<Operation>,
    {
        self.native_effects().remove_aspect(
            target.borrowed(),
            aspect.entity(),
            aspect.name(),
            selected_aspect::<F, Entity, Aspect>,
        )
    }

    pub(super) fn require_parent_not_removed(
        &self,
        entity: &str,
        id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        require_parent_not_removed(&self.effects, entity, id, locator)
    }
}

pub(super) fn aspect_conflict(
    entity: &str,
    id: EntityId,
    key: &AspectKey,
) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
        format!("{entity} {id:?}.{}", key.as_str()),
    )
}

pub(super) fn require_parent_not_removed(
    effects: &[model::WorthQueryApplicationRealizedEffect],
    entity: &str,
    id: EntityId,
    locator: &AspectFieldLocator,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if effects.iter().any(|effect| matches!(effect, model::WorthQueryApplicationRealizedEffect::RemoveAspect { entity_id, contract, .. }
            if *entity_id == id && contract.key() == locator.aspect().aspect_key())) {
            Err(aspect_conflict(entity, id, locator.aspect().aspect_key()))
        } else { Ok(()) }
}

fn selected_aspect<F, E, A>(
    catalog: &worth_query_declaration::facade::application_schema::fragment::FragmentCatalog,
) -> Option<&worth_query_declaration::facade::application_schema::fragment::AspectDefinition>
where
    F: worth_query_declaration::facade::application_schema::fragment::Fragment,
    E: worth_query_declaration::facade::application_schema::fragment::Entity,
    A: worth_query_declaration::facade::application_schema::fragment::Aspect,
{
    catalog.fragment_type::<F>()?.aspect::<E, A>()
}
