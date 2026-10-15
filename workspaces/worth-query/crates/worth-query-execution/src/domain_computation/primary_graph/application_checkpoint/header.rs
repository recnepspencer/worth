//! Intact current-contract refusal before interpreting native checkpoint bytes.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct CheckpointDecodeDenial {
    kind: Kind,
    message: String,
}
impl CheckpointDecodeDenial {
    pub(in crate::domain_computation::primary_graph) fn into_installation_denial(self) -> Denial {
        Denial::new(self.kind, self.message)
    }
}
impl From<String> for CheckpointDecodeDenial {
    fn from(message: String) -> Self {
        Self {
            kind: Kind::CheckpointRecoveryRejected,
            message,
        }
    }
}
impl std::ops::Deref for CheckpointDecodeDenial {
    type Target = str;
    fn deref(&self) -> &str {
        &self.message
    }
}
impl std::fmt::Display for CheckpointDecodeDenial {
    fn fmt(&self, writer: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writer.write_str(&self.message)
    }
}
impl std::error::Error for CheckpointDecodeDenial {}
pub(super) fn admit(bytes: &[u8]) -> Result<(), CheckpointDecodeDenial> {
    let body_start = MAGIC.len() + CHECKSUM_BYTES;
    // Check the old minimum framing before reading the declared version. A
    // well-formed displaced frame is unsupported, not a short current frame.
    if bytes.len() < body_start + 2 || &bytes[..MAGIC.len()] != MAGIC {
        return Err("Query application checkpoint header is invalid"
            .to_owned()
            .into());
    }
    let body = &bytes[body_start..];
    if Sha256::digest(body).as_slice() != &bytes[MAGIC.len()..body_start] {
        return Err("Query application checkpoint checksum differs"
            .to_owned()
            .into());
    }
    let version = u16::from_be_bytes([body[0], body[1]]);
    if version != FORMAT_VERSION {
        return Err(CheckpointDecodeDenial {
            kind: Kind::UnsupportedCheckpointFormat {
                observed: version,
                supported: FORMAT_VERSION,
            },
            message: format!("Query application checkpoint format {version} is unsupported"),
        });
    }
    if bytes.len() < HEADER_BYTES {
        return Err("Query application checkpoint header is invalid"
            .to_owned()
            .into());
    }
    let facts = u16::from_be_bytes([body[2], body[3]]);
    let instances = u16::from_be_bytes([body[4], body[5]]);
    if facts != WORKFLOW_FACTS || instances != WORKFLOW_INSTANCES {
        return Err(CheckpointDecodeDenial {
            kind: Kind::UnsupportedCheckpointWorkflowContract { facts, instances },
            message: "Query application checkpoint workflow contract is unsupported".to_owned(),
        });
    }
    Ok(())
}
