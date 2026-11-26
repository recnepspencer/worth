//! Value refusals carry the application's own `validate` reason verbatim.
//!
//! A `validate = ...` hook on an action or query input refuses with a
//! `&'static str` the application wrote for its end users; the summary borrows
//! it without allocating. Encoding faults and empty reasons keep the generic
//! invalid-input sentence.
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial;
use worth_query_declaration::facade::application_query::ApplicationQueryInputPreparationDenial;
use worth_query_declaration::facade::application_schema::{
    ApplicationValueEncodeDenial, ApplicationValueValidationDenial,
};

use super::request::BAD_INPUT;
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

impl WorthQueryUserFacingRefusal for ApplicationValueValidationDenial {
    fn user_summary(&self) -> Summary {
        let reason = self.reason();
        if reason.trim().is_empty() {
            BAD_INPUT
        } else {
            Summary::with_message(Category::Validation, reason)
        }
    }
}

impl WorthQueryUserFacingRefusal for ApplicationValueEncodeDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Validation(denial) => denial.user_summary(),
            Self::Canonical { .. } | Self::CodecRejected { .. } => BAD_INPUT,
        }
    }
}

impl WorthQueryUserFacingRefusal for ApplicationMutationIdentityDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Key(cause) | Self::Input(cause) => cause.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for ApplicationQueryInputPreparationDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::InputValidation(denial) => denial.user_summary(),
            Self::ParameterEncoding(cause) => cause.user_summary(),
        }
    }
}
