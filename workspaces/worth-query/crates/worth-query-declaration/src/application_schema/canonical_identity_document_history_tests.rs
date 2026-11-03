//! A graph domain cannot share a canonical schema identity with a record domain.
use super::{identity, portable, ApplicationSchemaMember};
use crate::{application_history::*, application_query::*};

fn definition(model_domain: ApplicationHistoryModelDomain) -> ErasedApplicationHistoryDefinition {
    ErasedApplicationHistoryDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationHistoryParts {
            identity: "ModelHistory".into(),
            history_type: portable("history.Model"),
            access: ApplicationQueryBindingReference::from_untrusted_parts(
                ApplicationQueryBindingReferenceParts {
                    binding: portable("ModelRead"),
                    input: portable("ModelReadInput"),
                    query_name: "ReadModel".into(),
                    query: portable("ReadModelQuery"),
                    parameters: portable("ModelParameters"),
                    result: portable("ModelResult"),
                    scope: portable("ModelScope"),
                    scope_entity: "Model".into(),
                    scope_aspect: "Identity".into(),
                    scope_field: "Key".into(),
                    scope_value: portable("ModelKey"),
                    scope_resolution: ApplicationQueryScopeResolutionMode::InputField,
                    scope_writable: false,
                    principal_binding: "Operator".into(),
                },
            ),
            metadata_disclosure: ApplicationQueryDisclosureContract::public(),
            model_domain,
            ownership: Vec::new(),
            navigation: None,
            window: None,
            bookmarks: None,
            operations: vec![ApplicationHistoryOperation {
                binding: "EditModel".into(),
                label: "Edit model".into(),
            }],
        },
    )
}

#[test]
fn owned_model_graph_domain_has_distinct_canonical_meaning_from_both_existing_domains() {
    let graph = definition(ApplicationHistoryModelDomain::OwnedModelGraph);
    graph.validate().unwrap();
    let graph = identity(&[ApplicationSchemaMember::ApplicationHistory { definition: graph }]);
    for domain in [
        ApplicationHistoryModelDomain::WholeComposite,
        ApplicationHistoryModelDomain::ScopeEntityRecord,
    ] {
        assert_ne!(
            graph,
            identity(&[ApplicationSchemaMember::ApplicationHistory {
                definition: definition(domain)
            }])
        );
    }
}
