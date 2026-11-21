//! A mixed original's recorded Signal leg, admitted after its own open.
use crate::durability::{DecodedWorldRecord, WorldPlannedSignalLegRecord, WorldRecord};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldPlannedSignalLeg {
    meaning: Arc<DecodedWorldRecord>,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldPlannedSignalLeg {
    pub(super) fn from_validated_journal(
        meaning: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        assert!(matches!(&meaning.record, WorldRecord::PlannedSignalLeg(_)));
        Self {
            meaning,
            position,
            digest,
        }
    }
    pub(crate) fn planned(&self) -> &WorldPlannedSignalLegRecord {
        let WorldRecord::PlannedSignalLeg(planned) = &self.meaning.record else {
            unreachable!("admitted planned Signal leg")
        };
        planned
    }
    pub(crate) fn leg(&self) -> &worth_runtime_bridge::facade::BridgePlannedDefinitionLeg {
        &self.planned().leg
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.meaning
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}
