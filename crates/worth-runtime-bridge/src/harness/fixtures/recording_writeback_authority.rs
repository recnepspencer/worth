use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use crate::adapter::{
    TruthWritebackAuthority, TruthWritebackAuthorityError, TruthWritebackReceipt,
    TruthWritebackRequest,
};
use crate::writeback::BridgeWritebackOutcomeClass;

#[derive(Debug, Clone, Default)]
struct RecordingTruthWritebackState {
    first_commit_by_family_and_causality:
        BTreeMap<RecordingTruthWritebackCommitKey, TruthWritebackReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RecordingTruthWritebackCommitKey {
    family_kind: crate::writeback::BridgeWritebackFamilyKind,
    causality_digest: Arc<str>,
}

impl RecordingTruthWritebackCommitKey {
    fn from_request(request: &TruthWritebackRequest) -> Self {
        Self {
            family_kind: request.family_kind(),
            causality_digest: Arc::from(request.causality_digest()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecordingTruthWritebackAuthority {
    state: Arc<RwLock<RecordingTruthWritebackState>>,
}

impl RecordingTruthWritebackAuthority {
    pub fn committed_causality_count(&self) -> usize {
        self.state
            .read()
            .expect("writeback authority lock poisoned")
            .first_commit_by_family_and_causality
            .len()
    }
}

impl TruthWritebackAuthority for RecordingTruthWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        let mut state = self
            .state
            .write()
            .expect("writeback authority lock poisoned");
        let commit_key = RecordingTruthWritebackCommitKey::from_request(&request);

        if let Some(existing_artifact) = state
            .first_commit_by_family_and_causality
            .get(&commit_key)
            .cloned()
        {
            let prior_receipt = existing_artifact;
            return Ok(TruthWritebackReceipt::canonical_noop_from_prior_receipt(
                &request,
                &prior_receipt,
            ));
        }

        let receipt =
            TruthWritebackReceipt::new(BridgeWritebackOutcomeClass::AuthoritativeCommit, &request);
        state
            .first_commit_by_family_and_causality
            .insert(commit_key, receipt.clone());

        Ok(receipt)
    }
}
