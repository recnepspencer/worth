#[derive(Debug, Clone)]
pub enum WorthQueryConditionalDeliveryDenial {
    Bridge {
        kind: worth_runtime_bridge::facade::BridgeConditionalDenialKind,
        detail: String,
    },
}

impl WorthQueryConditionalDeliveryDenial {
    pub(crate) fn bridge(denial: worth_runtime_bridge::facade::BridgeConditionalDenial) -> Self {
        Self::Bridge {
            kind: denial.kind(),
            detail: denial.detail().to_string(),
        }
    }
}
