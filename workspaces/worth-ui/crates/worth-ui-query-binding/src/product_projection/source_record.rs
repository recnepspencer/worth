#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiScalarProjectionSourceRecord {
    status: String,
    revision: u64,
}

impl WorthUiScalarProjectionSourceRecord {
    pub fn new(status: impl Into<String>, revision: u64) -> Result<Self, &'static str> {
        let status = status.into();
        if status.is_empty() {
            return Err("scalar projection status must not be empty");
        }
        if status.len() > 65_536 {
            return Err("scalar projection status exceeds the 65,536-byte product budget");
        }
        Ok(Self { status, revision })
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}
