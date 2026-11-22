//! Resolve installed declaration scopes into the concrete Native touch vocabulary.
use super::*;
pub(super) fn resolve_installed_touches(
    layout: &WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryOperationTouchContract,
) -> Result<BTreeMap<ResolvedApplicationTouch, usize>, ()> {
    let mut resolved = BTreeMap::new();
    for (index, scope) in contract.scopes().iter().enumerate() {
        let key = resolve_installed_touch(layout, scope)?;
        if resolved.insert(key, index).is_some() {
            return Err(());
        }
    }
    Ok(resolved)
}

fn resolve_installed_touch(
    layout: &WorthQueryPrimaryGraphLayout,
    scope: &WorthQueryOperationTouchScope,
) -> Result<ResolvedApplicationTouch, ()> {
    match scope {
        WorthQueryOperationTouchScope::RemoveAspect(scope) => {
            if layout.aspect_contract(scope.entity(), scope.contract().key())
                != Some(scope.contract())
            {
                return Err(());
            }
            Ok(ResolvedApplicationTouch::RemoveEntityAspect(
                layout.entity_kind(scope.entity()).ok_or(())?,
                scope.contract().key().clone(),
            ))
        }
        WorthQueryOperationTouchScope::InitializeRelationField(scope) => {
            let relation = scope.relation();
            resolve_relation_touch(layout, relation, true)?;
            if layout.relation_aspect_contract(relation.relation(), scope.contract().key().as_str())
                != Some(scope.contract())
                || scope.field_path().fields().len() != 1
            {
                return Err(());
            }
            let locator = layout
                .relation_field_locator(
                    relation.relation(),
                    scope.contract().key().as_str(),
                    scope.field_path().fields()[0].as_str(),
                )
                .ok_or(())?;
            if locator.field_path() != scope.field_path() {
                return Err(());
            }
            Ok(ResolvedApplicationTouch::InitializeRelationField(
                layout.relation(relation.relation()).ok_or(())?.kind,
                locator.clone(),
            ))
        }
        WorthQueryOperationTouchScope::CreateEntity(scope) => layout
            .entity_kind(scope.entity())
            .map(ResolvedApplicationTouch::CreateEntity)
            .ok_or(()),
        WorthQueryOperationTouchScope::DeleteEntity(scope) => layout
            .entity_kind(scope.entity())
            .map(ResolvedApplicationTouch::DeleteEntity)
            .ok_or(()),
        WorthQueryOperationTouchScope::WriteField(scope) => {
            if layout.aspect_contract(scope.entity(), scope.contract().key())
                != Some(scope.contract())
                || scope.field_path().fields().len() != 1
            {
                return Err(());
            }
            let field = &scope.field_path().fields()[0];
            let kind = layout.entity_kind(scope.entity()).ok_or(())?;
            let locator = layout
                .field_locator(
                    scope.entity(),
                    scope.contract().key().as_str(),
                    field.as_str(),
                )
                .cloned()
                .ok_or(())?;
            if locator.field_path() != scope.field_path() {
                return Err(());
            }
            Ok(ResolvedApplicationTouch::WriteEntityField(kind, locator))
        }
        WorthQueryOperationTouchScope::LinkRelation(scope) => {
            resolve_relation_touch(layout, scope, true)
        }
        WorthQueryOperationTouchScope::UnlinkRelation(scope) => {
            resolve_relation_touch(layout, scope, false)
        }
        WorthQueryOperationTouchScope::DeclaredDomain(_) => Err(()),
    }
}

fn resolve_relation_touch(
    layout: &WorthQueryPrimaryGraphLayout,
    scope: &worth_query_installation::facade::WorthQueryOperationRelationTouchScope,
    link: bool,
) -> Result<ResolvedApplicationTouch, ()> {
    let relation = layout.relation(scope.relation()).ok_or(())?;
    if layout.entity_kind(scope.from()) != Some(relation.from)
        || layout.entity_kind(scope.to()) != Some(relation.to)
    {
        return Err(());
    }
    Ok(if link {
        ResolvedApplicationTouch::LinkRelation(relation.kind)
    } else {
        ResolvedApplicationTouch::UnlinkRelation(relation.kind)
    })
}
