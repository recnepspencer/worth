use super::BridgeAsyncSourceDeclarationRejection;
use super::BridgeAsyncSourceDeclarationRejectionKind;
use worth_execution::report::{
    DenialClass::{self, *},
    Disposition::{self, *},
    ReportCode, Reportable,
};

impl std::fmt::Display for BridgeAsyncSourceDeclarationRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Bridge rejected the async source declaration.")
    }
}

impl std::error::Error for BridgeAsyncSourceDeclarationRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.signal_cause
            .as_deref()
            .map(|cause| cause as &dyn std::error::Error)
    }
}

impl Reportable for BridgeAsyncSourceDeclarationRejection {
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
        "Bridge rejected the async source declaration."
    }
}

fn report(
    denial: &BridgeAsyncSourceDeclarationRejection,
) -> (DenialClass, Disposition, &'static str) {
    match denial.kind() {
        BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected => (
            Invariant,
            Escalate,
            "bridge.async_source.signal_declaration_rejected",
        ),
    }
}
