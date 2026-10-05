use crate::WorthServerProductSupportPosture;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationSupportSnapshot {
    support_row: String,
    posture: WorthServerProductSupportPosture,
}

impl WorthServerProductOperationSupportSnapshot {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        super::execution_pipeline::read_batch_accounting::string(&self.support_row)
    }

    pub fn production_admitted(support_row: impl Into<String>) -> Self {
        Self {
            support_row: support_row.into(),
            posture: WorthServerProductSupportPosture::ProductionAdmitted,
        }
    }

    pub fn unsupported(support_row: impl Into<String>) -> Self {
        Self {
            support_row: support_row.into(),
            posture: WorthServerProductSupportPosture::Unsupported,
        }
    }

    pub fn unknown(support_row: impl Into<String>) -> Self {
        Self {
            support_row: support_row.into(),
            posture: WorthServerProductSupportPosture::Unknown,
        }
    }

    pub fn incompatible_basis(support_row: impl Into<String>) -> Self {
        Self {
            support_row: support_row.into(),
            posture: WorthServerProductSupportPosture::IncompatibleBasis,
        }
    }

    pub fn support_row(&self) -> &str {
        &self.support_row
    }

    pub(crate) fn posture(&self) -> WorthServerProductSupportPosture {
        self.posture.clone()
    }

    pub(super) fn canonical_label(&self) -> &'static str {
        match self.posture {
            WorthServerProductSupportPosture::ProductionAdmitted => "production-admitted",
            WorthServerProductSupportPosture::Unsupported => "unsupported",
            WorthServerProductSupportPosture::Unknown => "unknown",
            WorthServerProductSupportPosture::IncompatibleBasis => "incompatible-basis",
        }
    }
}
