use sha2::{Digest, Sha256};

use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::symbols::data::ClientKey;
use worth_foundational::facade::AspectFieldLocator;

pub(super) struct StrategyDigestBytes {
    bytes: Sha256,
}

impl StrategyDigestBytes {
    pub(super) fn digest(domain: &'static str, fill: impl FnOnce(&mut Self)) -> [u8; 32] {
        let mut bytes = Self::new(domain);
        fill(&mut bytes);
        bytes.bytes.finalize().into()
    }

    pub(super) fn hex_digest(domain: &'static str, fill: impl FnOnce(&mut Self)) -> String {
        let digest = Self::digest(domain, fill);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut text = String::with_capacity(64);
        for byte in digest {
            text.push(HEX[(byte >> 4) as usize] as char);
            text.push(HEX[(byte & 15) as usize] as char);
        }
        text
    }

    fn new(domain: &'static str) -> Self {
        let mut bytes = Self {
            bytes: Sha256::new(),
        };
        bytes.string(domain);
        bytes
    }

    pub(super) fn tag(&mut self, tag: u8) {
        self.bytes.update([tag]);
    }

    pub(super) fn bool(&mut self, value: bool) {
        self.tag(u8::from(value));
    }

    pub(super) fn u16(&mut self, value: u16) {
        self.bytes.update(value.to_le_bytes());
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.bytes.update(value.to_le_bytes());
    }

    pub(super) fn u64(&mut self, value: u64) {
        self.bytes.update(value.to_le_bytes());
    }

    pub(super) fn usize(&mut self, value: usize) {
        self.u64(value as u64);
    }

    pub(super) fn string(&mut self, value: &str) {
        self.u32(value.len() as u32);
        self.bytes.update(value.as_bytes());
    }

    pub(super) fn bytes(&mut self, value: &[u8]) {
        self.u32(value.len() as u32);
        self.bytes.update(value);
    }

    pub(super) fn digest_bytes(&mut self, value: &[u8; 32]) {
        self.bytes.update(value);
    }

    pub(super) fn optional<T>(&mut self, value: Option<T>, write: impl FnOnce(&mut Self, T)) {
        match value {
            Some(value) => {
                self.tag(1);
                write(self, value);
            }
            None => self.tag(0),
        }
    }

    pub(super) fn partition_id(&mut self, value: PartitionId) {
        self.u32(value.as_u32());
    }

    pub(super) fn kind_id(&mut self, value: KindId) {
        self.u32(value.as_u32());
    }

    pub(super) fn version_id(&mut self, value: VersionId) {
        self.u64(value.as_u64());
    }

    pub(super) fn entity_id(&mut self, value: EntityId) {
        self.partition_id(value.partition_id);
        self.u64(value.local_slot_value());
        self.u32(value.generation_value());
    }

    pub(super) fn relation_id(&mut self, value: RelationId) {
        self.partition_id(value.partition_id);
        self.u64(value.local_slot_value());
        self.u32(value.generation_value());
    }

    pub(super) fn client_key(&mut self, value: &ClientKey) {
        if let Some(raw) = value.as_raw_str() {
            self.tag(1);
            self.string(raw);
        } else if let Some(symbol) = value.as_symbol() {
            self.tag(2);
            self.u32(symbol.0);
        } else {
            self.tag(3);
            self.string(value.canonical_text().as_ref());
        }
    }

    pub(super) fn aspect_field_locator(&mut self, locator: &AspectFieldLocator) {
        self.bytes(&crate::aspect_wire::encode_aspect_field_locator(locator));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_decode_streamed_strategy_digest_preserves_original_primitive_bytes() {
        let mut previous = Vec::new();
        previous.extend_from_slice(&6_u32.to_le_bytes());
        previous.extend_from_slice(b"domain");
        previous.push(7);
        previous.push(1);
        previous.extend_from_slice(&19_u16.to_le_bytes());
        previous.extend_from_slice(&23_u32.to_le_bytes());
        previous.extend_from_slice(&29_u64.to_le_bytes());
        previous.extend_from_slice(&3_u32.to_le_bytes());
        previous.extend_from_slice(b"abc");
        previous.extend_from_slice(&2_u32.to_le_bytes());
        previous.extend_from_slice(&[5, 8]);
        previous.extend_from_slice(&[31; 32]);
        let digest = StrategyDigestBytes::digest("domain", |bytes| {
            bytes.tag(7);
            bytes.bool(true);
            bytes.u16(19);
            bytes.u32(23);
            bytes.u64(29);
            bytes.string("abc");
            bytes.bytes(&[5, 8]);
            bytes.digest_bytes(&[31; 32]);
        });
        assert_eq!(digest, <[u8; 32]>::from(Sha256::digest(previous)));
    }
}
