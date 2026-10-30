use sha2::{Digest, Sha256};

/// Canonical, typed fields of one construction recipe, in declared order.
#[derive(Default)]
pub struct SetupConstructionInputs {
    encoded: Vec<u8>,
}

impl SetupConstructionInputs {
    pub fn bytes(&mut self, name: &str, value: &[u8]) {
        self.field(0, name, value);
    }

    pub fn unsigned(&mut self, name: &str, value: u64) {
        self.field(1, name, &value.to_le_bytes());
    }

    pub fn boolean(&mut self, name: &str, value: bool) {
        self.field(2, name, &[u8::from(value)]);
    }

    pub fn fingerprint(&self, builder: &str) -> String {
        let mut hash = Sha256::new();
        hash.update(b"WORTH_TEST_COST_SETUP\x01");
        hash.update((builder.len() as u64).to_le_bytes());
        hash.update(builder.as_bytes());
        hash.update(&self.encoded);
        let mut fingerprint = String::with_capacity(64);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in hash.finalize() {
            fingerprint.push(char::from(HEX[usize::from(byte >> 4)]));
            fingerprint.push(char::from(HEX[usize::from(byte & 15)]));
        }
        fingerprint
    }

    fn field(&mut self, kind: u8, name: &str, value: &[u8]) {
        self.encoded.push(kind);
        self.encoded
            .extend_from_slice(&(name.len() as u64).to_le_bytes());
        self.encoded.extend_from_slice(name.as_bytes());
        self.encoded
            .extend_from_slice(&(value.len() as u64).to_le_bytes());
        self.encoded.extend_from_slice(value);
    }
}
