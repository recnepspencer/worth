use super::super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName, TraversalSelector,
};
use crate::authorized_projection::PolicyAspectMask;
use crate::ordinary::read::{
    current, declare, WorthQueryReadRelationshipProof, WorthQueryReadRelationshipProofs,
};
use crate::policy_basis::{BranchAccessGrant, PolicyEpoch, PolicyRuleSnapshot};
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};
use crate::tenant_basis::{SchemaVariantSnapshot, TenantBasisEpoch, TenantBindingSnapshot};

pub(super) fn local_identity_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_detail(
        "user",
        manager_schema(),
        |query| {
            query.project(
                AspectFieldSelector::new("identity", "id")
                    .expect("identity projection should build"),
            )
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("identity result field should build"),
            )
        },
    )
}

pub(super) fn local_identity_collection_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_collection(
        "user",
        manager_schema(),
        |query| {
            query.project(
                AspectFieldSelector::new("identity", "id")
                    .expect("identity projection should build"),
            )
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("identity result field should build"),
            )
        },
    )
}

pub(super) fn anchored_manager_graph_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.anchored_collection(
        "user",
        expanded_manager_schema(),
        |query| {
            query
                .traverse(
                    TraversalSelector::bounded("manager", 2)
                        .expect("manager traversal should build"),
                )
                .project(
                    AspectFieldSelector::new("identity", "id")
                        .expect("identity projection should build"),
                )
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("identity result field should build"),
            )
        },
    )
}

pub(super) fn local_manager_relationship_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_direct_edge_detail(
        "user",
        manager_schema(),
        manager_relation_name(),
        |query| {
            query.project(
                AspectFieldSelector::new("identity", "id")
                    .expect("identity projection should build"),
            )
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("identity result field should build"),
            )
        },
    )
}

pub(super) fn bounded_descendant_manager_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.anchored_bounded_descendant_detail(
        "user",
        expanded_manager_schema(),
        manager_relation_name(),
        2,
        |query| {
            query.project(
                AspectFieldSelector::new("identity", "id")
                    .expect("identity projection should build"),
            )
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("identity result field should build"),
            )
        },
    )
}

pub(super) fn local_policy_projection_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_detail(
        "user",
        policy_projection_schema(),
        |query| {
            query
                .project(identity_field())
                .project(display_name_field())
                .project(handle_field())
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("identity result field should build"),
            )
        },
    )
}

pub(super) fn local_policy_result_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_detail(
        "user",
        policy_projection_schema(),
        |query| {
            query
                .project(identity_field())
                .project(display_name_field())
        },
        |shape| {
            shape
                .field(
                    AuthoredResultShapeField::new("identity", "id", "id")
                        .expect("identity result field should build"),
                )
                .field(
                    AuthoredResultShapeField::new("profile", "display_name", "display_name")
                        .expect("profile result field should build"),
                )
        },
    )
}

pub(super) struct PolicyTenantInputs {
    pub(super) policy: PolicyRuleSnapshot,
    pub(super) tenant: TenantBindingSnapshot,
    pub(super) branch: BranchAccessGrant,
    pub(super) schema: SchemaVariantSnapshot,
}

pub(super) fn admitted_policy_tenant_inputs(epoch: u64, admits_query: bool) -> PolicyTenantInputs {
    let policy = PolicyRuleSnapshot::synthetic_authority_with_query_admission(
        "ordinary-policy",
        "ordinary-rules",
        PolicyEpoch::Synthetic(epoch),
        admits_query,
    );
    let tenant = TenantBindingSnapshot::synthetic_direct(
        "tenant-a",
        "main",
        "schema-a",
        TenantBasisEpoch::Synthetic(7),
    );
    let branch = BranchAccessGrant::synthetic_granted("main", &policy);
    let schema = SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "exact");
    PolicyTenantInputs {
        policy,
        tenant,
        branch,
        schema,
    }
}

pub(super) fn current_manager_relationship_context(
) -> crate::ordinary::read::WorthQueryCurrentRelationshipReadContext {
    let policy_tenant = admitted_policy_tenant_inputs(1, true);
    let relationships = WorthQueryReadRelationshipProofs::bounded(
        [WorthQueryReadRelationshipProof::direct_edge(
            manager_relation_name(),
        )],
        1,
        1,
    )
    .expect("manager relationship proof should be bounded");
    current()
        .under_policy_tenant(
            policy_tenant.policy,
            policy_tenant.tenant,
            policy_tenant.branch,
            policy_tenant.schema,
        )
        .with_relationship_proofs(relationships)
}

pub(super) fn current_bounded_manager_relationship_context(
) -> crate::ordinary::read::WorthQueryCurrentRelationshipReadContext {
    let policy_tenant = admitted_policy_tenant_inputs(1, true);
    let relationships = WorthQueryReadRelationshipProofs::bounded(
        [WorthQueryReadRelationshipProof::bounded_ancestor(
            manager_relation_name(),
            crate::ordinary::read::WorthQueryReadRelationshipDepth::new(2)
                .expect("bounded manager depth should author"),
        )],
        1,
        2,
    )
    .expect("bounded manager relationship proof should be bounded");
    current()
        .under_policy_tenant(
            policy_tenant.policy,
            policy_tenant.tenant,
            policy_tenant.branch,
            policy_tenant.schema,
        )
        .with_relationship_proofs(relationships)
}

pub(super) fn narrowing_policy_tenant_inputs(
    epoch: u64,
    projection_mask: PolicyAspectMask,
) -> PolicyTenantInputs {
    let policy = PolicyRuleSnapshot::synthetic_authority_with_projection(
        "ordinary-narrowing-policy",
        "ordinary-narrowing-rules",
        PolicyEpoch::Synthetic(epoch),
        projection_mask,
    );
    let tenant = TenantBindingSnapshot::synthetic_direct(
        "tenant-a",
        "main",
        "schema-a",
        TenantBasisEpoch::Synthetic(7),
    );
    let branch = BranchAccessGrant::synthetic_granted("main", &policy);
    let schema = SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "exact");
    PolicyTenantInputs {
        policy,
        tenant,
        branch,
        schema,
    }
}

pub(super) fn display_name_field() -> AspectFieldSelector {
    AspectFieldSelector::new("profile", "display_name")
        .expect("display-name field selector should build")
}

pub(super) fn handle_field() -> AspectFieldSelector {
    AspectFieldSelector::new("profile", "handle").expect("handle field selector should build")
}

fn identity_field() -> AspectFieldSelector {
    AspectFieldSelector::new("identity", "id").expect("identity field selector should build")
}

fn policy_projection_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "ordinary-policy-projection",
        [
            schema_field("identity", "id"),
            schema_field("profile", "display_name"),
            schema_field("profile", "handle"),
        ],
        [],
    )
}

fn schema_field(aspect: &str, field: &str) -> SchemaFieldView {
    SchemaFieldView::new(
        AspectName::new(aspect).expect("policy schema aspect should be valid"),
        FieldName::new(field).expect("policy schema field should be valid"),
        ScalarAspectType::String,
    )
}

pub(super) fn run_policy_context(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    epoch: u64,
) -> crate::ordinary::read::WorthQueryReadContextReceipt {
    let declaration = declare(local_identity_read).expect("read declaration should canonicalize");
    let policy_tenant = admitted_policy_tenant_inputs(epoch, true);
    let context = current().under_policy_tenant(
        policy_tenant.policy,
        policy_tenant.tenant,
        policy_tenant.branch,
        policy_tenant.schema,
    );
    let mut workspace = read_runtime(request_execution)
        .workspace(format!("ordinary-read-policy-epoch-{epoch}"))
        .expect("ordinary workspace should open");
    declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("policy context should execute")
        .context_receipt()
        .clone()
}

pub(super) fn run_relationship_context<const PROOF_COUNT: usize>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    proofs: [WorthQueryReadRelationshipProof; PROOF_COUNT],
) -> crate::ordinary::read::WorthQueryReadCompletion {
    let declaration = declare(local_manager_relationship_read)
        .expect("relationship read declaration should canonicalize");
    let policy_tenant = admitted_policy_tenant_inputs(1, true);
    let relationships = WorthQueryReadRelationshipProofs::bounded(proofs, 2, 2)
        .expect("relationship proof set should be bounded");
    let context = current()
        .under_policy_tenant(
            policy_tenant.policy,
            policy_tenant.tenant,
            policy_tenant.branch,
            policy_tenant.schema,
        )
        .with_relationship_proofs(relationships);
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-relationship-order")
        .expect("ordinary workspace should open");
    declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("relationship context should execute")
}
