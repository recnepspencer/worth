use std::collections::HashMap;
use std::sync::Mutex;

use worth_store_authority::{
    ControlStoreFencingAuthority, ControlStoreFencingPort, ControlStoreFencingProviderDenial,
    ControlStoreGeneration, ControlStoreSelectionCoordinates, ExternalFenceGrant,
    ExternalServeLeaseGrant, OperationalFencingAuthorityPort, OperationalFencingProviderDenial,
    PrimaryServeLeaseRequest, PrimaryServingAuthority, PromotionFenceOperationIdentity,
    StoreCurrentAuthorityIdentity,
};
use worth_store_formal_models::check_operational_recovery_records;
use worth_store_offline_verifier::ReplicaTargetVerificationBudget;
use worth_store_operations::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, OperationalOperationId,
    OperationalSecurityScope, OperationalTransitionId, ReplicaPromotionIntent,
    ReplicaPromotionPublicationDenial, ReplicaPromotionPublicationPort,
    ReplicaPromotionPublicationReceipt, ReplicaPromotionPublicationRequest,
};
use worth_store_replication::{
    durable_replica_target_identity, OldPrimaryDivergenceDisposition,
    OldPrimaryRejoinExecutionDenial, OldPrimaryRejoinExecutionPort,
    OldPrimaryRejoinExecutionRequest, OldPrimaryRejoinReceipt, ReplicationDisasterRecoveryOwner,
    ReplicationLineageIdentity, ReplicationPeerId,
};

use super::operational_recovery_authorization_fixture::{operator_assertion, ExactAuthorization};
use super::operational_recovery_replica_driver_fixture::DisasterRecoveryFixture;
use crate::{
    DrivenOperationalControlStore, DrivenOperationalTransition,
    OperationalRecoveryControlTransitionKind as Control, OperationalRecoveryProductionDriver,
    OperationalRecoveryYieldpoint as Point,
};

#[test]
fn promotion_owner_path_drives_every_durable_transition_and_reopens_exactly() {
    let fixture = DisasterRecoveryFixture::materialize();
    let authority = worth_store_test_support::layout_integrity_authority("s10-driver-bootstrap");
    {
        let control = fixture.control_store();
        let provider = ExactFencingProvider::for_empty_store(&control);
        let selected = ControlStoreFencingAuthority::for_current_store(
            authority.current_authority(),
            &provider,
        )
        .select_generation()
        .unwrap();
        let serving = PrimaryServingAuthority::for_selected_control_generation(
            authority.current_authority(),
            selected,
            &provider,
        )
        .unwrap();
        let old_primary_lease = serving.acquire(9, 10, 100).unwrap();
        let operation = OperationalOperationId::new("s10-driven-promotion").unwrap();
        let peer = ReplicationPeerId::from_declared_peer("replica-b").unwrap();
        let target_root = fixture.materialize_replica_target();
        let target_identity = durable_replica_target_identity(&target_root).unwrap();
        let history = ReplicationDisasterRecoveryOwner::classify_replica_history(
            peer.clone(),
            fixture.lineage(),
            fixture.frontier(),
            true,
            true,
            target_identity,
            fixture.lineage(),
        );
        let verified = fixture.verify();
        let security =
            OperationalSecurityScope::from_admission(authority.security_scope().receipt());
        let authorized = ReplicaPromotionIntent::new(
            operation,
            peer,
            target_identity,
            fixture.frontier(),
            authority.current_authority().authority_identity(),
            security,
        )
        .unwrap()
        .resolve(verified, history, old_primary_lease)
        .unwrap()
        .lower()
        .unwrap()
        .authorize(
            &ExactAuthorization,
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap();

        let driver = OperationalRecoveryProductionDriver::uninterrupted();
        let driven = DrivenOperationalControlStore::new(&control, &driver);
        let ready = authorized
            .ready_with_certification_control_store(
                &control,
                &driven,
                transition("promotion-authorization"),
                authority.current_authority(),
                30,
                AuthorizationRevocationObservation::NotRevoked { observed_at: 30 },
            )
            .unwrap();
        let fenced = completed(driver.promotion_fence(ready, &serving).unwrap());
        let durable = completed(
            driver
                .persist_promotion_fence(&fenced, transition("promotion-fence"))
                .unwrap(),
        );
        let executed = completed(
            driver
                .record_promotion(&durable, transition("promotion-record"))
                .unwrap(),
        );
        let verified = completed(
            driver
                .post_verify_promotion(
                    executed,
                    &target_root,
                    ReplicaTargetVerificationBudget::bounded(17).unwrap(),
                )
                .unwrap(),
        );
        let mut publication = ExactPublication;
        let published = completed(
            driver
                .publish_promotion(
                    verified,
                    &driven,
                    transition("promotion-publication"),
                    &mut publication,
                )
                .unwrap(),
        );
        let current = completed(
            driver
                .readmit_promotion(
                    published,
                    &driven,
                    transition("promotion-readmission"),
                    &serving,
                    40,
                    90,
                )
                .unwrap(),
        );
        assert_eq!(
            current.promotion_receipt().durable_target_identity(),
            target_identity
        );
        assert_eq!(current.serve_lease().epoch(), 11);

        let old_primary = ReplicationPeerId::from_declared_peer("old-primary-a").unwrap();
        let divergence = ReplicationDisasterRecoveryOwner::classify_replica_history(
            old_primary.clone(),
            ReplicationLineageIdentity::from_declared_lineage("lineage/divergent-old-primary")
                .unwrap(),
            fixture.frontier(),
            true,
            true,
            [0x81; 32],
            fixture.lineage(),
        );
        let plan = completed(
            driver
                .plan_old_primary_rejoin(
                    &current,
                    &driven,
                    transition("old-primary-rejoin-plan"),
                    old_primary,
                    divergence,
                    OldPrimaryDivergenceDisposition::RebootstrapAfterForensicRetention,
                    Some([0x82; 32]),
                )
                .unwrap(),
        );
        let resolved = completed(
            driver
                .execute_old_primary_rejoin(plan, &mut ExactRejoinOwner)
                .unwrap(),
        );
        let rejoined = completed(
            driver
                .complete_old_primary_rejoin(
                    resolved,
                    &driven,
                    transition("old-primary-rejoin-completion"),
                )
                .unwrap(),
        );
        assert_eq!(
            rejoined.receipt().forensic_retention_identity(),
            Some([0x83; 32])
        );
        assert_eq!(
            rejoined.receipt().rebootstrap_target_identity(),
            Some([0x84; 32])
        );

        let trace = driver.trace();
        for kind in [
            Control::AuthorizationConsumption,
            Control::ReplicaPromotionFence,
            Control::ReplicaPromotionRecord,
            Control::ReplicaPromotionPublication,
            Control::ReplicaPromotionReadmission,
            Control::OldPrimaryRejoinPlan,
            Control::OldPrimaryRejoinCompletion,
        ] {
            assert!(trace
                .reached()
                .contains(&Point::BeforeDurableControlTransition(kind)));
            assert!(trace
                .reached()
                .contains(&Point::AfterDurableControlTransition(kind)));
        }
        for point in [
            Point::BeforeOldPrimaryRejoinPlan,
            Point::AfterOldPrimaryRejoinPlan,
            Point::BeforeOldPrimaryRejoinExecution,
            Point::AfterOldPrimaryRejoinExecution,
            Point::BeforeOldPrimaryRejoinCompletion,
            Point::AfterOldPrimaryRejoinCompletion,
        ] {
            assert!(trace.reached().contains(&point));
        }
        assert_eq!(trace.control_artifact_identities().len(), 7);
        assert_eq!(
            control
                .observe_selection_coordinates()
                .unwrap()
                .unwrap()
                .generation()
                .get(),
            7,
        );
        let selected_provider = ExactFencingProvider::for_current_prefix(&control);
        let selection = ControlStoreFencingAuthority::for_current_store(
            authority.current_authority(),
            &selected_provider,
        );
        let worth_store_operations::ControlStoreTrustPosture::Selected(selected) =
            control.inspect_generations(&selection)
        else {
            panic!("current physical control prefix must be selected");
        };
        check_operational_recovery_records(selected.durable_records())
            .expect("the executed promotion history satisfies the recovery model");
    }
    assert_eq!(
        fixture
            .control_store()
            .observe_selection_coordinates()
            .unwrap()
            .unwrap()
            .generation()
            .get(),
        7,
    );
}

fn completed<T: std::fmt::Debug>(transition: DrivenOperationalTransition<T>) -> T {
    match transition {
        DrivenOperationalTransition::Completed(value) => value,
        other => panic!("uninterrupted driver returned {other:?}"),
    }
}

fn transition(label: &str) -> OperationalTransitionId {
    OperationalTransitionId::new(label).unwrap()
}

#[derive(Debug)]
struct ExactFencingProvider {
    coordinates: ControlStoreSelectionCoordinates,
    fences: Mutex<HashMap<PromotionFenceOperationIdentity, ExternalFenceGrant>>,
}

impl ExactFencingProvider {
    fn for_empty_store(control: &worth_store_operations::OperationalControlStore) -> Self {
        Self {
            coordinates: ControlStoreSelectionCoordinates::new(
                control.media_identity().fingerprint(),
                ControlStoreGeneration::initial(),
                [0; 32],
            ),
            fences: Mutex::new(HashMap::new()),
        }
    }

    fn for_current_prefix(control: &worth_store_operations::OperationalControlStore) -> Self {
        Self {
            coordinates: control
                .observe_selection_coordinates()
                .unwrap()
                .expect("promotion produced durable control history"),
            fences: Mutex::new(HashMap::new()),
        }
    }
}

impl ControlStoreFencingPort for ExactFencingProvider {
    fn selected_control_store(
        &self,
        _current_authority: StoreCurrentAuthorityIdentity,
    ) -> Result<ControlStoreSelectionCoordinates, ControlStoreFencingProviderDenial> {
        Ok(self.coordinates)
    }
}

impl OperationalFencingAuthorityPort for ExactFencingProvider {
    fn acquire_primary_serve_lease(
        &self,
        request: PrimaryServeLeaseRequest,
    ) -> Result<ExternalServeLeaseGrant, OperationalFencingProviderDenial> {
        Ok(ExternalServeLeaseGrant::from_provider(
            [0x61; 32],
            request.minimum_epoch_exclusive() + 1,
            request.requested_until_tick(),
            [0x62; 32],
        ))
    }

    fn renew_primary_serve_lease(
        &self,
        _current_token: [u8; 32],
        request: PrimaryServeLeaseRequest,
    ) -> Result<ExternalServeLeaseGrant, OperationalFencingProviderDenial> {
        self.acquire_primary_serve_lease(request)
    }

    fn revoke_and_advance_epoch(
        &self,
        old_lease_token: [u8; 32],
        minimum_epoch_exclusive: u64,
        operation_identity: PromotionFenceOperationIdentity,
    ) -> Result<ExternalFenceGrant, OperationalFencingProviderDenial> {
        let mut fences = self.fences.lock().unwrap();
        Ok(*fences.entry(operation_identity).or_insert_with(|| {
            ExternalFenceGrant::from_provider(
                old_lease_token,
                minimum_epoch_exclusive + 1,
                [0x62; 32],
                [0x63; 32],
                operation_identity,
            )
        }))
    }

    fn recover_fence(
        &self,
        operation_identity: PromotionFenceOperationIdentity,
    ) -> Result<Option<ExternalFenceGrant>, OperationalFencingProviderDenial> {
        Ok(self
            .fences
            .lock()
            .unwrap()
            .get(&operation_identity)
            .copied())
    }
}

struct ExactPublication;
struct ExactRejoinOwner;

impl OldPrimaryRejoinExecutionPort for ExactRejoinOwner {
    fn resolve_old_primary_divergence(
        &mut self,
        request: OldPrimaryRejoinExecutionRequest,
    ) -> Result<OldPrimaryRejoinReceipt, OldPrimaryRejoinExecutionDenial> {
        OldPrimaryRejoinReceipt::from_rejoin_owner(&request, Some([0x83; 32]), Some([0x84; 32]))
    }
}

impl ReplicaPromotionPublicationPort for ExactPublication {
    fn publish_promoted_replica(
        &mut self,
        request: ReplicaPromotionPublicationRequest,
    ) -> Result<ReplicaPromotionPublicationReceipt, ReplicaPromotionPublicationDenial> {
        Ok(ReplicaPromotionPublicationReceipt::from_publication_owner(
            [0x71; 32],
            request.target_identity(),
            request.promoted_epoch(),
        ))
    }
}
