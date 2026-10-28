use worth_store_authority::ControlStoreGeneration;
use worth_store_physical_backend::ControlMediaFault;

use crate::control_store::{
    decode_control_record, OperationalControlAppendDenial, OperationalControlRecord,
    OperationalControlRecordKind, OperationalControlStore, OperationalOperationId,
    OperationalTransitionId, PersistedControlRecordDecodeDenial,
};
use crate::OperationalControlStorePort;

use super::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, AuthorizedOperationalPlan,
};

#[derive(Debug)]
pub enum AuthorizationConsumptionDenial {
    Expired,
    Revoked,
    RevocationUnavailable,
    AlreadyConsumed,
    ConsumedByDifferentOperation,
    ConsumedForDifferentPlan,
    Control(OperationalControlAppendDenial),
    DamagedControlState(ControlMediaFault),
    InvalidControlState,
    MissingDurableConsumption,
    ConcurrentProgressDidNotConverge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizationConsumptionReceipt {
    authorization_identity: [u8; 32],
    plan_fingerprint: [u8; 32],
    control_generation: ControlStoreGeneration,
    idempotent_replay: bool,
    recovered_for_resume: bool,
    expires_at: u64,
}

impl AuthorizationConsumptionReceipt {
    pub const fn authorization_identity(self) -> [u8; 32] {
        self.authorization_identity
    }
    pub const fn plan_fingerprint(self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn control_generation(self) -> ControlStoreGeneration {
        self.control_generation
    }
    pub const fn idempotent_replay(self) -> bool {
        self.idempotent_replay
    }
    pub const fn recovered_for_resume(self) -> bool {
        self.recovered_for_resume
    }
    pub const fn expires_at(self) -> u64 {
        self.expires_at
    }
}

#[derive(Debug)]
pub(crate) struct ConsumedOperationalPlan<K> {
    authorized: AuthorizedOperationalPlan<K>,
    receipt: AuthorizationConsumptionReceipt,
}

impl<K> ConsumedOperationalPlan<K> {
    pub(crate) const fn authorized(&self) -> &AuthorizedOperationalPlan<K> {
        &self.authorized
    }
    pub(crate) const fn receipt(&self) -> AuthorizationConsumptionReceipt {
        self.receipt
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn consume_authorization_through<K>(
    control: &OperationalControlStore,
    append: &dyn OperationalControlStorePort,
    operation_id: OperationalOperationId,
    transition_id: OperationalTransitionId,
    authorized: AuthorizedOperationalPlan<K>,
    execution_plan_fingerprint: Option<[u8; 32]>,
    observed_at: u64,
    revocation: AuthorizationRevocationObservation,
) -> Result<ConsumedOperationalPlan<K>, AuthorizationConsumptionDenial> {
    validate_consumption_time(&authorized, observed_at, revocation)?;
    for _attempt in 0..64 {
        let expected_generation = control
            .physical()
            .observe_current_prefix()
            .map_err(AuthorizationConsumptionDenial::DamagedControlState)?
            .map(|(generation, _)| generation);
        let record = OperationalControlRecord::authorization_consumed(
            authorized.binding().authority_identity(),
            operation_id.clone(),
            transition_id.clone(),
            authorized.authorization_identity(),
            authorized.binding().fingerprint(),
            authorized.binding().operation_tag(),
            execution_plan_fingerprint,
            authorized.assertion_identity(),
            authorized.expires_at(),
            authorized.replay_policy() == AuthorizationReplayPolicy::ReplaySameOperationIdentity,
        );
        match append.compare_exchange_authorization_consumption(expected_generation, &record) {
            Ok(receipt) => {
                let expires_at = authorized.expires_at();
                if receipt.idempotent_replay()
                    && authorized.replay_policy()
                        != AuthorizationReplayPolicy::ReplaySameOperationIdentity
                {
                    return Err(AuthorizationConsumptionDenial::AlreadyConsumed);
                }
                return Ok(ConsumedOperationalPlan {
                    authorized,
                    receipt: AuthorizationConsumptionReceipt {
                        authorization_identity: record_authorization_identity(&record),
                        plan_fingerprint: record_plan_fingerprint(&record),
                        control_generation: receipt.generation(),
                        idempotent_replay: receipt.idempotent_replay(),
                        recovered_for_resume: false,
                        expires_at,
                    },
                });
            }
            Err(OperationalControlAppendDenial::Media(ControlMediaFault::GenerationMismatch {
                ..
            })) => continue,
            Err(OperationalControlAppendDenial::Media(
                ControlMediaFault::DuplicateTransitionConflict,
            )) => {
                let existing = find_consumption(control, authorized.authorization_identity())?
                    .ok_or(AuthorizationConsumptionDenial::InvalidControlState)?;
                return classify_existing(existing, operation_id, authorized);
            }
            Err(OperationalControlAppendDenial::Media(fault)) => {
                return Err(AuthorizationConsumptionDenial::DamagedControlState(fault))
            }
            Err(denial) => return Err(AuthorizationConsumptionDenial::Control(denial)),
        }
    }
    Err(AuthorizationConsumptionDenial::ConcurrentProgressDidNotConverge)
}

fn validate_consumption_time<K>(
    authorized: &AuthorizedOperationalPlan<K>,
    observed_at: u64,
    revocation: AuthorizationRevocationObservation,
) -> Result<(), AuthorizationConsumptionDenial> {
    if observed_at < authorized.issued_at() || observed_at > authorized.expires_at() {
        return Err(AuthorizationConsumptionDenial::Expired);
    }
    match revocation {
        AuthorizationRevocationObservation::NotRevoked {
            observed_at: revocation_time,
        } if revocation_time >= observed_at => Ok(()),
        AuthorizationRevocationObservation::Revoked { .. } => {
            Err(AuthorizationConsumptionDenial::Revoked)
        }
        _ => Err(AuthorizationConsumptionDenial::RevocationUnavailable),
    }
}

#[derive(Debug)]
struct ExistingConsumption {
    operation_id: OperationalOperationId,
    plan_fingerprint: [u8; 32],
    generation: ControlStoreGeneration,
    replay_same_operation_identity: bool,
    expires_at: u64,
}

fn find_consumption(
    control: &OperationalControlStore,
    authorization_identity: [u8; 32],
) -> Result<Option<ExistingConsumption>, AuthorizationConsumptionDenial> {
    let mut found = None;
    let mut denial = None;
    control
        .physical()
        .visit_records(|raw| {
            if found.is_some() || denial.is_some() {
                return;
            }
            let decoded = decode_control_record(raw.payload()).and_then(|persisted| {
                persisted.into_domain(|handle| {
                    control
                        .physical()
                        .read_recovery_object(handle)
                        .map_err(PersistedControlRecordDecodeDenial::Media)
                })
            });
            let Ok(record) = decoded else {
                denial = Some(AuthorizationConsumptionDenial::InvalidControlState);
                return;
            };
            if let OperationalControlRecordKind::AuthorizationConsumed {
                authorization_identity: observed,
                plan_fingerprint,
                replay_same_operation_identity,
                expires_at,
                ..
            } = record.kind()
                && *observed == authorization_identity
            {
                found = Some(ExistingConsumption {
                    operation_id: record.operation_id().clone(),
                    plan_fingerprint: *plan_fingerprint,
                    generation: raw.generation(),
                    replay_same_operation_identity: *replay_same_operation_identity,
                    expires_at: *expires_at,
                });
            }
        })
        .map_err(AuthorizationConsumptionDenial::DamagedControlState)?;
    if let Some(denial) = denial {
        return Err(denial);
    }
    Ok(found)
}

fn classify_existing<K>(
    existing: ExistingConsumption,
    operation_id: OperationalOperationId,
    authorized: AuthorizedOperationalPlan<K>,
) -> Result<ConsumedOperationalPlan<K>, AuthorizationConsumptionDenial> {
    if existing.operation_id != operation_id {
        return Err(AuthorizationConsumptionDenial::ConsumedByDifferentOperation);
    }
    if existing.plan_fingerprint != authorized.binding().fingerprint() {
        return Err(AuthorizationConsumptionDenial::ConsumedForDifferentPlan);
    }
    if !existing.replay_same_operation_identity {
        return Err(AuthorizationConsumptionDenial::AlreadyConsumed);
    }
    Ok(ConsumedOperationalPlan {
        receipt: AuthorizationConsumptionReceipt {
            authorization_identity: authorized.authorization_identity(),
            plan_fingerprint: authorized.binding().fingerprint(),
            control_generation: existing.generation,
            idempotent_replay: true,
            recovered_for_resume: false,
            expires_at: existing.expires_at,
        },
        authorized,
    })
}

pub(crate) fn recover_authorization_consumption(
    control: &OperationalControlStore,
    operation_id: &OperationalOperationId,
    authorization_identity: [u8; 32],
    plan_fingerprint: [u8; 32],
) -> Result<AuthorizationConsumptionReceipt, AuthorizationConsumptionDenial> {
    let mut recovered = None;
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
                denial = Some(AuthorizationConsumptionDenial::InvalidControlState);
                return;
            };
            let OperationalControlRecordKind::AuthorizationConsumed {
                authorization_identity: observed,
                plan_fingerprint: observed_plan,
                ..
            } = record.kind()
            else {
                return;
            };
            if *observed != authorization_identity {
                return;
            }
            if record.operation_id() != operation_id || *observed_plan != plan_fingerprint {
                denial = Some(AuthorizationConsumptionDenial::ConsumedForDifferentPlan);
                return;
            }
            if recovered.is_some() {
                denial = Some(AuthorizationConsumptionDenial::InvalidControlState);
                return;
            }
            recovered = Some(AuthorizationConsumptionReceipt {
                authorization_identity,
                plan_fingerprint,
                control_generation: raw.generation(),
                idempotent_replay: false,
                recovered_for_resume: true,
                expires_at: match record.kind() {
                    OperationalControlRecordKind::AuthorizationConsumed { expires_at, .. } => {
                        *expires_at
                    }
                    _ => unreachable!(),
                },
            });
        })
        .map_err(AuthorizationConsumptionDenial::DamagedControlState)?;
    if let Some(denial) = denial {
        return Err(denial);
    }
    recovered.ok_or(AuthorizationConsumptionDenial::MissingDurableConsumption)
}

pub(crate) fn record_recovery_staging_completion(
    control: &dyn OperationalControlStorePort,
    authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    operation_id: OperationalOperationId,
    authorization: AuthorizationConsumptionReceipt,
    execution_plan_fingerprint: [u8; 32],
    staged_media_identity: [u8; 32],
) -> Result<(), OperationalControlAppendDenial> {
    control.append(&OperationalControlRecord::recovery_staging_completed(
        authority_identity,
        operation_id,
        authorization.authorization_identity(),
        authorization.plan_fingerprint(),
        execution_plan_fingerprint,
        staged_media_identity,
    ))?;
    Ok(())
}

fn record_authorization_identity(record: &OperationalControlRecord) -> [u8; 32] {
    let OperationalControlRecordKind::AuthorizationConsumed {
        authorization_identity,
        ..
    } = record.kind()
    else {
        unreachable!("authorization consumption constructor must create consumption record")
    };
    *authorization_identity
}

fn record_plan_fingerprint(record: &OperationalControlRecord) -> [u8; 32] {
    let OperationalControlRecordKind::AuthorizationConsumed {
        plan_fingerprint, ..
    } = record.kind()
    else {
        unreachable!("authorization consumption constructor must create consumption record")
    };
    *plan_fingerprint
}
