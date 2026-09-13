use worth_proof::{
    Artifact, AssumptionBasis, AuthorityMarker, AuthorityWitness, CurrentValidity,
    FreshnessScopedBasis, NoProofs, PhaseMarker,
};
use worth_store_authority::StoreCurrentAuthorityWitness;

use crate::publication::BlobPublicationCrashBoundaryReport;

use super::{BlobReplayAdmissionDenial, BlobReplayAdmissionDenialKind};
#[cfg(any(test, feature = "certification-test-authority"))]
use super::{BlobReplaySourceIdentity, BlobReplaySourceIdentityKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReplaySourceKind {
    Wal,
    Checkpoint,
    Manifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobReplaySourceAdmission {
    kind: BlobReplaySourceKind,
    source_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobResumeReplayReadmission {
    readmitted_checkpoint_source: ReadmittedCheckpointSourceArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlobResumeReplayReadmissionPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlobResumeReplayReadmissionAuthority;

type ReadmittedCheckpointSourceArtifact = Artifact<
    BlobResumeReplayReadmissionPhase,
    String,
    NoProofs,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<String>>,
>;

impl PhaseMarker for BlobResumeReplayReadmissionPhase {}

impl AuthorityMarker for BlobResumeReplayReadmissionAuthority {}

impl BlobReplaySourceAdmission {
    pub fn from_replayable_wal_report(
        report: &BlobPublicationCrashBoundaryReport,
    ) -> Result<Self, BlobReplayAdmissionDenial> {
        if !report.replayable() {
            return Err(BlobReplayAdmissionDenial::new(
                BlobReplayAdmissionDenialKind::MissingWalSource,
                Some(report.classification_digest().to_owned()),
            ));
        }
        if report.replayable_durable_wal().is_none() {
            return Err(BlobReplayAdmissionDenial::new(
                BlobReplayAdmissionDenialKind::MissingWalSource,
                Some(report.classification_digest().to_owned()),
            ));
        }
        admit_replay_source(
            BlobReplaySourceKind::Wal,
            report.classification_digest(),
            BlobReplayAdmissionDenialKind::MissingWalSource,
        )
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn from_checkpoint_replay_identity(
        identity: &BlobReplaySourceIdentity,
    ) -> Result<Self, BlobReplayAdmissionDenial> {
        admit_checkpoint_replay_source(identity)
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn from_manifest_replay_identity(
        identity: &BlobReplaySourceIdentity,
    ) -> Result<Self, BlobReplayAdmissionDenial> {
        admit_manifest_replay_source(identity)
    }

    pub fn reject_backend_residue(digest: impl Into<String>) -> BlobReplayAdmissionDenial {
        BlobReplayAdmissionDenial::new(
            BlobReplayAdmissionDenialKind::BackendResidueRejected,
            Some(digest.into()),
        )
    }

    pub const fn kind(&self) -> BlobReplaySourceKind {
        self.kind
    }

    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }
}

impl BlobResumeReplayReadmission {
    pub fn from_checkpoint_source(
        source: &BlobReplaySourceAdmission,
        current_store_authority: StoreCurrentAuthorityWitness,
    ) -> Result<Self, BlobReplayAdmissionDenial> {
        let current_store_authority_digest = current_store_authority
            .identity()
            .aspect_key()
            .as_str()
            .to_owned();
        if source.kind() != BlobReplaySourceKind::Checkpoint {
            return Err(BlobReplayAdmissionDenial::new(
                BlobReplayAdmissionDenialKind::WrongReplaySourceForResumeSession,
                Some(source.source_digest().to_owned()),
            ));
        }
        if current_store_authority
            .identity()
            .aspect_key()
            .as_str()
            .is_empty()
        {
            return Err(BlobReplayAdmissionDenial::new(
                BlobReplayAdmissionDenialKind::MissingStoreAuthorityReadmission,
                None,
            ));
        }
        let authority =
            AuthorityWitness::from_authority_marker(BlobResumeReplayReadmissionAuthority);
        let readmitted_checkpoint_source =
            Artifact::<BlobResumeReplayReadmissionPhase, _, _, _>::with_current_basis(
                source.source_digest().to_owned(),
                source.source_digest().to_owned(),
                authority,
            )
            .bridge_trust_boundary()
            .readmit_with_authority(
                current_store_authority_digest,
                AuthorityWitness::from_authority_marker(BlobResumeReplayReadmissionAuthority),
            );
        Ok(Self {
            readmitted_checkpoint_source,
        })
    }

    pub fn checkpoint_source_digest(&self) -> &str {
        self.readmitted_checkpoint_source.payload().as_str()
    }

    pub fn current_store_authority_digest(&self) -> &str {
        self.readmitted_checkpoint_source
            .basis()
            .basis()
            .value()
            .as_str()
    }
}

#[cfg(any(test, feature = "certification-test-authority"))]
fn admit_checkpoint_replay_source(
    identity: &BlobReplaySourceIdentity,
) -> Result<BlobReplaySourceAdmission, BlobReplayAdmissionDenial> {
    admit_durability_replay_source(
        identity,
        BlobReplaySourceIdentityKind::Checkpoint,
        BlobReplaySourceKind::Checkpoint,
        BlobReplayAdmissionDenialKind::MissingCheckpointSource,
    )
}

#[cfg(any(test, feature = "certification-test-authority"))]
fn admit_manifest_replay_source(
    identity: &BlobReplaySourceIdentity,
) -> Result<BlobReplaySourceAdmission, BlobReplayAdmissionDenial> {
    admit_durability_replay_source(
        identity,
        BlobReplaySourceIdentityKind::Manifest,
        BlobReplaySourceKind::Manifest,
        BlobReplayAdmissionDenialKind::MissingManifestSource,
    )
}

#[cfg(any(test, feature = "certification-test-authority"))]
fn admit_durability_replay_source(
    identity: &BlobReplaySourceIdentity,
    expected_replay_kind: BlobReplaySourceIdentityKind,
    blob_source_kind: BlobReplaySourceKind,
    denial: BlobReplayAdmissionDenialKind,
) -> Result<BlobReplaySourceAdmission, BlobReplayAdmissionDenial> {
    if identity.kind() != expected_replay_kind || identity.digest().is_empty() {
        return Err(BlobReplayAdmissionDenial::new(
            denial,
            Some(identity.digest().to_owned()),
        ));
    }
    admit_replay_source(blob_source_kind, identity.digest(), denial)
}

fn admit_replay_source(
    kind: BlobReplaySourceKind,
    source_digest: &str,
    denial: BlobReplayAdmissionDenialKind,
) -> Result<BlobReplaySourceAdmission, BlobReplayAdmissionDenial> {
    if source_digest.is_empty() {
        return Err(BlobReplayAdmissionDenial::new(denial, None));
    }
    Ok(BlobReplaySourceAdmission {
        kind,
        source_digest: source_digest.to_owned(),
    })
}
