use super::super::{BaselineLsmLookupDisposition, BaselineLsmLookupSource};
use super::{
    BaselineLsmManifestPublicationExecution, PreparedLsmCompaction, PublishedLsmCompaction,
};
use worth_store_wal::{
    BlobWalRecordEnvelope, BlobWalRecordIdentity, PublicationDeclaration, PublicationScope,
};

impl PreparedLsmCompaction {
    pub fn input_identities(&self) -> [BlobWalRecordIdentity; 3] {
        self.membership.identities()
    }

    pub const fn output_publication(&self) -> &BlobWalRecordEnvelope {
        self.output.envelope()
    }
}

impl PublishedLsmCompaction {
    pub fn membership_replacement(
        &self,
    ) -> &worth_store_lsm_authority::PublishedLsmMembershipReplacement {
        &self.membership_replacement
    }

    pub const fn value_record(&self) -> &BlobWalRecordEnvelope {
        self.replay_tail.value()
    }

    pub const fn generation_record(&self) -> &BlobWalRecordEnvelope {
        self.replay_tail.generation()
    }

    pub const fn tombstone_record(&self) -> &BlobWalRecordEnvelope {
        self.replay_tail.tombstone()
    }

    pub fn observe_reader_cutover(
        &self,
        recovery: worth_store_physical_isolation::CompactionRecoveryEvidence,
        pre_cutover_read: worth_store_physical_isolation::StablePhysicalReadReceipt,
        post_cutover_read: worth_store_physical_isolation::StablePhysicalReadReceipt,
    ) -> Result<
        worth_store_physical_isolation::ReadDuringCompactionVerdict,
        worth_store_physical_isolation::CompactionReadInterlockDenial,
    > {
        worth_store_physical_isolation::execute_read_during_compaction_cutover(
            self.physical_compaction.clone(),
            recovery,
            pre_cutover_read,
            post_cutover_read,
        )
    }

    pub fn admit_lookup_source(&self) -> BaselineLsmLookupSource {
        BaselineLsmLookupSource::from_published_replacement(&self.membership_replacement)
    }

    pub fn publication_execution(&self) -> BaselineLsmManifestPublicationExecution {
        debug_assert!(matches!(
            self.wal_publication.publication_declaration().scope(),
            PublicationScope::WalFrame(_)
        ));
        debug_assert!(matches!(
            self.manifest_publication.scope(),
            PublicationScope::Manifest(_)
        ));
        BaselineLsmManifestPublicationExecution::from_published(self)
    }

    pub const fn compaction_publication_receipt(
        &self,
    ) -> &super::BaselineLsmCompactionPublicationReceipt {
        &self.compaction
    }

    pub fn lookup_disposition_for(&self, probe_sequence: u64) -> BaselineLsmLookupDisposition {
        if self.memtable_records[0].sequence() == probe_sequence {
            return BaselineLsmLookupDisposition::Memtable;
        }
        if self.sorted_run_records[1].sequence() == probe_sequence {
            return BaselineLsmLookupDisposition::SortedRun;
        }
        BaselineLsmLookupDisposition::NotFound
    }

    pub const fn memtable_records(&self) -> &[BlobWalRecordIdentity; 1] {
        &self.memtable_records
    }

    pub const fn sorted_run_records(&self) -> &[BlobWalRecordIdentity; 2] {
        &self.sorted_run_records
    }

    pub const fn wal_publication(&self) -> &BlobWalRecordEnvelope {
        &self.wal_publication
    }

    pub const fn manifest_publication(&self) -> &PublicationDeclaration {
        &self.manifest_publication
    }

    pub const fn physical_compaction(
        &self,
    ) -> &worth_store_physical_isolation::CompactionRewritePublication {
        &self.physical_compaction
    }
}
