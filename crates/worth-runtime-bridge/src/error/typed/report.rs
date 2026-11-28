use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable};

use super::{BridgeErrorKind, BridgeTypedError};

impl<K: BridgeErrorKind + Send + Sync + 'static> Reportable for BridgeTypedError<K> {
    fn denial_class(&self) -> DenialClass {
        self.kind.report().0
    }

    fn disposition(&self) -> Disposition {
        self.kind.report().1
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new(self.kind.report().2).expect("valid static Bridge report code")
    }

    fn forensic_source_detail(&self, source: &(dyn std::error::Error + 'static)) -> Option<String> {
        source
            .downcast_ref::<crate::error::BridgeExecutionDenial>()
            .map(|denial| denial.label().to_owned())
    }

    fn diagnostic_summary(&self) -> &'static str {
        self.kind.summary()
    }
}
