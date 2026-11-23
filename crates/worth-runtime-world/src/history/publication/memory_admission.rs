//! An original's capture, paid on its request before the attempt's first owner
//! effect. After the effect only its charge-free seal runs, so a request one
//! unit short stops with no effect rather than after its commit.
use super::memory_observation::{
    MemoryPublicationMeaning, MemoryPublicationReceiver, MemoryPublicationRecord,
};
use super::CanonicalPublicationEnvelope;
use std::sync::{atomic::AtomicBool, Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::MemoryProductBranchReference;
use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, AdmittedRelationalMemoryComponent,
    ObservedRelationalMemoryComponent, RelationalBranchBasisPort,
    RelationalRecordedComponentRefusal as Refusal,
};
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

/// Where an original's successor component comes from.
pub(crate) enum MemoryOriginalSuccessor<'a> {
    /// The successor this attempt's own prepared candidate will issue.
    Candidate(&'a PreparedRelationalCommitCandidate),
    /// A successor already issued before this request.
    Issued(&'a AdmittedRelationalBranchBasis),
}

#[derive(Debug)]
enum AdmittedSuccessor {
    Quoted(AdmittedRelationalMemoryComponent),
    Issued(ObservedRelationalMemoryComponent),
}

/// Every charge and reservation of one original's capture.
#[derive(Debug)]
pub(crate) struct AdmittedMemoryOriginal {
    meaning: Arc<MemoryPublicationMeaning>,
    before: ObservedRelationalMemoryComponent,
    after: AdmittedSuccessor,
    slot: crate::history::MemoryOriginalSlot,
    funding: Option<ExecutionMemoryReservation>,
}

impl CanonicalPublicationEnvelope {
    /// Pay this envelope's original capture on `execution` and retain it.
    pub(crate) fn admit_memory_original(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        history: &crate::history::CompositeHistoryCatalog,
        port: &RelationalBranchBasisPort,
        expected: &crate::branch::ProductBranchObservation,
        successor: MemoryOriginalSuccessor<'_>,
    ) -> Result<(), Refusal> {
        let admitted = self.admission(execution, history, port, expected, successor)?;
        let mut slot = self
            .admitted_original
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.replace(admitted).is_none(), "one admitted original");
        Ok(())
    }

    /// Install the original from its admission. An attempt resumed after its
    /// owner effects has none, and its own request admits it here, before the
    /// product movement that request performs.
    pub(crate) fn seal_memory_original(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        history: &crate::history::CompositeHistoryCatalog,
        port: &RelationalBranchBasisPort,
        expected: &crate::branch::ProductBranchObservation,
        successor: &AdmittedRelationalBranchBasis,
    ) -> Result<(), Refusal> {
        let admitted = self
            .admitted_original
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let admitted = match admitted {
            Some(admitted) => admitted,
            None => self.admission(
                execution,
                history,
                port,
                expected,
                MemoryOriginalSuccessor::Issued(successor),
            )?,
        };
        let after_component = match admitted.after {
            AdmittedSuccessor::Quoted(quote) => {
                port.observe_admitted_memory_component(quote, successor)?
            }
            AdmittedSuccessor::Issued(component) => component,
        };
        let record = Arc::new(MemoryPublicationRecord {
            meaning: admitted.meaning,
            receiver: MemoryPublicationReceiver::Live,
            before_component: admitted.before,
            after_component,
            committed: AtomicBool::new(false),
            _funding: admitted.funding,
        });
        self.install_memory_original(admitted.slot.fill(record));
        Ok(())
    }

    fn admission(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        history: &crate::history::CompositeHistoryCatalog,
        port: &RelationalBranchBasisPort,
        expected: &crate::branch::ProductBranchObservation,
        successor: MemoryOriginalSuccessor<'_>,
    ) -> Result<AdmittedMemoryOriginal, Refusal> {
        let bytes =
            std::mem::size_of::<MemoryPublicationRecord>() + 2 * std::mem::size_of::<usize>();
        let funding = execution
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let meaning_bytes = std::mem::size_of::<MemoryPublicationMeaning>()
            + 2 * expected.branch_identity().name().as_str().len()
            + 2 * std::mem::size_of::<usize>();
        let meaning_funding = execution
            .reserve_retained_memory(meaning_bytes as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let mut charged = None;
        let scope = execution.run_native_with_checkpoint(|context| {
            charged = Some(if context.has_accounting() {
                context.checkpoint((bytes + meaning_bytes) as u64)
            } else {
                context.poll()
            });
        });
        match (scope, charged) {
            (Ok(_), Some(Ok(()))) => {}
            (_, Some(Err(cause))) => return Err(Refusal::Request(cause.into())),
            (Err(cause), _) => return Err(Refusal::Request(cause.into())),
            (Ok(_), None) => unreachable!("original capture charged before copying"),
        }
        let head = |generation, commit| {
            MemoryProductBranchReference::from_observed(
                expected.owner_identity().get(),
                expected.branch_identity().name().as_str().to_owned(),
                expected.lifecycle_incarnation().ordinal(),
                generation,
                commit,
            )
            .expect("an actual World branch has a valid name")
        };
        let meaning = Arc::new(MemoryPublicationMeaning {
            before: head(
                expected.reference_generation().get(),
                expected.selected_commit().ordinal(),
            ),
            after: head(
                expected
                    .reference_generation()
                    .advance()
                    .expect("generation admitted before effects")
                    .get(),
                self.commit_identity().ordinal(),
            ),
            attempt:
                worth_foundational::facade::RecordedCompositePublicationAttemptIdentity::Memory {
                    owner: self.attempt_identity().owner_identity().get(),
                    ordinal: self.attempt_identity().ordinal(),
                },
            _funding: meaning_funding,
        });
        let before =
            port.observe_memory_component(execution, expected.basis().relational_basis())?;
        let after = match successor {
            MemoryOriginalSuccessor::Candidate(candidate) => AdmittedSuccessor::Quoted(
                port.admit_successor_memory_component(execution, candidate)?,
            ),
            MemoryOriginalSuccessor::Issued(basis) => {
                AdmittedSuccessor::Issued(port.observe_memory_component(execution, basis)?)
            }
        };
        let slot = history.reserve_memory_original(execution)?;
        Ok(AdmittedMemoryOriginal {
            meaning,
            before,
            after,
            slot,
            funding,
        })
    }
}
