pub const BRIDGE_PRODUCER_EXPORT_SCHEMA_V1: &str = "worth-runtime-bridge.producer-envelope.v1";

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeProducerAuthorityKind {
    RegisteredAuthoritativeSource,
    BridgeHarnessFixture,
    Unknown,
}

impl BridgeProducerAuthorityKind {
    pub fn canonical_label(self) -> &'static str {
        match self {
            Self::RegisteredAuthoritativeSource => "registered-authoritative-source",
            Self::BridgeHarnessFixture => "bridge-harness-fixture",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeProducerMetadata {
    authority_kind: BridgeProducerAuthorityKind,
    export_schema_version: Arc<str>,
    producer_semantics_version: Option<Arc<str>>,
    authoritative_source: Option<BridgeAuthoritativeSourceProvenance>,
    writeback_feedback_context: Option<crate::writeback::BridgeWritebackFeedbackContext>,
}

impl BridgeProducerMetadata {
    /// Describes an envelope returned by the runtime's registered authoritative
    /// source. This is deliberately not a Relational proof: the stronger
    /// owner-specific publication artifact remains in `worth-relational`.
    pub fn registered_authoritative_source() -> Self {
        Self::new(
            BridgeProducerAuthorityKind::RegisteredAuthoritativeSource,
            BRIDGE_PRODUCER_EXPORT_SCHEMA_V1,
        )
    }

    pub fn bridge_harness_fixture() -> Self {
        Self::new(
            BridgeProducerAuthorityKind::BridgeHarnessFixture,
            BRIDGE_PRODUCER_EXPORT_SCHEMA_V1,
        )
    }

    pub fn new(
        authority_kind: BridgeProducerAuthorityKind,
        export_schema_version: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            authority_kind,
            export_schema_version: export_schema_version.into(),
            producer_semantics_version: None,
            authoritative_source: None,
            writeback_feedback_context: None,
        }
    }

    pub fn with_producer_semantics_version(
        mut self,
        producer_semantics_version: impl Into<Arc<str>>,
    ) -> Self {
        self.producer_semantics_version = Some(producer_semantics_version.into());
        self
    }

    pub fn with_authoritative_source(
        mut self,
        source: BridgeAuthoritativeSourceProvenance,
    ) -> Self {
        self.authoritative_source = Some(source);
        self
    }

    pub fn authoritative_source(&self) -> Option<&BridgeAuthoritativeSourceProvenance> {
        self.authoritative_source.as_ref()
    }

    pub fn with_writeback_feedback_context(
        mut self,
        feedback_context: crate::writeback::BridgeWritebackFeedbackContext,
    ) -> Self {
        self.writeback_feedback_context = Some(feedback_context);
        self
    }

    pub fn authority_kind(&self) -> BridgeProducerAuthorityKind {
        self.authority_kind
    }

    pub fn export_schema_version(&self) -> &str {
        self.export_schema_version.as_ref()
    }

    pub fn producer_semantics_version(&self) -> Option<&str> {
        self.producer_semantics_version.as_deref()
    }

    pub fn writeback_feedback_context(
        &self,
    ) -> Option<&crate::writeback::BridgeWritebackFeedbackContext> {
        self.writeback_feedback_context.as_ref()
    }

    pub fn writeback_feedback_provenance_digest(&self) -> Option<&str> {
        self.writeback_feedback_context
            .as_ref()
            .map(|context| context.provenance_digest())
    }

    pub fn writeback_feedback_causality_digest(&self) -> Option<&str> {
        self.writeback_feedback_context
            .as_ref()
            .map(|context| context.causality_digest())
    }
}

impl CheapClone for BridgeProducerMetadata {}

pub type TruthCommitIdentity = BridgeIdentity<TruthCommitTag>;
pub type TruthPatchIdentity = BridgeIdentity<TruthPatchTag>;
pub type TruthBranchIdentity = BridgeIdentity<TruthBranchTag>;
pub type BridgeCommittedPatchDigest = BridgeIdentity<CommittedPatchDigestTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommittedPatchEnvelopeIdentity {
    producer_metadata: BridgeProducerMetadata,
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
}

impl BridgeCommittedPatchEnvelopeIdentity {
    pub fn new(
        commit_identity: TruthCommitIdentity,
        patch_identity: TruthPatchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
        branch_identity: TruthBranchIdentity,
    ) -> Self {
        Self::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            commit_identity,
            patch_identity,
            snapshot_identity,
            branch_identity,
        )
    }

    pub fn new_with_metadata(
        producer_metadata: BridgeProducerMetadata,
        commit_identity: TruthCommitIdentity,
        patch_identity: TruthPatchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
        branch_identity: TruthBranchIdentity,
    ) -> Self {
        Self {
            producer_metadata,
            commit_identity,
            patch_identity,
            snapshot_identity,
            branch_identity,
        }
    }

    pub fn producer_metadata(&self) -> &BridgeProducerMetadata {
        &self.producer_metadata
    }

    pub fn commit_identity(&self) -> &TruthCommitIdentity {
        &self.commit_identity
    }

    pub fn patch_identity(&self) -> &TruthPatchIdentity {
        &self.patch_identity
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BridgePatchEnvelopeHeader {
    producer_metadata: BridgeProducerMetadata,
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
}

impl BridgePatchEnvelopeHeader {
    pub(super) fn new(
        producer_metadata: BridgeProducerMetadata,
        commit_identity: TruthCommitIdentity,
        patch_identity: TruthPatchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
        branch_identity: TruthBranchIdentity,
    ) -> Self {
        Self {
            producer_metadata,
            commit_identity,
            patch_identity,
            snapshot_identity,
            branch_identity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BridgeEnvelopeCore<Body> {
    pub(super) header: BridgePatchEnvelopeHeader,
    pub(super) body: Body,
}

impl<Body> BridgeEnvelopeCore<Body> {
    pub(super) fn new(header: BridgePatchEnvelopeHeader, body: Body) -> Self {
        Self { header, body }
    }

    pub(super) fn producer_metadata(&self) -> &BridgeProducerMetadata {
        &self.header.producer_metadata
    }

    pub(super) fn commit_identity(&self) -> &TruthCommitIdentity {
        &self.header.commit_identity
    }

    pub(super) fn patch_identity(&self) -> &TruthPatchIdentity {
        &self.header.patch_identity
    }

    pub(super) fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.header.snapshot_identity
    }

    pub(super) fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.header.branch_identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CanonicalPatchEnvelopeBody {
    pub(super) patch_summary: BridgeCommittedPatchSummary,
    pub(super) patch_body: BridgeCommittedPatchBody,
    pub(super) digest: BridgeCommittedPatchDigest,
}
