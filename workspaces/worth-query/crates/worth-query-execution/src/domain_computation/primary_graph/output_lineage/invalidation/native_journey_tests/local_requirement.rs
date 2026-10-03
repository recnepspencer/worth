//! A post-effect lineage requirement cannot be hidden by an older Clean mark.

use super::*;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryCheckpointSourceIdentity,
    invariant_projection::{ConsumedOutputEvidence, ConsumedOutputVerification},
    output_lineage::{
        invalidation::FullVerificationReason, RecordedSourceIdentity,
        WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
    },
};

#[test]
fn local_full_verification_requirement_overrides_a_clean_actor_row() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let observation = product.observation();
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let (before_handle, before, retained_fact) = handle.with_runtime(|runtime| {
        let (snapshot, basis) = snapshot(runtime);
        let fact = field_fact(runtime, &snapshot, entity, status.clone());
        let facts: Arc<[_]> = Arc::from([fact]);
        (snapshot, basis, facts)
    });

    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.install_output_families(BTreeMap::from([(
        "local-requirement".to_owned(),
        vec![TypeId::of::<StatusOutput>()],
    )]));
    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity);
    lineage.record_restoration(
        TypeId::of::<StatusOutput>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::new(WorthQueryApplicationOutputCorrespondence::default()),
        RecordedSourceIdentity::Checkpoint(WorthQueryCheckpointSourceIdentity::new([0x71; 32])),
        [0x72; 32],
        None,
        [0x73; 32],
        Arc::clone(&retained_fact),
        None,
        None,
    );
    let identity = Arc::clone(
        &lineage
            .resolve_current_family(
                runtime_authority,
                &schema,
                scope,
                "local-requirement",
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                32,
            )
            .unwrap()
            .candidates
            .pop()
            .expect("the lineage owner selects its retained output")
            .settlement_identity,
    );
    // The old actor row has no posting for this source field. A failed
    // post-effect registration leaves it Clean while lineage retains the
    // actual fact and records a full-verification requirement.
    register(
        owner,
        Arc::clone(&identity),
        Arc::from([]),
        &before,
        OrdSet::new(),
    );
    lineage.require_settlement_verification(
        &identity,
        FullVerificationReason::NativeRevisionUnavailable,
    );
    handle.with_runtime_mut(|runtime| write_field(runtime, entity, status, "closed"));
    let (after_handle, after) = handle.with_runtime(snapshot);
    assert!(matches!(
        owner.currentness(&after, &identity, &mut owner.edit_admission()),
        Ok(SourceSettlementCurrentness::Clean)
    ));

    let candidate = lineage
        .resolve_current_family(
            runtime_authority,
            &schema,
            scope,
            "local-requirement",
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            32,
        )
        .unwrap()
        .candidates
        .pop()
        .expect("the local requirement stays on the selected owner row");
    assert_eq!(
        candidate.verification_requirement,
        Some(FullVerificationReason::NativeRevisionUnavailable)
    );
    let mut admission = owner.read_admission(1_000_000);
    let result = handle.with_runtime(|runtime| {
        ConsumedOutputEvidence::verify_candidate_with_admission(
            &candidate.settlement_identity,
            &candidate.observed_source_facts,
            &candidate.consumed_outputs,
            candidate.verification_requirement,
            &before,
            owner,
            runtime,
            &after_handle,
            &after,
            &mut admission,
        )
    });
    assert_eq!(result, Ok(ConsumedOutputVerification::ChangedDirectFact(0)));
    handle.with_runtime_mut(|runtime| {
        runtime
            .snapshots()
            .release_snapshot(&before_handle)
            .unwrap();
        runtime.snapshots().release_snapshot(&after_handle).unwrap();
    });
}
