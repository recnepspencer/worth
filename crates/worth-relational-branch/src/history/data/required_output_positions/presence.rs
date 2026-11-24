//! Durable current route presence, written only by the required-output position owner.
use crate::history::data::CanonicalRequiredOutputOrigin;
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use crate::identity::data::VersionId;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use serde::{Deserialize, Serialize};

pub mod admission;
pub mod history_fallback;
pub use history_fallback::require_current_owned_route;
pub mod current_index;
pub use current_index::CurrentPositions;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Posture {
    Published,
    CodecRetired { retirement: CommitId },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePresence {
    pub meaning: RequiredOutputPublicationMeaning,
    pub publication: CommitId,
    pub generation: VersionId,
    pub posture: Posture,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custody: Option<CommitId>,
    pub row_digest: [u8; 32],
    pub digest: [u8; 32],
}
pub type PresenceRow = (EntityId, RoutePresence);
impl RoutePresence {
    pub fn published(
        origin: &CanonicalRequiredOutputOrigin,
        custody: Option<CommitId>,
        row_digest: [u8; 32],
    ) -> Self {
        let mut value = Self {
            meaning: origin.meaning.clone(),
            publication: origin.commit.commit_id,
            generation: origin.commit.version_id,
            posture: Posture::Published,
            row_digest,
            custody,
            digest: [0; 32],
        };
        value.seal();
        value
    }
    pub fn custody(&self) -> CommitId {
        self.custody.unwrap_or(self.publication)
    }
    pub fn seal(&mut self) {
        self.digest = self.content_digest();
    }
    pub fn valid_digest(&self) -> bool {
        self.digest == self.content_digest()
    }
    fn content_digest(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut writer = RouteDigest(Sha256::new());
        writer.0.update(if self.custody.is_some() {
            b"worth.current-route.v2\0"
        } else {
            b"worth.current-route.v1\0"
        });
        if let Some(custody) = self.custody {
            writer.0.update(custody.0.to_be_bytes());
        }
        serde_json::to_writer(
            &mut writer,
            &(
                &self.meaning,
                self.publication,
                self.generation,
                &self.posture,
                self.row_digest,
            ),
        )
        .expect("the closed scalar position grammar serializes");
        writer.0.finalize().into()
    }
}
struct RouteDigest(sha2::Sha256);
impl std::io::Write for RouteDigest {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        use sha2::Digest;
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
