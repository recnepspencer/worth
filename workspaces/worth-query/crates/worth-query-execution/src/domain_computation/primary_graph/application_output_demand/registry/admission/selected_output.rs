//! An exact selector may rejoin the accepted row's actual lifecycle posture.
use super::super::{
    DemandRecord, DemandState, WorthQueryAcceptedOutputAuthority, WorthQueryOutputCheckpoint,
    WorthQueryOutputDemandKey,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

pub(in crate::domain_computation::primary_graph) struct SelectedOutputAdmission<'a> {
    pub(super) commit: &'a CompositeCommitIdentity,
    retained_key: Option<[u8; 32]>,
}

impl<'a> SelectedOutputAdmission<'a> {
    pub(in crate::domain_computation::primary_graph) fn new(
        commit: &'a CompositeCommitIdentity,
        retained_key: Option<[u8; 32]>,
    ) -> Self {
        Self {
            commit,
            retained_key,
        }
    }

    pub(super) fn matches_retained(
        &self,
        requested: &WorthQueryOutputDemandKey,
        actual: &WorthQueryOutputDemandKey,
        record: &DemandRecord,
    ) -> bool {
        let Some(expected) = self.retained_key else {
            return false;
        };
        if requested.producer != actual.producer
            || !requested.same_occurrence(actual)
            || requested.applicability.profile_kind() != actual.applicability.profile_kind()
            || !super::accepts_semantic_join(record)
        {
            return false;
        }
        let actual = match &record.state {
            DemandState::Output(output) => match &output.checkpoint {
                Some(
                    WorthQueryOutputCheckpoint::Published { receipt, .. }
                    | WorthQueryOutputCheckpoint::Delivered { receipt, .. },
                ) => *receipt.idempotency_binding().key_identity(),
                Some(WorthQueryOutputCheckpoint::Ready(completion)) => {
                    match &completion.authority {
                        WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                            *receipt.idempotency_binding().key_identity()
                        }
                        WorthQueryAcceptedOutputAuthority::Stable(published) => {
                            published.idempotency_key_identity()
                        }
                        WorthQueryAcceptedOutputAuthority::Restored(restored) => {
                            restored.checkpoint.idempotency_key
                        }
                    }
                }
                None => return output.published_commit.as_ref() == Some(self.commit),
            },
            _ => {
                return requested.source.same_semantic_source(&actual.source)
                    && super::super::succession::Succession::predecessor_of(&record.successor_of)
                        == Some(expected)
            }
        };
        // Stable aliases can change observation/source addresses without
        // changing the accepted output. The selector has compared this exact
        // idempotency candidate's source facts and native witness at its basis.
        actual == expected
    }

    pub(super) fn newest_retained_row<'rows>(
        &self,
        requested: &WorthQueryOutputDemandKey,
        rows: impl Iterator<Item = (&'rows WorthQueryOutputDemandKey, &'rows DemandRecord)>,
    ) -> Option<&'rows WorthQueryOutputDemandKey> {
        self.retained_key?;
        let mut newest = None;
        for (candidate, _) in
            rows.filter(|(key, record)| self.matches_retained(requested, key, record))
        {
            match newest {
                None => newest = Some(candidate),
                Some(current) => {
                    match super::super::refreshed_rejoin::refresh_order(candidate, current) {
                        Some(std::cmp::Ordering::Greater) => newest = Some(candidate),
                        Some(std::cmp::Ordering::Less) => {}
                        Some(std::cmp::Ordering::Equal) if candidate == current => {}
                        _ => return None,
                    }
                }
            }
        }
        newest
    }
}
