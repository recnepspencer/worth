use sha2::{Digest, Sha256};
use worth_foundational::CanonicalDigestId;
use worth_store_operations::OperationalControlSessionObservation;

#[cfg(test)]
use crate::OperationalRecoveryDriverTrace;
use crate::OperationalRecoveryYieldpoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalRecoveryCrashCutDenial {
    UnsupportedCut,
    CutWasNotLastObservation,
    NoBoundOperation,
    SameControlSession,
    SameProcess,
    ControlMediaSubstituted,
    BeforeCutChangedDurablePrefix,
    AfterCutDidNotAdvanceExactlyOnce,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalRecoveryCrashCutEvidence {
    yieldpoint: OperationalRecoveryYieldpoint,
    operation_identities: Vec<String>,
    evidence_identity: CanonicalDigestId,
}

impl OperationalRecoveryCrashCutEvidence {
    #[cfg(test)]
    pub(crate) fn from_fresh_process_reopen(
        yieldpoint: OperationalRecoveryYieldpoint,
        trace: &OperationalRecoveryDriverTrace,
        immediately_before: OperationalControlSessionObservation,
        reopened_after_crash: OperationalControlSessionObservation,
    ) -> Result<Self, OperationalRecoveryCrashCutDenial> {
        let is_before = match yieldpoint {
            OperationalRecoveryYieldpoint::BeforeDurableControlTransition(_) => true,
            OperationalRecoveryYieldpoint::AfterDurableControlTransition(_) => false,
            _ => return Err(OperationalRecoveryCrashCutDenial::UnsupportedCut),
        };
        if trace.reached().last() != Some(&yieldpoint) {
            return Err(OperationalRecoveryCrashCutDenial::CutWasNotLastObservation);
        }
        if trace.operation_identities().is_empty() {
            return Err(OperationalRecoveryCrashCutDenial::NoBoundOperation);
        }
        if immediately_before.session() == reopened_after_crash.session() {
            return Err(OperationalRecoveryCrashCutDenial::SameControlSession);
        }
        if immediately_before.process() == reopened_after_crash.process() {
            return Err(OperationalRecoveryCrashCutDenial::SameProcess);
        }
        if immediately_before.media_identity_fingerprint()
            != reopened_after_crash.media_identity_fingerprint()
        {
            return Err(OperationalRecoveryCrashCutDenial::ControlMediaSubstituted);
        }
        let before = immediately_before.coordinates();
        let reopened = reopened_after_crash.coordinates();
        if is_before {
            if before != reopened {
                return Err(OperationalRecoveryCrashCutDenial::BeforeCutChangedDurablePrefix);
            }
        } else if !advanced_exactly_once(before, reopened) {
            return Err(OperationalRecoveryCrashCutDenial::AfterCutDidNotAdvanceExactlyOnce);
        }
        let mut digest = Sha256::new();
        digest.update(b"worth-store-operational-recovery-control-crash-cut-v1");
        digest.update(yieldpoint.token().as_bytes());
        digest.update(immediately_before.session().fingerprint());
        digest.update(reopened_after_crash.session().fingerprint());
        digest.update(immediately_before.process().fingerprint());
        digest.update(reopened_after_crash.process().fingerprint());
        digest.update(immediately_before.media_identity_fingerprint());
        update_coordinates(&mut digest, before);
        update_coordinates(&mut digest, reopened);
        digest.update(trace.evidence_identity());
        Ok(Self {
            yieldpoint,
            operation_identities: trace.operation_identities().to_vec(),
            evidence_identity: CanonicalDigestId::new(digest.finalize().into()),
        })
    }

    pub(crate) fn from_external_process_reopen(
        yieldpoint: OperationalRecoveryYieldpoint,
        cut_trace_identity: [u8; 32],
        operation_identities: Vec<String>,
        immediately_before: OperationalControlSessionObservation,
        reopened_after_crash: OperationalControlSessionObservation,
        challenge: [u8; 32],
    ) -> Result<Self, OperationalRecoveryCrashCutDenial> {
        let expects_advance = matches!(
            yieldpoint,
            OperationalRecoveryYieldpoint::AfterDurableControlTransition(_)
        );
        if operation_identities.is_empty() {
            return Err(OperationalRecoveryCrashCutDenial::NoBoundOperation);
        }
        validate_reopen(!expects_advance, immediately_before, reopened_after_crash)?;
        let before = immediately_before.coordinates();
        let reopened = reopened_after_crash.coordinates();
        let mut digest = Sha256::new();
        digest.update(b"worth-store-operational-recovery-external-process-crash-cut-v1");
        digest.update(yieldpoint.token().as_bytes());
        digest.update(challenge);
        digest.update(cut_trace_identity);
        digest.update(immediately_before.process().fingerprint());
        digest.update(reopened_after_crash.process().fingerprint());
        digest.update(immediately_before.session().fingerprint());
        digest.update(reopened_after_crash.session().fingerprint());
        digest.update(immediately_before.media_identity_fingerprint());
        update_coordinates(&mut digest, before);
        update_coordinates(&mut digest, reopened);
        for operation in &operation_identities {
            digest.update((operation.len() as u64).to_be_bytes());
            digest.update(operation.as_bytes());
        }
        Ok(Self {
            yieldpoint,
            operation_identities,
            evidence_identity: CanonicalDigestId::new(digest.finalize().into()),
        })
    }

    pub const fn yieldpoint(&self) -> OperationalRecoveryYieldpoint {
        self.yieldpoint
    }

    pub fn operation_identities(&self) -> &[String] {
        &self.operation_identities
    }

    pub const fn evidence_identity(&self) -> [u8; 32] {
        *self.evidence_identity.bytes()
    }
}

fn validate_reopen(
    is_before: bool,
    immediately_before: OperationalControlSessionObservation,
    reopened_after_crash: OperationalControlSessionObservation,
) -> Result<(), OperationalRecoveryCrashCutDenial> {
    if immediately_before.session() == reopened_after_crash.session() {
        return Err(OperationalRecoveryCrashCutDenial::SameControlSession);
    }
    if immediately_before.process() == reopened_after_crash.process() {
        return Err(OperationalRecoveryCrashCutDenial::SameProcess);
    }
    if immediately_before.media_identity_fingerprint()
        != reopened_after_crash.media_identity_fingerprint()
    {
        return Err(OperationalRecoveryCrashCutDenial::ControlMediaSubstituted);
    }
    let before = immediately_before.coordinates();
    let reopened = reopened_after_crash.coordinates();
    if is_before && before != reopened {
        return Err(OperationalRecoveryCrashCutDenial::BeforeCutChangedDurablePrefix);
    }
    if !is_before && !advanced_exactly_once(before, reopened) {
        return Err(OperationalRecoveryCrashCutDenial::AfterCutDidNotAdvanceExactlyOnce);
    }
    Ok(())
}

fn advanced_exactly_once(
    before: Option<worth_store_authority::ControlStoreSelectionCoordinates>,
    reopened: Option<worth_store_authority::ControlStoreSelectionCoordinates>,
) -> bool {
    match (before, reopened) {
        (None, Some(reopened)) => reopened.generation().get() == 1,
        (Some(before), Some(reopened)) => {
            reopened.generation().get() == before.generation().get().saturating_add(1)
                && reopened.prefix_digest() != before.prefix_digest()
        }
        _ => false,
    }
}

fn update_coordinates(
    digest: &mut Sha256,
    coordinates: Option<worth_store_authority::ControlStoreSelectionCoordinates>,
) {
    match coordinates {
        Some(coordinates) => {
            digest.update([1]);
            digest.update(coordinates.media_identity_fingerprint());
            digest.update(coordinates.generation().get().to_be_bytes());
            digest.update(coordinates.prefix_digest());
        }
        None => digest.update([0]),
    }
}
