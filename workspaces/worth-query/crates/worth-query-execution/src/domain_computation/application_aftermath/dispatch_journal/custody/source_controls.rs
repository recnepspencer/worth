//! Home source facts share the journal's original publication and readmission owner.
use super::settlement::DispatchJournalDriver;
use super::*;
use worth_execution::ExecutionRequest;

pub(in crate::domain_computation) enum QueryInboundSourceControlObservation {
    Recorded(Option<InboundSourceControl>),
    /// This control is not yet a recorded fact; consumption remains fenced.
    Pending,
}

fn source_key(source: &str) -> CanonicalDigestId {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"worth-query.home-inbound-source-control.v1\0");
    digest.update(
        u64::try_from(source.len())
            .expect("addressable source identity")
            .to_le_bytes(),
    );
    digest.update(source.as_bytes());
    CanonicalDigestId::new(digest.finalize().into())
}

impl WorthQueryDispatchJournal {
    pub(in crate::domain_computation) fn source_control(
        &self,
        source: &str,
    ) -> QueryInboundSourceControlObservation {
        let source = source_key(source);
        let guard = self.state();
        if let Some(state) = guard.as_ref() {
            if let Some(QueryDispatchRecord::SourceControl {
                source: pending, ..
            }) = state.pending.as_ref().map(QueryDispatchFrame::record)
                && pending == &source
            {
                return QueryInboundSourceControlObservation::Pending;
            }
            return QueryInboundSourceControlObservation::Recorded(
                state.ledger.source_control(&source),
            );
        }
        drop(guard);
        QueryInboundSourceControlObservation::Recorded(
            self.process_local_source_controls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&source)
                .copied(),
        )
    }

    /// The actual runtime checks Directory support and attachment first. Memory-only
    /// source facts remain in this owner; a Home folds only its committed ledger.
    pub(in crate::domain_computation) fn record_source_control(
        &self,
        source: &str,
        posture: InboundSourceControl,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        let source = source_key(source);
        let Some(medium) = self.medium.get().map(std::sync::Arc::clone) else {
            request
                .consult()
                .map_err(QueryDispatchJournalDenial::Execution)?;
            let mut local = self
                .process_local_source_controls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            local
                .entry(source)
                .and_modify(|old| *old = (*old).max(posture))
                .or_insert(posture);
            return Ok(());
        };
        let driver = DispatchJournalDriver::lock(self, request)?;
        driver.settle(medium.as_ref(), request)?;
        if self
            .state()
            .as_ref()
            .expect("admitted dispatch journal")
            .ledger
            .source_control(&source)
            .is_some_and(|old| old >= posture)
        {
            return Ok(());
        }
        match driver.publish(
            medium.as_ref(),
            QueryDispatchRecordIntent::SourceControl { source, posture },
            request,
        )? {
            QueryDispatchRecord::SourceControl { .. } => Ok(()),
            QueryDispatchRecord::Attempt(_) | QueryDispatchRecord::Observation { .. } => {
                unreachable!("a source control intent records only a source control")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::DispatchJournalState;
    use super::*;

    #[test]
    fn pending_source_data_is_distinct_from_a_recorded_security_fact() {
        let journal = WorthQueryDispatchJournal::process_local();
        let ledger =
            QueryDispatchLedger::empty(HomeIdentity::from_observed_bytes([1; 16]).unwrap());
        let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
        let pending = ledger
            .prepare(
                QueryDispatchRecordIntent::SourceControl {
                    source: source_key("source"),
                    posture: InboundSourceControl::Revoked,
                },
                &mut generations,
            )
            .unwrap();
        // DATA-only owner state: no medium, fate witness or Home service is constructed.
        *journal.state() = Some(DispatchJournalState {
            ledger,
            generations,
            pending: Some(pending),
            unrecorded: BTreeMap::new(),
        });
        assert!(matches!(
            journal.source_control("source"),
            QueryInboundSourceControlObservation::Pending
        ));
        assert!(matches!(
            journal.source_control("other-source"),
            QueryInboundSourceControlObservation::Recorded(None)
        ));
        assert!(!journal.is_durable());
    }
}
