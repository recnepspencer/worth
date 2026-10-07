//! Hand-assembled wire bytes for a terminal head retirement projection. This
//! is the layout oracle: it shares no code with the encoder under test.

use super::*;
use crate::ReleaseCustodyHeadNodeWriteV1;

pub(super) struct WireBody {
    pub(super) mutation_tag: u8,
    pub(super) source_root: HeadRef,
    pub(super) expected_prior: HeadEntry,
    pub(super) result_root: Option<HeadRef>,
    pub(super) result_next_block: u64,
    pub(super) path: Vec<ReleaseCustodyHeadPathNodeV1>,
    pub(super) writes: Vec<ReleaseCustodyHeadNodeWriteV1>,
}

fn put_field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}

fn put_ref(target: &mut Vec<u8>, reference: HeadRef) {
    let mut encoded = [0; 104];
    reference.encode_into(&mut encoded);
    target.extend_from_slice(&encoded);
}

fn put_frame(target: &mut Vec<u8>, reference: HeadRef, frame: &[u8]) {
    let mut item = Vec::new();
    put_ref(&mut item, reference);
    put_field(&mut item, frame);
    put_field(target, &item);
}

impl WireBody {
    pub(super) fn of(retirement: &PersistedTerminalReleaseHeadRetirementV1) -> Self {
        Self {
            mutation_tag: 2,
            source_root: retirement.source_root(),
            expected_prior: retirement.expected_prior(),
            result_root: retirement.result_root(),
            result_next_block: retirement.result_next_block(),
            path: retirement.source_path().to_vec(),
            writes: retirement.node_writes().to_vec(),
        }
    }

    fn body(&self) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&TREE.to_le_bytes());
        put_field(&mut body, &basis().encode());
        put_ref(&mut body, self.source_root);
        body.extend_from_slice(&2_u64.to_le_bytes());
        body.push(self.mutation_tag);
        let mut entry = [0; 312];
        self.expected_prior.encode_into(&mut entry);
        body.extend_from_slice(&entry);
        match self.result_root {
            None => body.push(0),
            Some(reference) => {
                body.push(1);
                put_ref(&mut body, reference);
            }
        }
        body.extend_from_slice(&self.result_next_block.to_le_bytes());
        body.extend_from_slice(&(self.path.len() as u64).to_le_bytes());
        for node in &self.path {
            put_frame(&mut body, node.reference(), node.frame());
        }
        body.extend_from_slice(&(self.writes.len() as u64).to_le_bytes());
        for write in &self.writes {
            put_frame(&mut body, write.reference(), write.frame());
        }
        body
    }

    /// The whole record-less projection: no record, frame, placement, segment
    /// update or manifest, then operation tag 9 with the declaration binding.
    pub(super) fn projection_bytes(&self) -> Vec<u8> {
        self.projection_bytes_with_body(|_| {})
    }

    /// The same projection with its retirement body edited before framing.
    pub(super) fn projection_bytes_with_body(&self, edit: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut bytes = Vec::new();
        put_field(&mut bytes, CURRENT_RECOVERY_PROJECTION_DOMAIN);
        bytes.extend_from_slice(&SOURCE_GENERATION.to_le_bytes());
        put_field(&mut bytes, &root_state().encode());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.push(0);
        for _ in 0..4 {
            bytes.extend_from_slice(&0_u64.to_le_bytes());
        }
        let mut binding = vec![9];
        binding.extend_from_slice(&[1; 16]);
        binding.extend_from_slice(&2_u64.to_le_bytes());
        binding.extend_from_slice(&DECLARATION_SHA256);
        put_field(&mut bytes, &binding);
        let mut body = self.body();
        edit(&mut body);
        put_field(&mut bytes, &body);
        bytes
    }
}
