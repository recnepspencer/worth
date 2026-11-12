//! Original World opens and actual Native/Signal effects produce every codec input.
use super::super::artifact_source::Fixture;
use super::*;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
pub(in super::super) struct Journey {
    pub fixture: Fixture,
    pub expected: Vec<u8>,
    pub open: Vec<u8>,
    pub occurrence: Vec<u8>,
    pub signal: Option<Vec<u8>>,
}
impl Journey {
    pub fn new(request: &ExecutionRequest<'_, '_>, tag: u8) -> Self {
        let mut fixture = Fixture::journal(request);
        let root = match fixture.offer(request, "primary") {
            WorldDurableBootstrapOutcome::Performed(value) => value.product_branch().clone(),
            other => panic!("real bootstrap: {other:?}"),
        };
        let mut journey = Self::from_fixture(fixture);
        journey.advance(request, root, tag);
        journey
    }
    pub fn from_fixture(fixture: Fixture) -> Self {
        Self {
            fixture,
            expected: Vec::new(),
            open: Vec::new(),
            occurrence: Vec::new(),
            signal: None,
        }
    }
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        root: ProductBranchObservation,
        tag: u8,
    ) {
        let port = self.fixture.world.publication_port();
        let cancellation = RuntimeWorldCancellationSource::new();
        let opened = if tag == 1 {
            port.open_publication_without_signal(
                request,
                root.clone(),
                self.fixture.recorder.program_source(&root),
                RelationalTransactionIntent::ordinary(),
                &cancellation.token(),
                None,
            )
        } else {
            port.prepare_query_publication_with_signal(
                request,
                root.clone(),
                self.fixture.recorder.program_source(&root),
                (tag == 3).then(RelationalTransactionIntent::ordinary),
                &cancellation.token(),
                None,
            )
            .unwrap()
            .advance(request)
        };
        let WorldOpenAttemptOutcome::Opened(opened) = opened else {
            panic!("actual open: {opened:?}")
        };
        let proposal = if tag != 2 {
            let mut transaction =
                worth_testkit_relational::aspect_storage::publications::begin_main_transaction(
                    &self.fixture.native,
                );
            transaction
                .push_batch(
                    worth_testkit_relational::durable_publication::batch_create(&format!(
                        "wire-{tag}"
                    )),
                    request.allocation_policy(),
                )
                .unwrap();
            Some(transaction.validate(request, &self.fixture.native).unwrap())
        } else {
            None
        };
        let outcome = if tag == 1 {
            port.execute_durable_without_signal(
                *request,
                opened
                    .prepare_native(&self.fixture.native, request, proposal.unwrap())
                    .unwrap(),
                &cancellation.token(),
            )
        } else {
            port.execute_durable_with_signal(
                *request,
                opened
                    .prepare_signal(&self.fixture.native, request, proposal)
                    .unwrap(),
                &mut (),
                &cancellation.token(),
                |_| Ok(()),
            )
        };
        let performed = match outcome {
            WorldDurablePublicationOutcome::Performed(value) => value,
            other => panic!("actual durable occurrence: {other:?}"),
        };
        let mut owners = vec![tag];
        if tag != 2 {
            let result = performed
                .component_results()
                .relational_commit_result()
                .unwrap();
            let (_, basis) = self
                .fixture
                .native
                .observe_branch(&self.fixture.native.main_branch_identity())
                .unwrap();
            let recorded = self
                .fixture
                .native
                .owner_component_services()
                .basis_port()
                .export_recorded_current_component(request, &basis)
                .unwrap();
            let worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(
                actual,
            ) = recorded.target()
            else {
                panic!("real Native result")
            };
            assert_eq!(actual.receipt(), &result.outcome().commit);
            owners.extend_from_slice(&actual.receipt().commit_id.0.to_le_bytes());
            owners.extend_from_slice(&actual.position().0.to_le_bytes());
            owners.extend_from_slice(actual.artifact_digest());
        }
        if tag != 1 {
            let output = performed
                .component_results()
                .signal()
                .durable_advance_output()
                .expect("actual Signal output");
            let bytes = output
                .branch()
                .source_point()
                .encode_bytes(request)
                .unwrap()
                .as_ref()
                .to_vec();
            owners.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            owners.extend_from_slice(&bytes);
            owners.resize(owners.len()+worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES-bytes.len(),0);
            self.signal = Some(bytes);
        }
        let subjects = self.fixture.recorder.subjects.lock().unwrap();
        self.open = subjects
            .iter()
            .rev()
            .find(|subject| subject.journal() == WorldDurableJournal::Custody)
            .unwrap()
            .as_bytes()
            .to_vec();
        self.occurrence = subjects
            .iter()
            .rev()
            .find(|subject| subject.journal() == WorldDurableJournal::Occurrences)
            .unwrap()
            .as_bytes()
            .to_vec();
        let mut payload = self.open[52..].to_vec();
        payload[0] = 4;
        payload.extend_from_slice(&owners);
        payload.extend_from_slice(&1u64.to_le_bytes());
        self.expected = frame(b"WWRLDART", 5, &payload);
        drop(subjects);
        drop(performed.consume());
        drop(root);
    }
    pub fn data(&self, request: &ExecutionRequest<'_, '_>) -> PreparedWorldHomeData {
        PreparedWorldHomeData::from_home_source(
            self.fixture.recorder.binding().clone(),
            self.fixture.recorder.snapshot_for_court(),
            request,
        )
        .unwrap()
    }
    pub fn checkpoint(&self, request: &ExecutionRequest<'_, '_>) -> PreparedWorldCheckpoint {
        self.fixture
            .world
            .checkpoint_port()
            .unwrap()
            .prepare_home_checkpoint(request)
            .unwrap()
    }
    pub fn refusal(
        &self,
        request: &ExecutionRequest<'_, '_>,
        selected: Option<Vec<u8>>,
        change: impl FnOnce(&mut Vec<u8>),
    ) -> WorldReadmissionDenial {
        let selected = selected.map(|value| bytes(request, &value));
        PreparedWorldHomeData::from_home_source(
            self.fixture.recorder.binding().clone(),
            self.fixture.recorder.data_snapshot(selected, change),
            request,
        )
        .unwrap_err()
    }
}
pub(super) fn bytes(
    request: &ExecutionRequest<'_, '_>,
    input: &[u8],
) -> worth_execution::ExecutionImmutableBytes {
    let mut bytes =
        worth_execution::ExecutionRequestArrayBuilder::allocate(*request, input.len()).unwrap();
    bytes.extend_from_slice(input).unwrap();
    bytes.seal_immutable_bytes().unwrap()
}
pub(in super::super) fn frame(magic: &[u8; 8], version: u32, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::from(magic);
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&Sha256::digest(payload));
    bytes.extend_from_slice(payload);
    bytes
}
pub(in super::super) fn rehash(bytes: &mut [u8]) {
    let hash = Sha256::digest(&bytes[52..]);
    bytes[20..52].copy_from_slice(&hash);
}
