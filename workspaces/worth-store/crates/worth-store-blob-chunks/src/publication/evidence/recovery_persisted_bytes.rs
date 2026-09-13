use std::str;

use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::{BlobPublicationCrashEdge, BlobPublicationObservedSource};

const FORMAT_VERSION: &str = "worth-store.partial-publication.v1";
const BEFORE_WAL_APPEND: &str = "before-wal-append";
const AFTER_WAL_APPEND_BEFORE_DURABILITY: &str = "after-wal-append-before-durability";
const DURING_CHECKPOINT_CUTOVER: &str = "during-checkpoint-cutover";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationPersistedBytes {
    bytes: Vec<u8>,
}

impl BlobPublicationPersistedBytes {
    #[cfg(test)]
    pub(crate) fn from_replay_read_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    #[cfg(test)]
    pub(crate) fn before_wal_append(operation_digest: impl Into<String>) -> Self {
        encode_persisted_record([FORMAT_VERSION, BEFORE_WAL_APPEND, &operation_digest.into()])
    }

    pub fn after_wal_append_before_durability(
        wal_range: WalLsnRange,
        operation_digest: impl Into<String>,
    ) -> Self {
        let start = wal_range.start().get().to_string();
        let end_exclusive = wal_range.end_exclusive().get().to_string();
        let operation_digest = operation_digest.into();
        encode_persisted_record([
            FORMAT_VERSION,
            AFTER_WAL_APPEND_BEFORE_DURABILITY,
            &start,
            &end_exclusive,
            &operation_digest,
        ])
    }

    pub fn during_checkpoint_cutover(checkpoint_digest: impl Into<String>) -> Self {
        let checkpoint_digest = checkpoint_digest.into();
        encode_persisted_record([
            FORMAT_VERSION,
            DURING_CHECKPOINT_CUTOVER,
            &checkpoint_digest,
        ])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn persisted_bytes_digest(&self) -> String {
        persisted_bytes_digest(&self.bytes)
    }

    pub fn observe(&self) -> BlobPublicationObservedSource {
        decode_persisted_bytes(&self.bytes)
    }
}

fn decode_persisted_bytes(bytes: &[u8]) -> BlobPublicationObservedSource {
    let Ok(text) = str::from_utf8(bytes) else {
        return invalid_persisted_bytes(bytes);
    };
    let fields = text.split('\n').collect::<Vec<_>>();
    match fields.as_slice() {
        [FORMAT_VERSION, BEFORE_WAL_APPEND, operation_digest] => {
            BlobPublicationObservedSource::persisted_crash_edge(
                BlobPublicationCrashEdge::before_wal_append(*operation_digest),
            )
        }
        [FORMAT_VERSION, AFTER_WAL_APPEND_BEFORE_DURABILITY, start, end_exclusive, operation_digest] => {
            decode_wal_append_before_durability(start, end_exclusive, operation_digest)
        }
        [FORMAT_VERSION, DURING_CHECKPOINT_CUTOVER, checkpoint_digest] => {
            BlobPublicationObservedSource::persisted_crash_edge(
                BlobPublicationCrashEdge::during_checkpoint_cutover(*checkpoint_digest),
            )
        }
        _ => invalid_persisted_bytes(bytes),
    }
}

fn decode_wal_append_before_durability(
    start: &str,
    end_exclusive: &str,
    operation_digest: &str,
) -> BlobPublicationObservedSource {
    let Ok(start) = start.parse::<u64>() else {
        return invalid_persisted_text(operation_digest);
    };
    let Ok(end_exclusive) = end_exclusive.parse::<u64>() else {
        return invalid_persisted_text(operation_digest);
    };
    let Ok(wal_range) = WalLsnRange::new(
        LogSequenceNumber::new(start),
        LogSequenceNumber::new(end_exclusive),
    ) else {
        return invalid_persisted_text(operation_digest);
    };
    BlobPublicationObservedSource::persisted_crash_edge(
        BlobPublicationCrashEdge::after_wal_append_before_durability(wal_range, operation_digest),
    )
}

fn invalid_persisted_bytes(bytes: &[u8]) -> BlobPublicationObservedSource {
    BlobPublicationObservedSource::insufficient_persisted_evidence(format!(
        "blob-publication:invalid-bytes:{}",
        bytes.len()
    ))
}

fn invalid_persisted_text(text: &str) -> BlobPublicationObservedSource {
    BlobPublicationObservedSource::insufficient_persisted_evidence(format!(
        "blob-publication:invalid-record:{text}"
    ))
}

fn encode_persisted_record<'a>(
    fields: impl IntoIterator<Item = &'a str>,
) -> BlobPublicationPersistedBytes {
    BlobPublicationPersistedBytes {
        bytes: fields
            .into_iter()
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes(),
    }
}

fn persisted_bytes_digest(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push_str(&format!("{byte:02x}"));
    }
    format!(
        "partial-publication-persisted-bytes:v1:len={}:hex={encoded}",
        bytes.len()
    )
}
