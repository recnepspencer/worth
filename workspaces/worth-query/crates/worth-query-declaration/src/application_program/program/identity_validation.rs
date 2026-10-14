//! Program identity admission, including bounded canonical revision failure.
use super::{
    ApplicationProgramIdentity, ApplicationProgramRevisionBudgetDenial,
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
};

pub(super) fn require_identity(identity: &str) -> Result<(), ApplicationProgramValidationDenial> {
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(denial(
            ApplicationProgramValidationDenialKind::InvalidIdentity,
            identity,
        ))
    } else {
        Ok(())
    }
}

pub(in crate::application_program) fn deny_canonical_revision_budget(
    identity: &ApplicationProgramIdentity,
    budget: ApplicationProgramRevisionBudgetDenial,
) -> ApplicationProgramValidationDenial {
    denial(
        ApplicationProgramValidationDenialKind::CanonicalRevisionBudgetExceeded,
        format!("{}: {budget}", identity.as_str()),
    )
}

pub(super) fn denial(
    kind: ApplicationProgramValidationDenialKind,
    subject: impl Into<String>,
) -> ApplicationProgramValidationDenial {
    ApplicationProgramValidationDenial {
        kind,
        subject: subject.into(),
    }
}
