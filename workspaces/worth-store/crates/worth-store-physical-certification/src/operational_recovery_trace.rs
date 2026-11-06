use sha2::{Digest, Sha256};
use worth_foundational::CanonicalDigestId;

use crate::OperationalRecoveryYieldpoint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalRecoveryDriverTrace {
    reached: Vec<OperationalRecoveryYieldpoint>,
    operation_identities: Vec<String>,
    control_artifact_identities: Vec<CanonicalDigestId>,
    inspection_evidence_identity: Option<CanonicalDigestId>,
    truth_evidence_identity: Option<CanonicalDigestId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalRecoveryTraceJoinDenial {
    Empty,
    InspectionEvidenceConflict,
    TruthEvidenceConflict,
}

impl OperationalRecoveryDriverTrace {
    pub(super) fn from_observations(
        reached: Vec<OperationalRecoveryYieldpoint>,
        operation_identities: Vec<String>,
        control_artifact_identities: Vec<CanonicalDigestId>,
        inspection_evidence_identity: Option<CanonicalDigestId>,
        truth_evidence_identity: Option<CanonicalDigestId>,
    ) -> Self {
        Self {
            reached,
            operation_identities,
            control_artifact_identities,
            inspection_evidence_identity,
            truth_evidence_identity,
        }
    }

    pub fn reached(&self) -> &[OperationalRecoveryYieldpoint] {
        &self.reached
    }

    pub fn operation_identities(&self) -> &[String] {
        &self.operation_identities
    }

    pub fn control_artifact_identities(&self) -> &[CanonicalDigestId] {
        &self.control_artifact_identities
    }

    pub const fn inspection_evidence_identity(&self) -> Option<[u8; 32]> {
        match self.inspection_evidence_identity {
            Some(digest) => Some(*digest.bytes()),
            None => None,
        }
    }

    pub const fn truth_evidence_identity(&self) -> Option<[u8; 32]> {
        match self.truth_evidence_identity {
            Some(digest) => Some(*digest.bytes()),
            None => None,
        }
    }

    pub fn evidence_identity(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth-store-operational-recovery-driver-trace-v1");
        digest.update((self.reached.len() as u64).to_be_bytes());
        for point in &self.reached {
            let token = point.token();
            digest.update((token.len() as u64).to_be_bytes());
            digest.update(token.as_bytes());
        }
        digest.update((self.operation_identities.len() as u64).to_be_bytes());
        for operation in &self.operation_identities {
            digest.update((operation.len() as u64).to_be_bytes());
            digest.update(operation.as_bytes());
        }
        digest.update((self.control_artifact_identities.len() as u64).to_be_bytes());
        for identity in &self.control_artifact_identities {
            digest.update(identity.bytes());
        }
        update_optional_identity(
            &mut digest,
            self.inspection_evidence_identity
                .map(|digest| *digest.bytes()),
        );
        update_optional_identity(
            &mut digest,
            self.truth_evidence_identity.map(|digest| *digest.bytes()),
        );
        digest.finalize().into()
    }

    pub fn join(
        traces: impl IntoIterator<Item = Self>,
    ) -> Result<Self, OperationalRecoveryTraceJoinDenial> {
        let mut traces = traces.into_iter();
        let Some(first) = traces.next() else {
            return Err(OperationalRecoveryTraceJoinDenial::Empty);
        };
        let mut joined = first;
        for trace in traces {
            joined.reached.extend(trace.reached);
            joined
                .operation_identities
                .extend(trace.operation_identities);
            joined
                .control_artifact_identities
                .extend(trace.control_artifact_identities);
            joined.inspection_evidence_identity = join_optional_identity(
                joined.inspection_evidence_identity,
                trace.inspection_evidence_identity,
                OperationalRecoveryTraceJoinDenial::InspectionEvidenceConflict,
            )?;
            joined.truth_evidence_identity = join_optional_identity(
                joined.truth_evidence_identity,
                trace.truth_evidence_identity,
                OperationalRecoveryTraceJoinDenial::TruthEvidenceConflict,
            )?;
        }
        // Yieldpoints are an ordered event stream, not a coverage set. The
        // same durable transition kind may occur for several ordinary owner
        // operations in one scenario, and certification must be able to
        // account for every occurrence in execution order.
        joined.operation_identities.sort();
        joined.operation_identities.dedup();
        joined.control_artifact_identities.sort_unstable();
        joined.control_artifact_identities.dedup();
        Ok(joined)
    }
}

fn join_optional_identity(
    left: Option<CanonicalDigestId>,
    right: Option<CanonicalDigestId>,
    conflict: OperationalRecoveryTraceJoinDenial,
) -> Result<Option<CanonicalDigestId>, OperationalRecoveryTraceJoinDenial> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(conflict),
        (Some(identity), _) | (_, Some(identity)) => Ok(Some(identity)),
        (None, None) => Ok(None),
    }
}

fn update_optional_identity(digest: &mut Sha256, identity: Option<[u8; 32]>) {
    match identity {
        Some(identity) => {
            digest.update([1]);
            digest.update(identity);
        }
        None => digest.update([0]),
    }
}

#[cfg(test)]
mod tests {
    use crate::OperationalRecoveryControlTransitionKind;

    use super::*;

    #[test]
    fn joining_traces_preserves_repeated_transition_events_in_order() {
        let before = OperationalRecoveryYieldpoint::BeforeDurableControlTransition(
            OperationalRecoveryControlTransitionKind::RecoveryStagingCompletion,
        );
        let after = OperationalRecoveryYieldpoint::AfterDurableControlTransition(
            OperationalRecoveryControlTransitionKind::RecoveryStagingCompletion,
        );
        let first = OperationalRecoveryDriverTrace::from_observations(
            vec![before, after],
            vec!["restore".to_owned()],
            vec![CanonicalDigestId::new([1; 32])],
            None,
            None,
        );
        let second = OperationalRecoveryDriverTrace::from_observations(
            vec![before, after],
            vec!["pitr".to_owned()],
            vec![CanonicalDigestId::new([2; 32])],
            None,
            None,
        );

        let joined = OperationalRecoveryDriverTrace::join([first, second]).unwrap();

        assert_eq!(joined.reached(), &[before, after, before, after]);
    }
}
