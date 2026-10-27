//! One canonical owner for derived recipes and their descriptive associations.
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::{
    ApplicationDerivedRelationNode as Node, ApplicationDerivedRelationSourceKind as SourceKind,
    ApplicationSchemaMember,
};

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    match member {
        ApplicationSchemaMember::DerivedRelation { definition } => {
            let parts = definition.parts();
            basis.text(format!("{prefix}.kind"), "derived-relation");
            basis.text(format!("{prefix}.relation"), &parts.relation);
            basis.text(format!("{prefix}.from"), &parts.from);
            basis.text(format!("{prefix}.to"), &parts.to);
            basis.usize(format!("{prefix}.node-count"), parts.expression.nodes.len());
            for (index, node) in parts.expression.nodes.iter().enumerate() {
                append_node(basis, &format!("{prefix}.node[{index}]"), node);
            }
            super::relation_graph::append(basis, prefix, parts.laws.borrowed(), None);
        }
        ApplicationSchemaMember::RuleReadableCollection { association } => {
            basis.text(format!("{prefix}.kind"), "rule-readable-collection");
            for (key, value) in [
                ("relation", &association.relation),
                ("feature", &association.feature),
                ("collection", &association.collection),
                ("computation", &association.computation),
                ("output-artifact", &association.output_artifact),
                ("completeness", &association.completeness),
            ] {
                basis.text(format!("{prefix}.{key}"), value);
            }
            basis.u64(
                format!("{prefix}.feature-major"),
                association.feature_major.into(),
            );
            basis.u64(
                format!("{prefix}.feature-minor"),
                association.feature_minor.into(),
            );
        }
        ApplicationSchemaMember::OperationDerivedRead {
            operation,
            relation,
        } => {
            basis.text(format!("{prefix}.kind"), "operation-derived-read");
            basis.text(format!("{prefix}.operation"), operation);
            basis.text(format!("{prefix}.relation"), relation);
        }
        _ => unreachable!("derived member router"),
    }
}

fn append_node(basis: &mut ApplicationSchemaCanonicalBasis, prefix: &str, node: &Node) {
    match node {
        Node::Source(source) => {
            basis.text(
                format!("{prefix}.kind"),
                match source.kind {
                    SourceKind::Stored => "stored",
                    SourceKind::Derived => "derived",
                },
            );
            basis.text(format!("{prefix}.relation"), &source.relation);
            basis.text(format!("{prefix}.from"), &source.from);
            basis.text(format!("{prefix}.to"), &source.to);
        }
        Node::Compose { left, right } => {
            basis.text(format!("{prefix}.kind"), "compose");
            basis.usize(format!("{prefix}.left"), *left);
            basis.usize(format!("{prefix}.right"), *right);
        }
        Node::Reverse { input } | Node::WithoutSelfEdges { input } => {
            basis.text(
                format!("{prefix}.kind"),
                if matches!(node, Node::Reverse { .. }) {
                    "reverse"
                } else {
                    "without-self-edges"
                },
            );
            basis.usize(format!("{prefix}.input"), *input);
        }
    }
}
