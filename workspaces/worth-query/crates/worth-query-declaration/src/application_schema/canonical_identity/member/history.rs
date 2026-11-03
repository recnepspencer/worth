use crate::application_history::ErasedApplicationHistoryDefinition;
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationHistoryDefinition,
) {
    let parts = definition.parts();
    basis.text(format!("{prefix}.kind"), "application-history");
    basis.text(format!("{prefix}.identity"), &parts.identity);
    basis.text(
        format!("{prefix}.model-domain"),
        match parts.model_domain {
            crate::application_history::ApplicationHistoryModelDomain::WholeComposite => {
                "whole-composite"
            }
            crate::application_history::ApplicationHistoryModelDomain::ScopeEntityRecord => {
                "scope-entity-record"
            }
            crate::application_history::ApplicationHistoryModelDomain::OwnedModelGraph => {
                "owned-model-graph"
            }
        },
    );
    basis.text(
        format!("{prefix}.history-type"),
        parts.history_type.as_str(),
    );
    for (field, value) in parts.access.canonical_fields() {
        basis.text(format!("{prefix}.access.{field}"), value);
    }
    basis.extend_embedded(
        parts.metadata_disclosure.canonical_basis().basis(),
        &format!("{prefix}.metadata-disclosure"),
        worth_foundational::facade::CanonicalBasisEntryKind::Identity,
    );
    basis.usize(format!("{prefix}.operation-count"), parts.operations.len());
    // Preserve the existing canonical basis for every unowned definition.
    if !parts.ownership.is_empty() {
        basis.usize(format!("{prefix}.ownership-count"), parts.ownership.len());
        for (index, relation) in parts.ownership.iter().enumerate() {
            basis.text(
                format!("{prefix}.ownership[{index}].relation"),
                &relation.relation,
            );
            basis.text(format!("{prefix}.ownership[{index}].from"), &relation.from);
            basis.text(format!("{prefix}.ownership[{index}].to"), &relation.to);
        }
    }
    if let Some(navigation) = &parts.navigation {
        for (field, value) in navigation.canonical_fields() {
            basis.text(format!("{prefix}.snapshot-navigation.{field}"), value);
        }
    }
    for (role, policy) in [("window", &parts.window), ("bookmarks", &parts.bookmarks)] {
        if let Some(policy) = policy {
            for (field, value) in policy.canonical_fields() {
                basis.text(format!("{prefix}.{role}.{field}"), value);
            }
        }
    }
    for (index, operation) in parts.operations.iter().enumerate() {
        basis.text(
            format!("{prefix}.operation[{index}].binding"),
            &operation.binding,
        );
        basis.text(
            format!("{prefix}.operation[{index}].label"),
            &operation.label,
        );
    }
}
