#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityComparisonLimits {
    pub(super) input_bytes: u64,
    pub(super) artifacts: u64,
    pub(super) report_bytes: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityComparisonLimitsDenial {
    ZeroInputBytes,
    ZeroArtifacts,
    ZeroReportBytes,
}
impl PhysicalIntegrityComparisonLimits {
    pub const fn new(
        input_bytes: u64,
        artifacts: u64,
        report_bytes: u64,
    ) -> Result<Self, PhysicalIntegrityComparisonLimitsDenial> {
        if input_bytes == 0 {
            return Err(PhysicalIntegrityComparisonLimitsDenial::ZeroInputBytes);
        }
        if artifacts == 0 {
            return Err(PhysicalIntegrityComparisonLimitsDenial::ZeroArtifacts);
        }
        if report_bytes == 0 {
            return Err(PhysicalIntegrityComparisonLimitsDenial::ZeroReportBytes);
        }
        Ok(Self {
            input_bytes,
            artifacts,
            report_bytes,
        })
    }
    pub const fn maximum_input_bytes(self) -> u64 {
        self.input_bytes
    }
    pub const fn maximum_artifacts(self) -> u64 {
        self.artifacts
    }
    pub const fn maximum_report_bytes(self) -> u64 {
        self.report_bytes
    }
}
impl Default for PhysicalIntegrityComparisonLimits {
    fn default() -> Self {
        Self {
            input_bytes: 16 * 1024 * 1024,
            artifacts: 100_000,
            report_bytes: 64 * 1024 * 1024,
        }
    }
}
