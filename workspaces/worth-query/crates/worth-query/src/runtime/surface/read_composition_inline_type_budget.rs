//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryReadGraph>(
            "workspaces/worth-query/crates/worth-query/src/runtime/surface/read_composition.rs",
            "WorthQueryReadGraph",
            128 * 1024,
        );
        owner.field::<WorthQueryEvidenceIdentity>("identity");
        owner.field::<WorthQueryApplicationCanonicalArtifact>("canonical_planning");
        owner.field::<WorthQueryReadGraphFamily>("family");
        owner.field::<WorthQueryReadScopeClass>("scope_class");
        owner.field::<SchemaBasisDigest>("schema_basis");
        owner.field::<Vec<WorthQueryReadBuiltInOperator>>("built_in_operators");
        owner
            .field::<Vec<WorthQueryGraphReadDomainOperationDeclaration>>("domain_graph_operations");
        owner.field::<Vec<WorthQueryInstalledGraphReadOperation>>("installed_operation_bindings");
        owner.field::<usize>("declared_traversal_clause_count");
        owner.field::<usize>("declared_traversal_depth_limit");
        owner.field::<Option<RelationshipProofAdmission>>("relationship_proof_admission");
        owner.field::<Option<PolicyAwareCurrentPlan>>("policy_aware_plan");
        owner.field::<Option<AuthorizedProjectionArtifact>>("authorized_projection");
        owner.field::<CanonicalQueryBundle>("canonical");
        owner.field::<ValidatedQueryBundle>("validated");
        owner.field::<DeclarativeLiveQueryRequest>("declarative_request");
        owner.field::<QuerySchemaView>("schema_view");
        owner.field::<ExecutionPlanBundle>("execution_plan");
    }
}
