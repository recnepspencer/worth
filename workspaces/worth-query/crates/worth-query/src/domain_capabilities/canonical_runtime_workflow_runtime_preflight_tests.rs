use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::test_support::{declaration_target, ready, success};
use super::{
    materialize_lowered_mutation_intent_declaration, materialize_query_workflow_declaration,
    WorthQueryWorkflowContributionAuthoring,
};
use crate::harness::fixtures::execution_preflights;
use crate::workflow::{bind_workflow_context, WorkflowBindingSource};

#[test]
fn workflow_runtime_preflight_materializer_preserves_real_preflight_query_and_basis_identity(
) -> Result<(), IdentityDerivationDenial> {
    let preflight = execution_preflights::direct_runtime_preflight();
    let expected = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime preflight should bind");

    let declaration = success(materialize_query_workflow_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection_from_preflight(
            "spatial.runtime.preflight.review",
            "domain workflow should reuse the real runtime preflight binding",
            preflight.clone(),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-runtime-preflight")),
    )?)?);

    assert_ne!(
        declaration.binding().binding_digest(),
        expected.binding_digest()
    );
    assert_eq!(
        declaration.binding().source_for_reporting(),
        expected.source_for_reporting()
    );
    assert_eq!(
        declaration
            .binding()
            .runtime_snapshot_identity()
            .map(|identity| identity.evidence_identity()),
        Some(
            crate::memory_workspace::WorthQuerySnapshotIdentity::preview(
                preflight.basis().identity().snapshot_identity().clone(),
            )
            .evidence_identity()
        )
    );
    assert_eq!(
        declaration.binding().query_for_reporting(),
        expected.query_for_reporting()
    );
    assert_eq!(
        declaration.binding().basis_for_reporting(),
        expected.basis_for_reporting()
    );

    Ok(())
}

#[test]
fn workflow_runtime_preflight_materializer_is_stronger_than_snapshot_only_surrogate(
) -> Result<(), IdentityDerivationDenial> {
    let snapshot_identity = crate::harness::fixtures::resolved_bases::primary_snapshot_identity();
    let snapshot_evidence = snapshot_identity.evidence_identity();
    let preflight =
        execution_preflights::runtime_preflight_with_snapshot_identity(snapshot_identity);

    let real = success(materialize_query_workflow_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection_from_preflight(
            "spatial.runtime.preflight.real",
            "real runtime preflight should preserve full query and basis identity",
            preflight.clone(),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-runtime-real")),
    )?)?);
    let surrogate = success(materialize_query_workflow_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection(
            "spatial.runtime.preflight.synthetic",
            "snapshot-token-only workflow semantics remain a weaker surrogate path",
            crate::memory_workspace::WorthQuerySnapshotIdentity::preview(snapshot_evidence),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-runtime-synthetic")),
    )?)?);

    assert_ne!(
        real.binding().binding_digest(),
        surrogate.binding().binding_digest()
    );
    assert_ne!(
        real.binding().query_for_reporting(),
        surrogate.binding().query_for_reporting()
    );
    assert_eq!(
        real.binding()
            .runtime_snapshot_identity()
            .map(|identity| identity.evidence_identity()),
        surrogate
            .binding()
            .runtime_snapshot_identity()
            .map(|identity| identity.evidence_identity())
    );

    Ok(())
}

#[test]
fn workflow_runtime_preflight_lowering_from_bundle_preserves_real_runtime_authority(
) -> Result<(), IdentityDerivationDenial> {
    let preflight = execution_preflights::direct_runtime_preflight();
    let authority_binding_identity = crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowMutationLowering,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("test_authority_binding"),
        "runtime-preflight",
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("binding"),
        "authority-binding:preflight",
    )
    .seal()?;

    let lowered = success(materialize_lowered_mutation_intent_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_mutation_reconciliation_from_preflight(
            "spatial.runtime.preflight.lowering",
            "mutation lowering should preserve real runtime preflight authority",
            preflight.clone(),
            authority_binding_identity.clone(),
            EntityId::new(PartitionId(7), 11, 0),
            crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                "name", "name", "after",
            )
            .expect("name patch should build"),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-runtime-lowering")),
    )?));

    assert_eq!(
        lowered
            .authority_binding()
            .runtime_snapshot_identity()
            .map(|identity| identity.evidence_identity()),
        Some(
            crate::memory_workspace::WorthQuerySnapshotIdentity::preview(
                preflight.basis().identity().snapshot_identity().clone(),
            )
            .evidence_identity()
        )
    );
    assert_eq!(
        lowered.authority_binding().binding_digest(),
        authority_binding_identity.as_str()
    );
    assert_eq!(lowered.counters().workflow_mutation_lowering_count(), 1);

    Ok(())
}

#[test]
fn workflow_runtime_preflight_materializer_keeps_scope_distinct_across_targets(
) -> Result<(), IdentityDerivationDenial> {
    let preflight = execution_preflights::direct_runtime_preflight();

    let left = success(materialize_query_workflow_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection_from_preflight(
            "spatial.runtime.preflight.scope",
            "real runtime preflight bindings must stay scoped by contribution target",
            preflight.clone(),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-runtime-left")),
    )?)?);
    let right = success(materialize_query_workflow_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection_from_preflight(
            "spatial.runtime.preflight.scope",
            "real runtime preflight bindings must stay scoped by contribution target",
            preflight.clone(),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-runtime-right")),
    )?)?);

    assert_eq!(
        left.binding().query_for_reporting(),
        right.binding().query_for_reporting()
    );
    assert_eq!(
        left.binding().basis_for_reporting(),
        right.binding().basis_for_reporting()
    );
    assert_ne!(
        left.binding().binding_digest(),
        right.binding().binding_digest()
    );
    assert_ne!(
        left.report().declaration_digest(),
        right.report().declaration_digest()
    );

    Ok(())
}
