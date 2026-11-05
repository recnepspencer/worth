//! Actual directory readmission and original-claim progression shared by these courts.
use super::*;
use worth_relational::facade::runtime::PreparedRelationalHomeProgramContracts;
use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, WorldHomeTransitionEffectProgress as Progress,
    WorldReadmittedPublication, WorldRecoveredUnpublishedEffects,
};
use worth_store_application::AdmittedApplicationHomeProgramStep;
use worth_store_testkit_application_oracle::world_home::WorldHomeFixture;

pub(super) struct CurrentOriginal {
    pub(super) fixture: WorldHomeFixture,
    pub(super) grant: AdmittedApplicationHomeProgramStep,
    pub(super) handle: ProductUnpublishedRecoveryHandle,
    pub(super) claim: Option<WorldRecoveredUnpublishedEffects>,
    pub(super) executions: Arc<AtomicUsize>,
    pub(super) contracts: PreparedRelationalHomeProgramContracts,
}

pub(super) fn assert_live_head(
    fixture: &WorldHomeFixture,
    publication: &WorldReadmittedPublication,
) {
    let live = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let after = publication.after();
    assert_eq!(
        (
            live.owner_identity(),
            live.branch_identity(),
            live.lifecycle_incarnation(),
            live.reference_generation(),
            live.selected_commit(),
            live.basis().identity()
        ),
        (
            after.owner_identity(),
            after.branch_identity(),
            after.lifecycle_incarnation(),
            after.reference_generation(),
            after.selected_commit(),
            after.basis().identity()
        ),
        "Continued installs the actual authoritative live head before returning"
    );
    assert_eq!(
        live.publication_attempt(),
        Some(publication.attempt_identity())
    );
}
impl CurrentOriginal {
    pub(super) fn open(path: &std::path::Path, request: &ExecutionRequest<'_, '_>) -> Self {
        let fixture = original::reopen(path, request);
        let executions = Arc::new(AtomicUsize::new(0));
        let contracts = original::contracts(&fixture, request, executions.clone());
        let page = fixture
            .home
            .world()
            .inspection_port()
            .recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(page.rows().len(), 1);
        let handle = page.rows()[0].handle().clone();
        let claim = fixture
            .home
            .world()
            .recovery_port()
            .inspect_recovered_effects(&handle)
            .unwrap();
        let grant = fixture
            .home
            .admit_readmitted_home_program_transition_step(
                0,
                &claim,
                HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
                1,
                HomeProgramTransitionTargetMeaning::from_recorded(0, contracts.catalog_digest()),
                request,
            )
            .unwrap();
        Self {
            fixture,
            grant,
            handle,
            claim: Some(claim),
            executions,
            contracts,
        }
    }
    pub(super) fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Outcome {
        let cancellation = RuntimeWorldCancellationSource::new();
        self.fixture
            .home
            .world()
            .publication_port()
            .continue_home_transition_effect(
                request,
                self.grant.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(
                    self.grant.plan().identity(),
                    0,
                ),
                self.grant.original(),
                self.grant.admission(),
                self.claim.take().unwrap(),
                &cancellation.token(),
                None,
            )
    }
    pub(super) fn until_acknowledged(&mut self, request: &ExecutionRequest<'_, '_>) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            assert!(std::time::Instant::now() < deadline);
            match self.advance(request) {
                Outcome::Pending(claim) => self.claim = Some(claim),
                other @ (Outcome::Continued(_)
                | Outcome::EvidenceRequired(_)
                | Outcome::Indeterminate(_)
                | Outcome::ReopenRequired(_)
                | Outcome::Refused { .. }) => panic!("actual ACK must be exposed: {other:?}"),
            }
            if self
                .claim
                .as_ref()
                .unwrap()
                .home_transition_effect_progress()
                .unwrap()
                == Some(Progress::Acknowledged)
            {
                return;
            }
            std::thread::yield_now();
        }
    }
    pub(super) fn finish(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> WorldReadmittedPublication {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            assert!(std::time::Instant::now() < deadline);
            match self.advance(request) {
                Outcome::Continued(publication) => return publication,
                Outcome::Pending(claim) => self.claim = Some(claim),
                other @ (Outcome::EvidenceRequired(_)
                | Outcome::Indeterminate(_)
                | Outcome::ReopenRequired(_)
                | Outcome::Refused { .. }) => panic!("same original must finish: {other:?}"),
            }
        }
    }
    pub(super) fn refuse_release_and_reclaim(&mut self, request: &ExecutionRequest<'_, '_>) {
        let recovery = self.fixture.home.world().recovery_port();
        let claim = match recovery.release_recovered_effects(request, self.claim.take().unwrap()) {
            WorldRecoveredReleaseOutcome::AdmissionRefused {
                cause: RuntimeWorldRecoveryDenial::Busy,
                custody,
            } => custody,
            other => panic!("continuation custody refuses independent disposition: {other:?}"),
        };
        assert_eq!(self.executions.load(Ordering::SeqCst), 0);
        let ids = (
            claim.attempt_identity().ordinal(),
            claim.reserved_commit_identity().ordinal(),
            claim.identity().ordinal(),
        );
        let receipt = claim.native_receipt();
        drop(claim);
        let claim = recovery.inspect_recovered_effects(&self.handle).unwrap();
        assert_eq!(
            (
                claim.attempt_identity().ordinal(),
                claim.reserved_commit_identity().ordinal(),
                claim.identity().ordinal()
            ),
            ids
        );
        assert_eq!(claim.native_receipt(), receipt);
        self.claim = Some(claim);
    }
}
