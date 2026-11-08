//! Immutable in-memory read-cut fixture; publication is deliberately outside this court.
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;

#[derive(Debug)]
pub(super) struct RecordedCut {
    binding: SignalDurableHomeBinding,
    admission: SignalDurableOwnerAdmissionRegistry,
    records: Arc<Records>,
}
#[derive(Debug)]
struct Records {
    owner: DurableSignalOwnerIdentity,
    definitions: Vec<ExecutionImmutableBytes>,
    advances: Vec<ExecutionImmutableBytes>,
}
impl RecordedCut {
    pub(super) fn new(
        definitions: &[SignalDefinitionTransitionArtifact],
        advances: &[SignalBranchAdvanceArtifact],
        execution: &ExecutionRequest<'_, '_>,
        build: &str,
    ) -> Arc<Self> {
        let binding = SignalDurableHomeBinding::from_admitted_adapter(
            definitions[0].identity().owner(),
            HomeIncarnation::initial(),
            [9; 32],
            ApplicationBuildIdentity::parse(build).unwrap(),
        );
        let admission = SignalDurableOwnerAdmissionRegistry::new(&binding, execution).unwrap();
        let copy = |bytes: &[u8]| {
            let mut out = ExecutionRequestArrayBuilder::allocate(*execution, bytes.len()).unwrap();
            out.extend_from_slice(bytes).unwrap();
            out.seal().unwrap().into_immutable_bytes()
        };
        let records = Arc::new(Records {
            owner: binding.owner(),
            definitions: definitions
                .iter()
                .map(|artifact| copy(artifact.as_bytes()))
                .collect(),
            advances: advances
                .iter()
                .map(|artifact| copy(artifact.as_bytes()))
                .collect(),
        });
        Arc::new(Self {
            binding,
            admission,
            records,
        })
    }
}
#[derive(Debug)]
struct ReadCut(Arc<Records>);
impl SignalDurableReadSource for ReadCut {
    fn owner(&self) -> DurableSignalOwnerIdentity {
        self.0.owner
    }
    fn identity_contribution(&self) -> SignalRecordedIdentityContribution {
        SignalRecordedIdentityContribution::empty(self.owner())
    }
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, SignalDurableMediumDenial> {
        Ok(match journal {
            SignalDurableJournal::Definitions => self.0.definitions.len(),
            SignalDurableJournal::Advances => self.0.advances.len(),
        } as u64)
    }
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, SignalDurableMediumDenial> {
        let records = match journal {
            SignalDurableJournal::Definitions => &self.0.definitions,
            SignalDurableJournal::Advances => &self.0.advances,
        };
        records
            .get(
                position
                    .checked_sub(1)
                    .and_then(|index| usize::try_from(index).ok())
                    .ok_or(SignalDurableMediumDenial::Unavailable)?,
            )
            .cloned()
            .ok_or(SignalDurableMediumDenial::Unavailable)
    }
}
impl SignalDurableMedium for RecordedCut {
    fn binding(&self) -> &SignalDurableHomeBinding {
        &self.binding
    }
    fn owner_admission_registry(&self) -> &SignalDurableOwnerAdmissionRegistry {
        &self.admission
    }
    fn publication_task_posture(&self) -> SignalPublicationTaskPosture {
        SignalPublicationTaskPosture::Terminal
    }
    fn capture_recovery_source(
        &self,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn SignalDurableReadSource>, SignalDurableMediumDenial> {
        Ok(Box::new(ReadCut(Arc::clone(&self.records))))
    }
    fn advance_definition(
        &self,
        _artifact: &SignalDefinitionTransitionArtifact,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        panic!("readmission must not publish an owner effect")
    }
    fn advance_branch(
        &self,
        _artifact: &SignalBranchAdvanceArtifact,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        panic!("readmission must not publish an owner effect")
    }
    fn retire(
        &self,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalDurableMediumDenial> {
        panic!("immutable read-cut fixture is not a physical attachment")
    }
}
