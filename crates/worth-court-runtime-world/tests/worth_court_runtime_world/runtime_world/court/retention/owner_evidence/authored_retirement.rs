//! Public retirement consumes an actual stopped Signal allocation, never an Open owner.
use super::super::artifact_source::{signal_medium::Medium, UnboundDelivery};
use super::*;
use std::sync::Arc;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::durability::*;
struct EmptyBindings;
impl BridgeAuthoredInstallationBindings for EmptyBindings {
    fn dependency(
        &mut self,
        _: DurableSignalBranchIdentity,
        _: BridgeSemanticClaimReference,
        _: &BridgeAuthoredDependencyDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        panic!("empty programme has no dependencies")
    }
    fn providers(
        &mut self,
        _: DurableSignalBranchIdentity,
        _: &BridgeConditionalContract,
        _: &BridgeConditionalLocation,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        panic!("empty programme has no providers")
    }
}
fn programme(
    request: &ExecutionRequest<'_, '_>,
) -> (
    Arc<worth_relational::facade::runtime::RelationalRuntime>,
    BridgeReadmittedAuthoredAssembly,
    worth_proof::BridgeProgramInstallationBinding,
    worth_proof::BridgeProgramInstallationBinding,
) {
    let (native, _medium) = worth_testkit_relational::initial_input_source::initialized(&request);
    let native = Arc::new(native);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(native.clone(), "retirement-court").unwrap();
    let bridge = RuntimeBridgeBuilder::new()
        .with_signal_sink(UnboundDelivery)
        .with_source_adapter(source.clone())
        .with_relational_source(source)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("retirement-court"),
            TruthPatchScope::for_target(
                MappingSelector::exact("retirement-court"),
                AspectKey::new("profile").unwrap(),
                TruthPatchTargetSelector::authoritative_aspect(),
            ),
            SnapshotReadContract::scalar(
                AspectKey::new("profile").unwrap(),
                ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("retirement-court"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .unwrap();
    let builder = Box::new(
        BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
            bridge.clone(),
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap(),
    );
    let original_binding = builder
        .retain_fresh_authored_bindings(&request)
        .unwrap()
        .installation_binding();
    let other = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap();
    let foreign_binding = other
        .retain_fresh_authored_bindings(&request)
        .unwrap()
        .installation_binding();
    drop(other);
    let medium = Medium::new(&request);
    let mut publication = builder
        .prepare_initial_authored_source(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(medium.binding().owner()),
            &request,
        )
        .unwrap_or_else(|cause| panic!("initial programme: {:?}", cause.cause()));
    assert_eq!(
        publication.advance(&request).unwrap(),
        SignalInitialSourceProgress::Published
    );
    let owner = *publication
        .try_into_published()
        .unwrap_or_else(|_| panic!("original must commit"))
        .readmit_authored_owner(&mut EmptyBindings, &request)
        .unwrap();
    (native, owner, original_binding, foreign_binding)
}
#[test]
fn authored_retirement_refuses_an_open_owner_and_completes_after_the_same_owner_stops() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let (native, owner, _, _) = programme(&request);
    let source = owner.signal_owner().source_port();
    let branch = source.branches().unwrap()[0].0;
    let before = source.observe(branch).unwrap();
    let (owner, cause) = owner
        .into_home_retirement(&request, &native)
        .err()
        .expect("Open owner cannot become a retirement task");
    assert!(
        matches!(cause,BridgeHomeRetirementDenial::AuthoredSignal(cause) if matches!(*cause,SignalAuthoredHomeStopDenial::Owner(worth_signal::facade::runtime::SignalOwnerStopDenial::OwnerUnavailable)))
    );
    assert_eq!(source.observe(branch).unwrap(), before);
    owner
        .signal_owner()
        .prepare_home_stop(&request)
        .unwrap()
        .commit();
    // Store completes stop before owner retirement; repeating that completion
    // must observe the same stopped ledgers and retain their original allocation.
    owner.signal_owner().commit_home_stop(&request).unwrap();
    let mut retirement = owner
        .into_home_retirement(&request, &native)
        .unwrap_or_else(|(_, cause)| panic!("same stopped owner: {cause:?}"));
    let mut complete = false;
    for _ in 0..32 {
        if retirement.advance(&request, &native).unwrap() == BridgeHomeRetirementProgress::Complete
        {
            complete = true;
            break;
        }
    }
    assert!(complete, "actual stopped owners finish bounded retirement");
    drop(retirement);
}

#[test]
fn same_source_programme_installations_cannot_exchange_admission_bindings() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let (native, owner, original, foreign) = programme(&request);
    owner.admit_programme_binding(&original).unwrap();
    assert_eq!(
        owner.admit_programme_binding(&foreign),
        Err(worth_proof::BridgeProgramInstallationDenial::ForeignInstallation)
    );
    owner
        .signal_owner()
        .prepare_home_stop(&request)
        .unwrap()
        .commit();
    let mut retirement = owner
        .into_home_retirement(&request, &native)
        .unwrap_or_else(|(_, cause)| panic!("same owner: {cause:?}"));
    for _ in 0..32 {
        if retirement.advance(&request, &native).unwrap() == BridgeHomeRetirementProgress::Complete
        {
            return;
        }
    }
    panic!("original owners must complete retirement");
}
