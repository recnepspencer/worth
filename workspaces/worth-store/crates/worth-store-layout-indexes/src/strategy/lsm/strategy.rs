use worth_store_lsm_authority::{
    open_lsm_membership, persist_lsm_membership_record, AdmittedLsmReplaySource, LsmMembershipKey,
    LsmMembershipRecord, LsmMembershipSession, LsmReplaySourceDenial,
};
use worth_store_wal::{
    BlobWalRecordEnvelope, CheckpointArtifactObservation, WalFrameArtifactObservation,
    WalSecurityMetadataCarrier,
};

use super::{
    AdmittedLsmCompactionDemand, BaselineLsmCompactionPlan, BaselineLsmExecutionAdmissionDenial,
    LsmPhysicalCompactionIntent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LsmStrategy;

pub const fn lsm_strategy() -> LsmStrategy {
    LsmStrategy
}

impl LsmStrategy {
    pub fn readmit_lookup_source(
        self,
        family: crate::AdmittedPhysicalArtifactFamily,
        replacement: &worth_store_lsm_authority::PublishedLsmMembershipReplacement,
    ) -> Result<super::BaselineLsmLookupSource, BaselineLsmExecutionAdmissionDenial> {
        let key = replacement.key();
        if key.authority_identity() != family.authority_identity()
            || key.security_identity() != family.security_identity()
        {
            return Err(BaselineLsmExecutionAdmissionDenial::RecordKeyScopeMismatch);
        }
        Ok(super::BaselineLsmLookupSource::from_published_replacement(
            replacement,
        ))
    }

    pub fn open_index(
        self,
        durable_anchor: &WalFrameArtifactObservation,
        current_scope: &worth_store_security::StoreCurrentSecurityScopeWitnessSet,
    ) -> Result<LsmMembershipSession, BaselineLsmExecutionAdmissionDenial> {
        open_lsm_membership(durable_anchor, current_scope)
            .into_result()
            .map_err(super::execution::map_membership_denial)
    }

    pub fn admit_key(
        self,
        metadata: WalSecurityMetadataCarrier,
        admission: super::BaselineLsmCompactionAdmission,
    ) -> Result<LsmMembershipKey, BaselineLsmExecutionAdmissionDenial> {
        let selected = admission.selected();
        let identity = selected.request_identity();
        let family = selected.admitted_family();
        if metadata.authority_identity() != family.authority_identity()
            || metadata.security_identity() != family.security_identity()
        {
            return Err(BaselineLsmExecutionAdmissionDenial::RecordKeyScopeMismatch);
        }
        LsmMembershipKey::admit(metadata, identity.canonical_key())
            .ok_or(BaselineLsmExecutionAdmissionDenial::CanonicalKeyRequired)
    }

    pub fn persist_record(
        self,
        session: &mut LsmMembershipSession,
        envelope: BlobWalRecordEnvelope,
        durable: &WalFrameArtifactObservation,
        key: LsmMembershipKey,
    ) -> Result<LsmMembershipRecord, BaselineLsmExecutionAdmissionDenial> {
        let record = LsmMembershipRecord::admit(envelope, durable, key)
            .ok_or(BaselineLsmExecutionAdmissionDenial::DurableRecordBindingMismatch)?;
        persist_lsm_membership_record(session, record.clone())
            .into_result()
            .map_err(super::execution::map_membership_denial)?;
        Ok(record)
    }

    pub fn lower_compaction(
        self,
        session: &LsmMembershipSession,
        key: LsmMembershipKey,
        admission: super::BaselineLsmCompactionAdmission,
    ) -> Result<BaselineLsmCompactionPlan, BaselineLsmExecutionAdmissionDenial> {
        BaselineLsmCompactionPlan::lower_from_persisted(session, key, admission)
    }

    pub fn admit_compaction_demand(
        self,
        plan: BaselineLsmCompactionPlan,
        output_append: WalFrameArtifactObservation,
        physical_intent: LsmPhysicalCompactionIntent,
    ) -> Result<AdmittedLsmCompactionDemand, BaselineLsmExecutionAdmissionDenial> {
        AdmittedLsmCompactionDemand::admit(plan, output_append, physical_intent)
    }

    pub fn admit_replay_source(
        self,
        plan: &BaselineLsmCompactionPlan,
        checkpoint: Option<&CheckpointArtifactObservation>,
    ) -> Result<AdmittedLsmReplaySource, LsmReplaySourceDenial> {
        AdmittedLsmReplaySource::admit_recovered_membership(plan.replay_membership(), checkpoint)
    }
}
