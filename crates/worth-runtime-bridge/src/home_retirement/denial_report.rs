use super::BridgeHomeRetirementDenial;
use worth_execution::report::{
    DenialClass::{self, *},
    Disposition::{self, *},
    ReportCode, Reportable,
};

impl std::fmt::Display for BridgeHomeRetirementDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Bridge Home retirement was denied.")
    }
}

impl std::error::Error for BridgeHomeRetirementDenial {}

impl Reportable for BridgeHomeRetirementDenial {
    fn denial_class(&self) -> DenialClass {
        report(self).0
    }

    fn disposition(&self) -> Disposition {
        report(self).1
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new(report(self).2).expect("valid static Bridge report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Bridge Home retirement was denied."
    }
}

fn report(denial: &BridgeHomeRetirementDenial) -> (DenialClass, Disposition, &'static str) {
    match denial {
        BridgeHomeRetirementDenial::Busy => (Conflict, Final, "bridge.home_retirement.busy"),
        BridgeHomeRetirementDenial::Unsupported => (
            InvalidInput,
            CorrectInput,
            "bridge.home_retirement.unsupported",
        ),
        BridgeHomeRetirementDenial::ForeignOwner => (
            Authorization,
            Reauthorize,
            "bridge.home_retirement.foreign_owner",
        ),
        BridgeHomeRetirementDenial::NativeAlias(_) => {
            (Unavailable, Escalate, "bridge.home_retirement.native_alias")
        }
        BridgeHomeRetirementDenial::Execution(_) => {
            (Unavailable, Escalate, "bridge.home_retirement.execution")
        }
        BridgeHomeRetirementDenial::Signal(_) => {
            (Unavailable, Escalate, "bridge.home_retirement.signal")
        }
        BridgeHomeRetirementDenial::AuthoredSignal(_) => (
            Unavailable,
            Escalate,
            "bridge.home_retirement.authored_signal",
        ),
    }
}
