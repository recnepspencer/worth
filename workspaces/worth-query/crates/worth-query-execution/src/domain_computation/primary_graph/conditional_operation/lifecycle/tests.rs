use super::*;
use crate::domain_computation::primary_graph::conditional_operation::installation::WorthQueryConditionalRuntimeInstallationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::{
    WorthQueryNamedClockFailure, WorthQueryNamedClockFailureKind,
};

struct TestSchema;

struct InstalledClock {
    identity: String,
    lease: Arc<ConditionalClockLease>,
}

impl WorthQueryInstalledConditionalOperation<TestSchema> for InstalledClock {
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

    fn lowering_anchor(
        &self,
    ) -> Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering> {
        panic!("test operation has no Bridge lowering")
    }

    fn select_product_binding(
        &mut self,
        _bridge: &BridgeSealedRuntimeAssembly,
        _runtime: &WorthQueryPrimaryGraphApplicationRuntime<TestSchema>,
        _truth: &WorthQueryConditionalTruthBasis,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn reconstruct(
        &mut self,
        _runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
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
        _runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            TestSchema,
        >,
        _bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        _product: &crate::basis::WorthQueryProductBranchLease,
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
        _bridge: &BridgeSealedRuntimeAssembly,
        _runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
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
    ) -> crate::domain_computation::primary_graph::conditional_operation::temporal_reconstruction::WorthQueryTemporalReconstructionWork{
        Default::default()
    }

    fn lifecycle_resources(
        &self,
    ) -> crate::domain_computation::primary_graph::conditional_operation::lifecycle_inventory::WorthQueryConditionalOperationLiveness{
        crate::domain_computation::primary_graph::conditional_operation::lifecycle_inventory::WorthQueryConditionalOperationLiveness {
            binding: Default::default(),
            lease: Default::default(),
            wakes: Vec::new(),
            intents: Vec::new(),
            attempts: Vec::new(),
        }
    }
}

#[test]
fn registry_requires_the_exact_private_installation_lease() {
    let installed_lease = Arc::new(ConditionalClockLease);
    let foreign_lease = Arc::new(ConditionalClockLease);
    let mut registry = WorthQueryConditionalOperationRegistry::<TestSchema>::default();
    registry
        .install(Box::new(InstalledClock {
            identity: "clock-binding".to_string(),
            lease: Arc::clone(&installed_lease),
        }))
        .unwrap();

    assert!(registry
        .admit_clock("clock-binding", &installed_lease)
        .is_some());
    assert!(!registry
        .admit_clock("clock-binding", &foreign_lease)
        .is_some());
    assert!(!registry
        .admit_clock("another-binding", &installed_lease)
        .is_some());
}

#[test]
fn parked_operation_does_not_hold_registry_lookup_or_another_operation_cell() {
    let first_lease = Arc::new(ConditionalClockLease);
    let second_lease = Arc::new(ConditionalClockLease);
    let mut registry = WorthQueryConditionalOperationRegistry::<TestSchema>::default();
    for (identity, lease) in [("first", &first_lease), ("second", &second_lease)] {
        registry
            .install(Box::new(InstalledClock {
                identity: identity.to_string(),
                lease: Arc::clone(lease),
            }))
            .unwrap();
    }
    let registry = std::sync::Mutex::new(registry);
    let first = registry
        .lock()
        .unwrap()
        .admit_clock("first", &first_lease)
        .unwrap();
    let (parked_tx, parked_rx) = std::sync::mpsc::sync_channel(0);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(0);
    let (completed_tx, completed_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        scope.spawn(move || {
            let _active = first.lock_operation();
            parked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });
        parked_rx.recv().unwrap();
        scope.spawn(|| {
            let second = registry
                .lock()
                .unwrap()
                .admit_clock("second", &second_lease)
                .unwrap();
            let active = second.lock_operation();
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
