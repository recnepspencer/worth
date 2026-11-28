//! Shared projection shape for static, audience-safe World denial descriptions.

macro_rules! world_denial_report {
    ($ty:ty, $domain:literal, {
        $($pattern:pat => ($code:literal, $class:ident, $disposition:expr, $summary:literal)),+ $(,)?
    }) => {
        impl $ty {
            fn report_description(&self) -> (
                worth_execution::report::ReportCode,
                worth_execution::report::DenialClass,
                worth_execution::report::Disposition,
                &'static str,
            ) {
                match self {
                    $($pattern => (
                        const {
                            match worth_execution::report::ReportCode::new(
                                concat!("world.", $domain, ".", $code)
                            ) {
                                Ok(code) => code,
                                Err(_) => panic!("invalid World report code"),
                            }
                        },
                        worth_execution::report::DenialClass::$class,
                        $disposition,
                        $summary,
                    )),+
                }
            }
        }

        impl std::fmt::Display for $ty {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.report_description().3)
            }
        }

        impl worth_execution::report::Reportable for $ty {
            fn denial_class(&self) -> worth_execution::report::DenialClass {
                self.report_description().1
            }

            fn disposition(&self) -> worth_execution::report::Disposition {
                self.report_description().2
            }

            fn report_code(&self) -> worth_execution::report::ReportCode {
                self.report_description().0
            }

            fn diagnostic_summary(&self) -> &'static str {
                self.report_description().3
            }
        }
    };
}
pub(crate) use world_denial_report;
