use worth_query_declaration::facade::application_schema::ApplicationRelationCardinality;
use worth_query_declaration::facade::application_schema::ApplicationRelationCrossContextPolicy;
use worth_query_declaration::facade::application_schema::ApplicationRelationDeletionPolicy;
use worth_query_declaration::facade::application_schema::ApplicationRelationEndpoints;
use worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity;

pub const fn issued_lifecycle_relation() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(
            true,
            ApplicationRelationCrossContextPolicy::AllowExplicit,
        ),
        ApplicationRelationCardinality::unbounded(),
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
    )
}
