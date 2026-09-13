/// Persisted family format version and, when present, its enclosing schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalIntegrityFormatVersion {
    format_version: u16,
    envelope_schema: Option<u16>,
}

impl PhysicalIntegrityFormatVersion {
    pub const fn new(format_version: u16, envelope_schema: Option<u16>) -> Self {
        Self {
            format_version,
            envelope_schema,
        }
    }

    pub const fn format_version(self) -> u16 {
        self.format_version
    }

    pub const fn envelope_schema(self) -> Option<u16> {
        self.envelope_schema
    }
}
