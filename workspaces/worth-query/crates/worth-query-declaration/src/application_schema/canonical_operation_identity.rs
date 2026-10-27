use super::canonical_basis::ApplicationSchemaCanonicalBasis;
use super::ApplicationOperationProgramTarget;

pub(super) fn append_operation_target(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    target: &ApplicationOperationProgramTarget,
) {
    match target {
        ApplicationOperationProgramTarget::Create { entity } => {
            basis.text(format!("{prefix}.action"), "create");
            basis.text(format!("{prefix}.entity"), entity);
        }
        ApplicationOperationProgramTarget::Delete { entity } => {
            basis.text(format!("{prefix}.action"), "delete");
            basis.text(format!("{prefix}.entity"), entity);
        }
        ApplicationOperationProgramTarget::Write {
            entity,
            aspect,
            field,
        } => {
            basis.text(format!("{prefix}.action"), "write");
            basis.text(format!("{prefix}.entity"), entity);
            basis.text(format!("{prefix}.aspect"), aspect);
            basis.text(format!("{prefix}.field"), field);
        }
        ApplicationOperationProgramTarget::Link { relation, from, to } => {
            append_relation_target(basis, prefix, "link", relation, from, to);
        }
        ApplicationOperationProgramTarget::InitializeRelationField {
            relation,
            from,
            to,
            aspect,
            field,
        } => {
            append_relation_target(
                basis,
                prefix,
                "initialize-relation-field",
                relation,
                from,
                to,
            );
            basis.text(format!("{prefix}.aspect"), aspect);
            basis.text(format!("{prefix}.field"), field);
        }
        ApplicationOperationProgramTarget::Unlink { relation, from, to } => {
            append_relation_target(basis, prefix, "unlink", relation, from, to);
        }
        ApplicationOperationProgramTarget::Emit { effect } => {
            basis.text(format!("{prefix}.action"), "emit");
            basis.text(format!("{prefix}.effect"), effect);
        }
    }
}

fn append_relation_target(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    action: &str,
    relation: &str,
    from: &str,
    to: &str,
) {
    basis.text(format!("{prefix}.action"), action);
    basis.text(format!("{prefix}.relation"), relation);
    basis.text(format!("{prefix}.from"), from);
    basis.text(format!("{prefix}.to"), to);
}
