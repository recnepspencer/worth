//! Shared contents and references retain distinct canonical meanings.
use super::{ApplicationSchemaCanonicalBasis, ApplicationSchemaMember};
pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    match member {
        ApplicationSchemaMember::DecisionReadScope { definition } => {
            basis.text(format!("{prefix}.kind"), "decision-read-scope");
            basis.text(format!("{prefix}.identity"), definition.identity());
            for (index, target) in definition.targets().iter().enumerate() {
                crate::application_schema::canonical_decision_read_identity::append_decision_read_target(
                    basis, &format!("{prefix}.target[{index}]"), target);
            }
            for (index, relation) in definition.derived_relations().iter().enumerate() {
                basis.text(format!("{prefix}.derived[{index}]"), relation);
            }
        }
        ApplicationSchemaMember::OperationDecisionReadScope { operation, scope } => {
            basis.text(format!("{prefix}.kind"), "operation-decision-read-scope");
            basis.text(format!("{prefix}.operation"), operation);
            basis.text(format!("{prefix}.scope"), scope);
        }
        _ => unreachable!("decision-read scope member router"),
    }
}
