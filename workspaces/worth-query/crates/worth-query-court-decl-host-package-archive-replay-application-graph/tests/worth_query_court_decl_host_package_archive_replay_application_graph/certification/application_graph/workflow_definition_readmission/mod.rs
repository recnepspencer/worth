pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_definition_readmission::*;

#[test]
fn current_draft_policy_drift_refuses_before_publication() {
    use worth_query_package_archive::facade::{
        decode_workflow_definition_draft, encode_workflow_definition_draft,
        WorthQueryPackageArchiveLimits, WorthQueryWorkflowDefinitionDraftDenialKind as Kind,
    };
    let application = publish_workflow_on_first_program();
    let definition =
        super::workflow_execution::definition::<OccurrenceSetRetentionBinding>().unwrap();
    let bytes =
        encode_workflow_definition_draft(&definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    let head = application.runtime().current_world();
    for (field, expected) in [
        (1, Kind::ChangedObservationPolicy),
        (2, Kind::ChangedActorPolicy),
    ] {
        let altered = replace_header_text(&bytes, field, "unsupported-policy");
        let draft =
            decode_workflow_definition_draft(&altered, WorthQueryPackageArchiveLimits::DEFAULT)
                .unwrap();
        let denial = draft
            .author(application.workflow_spec())
            .err()
            .expect("changed policy refuses");
        assert_eq!(denial.kind(), expected);
        assert_eq!(application.runtime().current_world(), head);
    }
}

pub(crate) mod current;
