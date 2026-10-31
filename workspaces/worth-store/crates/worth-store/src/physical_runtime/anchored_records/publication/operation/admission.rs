//! Admission and planning may read the selected value and hash all input bytes.
//! They hold only a short owner reservation, never its mutex across that work.
use super::super::{
    admission::AdmittedMembers,
    batch_capacity::BatchCapacity,
    current::{CurrentDecision, RequestedPayload},
    member_writes::{MemberResolution, MemberWritePlan},
    AnchoredRecordHeader, AnchoredRecordPublication,
};
use super::*;
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, RecordAppendDenial,
};
use sha2::{Digest, Sha256};

pub(super) struct PublicationPlan {
    pub(super) name: super::super::super::AnchoredRecordName,
    pub(super) expected_generation: u64,
    pub(super) payload: Vec<u8>,
    pub(super) header: AnchoredRecordHeader,
    pub(super) members: MemberWritePlan,
    pub(super) resolution: MemberResolution,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) deadline: PhysicalMutationDeadline,
    pub(super) locators: Vec<AnchoredRecordMemberLocator>,
    pub(super) record_sha256: [u8; 32],
}
pub(super) struct FinalSelection {
    pub(super) name: super::super::super::AnchoredRecordName,
    pub(super) membership: super::super::super::membership::PreparedValueMembership,
    pub(super) record_sha256: [u8; 32],
}
struct AdmissionReservation {
    owner: Arc<Mutex<AnchoredRecordState>>,
}
impl Drop for AdmissionReservation {
    fn drop(&mut self) {
        self.owner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .admitting = false;
    }
}
impl AnchoredRecords<'_> {
    /// Hashes the input and may read the selected anchored value. Substrate
    /// I/O/allocation and short locks are cooperative, not hard real-time.
    pub fn start_publication(
        &self,
        publication: AnchoredRecordPublication,
        interruption: worth_execution::ExecutionInterruptionView,
    ) -> AnchoredRecordPublicationStart {
        self.admit_publication(publication, Some(Arc::new(interruption)))
    }
    pub(in crate::physical_runtime::anchored_records) fn admit_publication(
        &self,
        publication: AnchoredRecordPublication,
        interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> AnchoredRecordPublicationStart {
        match self.plan_publication(publication, interruption) {
            Ok(start) => start,
            Err(failure) => AnchoredRecordPublicationStart::Refused(failure),
        }
    }
    fn plan_publication(
        &self,
        publication: AnchoredRecordPublication,
        interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> Result<AnchoredRecordPublicationStart, AnchoredRecordOperationFailure> {
        type Failure = AnchoredRecordPublicationFailure;
        if let Some(stop) = interruption
            .as_ref()
            .and_then(|view| view.check_live().err())
        {
            return Err(AnchoredRecordOperationFailure::OriginalRequestStopped(stop));
        }
        if self
            .runtime
            .journal_registry()
            .reserves(publication.name.anchor())
        {
            return Err(Failure::ReservedForOrderedJournal.into());
        }
        if publication.payload.is_empty() {
            return Err(Failure::EmptyPayload.into());
        }
        let owner = self.runtime.anchored_publication_owner();
        {
            let mut state = owner.lock().unwrap_or_else(|p| p.into_inner());
            if state.unresolved.contains(&publication.name.anchor()) {
                return Err(Failure::Unresolved.into());
            }
            let active = state
                .active
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|active| {
                    let state = active.state();
                    state.live
                });
            if state.admitting || active {
                return Err(AnchoredRecordOperationFailure::Busy);
            }
            state.admitting = true;
        }
        let _reservation = AdmissionReservation {
            owner: owner.clone(),
        };
        let name = publication.name;
        let capacity = BatchCapacity::of(self.runtime, publication.placement);
        let members = AdmittedMembers::admit(publication.members, capacity)?;
        let payload = RequestedPayload {
            sha256: Sha256::digest(&publication.payload).into(),
            bytes: publication.payload.len() as u64,
        };
        let target = match self.decide_against_current(
            name,
            publication.expected_generation,
            payload,
            &members,
        )? {
            CurrentDecision::AlreadyPublished(membership) => {
                return Ok(AnchoredRecordPublicationStart::AlreadyPublished(
                    AnchoredRecordPublished::AlreadyPublished(membership),
                ))
            }
            CurrentDecision::Replace { target } => target,
        };
        let header = AnchoredRecordHeader::describe(
            name.anchor(),
            target,
            &publication.payload,
            members.len(),
        );
        if header.record_bytes() > capacity.bytes {
            return Err(Failure::Batch(RecordAppendDenial::BatchByteLimitExceeded).into());
        }
        let content = members.content_digest(name, target, payload.sha256);
        let runtime = self.runtime.runtime_identity();
        let (members, resolution) =
            MemberWritePlan::plan(members, name, target, content, runtime.get(), capacity)?;
        let identity = AnchoredRecordPublicationIdentity {
            runtime,
            name,
            generation: target,
            content,
        };
        let shared = Arc::new(PublicationShared {
            identity,
            owner: Arc::downgrade(&owner),
            state: Mutex::new(PublicationState {
                plan: Some(PublicationPlan {
                    name,
                    expected_generation: publication.expected_generation,
                    payload: publication.payload,
                    header,
                    members,
                    resolution,
                    placement: publication.placement,
                    deadline: publication.deadline,
                    locators: Vec::new(),
                    record_sha256: [0; 32],
                }),
                final_selection: None,
                recovery_alive: true,
                prefix: Vec::new(),
                written_batches: 0,
                accepted: None,
                contents: Vec::new(),
                kind: AnchoredRecordStepKind::Members { batch_index: 0 },
                last_terminal: None,
                terminal: None,
                stopped: false,
                live: true,
                interruption,
                returned_custody: None,
            }),
        });
        owner.lock().unwrap_or_else(|p| p.into_inner()).active = Some(Arc::downgrade(&shared));
        let custody =
            LinearResource::mint(identity, &AnchoredPublicationCustodyAuthority::witness());
        Ok(AnchoredRecordPublicationStart::Ready {
            operation: AnchoredRecordPublicationOperation {
                shared: shared.clone(),
                custody: Some(custody),
            },
            recovery: AnchoredRecordPublicationRecovery { shared },
        })
    }
}
