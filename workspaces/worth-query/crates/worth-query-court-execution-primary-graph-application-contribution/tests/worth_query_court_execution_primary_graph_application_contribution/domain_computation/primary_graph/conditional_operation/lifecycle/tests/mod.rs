use std::sync::Arc;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::installation::clock_handle::ConditionalClockLease;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::clock_observation::erased::ErasedClockObservationOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryConditionalClockObservationFailureKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::lifecycle::registry::WorthQueryConditionalOperationRegistry;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::lifecycle::installed_operation::WorthQueryConditionalRetainedResourceCounts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryConditionalTruthBasis;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::lifecycle::installed_operation::WorthQueryInstalledConditionalOperation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::lifecycle::installed_operation::WorthQueryPreparedConditionalRuntimeBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::lifecycle::clock_source_observation::isolate_clock_source;
use worth_query_execution::facade::domain_computation::WorthQueryConditionalRuntimeInstallationDenial;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::WorthQueryNamedClockFailure;
use worth_query_installation::facade::WorthQueryNamedClockFailureKind;
use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

pub struct TestSchema;

pub struct InstalledClock {
    pub identity: String,
    pub lease: Arc<ConditionalClockLease>,
}

impl WorthQueryInstalledConditionalOperation<TestSchema> for InstalledClock {
    fn pending_managed_cleanup(
        &self,
    ) -> Option<
        worth_query_execution::facade::domain_computation::WorthQueryMutationCleanupFailureKind,
    > {
        None
    }

    fn binding_identity(&self) -> &str {
        &self.identity
    }

    fn installation_canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence {
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero()
    }

    fn clock_lease(&self) -> Arc<ConditionalClockLease> {
        Arc::clone(&self.lease)
    }

    fn operation_anchor(
        &self,
    ) -> Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering> {
        panic!("test operation has no Bridge lowering")
    }

    fn selected_lowering(
        &self,
    ) -> Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering> {
        panic!("test operation has no selected Bridge lowering")
    }

    fn select_product_binding(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _bridge: &BridgeSealedRuntimeAssembly,
        _runtime: &WorthQueryPrimaryGraphApplicationRuntime<TestSchema>,
        _truth: &WorthQueryConditionalTruthBasis,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn reconstruct(
        &mut self,
        _runtime: &worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime<
            TestSchema,
        >,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn reconcile_reconstruction(
        &mut self,
        _bridge: &mut BridgeSealedRuntimeAssembly,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn prepare_derived_runtime_reinstallation(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _runtime: &worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime<
            TestSchema,
        >,
        _bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        _product: &worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryPreparedConditionalRuntimeBinding,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        unreachable!("registry lease fixture does not install a runtime binding")
    }

    fn apply_derived_runtime_reinstallation(
        &mut self,
        _prepared: WorthQueryPreparedConditionalRuntimeBinding,
    ) {
        unreachable!("registry lease fixture does not install a runtime binding")
    }

    fn reconcile_prepared_runtime_reinstallation(
        &self,
        _bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        _prepared: &mut WorthQueryPreparedConditionalRuntimeBinding,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn observe_clock(
        &mut self,
        _phase: &WorthQueryAdvancementPhase<'_>,

        _bridge: &BridgeSealedRuntimeAssembly,
        _runtime: &worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime<
                TestSchema,
            >,
        _truth: &WorthQueryConditionalTruthBasis,
    ) -> ErasedClockObservationOutcome {
        ErasedClockObservationOutcome::Stale
    }

    fn retained_resource_counts(&self) -> WorthQueryConditionalRetainedResourceCounts {
        Default::default()
    }

    fn reconstruction_work(
        &self,
    ) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::temporal_reconstruction::WorthQueryTemporalReconstructionWork{
        Default::default()
    }

    fn lifecycle_resources(
        &self,
    ) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::lifecycle_inventory::WorthQueryConditionalOperationLiveness{
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::lifecycle_inventory::WorthQueryConditionalOperationLiveness::new_for_test(Default::default(), Default::default(), Vec::new(), Vec::new(), Vec::new())
    }
}

#[test]
fn registry_requires_the_exact_private_installation_lease() {
    let installed_lease = Arc::new(ConditionalClockLease);
    let foreign_lease = Arc::new(ConditionalClockLease);
    let mut registry = WorthQueryConditionalOperationRegistry::<TestSchema>::default();
    registry
        .install_for_test(Box::new(InstalledClock {
            identity: "clock-binding".to_string(),
            lease: Arc::clone(&installed_lease),
        }))
        .unwrap();

    assert!(registry
        .admit_clock_for_test("clock-binding", &installed_lease)
        .is_some());
    assert!(registry
        .admit_clock_for_test("clock-binding", &foreign_lease)
        .is_none());
    assert!(registry
        .admit_clock_for_test("another-binding", &installed_lease)
        .is_none());
}

#[test]
fn parked_operation_does_not_hold_registry_lookup_or_another_operation_cell() {
    let first_lease = Arc::new(ConditionalClockLease);
    let second_lease = Arc::new(ConditionalClockLease);
    let mut registry = WorthQueryConditionalOperationRegistry::<TestSchema>::default();
    for (identity, lease) in [("first", &first_lease), ("second", &second_lease)] {
        registry
            .install_for_test(Box::new(InstalledClock {
                identity: identity.to_string(),
                lease: Arc::clone(lease),
            }))
            .unwrap();
    }
    let registry = std::sync::Mutex::new(registry);
    let first = registry
        .lock()
        .unwrap()
        .admit_clock_for_test("first", &first_lease)
        .unwrap();
    let (parked_tx, parked_rx) = std::sync::mpsc::sync_channel(0);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(0);
    let (completed_tx, completed_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        scope.spawn(move || {
            let _active = first.lock_operation_for_test();
            parked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });
        parked_rx.recv().unwrap();
        scope.spawn(|| {
            let second = registry
                .lock()
                .unwrap()
                .admit_clock_for_test("second", &second_lease)
                .unwrap();
            let active = second.lock_operation_for_test();
            completed_tx
                .send(active.binding_identity().to_string())
                .unwrap();
        });
        let completion = completed_rx.recv_timeout(std::time::Duration::from_secs(5));
        release_tx.send(()).unwrap();
        assert_eq!(completion.unwrap(), "second");
    });
}

#[test]
fn clock_source_failure_and_panic_are_isolated_as_typed_postures() {
    let unavailable = isolate_clock_source::<()>(|| {
        Err(WorthQueryNamedClockFailure::new(
            WorthQueryNamedClockFailureKind::SourceUnavailable,
            "timer service unavailable",
        ))
    })
    .unwrap_err();
    assert!(matches!(
        unavailable,
        ErasedClockObservationOutcome::Failed {
            kind: WorthQueryConditionalClockObservationFailureKind::SourceUnavailable,
            ..
        }
    ));

    let panicked = isolate_clock_source::<()>(|| panic!("clock panic")).unwrap_err();
    assert!(matches!(
        panicked,
        ErasedClockObservationOutcome::Failed {
            kind: WorthQueryConditionalClockObservationFailureKind::SourcePanicked,
            ..
        }
    ));
}
