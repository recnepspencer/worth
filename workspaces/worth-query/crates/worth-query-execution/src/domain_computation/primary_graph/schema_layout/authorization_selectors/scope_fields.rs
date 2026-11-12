//! Scope selectors retain one compiled field set and references from capability operations.
use super::{
    insert, ApplicationOperationDecisionReadTarget as Target, ApplicationSchemaMember as Member,
    EntityFields,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) type SharedSelectorFields = BTreeMap<String, Vec<Arc<EntityFields>>>;

pub(super) fn lower(
    members: &[Member],
    capabilities: &BTreeMap<String, EntityFields>,
) -> SharedSelectorFields {
    let definitions = members
        .iter()
        .filter_map(|member| {
            let Member::DecisionReadScope { definition } = member else {
                return None;
            };
            let mut fields = EntityFields::new();
            for target in definition.targets() {
                insert_target(members, &mut fields, target);
            }
            Some((definition.identity(), Arc::new(fields)))
        })
        .collect::<BTreeMap<_, _>>();
    let mut references = SharedSelectorFields::new();
    for member in members {
        if let Member::OperationDecisionReadScope { operation, scope } = member
            && capabilities.contains_key(operation)
            && let Some(fields) = definitions.get(scope.as_str())
        {
            references
                .entry(operation.clone())
                .or_default()
                .push(Arc::clone(fields));
        }
    }
    references
}

pub(super) fn insert_target(members: &[Member], selected: &mut EntityFields, target: &Target) {
    match target {
        Target::Field {
            entity,
            aspect,
            field,
        } => insert(selected, entity, aspect, field),
        Target::Entity { entity } => {
            for member in members {
                if let Member::Field {
                    entity: field_entity,
                    aspect,
                    field,
                    equality_queryable: true,
                    ..
                } = member
                    && field_entity == entity
                {
                    insert(selected, entity, aspect, field);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_declaration::facade::application_schema::ApplicationDecisionReadScopeMeaning;

    #[test]
    fn shared_selector_fields_keep_exact_membership_and_one_allocation() {
        let definition = ApplicationDecisionReadScopeMeaning::from_untrusted_parts(
            vec![Target::Field {
                entity: "Record".into(),
                aspect: "Facts".into(),
                field: "Allowed".into(),
            }],
            Vec::new(),
        )
        .unwrap();
        let mut members = ["Read", "Inspect", "NotACapability"]
            .into_iter()
            .map(|operation| Member::OperationDecisionReadScope {
                operation: operation.into(),
                scope: definition.identity().into(),
            })
            .collect::<Vec<_>>();
        members.push(Member::DecisionReadScope { definition });
        let capabilities = ["Read", "Inspect", "Other"]
            .into_iter()
            .map(|name| (name.into(), EntityFields::new()))
            .collect();
        let references = lower(&members, &capabilities);
        assert!(Arc::ptr_eq(
            &references["Read"][0],
            &references["Inspect"][0]
        ));
        assert!(super::super::contains(
            &references["Read"][0],
            "Record",
            "Facts",
            "Allowed"
        ));
        assert!(!super::super::contains(
            &references["Read"][0],
            "Record",
            "Facts",
            "Outside"
        ));
        assert!(!references.contains_key("Other"));
        assert!(!references.contains_key("NotACapability"));
    }
}
