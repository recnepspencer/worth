//! In-memory byte transport; only Signal authors frames and admits their ACKs.
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
#[derive(Clone, Debug)]
struct Row {
    journal: SignalDurableJournal,
    digest: [u8; 32],
    bytes: Vec<u8>,
}
#[derive(Debug)]
pub(in super::super) struct Medium {
    binding: SignalDurableHomeBinding,
    registry: SignalDurableOwnerAdmissionRegistry,
    rows: Mutex<Vec<Row>>,
}
impl Medium {
    pub(in super::super) fn new(request: &ExecutionRequest<'_, '_>) -> Arc<Self> {
        let binding = SignalDurableHomeBinding::from_admitted_adapter(
            DurableSignalOwnerIdentity::from_recorded(
                HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
                SignalOwnerSlotIdentity::from_recorded(0),
            ),
            HomeIncarnation::initial(),
            [3; 32],
            ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111").unwrap(),
        );
        let registry = SignalDurableOwnerAdmissionRegistry::new(&binding, request).unwrap();
        Arc::new(Self {
            binding,
            registry,
            rows: Mutex::new(Vec::new()),
        })
    }
    pub(super) fn successor(&self, request: &ExecutionRequest<'_, '_>) -> Arc<Self> {
        let binding = SignalDurableHomeBinding::from_admitted_adapter(
            self.binding.owner(),
            self.binding.incarnation().successor().unwrap(),
            self.binding.program_revision(),
            self.binding.build_identity(),
        );
        let registry = SignalDurableOwnerAdmissionRegistry::new(&binding, request).unwrap();
        Arc::new(Self {
            binding,
            registry,
            rows: Mutex::new(self.rows.lock().unwrap().clone()),
        })
    }
    fn append(
        &self,
        journal: SignalDurableJournal,
        bytes: &[u8],
        digest: [u8; 32],
    ) -> SignalPublicationObservation {
        let mut rows = self.rows.lock().unwrap();
        let position = match rows
            .iter()
            .filter(|row| row.journal == journal)
            .position(|row| row.digest == digest)
        {
            Some(position) => position + 1,
            None => {
                let position = rows.iter().filter(|row| row.journal == journal).count() + 1;
                rows.push(Row {
                    journal,
                    digest,
                    bytes: bytes.to_vec(),
                });
                position
            }
        };
        SignalPublicationObservation::Finished(SignalPublicationCompletion::committed(
            journal,
            std::num::NonZeroU64::new(position as u64).unwrap(),
            digest,
        ))
    }
}
#[derive(Debug)]
struct Source {
    owner: DurableSignalOwnerIdentity,
    rows: Vec<Row>,
}
impl SignalDurableReadSource for Source {
    fn owner(&self) -> DurableSignalOwnerIdentity {
        self.owner
    }
    fn identity_contribution(&self) -> SignalRecordedIdentityContribution {
        SignalRecordedIdentityContribution::empty(self.owner)
    }
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, SignalDurableMediumDenial> {
        Ok(self
            .rows
            .iter()
            .filter(|row| row.journal == journal)
            .count() as u64)
    }
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, SignalDurableMediumDenial> {
        let row = self
            .rows
            .iter()
            .filter(|row| row.journal == journal)
            .nth(
                position
                    .checked_sub(1)
                    .ok_or(SignalDurableMediumDenial::Unavailable)? as usize,
            )
            .ok_or(SignalDurableMediumDenial::Unavailable)?;
        let mut bytes = ExecutionRequestArrayBuilder::allocate(*request, row.bytes.len())
            .map_err(SignalDurableMediumDenial::Backing)?;
        bytes
            .extend_from_slice(&row.bytes)
            .map_err(SignalDurableMediumDenial::Backing)?;
        Ok(bytes
            .seal_immutable_bytes()
            .map_err(SignalDurableMediumDenial::Backing)?)
    }
}
impl SignalDurableMedium for Medium {
    fn binding(&self) -> &SignalDurableHomeBinding {
        &self.binding
    }
    fn owner_admission_registry(&self) -> &SignalDurableOwnerAdmissionRegistry {
        &self.registry
    }
    fn publication_task_posture(&self) -> SignalPublicationTaskPosture {
        SignalPublicationTaskPosture::Terminal
    }
    fn capture_recovery_source(
        &self,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn SignalDurableReadSource>, SignalDurableMediumDenial> {
        Ok(Box::new(Source {
            owner: self.binding.owner(),
            rows: self.rows.lock().unwrap().clone(),
        }))
    }
    fn advance_definition(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        _: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.append(
            SignalDurableJournal::Definitions,
            artifact.as_bytes(),
            artifact.digest(),
        )
    }
    fn advance_branch(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        _: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.append(
            SignalDurableJournal::Advances,
            artifact.as_bytes(),
            artifact.digest(),
        )
    }
    fn retry_definition_after_no_effect(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        _: &worth_proof::ProvenNoEffectPublication<SignalNoEffectOriginal>,
        request: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.advance_definition(artifact, request)
    }
    fn retry_branch_after_no_effect(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        _: &worth_proof::ProvenNoEffectPublication<SignalNoEffectOriginal>,
        request: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.advance_branch(artifact, request)
    }
    fn retire(&self, _: &ExecutionRequest<'_, '_>) -> Result<(), SignalDurableMediumDenial> {
        Ok(())
    }
}
pub(super) fn owner(
    request: &ExecutionRequest<'_, '_>,
) -> (SignalDurableOwner<(), (), (), (), ()>, Arc<Medium>) {
    let medium = Medium::new(request);
    let program = Box::new(worth_signal::facade::SignalRuntime::build_for::<()>(
        worth_signal::facade::SignalGraph::new(),
    ));
    let empty = ExecutionRequestArrayBuilder::allocate(*request, 0)
        .unwrap()
        .seal_immutable_bytes()
        .unwrap();
    let mut publication = program
        .prepare_initial_authored_source(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(medium.binding().owner()),
            SignalDurableTierVocabulary::declare([((), "base")]).unwrap(),
            request,
            |_| Ok(SignalBridgeAuthoredSection::from_bridge_bytes(empty).unwrap()),
        )
        .unwrap_or_else(|refusal| panic!("initial Signal source: {:?}", refusal.cause()));
    assert_eq!(
        publication.advance(request).unwrap(),
        SignalInitialSourceProgress::Published
    );
    let owner = publication
        .try_into_published()
        .unwrap_or_else(|_| panic!("source remained pending"))
        .readmit_authored_owner(&mut SignalOnlyOwnerInstallation, request)
        .unwrap();
    (owner, medium)
}
pub(super) fn readmit(
    medium: Arc<Medium>,
    request: &ExecutionRequest<'_, '_>,
) -> SignalDurableOwner<(), (), (), (), ()> {
    Box::new(worth_signal::facade::SignalRuntime::build_for::<()>(
        worth_signal::facade::SignalGraph::new(),
    ))
    .readmit_authored_owner(
        medium.clone(),
        SignalRecordedIdentityContribution::empty(medium.binding().owner()),
        SignalDurableTierVocabulary::declare([((), "base")]).unwrap(),
        &mut SignalOnlyOwnerInstallation,
        request,
    )
    .unwrap()
}
