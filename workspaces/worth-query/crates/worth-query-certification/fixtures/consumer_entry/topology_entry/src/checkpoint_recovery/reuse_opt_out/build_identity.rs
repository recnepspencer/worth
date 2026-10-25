//! One real None-producer checkpoint across same and changed application builds.
use super::*;
use worth_query_consumer_values::{
    PlanarCurrentOutputExpectation, PlanarMutationDenial, PlanarOperation,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationOutputDemandSettlement,
    WorthQueryApplicationRequest, WorthQueryOutputSettlementPosture as Posture,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

type Application =
    application_installation::WorthQueryProgramApplicationRuntime<NoReuseSchema, NoReuseProgram>;
type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, NoReuseSchema>;

#[test]
fn none_producers_reopen_same_build_and_regenerate_changed_build_with_original_entities() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let build_a = ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap();
    let build_b = ApplicationBuildIdentity::parse("22222222-2222-2222-2222-222222222222").unwrap();
    let mut original =
        install_no_reuse_with_build(None, build_a, Some(provider_observation.clone()));
    let (source_entity, final_entity, original_root_commit, original_final_commit) = {
        let (scope, principal) = support::authenticate(&original);
        let request = original.request(&principal, &scope);
        let root = root_demand(&request, &original);
        let source_entity = root
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<NoReuseSchema>>()
            .unwrap()
            .entity_id();
        drop(root);
        let first = final_demand(&request, &original);
        let entity = first
            .outputs_of::<FinalPlanarOutputs>()
            .unwrap()
            .entity::<FinalAnchorOutput<NoReuseSchema>>()
            .unwrap()
            .entity_id();
        drop(first);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: support::length(9),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&0x2712_0101_u64)
            .execute_performed::<NoReuseProgram, program::Root>(
                &original,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        drop(source);
        let updated_root = root_demand(&request, &original);
        assert_eq!(updated_root.posture(), Posture::Performed);
        assert!(updated_root.producer_contacts_in_this_demand() > 0);
        assert_eq!(
            updated_root
                .outputs_of::<PlanarOutputs>()
                .unwrap()
                .entity::<PlanarAnchorOutput<NoReuseSchema>>()
                .unwrap()
                .entity_id(),
            source_entity
        );
        assert_root_value(&request);
        let original_root_commit = (*updated_root
            .application_commit_receipt()
            .unwrap()
            .committed_product_publication()
            .composite_commit())
        .clone();
        drop(updated_root);
        let preserved = final_demand(&request, &original);
        assert_eq!(preserved.posture(), Posture::Performed);
        assert_eq!(
            preserved
                .outputs_of::<FinalPlanarPreserveOutputs>()
                .unwrap()
                .entity::<FinalPreservedAnchorOutput<NoReuseSchema>>()
                .unwrap()
                .entity_id(),
            entity
        );
        assert_values(&request);
        // Retain only the commit descriptor, not A's receipt or runtime custody.
        let original_final_commit = (*preserved
            .application_commit_receipt()
            .unwrap()
            .committed_product_publication()
            .composite_commit())
        .clone();
        (
            source_entity,
            entity,
            original_root_commit,
            original_final_commit,
        )
    };
    let checkpoint = original
        .with_host_advancement(|phase| {
            original
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    let saved = checkpoint.bytes().to_vec();
    drop(checkpoint);
    assert!(original.close_live_delivery().owner_terminal());
    original.close_conditional_runtime().unwrap();
    drop(original);

    reset_contacts(&provider_observation);
    let mut same = install_no_reuse_with_build(
        Some(reopen(&saved)),
        build_a,
        Some(provider_observation.clone()),
    );
    {
        let (scope, principal) = support::authenticate(&same);
        let request = same.request(&principal, &scope);
        assert_eq!(verify_current(&request, &same, 0x2712_0102), Ok(()));
        let reused_root = root_demand(&request, &same);
        assert_eq!(reused_root.posture(), Posture::RecoveredPerformed);
        assert_eq!(reused_root.producer_contacts_in_this_demand(), 0);
        assert_eq!(
            reused_root
                .outputs_of::<PlanarOutputs>()
                .unwrap()
                .entity::<PlanarAnchorOutput<NoReuseSchema>>()
                .unwrap()
                .entity_id(),
            source_entity
        );
        assert_root_value(&request);
        drop(reused_root);
        let reused = final_demand(&request, &same);
        assert_eq!(reused.producer_contacts_in_this_demand(), 0);
        assert_eq!(reused.posture(), Posture::RecoveredPerformed);
        assert_eq!(
            reused
                .outputs_of::<FinalPlanarPreserveOutputs>()
                .unwrap()
                .entity::<FinalPreservedAnchorOutput<NoReuseSchema>>()
                .unwrap()
                .entity_id(),
            final_entity
        );
        assert_values(&request);
        assert_eq!(provider_observation.count(), 0);
        assert_eq!(crate::final_output::provider_contacts(), 0);
    }
    assert!(same.close_live_delivery().owner_terminal());
    same.close_conditional_runtime().unwrap();
    drop(same);

    reset_contacts(&provider_observation);
    let mut changed = install_no_reuse_with_build(
        Some(reopen(&saved)),
        build_b,
        Some(provider_observation.clone()),
    );
    {
        let (scope, principal) = support::authenticate(&changed);
        let request = changed.request(&principal, &scope);
        // Raw source/value preservation does not make the old output Current.
        assert_values(&request);
        assert_eq!(
            verify_current(&request, &changed, 0x2712_0103),
            Err(PlanarMutationDenial::CurrentOutputMissing)
        );
        assert_eq!(provider_observation.count(), 0);
        assert_eq!(crate::final_output::provider_contacts(), 0);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let selected = support::host_call(&changed, |phase| {
            changed.select_output_producer::<PlanarFinalOutputFamily>(
                phase,
                &source.observed_sources()[0],
                "planar-final",
            )
        })
        .unwrap();
        assert_eq!(
            selected.applicability().lifecycle(),
            WorthQueryProducerLifecyclePosture::Preserve
        );
        drop(source);
        // Final consumes authored PlanarRead, so exercise Root explicitly once.
        let root = root_demand(&request, &changed);
        assert_eq!(root.posture(), Posture::Performed);
        assert!(root.producer_contacts_in_this_demand() > 0);
        assert_ne!(
            root.application_commit_receipt()
                .unwrap()
                .committed_product_publication()
                .composite_commit(),
            &original_root_commit,
            "build B cannot replay build A's original root publication",
        );
        assert_root_value(&request);
        assert_eq!(
            root.outputs_of::<PlanarOutputs>()
                .unwrap()
                .entity::<PlanarAnchorOutput<NoReuseSchema>>()
                .unwrap()
                .entity_id(),
            source_entity
        );
        drop(root);
        // Isolate the final publication: the root commit cannot satisfy this check.
        let before = support::retain_read(&changed, &request).unwrap();
        let regenerated = final_demand(&request, &changed);
        let after = support::retain_read(&changed, &request).unwrap();
        assert_eq!(regenerated.posture(), Posture::Performed);
        assert!(regenerated.producer_contacts_in_this_demand() > 0);
        assert!(provider_observation.count() > 0);
        assert!(crate::final_output::provider_contacts() > 0);
        assert_ne!(
            before.selected_commit(),
            after.selected_commit(),
            "the final demand must advance the native commit"
        );
        assert_ne!(
            regenerated
                .application_commit_receipt()
                .unwrap()
                .committed_product_publication()
                .composite_commit(),
            &original_final_commit,
            "build B cannot replay build A's original final publication",
        );
        assert_eq!(
            regenerated
                .application_commit_receipt()
                .unwrap()
                .committed_product_publication()
                .composite_commit(),
            after.selected_commit()
        );
        assert_eq!(
            regenerated
                .outputs_of::<FinalPlanarPreserveOutputs>()
                .unwrap()
                .entity::<FinalPreservedAnchorOutput<NoReuseSchema>>()
                .unwrap()
                .entity_id(),
            final_entity
        );
        assert_values(&request);
        assert_eq!(verify_current(&request, &changed, 0x2712_0104), Ok(()));
    }
    assert!(changed.close_live_delivery().owner_terminal());
    changed.close_conditional_runtime().unwrap();
    drop(changed);
}

fn reopen(bytes: &[u8]) -> application_installation::WorthQueryApplicationCheckpoint {
    application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.to_vec())
}

fn reset_contacts(provider_observation: &crate::producer::ProviderContacts) {
    provider_observation.reset();
    crate::final_output::reset_provider_contacts();
}

fn root_demand<'a, 'p, 's>(
    request: &Request<'a, 'p, 's>,
    application: &'a Application,
) -> WorthQueryApplicationOutputDemandSettlement<PlanarQuery> {
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<NoReuseProgram, program::Root>(application)
        .unwrap();
    for _ in 0..64 {
        if let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.advance(request).unwrap()
        {
            return settled;
        }
    }
    panic!("the real root demand must settle");
}

fn final_demand<'a, 'p, 's>(
    request: &Request<'a, 'p, 's>,
    application: &'a Application,
) -> WorthQueryApplicationOutputDemandSettlement<PlanarQuery> {
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<NoReuseProgram, FinalConnection>(application)
        .unwrap();
    for _ in 0..64 {
        if let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.advance(request).unwrap()
        {
            return settled;
        }
    }
    panic!("the real final demand must settle");
}

fn assert_values(request: &Request<'_, '_, '_>) {
    assert_root_value(request);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 1);
    assert_eq!(source.rows()[0].y, support::length(9));
    let output = request
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        output.rows().len(),
        1,
        "Preserve must not create a second entity under the key"
    );
    assert_eq!(output.rows()[0].value, support::length(11));
}

fn assert_root_value(request: &Request<'_, '_, '_>) {
    let root = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(root.rows().len(), 1);
    assert_eq!(root.rows()[0].value, support::length(10));
}

fn verify_current<'a, 'p, 's>(
    request: &Request<'a, 'p, 's>,
    application: &'a Application,
    key: u64,
) -> Result<(), PlanarMutationDenial> {
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    match request.advanced().mutate(PlanarEdit(PlanarMutation {
        scope_key: "anchor-a".into(),
        operation: PlanarOperation::VerifyFinalCurrentOutputs(vec![PlanarCurrentOutputExpectation {
            producer_key: "anchor-a".into(), output_key: "final:anchor-a".into(),
        }]),
    })).expect_source(source.observed_sources()[0].clone()).idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
    {
        Ok(Outcome::Committed { .. }) => Ok(()),
        Ok(Outcome::DomainDenied(PlanarMutationDenial::CurrentOutputMissing)) => {
            Err(PlanarMutationDenial::CurrentOutputMissing)
        }
        other => panic!("the distinct verification command must commit or return exactly CurrentOutputMissing: {other:?}"),
    }
}
