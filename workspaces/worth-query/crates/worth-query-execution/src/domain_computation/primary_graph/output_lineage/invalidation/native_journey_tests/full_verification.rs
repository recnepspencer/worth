//! Real Native probes and immutable pre-verification capture; no synthetic stamp.

use super::super::{FullVerificationDecision, FullVerificationStop};
use super::*;

#[test]
fn full_verification_uses_native_facts_and_retains_the_exact_captured_image() {
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
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: TypeId::of::<LabelOutput>(),
    };
    let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_open_runtime_mut(|runtime| {
        let (old_handle, old_basis) = snapshot(runtime);
        use crate::domain_computation::primary_graph::invariant_projection::{
            ConsumedOutputEvidence, ConsumedOutputVerification,
        };
        assert_eq!(
            ConsumedOutputEvidence::verify_many_with_admission(
                &[],
                owner,
                runtime,
                &old_handle,
                &old_basis,
                &mut owner.read_admission(0)
            ),
            Ok(ConsumedOutputVerification::Current),
            "an empty closure needs no actor image or fact authority"
        );
        register(
            owner,
            identity.clone(),
            Arc::from([field_fact(runtime, &old_handle, entity, label.clone())]),
            &old_basis,
            OrdSet::new(),
        );
        let old = owner
            .capture_full_verification(
                runtime,
                &old_handle,
                &old_basis,
                &mut owner.edit_admission(),
            )
            .unwrap();
        assert_eq!(
            old.verify_root(&identity, None, &mut owner.edit_admission()),
            Err(FullVerificationStop::OutputEvidenceUnavailable),
            "source-only test registration cannot manufacture full output evidence"
        );
        drop(old);
        write_field(runtime, entity, label.clone(), "changed-for-full-oracle");
        let (changed_handle, changed_basis) = snapshot(runtime);
        let captured = owner
            .capture_full_verification(
                runtime,
                &changed_handle,
                &changed_basis,
                &mut owner.edit_admission(),
            )
            .unwrap();
        assert_eq!(
            captured.verify_root(&identity, None, &mut owner.edit_admission()),
            Ok(FullVerificationDecision::Changed)
        );
        // A later same-position registration replaces the actor's row. The
        // captured oracle still owns the earlier facts and cannot read that edit.
        register(
            owner,
            identity.clone(),
            Arc::from([field_fact(runtime, &changed_handle, entity, label)]),
            &changed_basis,
            OrdSet::new(),
        );
        assert!(matches!(
            currentness(owner, &changed_basis, &identity),
            SourceSettlementCurrentness::Clean
        ));
        assert_eq!(
            captured.verify_root(&identity, None, &mut owner.edit_admission()),
            Err(FullVerificationStop::ActorImageChanged),
            "an intervening actor edit cannot manufacture an equivalence mismatch"
        );
        let fresh = owner
            .capture_full_verification(
                runtime,
                &changed_handle,
                &changed_basis,
                &mut owner.edit_admission(),
            )
            .unwrap();
        assert_eq!(
            fresh.verify_root(&identity, None, &mut owner.edit_admission()),
            Err(FullVerificationStop::OutputEvidenceUnavailable)
        );
        assert!(
            owner
                .capture_full_verification(
                    runtime,
                    &old_handle,
                    &changed_basis,
                    &mut owner.edit_admission()
                )
                .is_err(),
            "Native and actor provenance cannot be substituted"
        );
        drop(captured);
        drop(fresh);
        runtime.snapshots().release_snapshot(&old_handle).unwrap();
        runtime
            .snapshots()
            .release_snapshot(&changed_handle)
            .unwrap();
    });
}
