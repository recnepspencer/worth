//! The existing journal tail/codec algorithm, outside short admission locks.
use super::{operation::*, outcome::*, recovery::*, *};
use std::sync::{Arc, Mutex, Weak};
use worth_proof::LinearResource;
use worth_store_physical_format::{RecordAnchor, MAXIMUM_RECORD_ANCHORS};
struct AdmissionReservation {
    owner: Arc<Mutex<OrderedJournalState>>,
}
impl Drop for AdmissionReservation {
    fn drop(&mut self) {
        self.owner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .admitting = false;
    }
}
impl OrderedJournal<'_> {
    /// Reads the protected head and encodes one successor. I/O, hashing,
    /// allocation and short locks are cooperative, not hard real-time.
    pub fn start_publication(
        &self,
        publication: OrderedJournalPublication,
        interruption: worth_execution::ExecutionInterruptionView,
    ) -> OrderedJournalPublicationStart {
        self.start_owned(publication, Some(Arc::new(interruption)))
    }
    pub(super) fn start_owned(
        &self,
        publication: OrderedJournalPublication,
        interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> OrderedJournalPublicationStart {
        match self.admit_owned(publication, interruption) {
            Ok(start) => start,
            Err(cause) => OrderedJournalPublicationStart::Refused(cause),
        }
    }
    fn admit_owned(
        &self,
        mut publication: OrderedJournalPublication,
        interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> Result<OrderedJournalPublicationStart, OrderedJournalOperationFailure> {
        if let Some(stop) = interruption.as_ref().and_then(|v| v.check_live().err()) {
            return Err(OrderedJournalOperationFailure::OriginalRequestStopped(stop));
        }
        let owner = self.runtime.ordered_journal_owner();
        {
            let mut state = owner.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(position) = state.unresolved.get(&self.name) {
                return Err(Failure::Unresolved {
                    position: *position,
                }
                .into());
            }
            let active = state
                .active
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|active| active.state().live);
            if state.admitting || active {
                return Err(OrderedJournalOperationFailure::Busy);
            }
            state.admitting = true;
        }
        let _reservation = AdmissionReservation {
            owner: owner.clone(),
        };
        if publication.payload.is_empty() {
            return Err(Failure::EmptyPayload.into());
        }
        let digest: [u8; 32] = Sha256::digest(&publication.payload).into();
        let source = OrderedJournalPublicationSource {
            runtime: self.runtime.runtime_identity(),
            journal: self.name,
            position: publication.position,
            digest,
            bytes: publication.payload.len() as u64,
        };
        let reader = self.reader().map_err(Failure::Read)?;
        let head = self.head_anchor(&reader);
        let head_position = head.map_or(0, RecordAnchor::generation);
        if publication.position >= 1 && publication.position <= head_position {
            let entry = self
                .locate(&reader, publication.position)
                .map_err(Failure::Read)?;
            return if entry.payload_sha256() == digest {
                Ok(OrderedJournalPublicationStart::AlreadyCommitted(
                    OrderedJournalCommitted {
                        terminal: Arc::new(OrderedJournalPublicationTerminal {
                            source,
                            mutation: None,
                            outcome: None,
                            published: Some(OrderedJournalPublished::AlreadyCommitted(entry)),
                            failure: None,
                        }),
                    },
                ))
            } else {
                Err(Failure::PositionConflict(entry).into())
            };
        }
        if Some(publication.position) != head_position.checked_add(1) {
            return Err(Failure::PositionGap {
                head: head_position,
                requested: publication.position,
            }
            .into());
        }
        let anchor_table_full =
            reader.selected_record_anchors().iter().count() == MAXIMUM_RECORD_ANCHORS;
        if head.is_none() && anchor_table_full {
            return Err(Failure::AnchorTableFull.into());
        }
        let head_header = match head {
            Some(anchor) => Some((
                read_header(&reader, self, anchor.record()).map_err(Failure::Read)?,
                anchor.record(),
            )),
            None => None,
        };
        drop(reader);
        let header = JournalRecordHeader::successor(
            self.name,
            head_header
                .as_ref()
                .map(|(header, record)| (header, *record)),
            &publication.payload,
        )
        .filter(|header| header.position == publication.position)
        .ok_or(Failure::Read(OrderedJournalReadFailure::Damaged))?;

        let plan = AnchoredRecordAppend {
            record: header.encode_record(&publication.payload),
            anchor: self.name.anchor(),
            expected_generation: head_position,
            idempotency: material(self.name, publication.position, digest),
            placement: publication.placement,
            deadline: publication.deadline,
        };
        let shared = Arc::new(JournalPublicationShared {
            source,
            owner: Arc::downgrade(&owner),
            state: Mutex::new(JournalProgress {
                plan: Some(plan),
                input: publication.input_memory.take(),
                interruption,
                preparing: false,
                stopped: false,
                live: true,
                recovery_alive: true,
                endpoint_live: true,
                returned: None,
                mutation: None,
                attempt: Weak::new(),
                existing: None,
                refusal: None,
                unresolved_duplicate: false,
                fact: None,
                settling: false,
                terminal: None,
                visibility: JournalTerminalVisibility::AwaitingBasePublication,
            }),
        });
        owner.lock().unwrap_or_else(|p| p.into_inner()).active = Some(Arc::downgrade(&shared));
        let custody = LinearResource::mint(source, &OrderedJournalPublicationAuthority::witness());
        Ok(OrderedJournalPublicationStart::Ready {
            operation: OrderedJournalPublicationOperation {
                shared: shared.clone(),
                custody: Some(custody),
            },
            recovery: OrderedJournalPublicationRecovery { shared },
        })
    }
}
