use sha2::{Digest, Sha256};

use crate::merge::data::{
    MergeIntent, NormalizedRelationalMergeRequest, RelationalMergeCorrespondencePosture,
    RelationalMergeRequestFamily, RelationalMergeSchemaReconciliationPosture, RelationalMergeScope,
    RelationalMergeTopologyIntent,
};

pub(crate) fn normalized_merge_request_digest(
    request: &NormalizedRelationalMergeRequest,
) -> String {
    let mut bytes = DigestBytes::new("worth.relational.merge.normalized-request.v1");
    bytes.normalized_request(request);
    bytes.finish()
}

struct DigestBytes {
    bytes: Sha256,
}

impl DigestBytes {
    fn new(domain: &'static str) -> Self {
        let mut bytes = Self {
            bytes: Sha256::new(),
        };
        bytes.str(domain);
        bytes
    }

    fn finish(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let digest = self.bytes.finalize();
        let mut text = String::with_capacity(64);
        for byte in digest {
            text.push(char::from(HEX[usize::from(byte >> 4)]));
            text.push(char::from(HEX[usize::from(byte & 15)]));
        }
        text
    }

    fn tag(&mut self, tag: u8) {
        self.bytes.update([tag]);
    }

    fn usize(&mut self, value: usize) {
        self.bytes.update((value as u64).to_be_bytes());
    }

    fn str(&mut self, value: &str) {
        self.usize(value.len());
        self.bytes.update(value.as_bytes());
    }

    fn normalized_request(&mut self, request: &NormalizedRelationalMergeRequest) {
        self.request_family(request.family());
        self.str(&request.target_branch().0);
        self.str(&request.source_branch().0);
        self.merge_intent(request.merge_intent());
        self.correspondence_posture(request.correspondence_posture());
        self.schema_posture(request.schema_reconciliation_posture());
        self.topology_intent(request.topology_intent());
    }

    fn request_family(&mut self, family: RelationalMergeRequestFamily) {
        match family {
            RelationalMergeRequestFamily::FullBranchReconciliation => self.tag(1),
        }
    }

    fn merge_intent(&mut self, intent: MergeIntent) {
        match intent {
            MergeIntent::ReconcileIntoTarget => self.tag(1),
        }
    }

    fn correspondence_posture(&mut self, posture: RelationalMergeCorrespondencePosture) {
        match posture {
            RelationalMergeCorrespondencePosture::Advisory => self.tag(1),
            RelationalMergeCorrespondencePosture::Strict => self.tag(2),
        }
    }

    fn schema_posture(&mut self, posture: RelationalMergeSchemaReconciliationPosture) {
        match posture {
            RelationalMergeSchemaReconciliationPosture::Participate => self.tag(1),
            RelationalMergeSchemaReconciliationPosture::RequireCompatibility => self.tag(2),
        }
    }

    fn topology_intent(&mut self, intent: RelationalMergeTopologyIntent) {
        match intent {
            RelationalMergeTopologyIntent::PreserveTopologySemantics => self.tag(1),
            RelationalMergeTopologyIntent::RequireStrictTopologyStability => self.tag(2),
        }
    }

    #[allow(dead_code)]
    fn scope(&mut self, scope: RelationalMergeScope) {
        match scope {
            RelationalMergeScope::FullBranch => self.tag(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_decode_streamed_merge_request_digest_preserves_prior_primitive_bytes() {
        let mut actual = DigestBytes::new("worth.relational.merge.normalized-request.v1");
        actual.tag(1);
        actual.str("target-\u{03bb}");
        actual.str("source");
        for tag in [1, 2, 1, 2] {
            actual.tag(tag);
        }
        let mut old = Vec::new();
        let domain = "worth.relational.merge.normalized-request.v1";
        old.extend_from_slice(&(domain.len() as u64).to_be_bytes());
        old.extend_from_slice(domain.as_bytes());
        old.push(1);
        for value in ["target-\u{03bb}", "source"] {
            old.extend_from_slice(&(value.len() as u64).to_be_bytes());
            old.extend_from_slice(value.as_bytes());
        }
        old.extend_from_slice(&[1, 2, 1, 2]);
        let expected = Sha256::digest(old)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(actual.finish(), expected);
    }
}
