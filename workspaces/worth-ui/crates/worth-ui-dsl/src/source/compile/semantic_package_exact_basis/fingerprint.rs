pub(super) struct Fingerprint(u64);

impl Fingerprint {
    pub(super) fn new(domain: &str) -> Self {
        let mut value = Self(0xcbf2_9ce4_8422_2325);
        value.fold_text(domain);
        value
    }

    pub(super) fn fold_optional_text(&mut self, text: Option<&str>) {
        match text {
            Some(text) => {
                self.fold_u64(1);
                self.fold_text(text);
            }
            None => self.fold_u64(0),
        }
    }

    pub(super) fn fold_optional_bool(&mut self, value: Option<bool>) {
        match value {
            Some(value) => self.fold_u64(if value { 2 } else { 1 }),
            None => self.fold_u64(0),
        }
    }

    pub(super) fn fold_text(&mut self, text: &str) {
        self.fold_usize(text.len());
        for byte in text.as_bytes() {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }

    pub(super) fn fold_texts(&mut self, values: &[String]) {
        self.fold_usize(values.len());
        for value in values {
            self.fold_text(value);
        }
    }

    pub(super) fn fold_usize(&mut self, value: usize) {
        self.fold_u64(value as u64);
    }

    pub(super) fn fold_bool(&mut self, value: bool) {
        self.fold_u64(u64::from(value));
    }

    pub(super) fn fold_bytes(&mut self, values: &[u8]) {
        self.fold_usize(values.len());
        for value in values {
            self.fold_u64(u64::from(*value));
        }
    }

    pub(super) fn fold_u16(&mut self, value: u16) {
        self.fold_u64(u64::from(value));
    }

    pub(super) fn fold_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }

    pub(super) fn finish(self) -> u64 {
        self.0
    }
}
