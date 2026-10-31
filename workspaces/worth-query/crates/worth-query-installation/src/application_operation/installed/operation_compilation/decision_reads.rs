//! Atomic decision targets consume the same installed scopes as operation reads.
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;

use crate::domain_operation::{
    WorthQueryOperationApplicationProjectionScope, WorthQueryOperationGraphReadContract,
    WorthQueryOperationGraphReadScope,
};

pub(super) fn from_installed(
    reads: &WorthQueryOperationGraphReadContract,
) -> Result<Vec<ApplicationOperationDecisionReadTarget>, ()> {
    let mut targets = Vec::new();
    for scope in reads.roles().iter().flat_map(|role| role.read_scopes()) {
        match scope {
            // These scopes retain their own installed selection authority;
            // neither is an atomic relation target.
            WorthQueryOperationGraphReadScope::Components(_)
            | WorthQueryOperationGraphReadScope::DerivedSource(_) => {}
            WorthQueryOperationGraphReadScope::Entity(entity) => {
                targets.push(ApplicationOperationDecisionReadTarget::Entity {
                    entity: entity.semantic_key().to_owned(),
                });
            }
            WorthQueryOperationGraphReadScope::NativeProjection(scope) => {
                append_projection(scope, &mut targets)?;
            }
            WorthQueryOperationGraphReadScope::Relation(relation) => {
                targets.push(ApplicationOperationDecisionReadTarget::Relation {
                    relation: relation.relation().to_owned(),
                    from: relation.from().to_owned(),
                    to: relation.to().to_owned(),
                });
            }
        }
    }
    Ok(targets)
}

fn append_projection(
    scope: &WorthQueryOperationApplicationProjectionScope,
    targets: &mut Vec<ApplicationOperationDecisionReadTarget>,
) -> Result<(), ()> {
    let mut append = |field: &worth_foundational::facade::FieldKey| {
        targets.push(ApplicationOperationDecisionReadTarget::Field {
            entity: scope.entity().semantic_key().to_owned(),
            aspect: scope.aspect().as_str().to_owned(),
            field: field.as_str().to_owned(),
        });
    };
    if let Some(field) = scope.bound_scalar_field() {
        append(field);
    } else {
        // Whole Struct keeps no closed atomic-field inventory here.
        for path in scope.projection().mask().paths() {
            let [field] = path.fields() else {
                return Err(());
            };
            append(field);
        }
    }
    Ok(())
}
