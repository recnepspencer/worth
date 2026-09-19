use worth_query_declaration::facade::application_schema::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
};

pub(super) const fn issued_lifecycle_relation() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(
            true,
            ApplicationRelationCrossContextPolicy::AllowExplicit,
        ),
        ApplicationRelationCardinality::unbounded(),
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
    )
}
