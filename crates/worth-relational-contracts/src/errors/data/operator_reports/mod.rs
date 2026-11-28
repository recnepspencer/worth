//! Stable, audience-safe report projections of Relational boundary refusals.
use std::{error::Error, fmt};
use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable};

macro_rules! report {
    ($type:ty, $value:ident => $description:expr) => {
        impl $type {
            fn report_description(&self) -> (DenialClass, Disposition, &'static str, &'static str) {
                let $value = self;
                $description
            }
        }

        impl Reportable for $type {
            fn denial_class(&self) -> DenialClass {
                self.report_description().0
            }

            fn disposition(&self) -> Disposition {
                self.report_description().1
            }

            fn report_code(&self) -> ReportCode {
                ReportCode::new(self.report_description().2)
                    .expect("valid static Relational report code")
            }

            fn diagnostic_summary(&self) -> &'static str {
                self.report_description().3
            }
        }
    };
}

macro_rules! display_error {
    ($type:ty, $value:ident => $display:expr; $error:ident => $source:expr) => {
        impl fmt::Display for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let $value = self;
                formatter.write_str($display)
            }
        }

        impl Error for $type {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                let $error = self;
                $source
            }
        }
    };
    ($type:ty) => {
        display_error!($type, value => value.diagnostic_summary(); _value => None);
    };
}

mod boundary;
mod commit;
mod domain;
mod publication_admission;

#[cfg(test)]
mod tests;
