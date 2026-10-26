use std::collections::HashMap;

use crate::control_store::{decode_control_record, PersistedControlRecordDecodeDenial};
use crate::{
    OperationalControlRecordKind, OperationalControlStore, OperationalOperationId,
    RepairRecoveryTopology,
};

use super::{RepairExecutionDisposition, RepairJournalDenial};

pub(super) struct ReopenedRepairJournal {
    pub(super) opened: Option<([u8; 32], [u8; 32], u64, RepairRecoveryTopology)>,
    pub(super) completed: HashMap<[u8; 32], ([u8; 32], u8)>,
    pub(super) started: HashMap<[u8; 32], u8>,
    pub(super) disposition: Option<RepairExecutionDisposition>,
}

pub(super) fn scan(
    control: &OperationalControlStore,
    operation: &OperationalOperationId,
) -> Result<ReopenedRepairJournal, RepairJournalDenial> {
    let mut state = ReopenedRepairJournal {
        opened: None,
        completed: HashMap::new(),
        started: HashMap::new(),
        disposition: None,
    };
    let mut denial = None;
    control
        .physical()
        .visit_records(|raw| {
            if denial.is_some() {
                return;
            }
            let record = decode_control_record(raw.payload()).and_then(|persisted| {
                persisted.into_domain(|handle| {
                    control
                        .physical()
                        .read_recovery_object(handle)
                        .map_err(PersistedControlRecordDecodeDenial::Media)
                })
            });
            let Ok(record) = record else {
                denial = Some(RepairJournalDenial::InvalidHistory);
                return;
            };
            if record.operation_id() == operation
                && let Err(error) = observe_record(&mut state, record.kind())
            {
                denial = Some(error);
            }
        })
        .map_err(|_| RepairJournalDenial::Media)?;
    denial.map_or(Ok(state), Err)
}

fn observe_record(
    state: &mut ReopenedRepairJournal,
    kind: &OperationalControlRecordKind,
) -> Result<(), RepairJournalDenial> {
    match kind {
        OperationalControlRecordKind::RepairExecutionOpened {
            authorization_identity,
            plan_fingerprint,
            owner_node_count,
            topology_tag,
        } => {
            if state.opened.is_some() {
                return Err(RepairJournalDenial::InvalidHistory);
            }
            state.opened = Some((
                *authorization_identity,
                *plan_fingerprint,
                *owner_node_count,
                RepairRecoveryTopology::from_tag(*topology_tag)
                    .ok_or(RepairJournalDenial::InvalidHistory)?,
            ));
        }
        OperationalControlRecordKind::RepairOwnerEffectStarted {
            plan_fingerprint,
            node_fingerprint,
            owner_tag,
        } => {
            validate_open(state, *plan_fingerprint)?;
            state
                .started
                .try_reserve(1)
                .map_err(|_| RepairJournalDenial::InvalidHistory)?;
            if *owner_tag == 0
                || state
                    .started
                    .insert(*node_fingerprint, *owner_tag)
                    .is_some()
            {
                return Err(RepairJournalDenial::InvalidHistory);
            }
        }
        OperationalControlRecordKind::RepairOwnerReceiptPersisted {
            plan_fingerprint,
            node_fingerprint,
            receipt_fingerprint,
            owner_tag,
        } => {
            validate_open(state, *plan_fingerprint)?;
            if state.started.get(node_fingerprint) != Some(owner_tag) {
                return Err(RepairJournalDenial::InvalidHistory);
            }
            state
                .completed
                .try_reserve(1)
                .map_err(|_| RepairJournalDenial::InvalidHistory)?;
            if state
                .completed
                .insert(*node_fingerprint, (*receipt_fingerprint, *owner_tag))
                .is_some()
            {
                return Err(RepairJournalDenial::DuplicateOwnerReceipt);
            }
        }
        OperationalControlRecordKind::RepairDispositionRecorded {
            plan_fingerprint,
            disposition_tag,
            disposition_basis,
        } => {
            validate_open(state, *plan_fingerprint)?;
            if *disposition_basis == [0; 32] || state.disposition.is_some() {
                return Err(RepairJournalDenial::InvalidHistory);
            }
            let disposition = disposition_from_tag(*disposition_tag)?;
            let (_, _, expected, topology) =
                state.opened.ok_or(RepairJournalDenial::InvalidHistory)?;
            let mutating_starts = state
                .started
                .values()
                .filter(|owner_tag| **owner_tag != 2)
                .count() as u64;
            if (disposition == RepairExecutionDisposition::Executed
                && state.completed.len() as u64 != expected)
                || (disposition == RepairExecutionDisposition::Abandoned && mutating_starts != 0)
                || (disposition == RepairExecutionDisposition::IsolatedForRecovery
                    && (topology != RepairRecoveryTopology::NonCurrentAuthorityAffecting
                        || mutating_starts == 0))
            {
                return Err(RepairJournalDenial::InvalidHistory);
            }
            state.disposition = Some(disposition);
        }
        _ => {}
    }
    Ok(())
}

fn validate_open(state: &ReopenedRepairJournal, plan: [u8; 32]) -> Result<(), RepairJournalDenial> {
    match state.opened {
        Some((_, opened, _, _)) if opened == plan && state.disposition.is_none() => Ok(()),
        Some(_) => Err(RepairJournalDenial::PlanMismatch),
        None => Err(RepairJournalDenial::InvalidHistory),
    }
}

const fn disposition_from_tag(
    value: u8,
) -> Result<RepairExecutionDisposition, RepairJournalDenial> {
    match value {
        1 => Ok(RepairExecutionDisposition::Executed),
        2 => Ok(RepairExecutionDisposition::Abandoned),
        3 => Ok(RepairExecutionDisposition::Indeterminate),
        4 => Ok(RepairExecutionDisposition::IsolatedForRecovery),
        _ => Err(RepairJournalDenial::InvalidHistory),
    }
}
