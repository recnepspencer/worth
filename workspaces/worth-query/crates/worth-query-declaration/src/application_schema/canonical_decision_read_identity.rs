use super::canonical_basis::ApplicationSchemaCanonicalBasis;
use super::ApplicationOperationDecisionReadTarget;

pub(super) fn append_decision_read_target(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    target: &ApplicationOperationDecisionReadTarget,
) {
    match target {
        ApplicationOperationDecisionReadTarget::ManagedIndex { parent } => {
            basis.text(format!("{prefix}.kind"), "managed-index");
            for (field, value) in parent.canonical_fields() {
                basis.text(format!("{prefix}.parent.{field}"), value);
            }
        }
        ApplicationOperationDecisionReadTarget::SharedOwnerManagedIndex {
            parent,
            relation,
            owner,
            member,
        } => {
            basis.text(format!("{prefix}.kind"), "shared-owner-managed-index");
            for (field, value) in parent.canonical_fields() {
                basis.text(format!("{prefix}.parent.{field}"), value);
            }
            basis.text(format!("{prefix}.relation"), relation);
            basis.text(format!("{prefix}.owner"), owner);
            basis.text(format!("{prefix}.member"), member);
        }
        ApplicationOperationDecisionReadTarget::Entity { entity } => {
            basis.text(format!("{prefix}.kind"), "entity");
            basis.text(format!("{prefix}.entity"), entity);
        }
        ApplicationOperationDecisionReadTarget::Field {
            entity,
            aspect,
            field,
        } => {
            basis.text(format!("{prefix}.kind"), "field");
            basis.text(format!("{prefix}.entity"), entity);
            basis.text(format!("{prefix}.aspect"), aspect);
            basis.text(format!("{prefix}.field"), field);
        }
        ApplicationOperationDecisionReadTarget::Relation { relation, from, to } => {
            basis.text(format!("{prefix}.kind"), "relation");
            basis.text(format!("{prefix}.relation"), relation);
            basis.text(format!("{prefix}.from"), from);
            basis.text(format!("{prefix}.to"), to);
        }
    }
}
