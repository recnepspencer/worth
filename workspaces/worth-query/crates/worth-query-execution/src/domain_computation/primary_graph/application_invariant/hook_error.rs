//! Application causes stay values until the installed rule applies its codec.
use super::WorthQueryInvariantAccessDenial;

#[derive(Debug)]
pub enum WorthQueryApplicationInvariantError<Cause = ()> {
    Cause(Cause),
    Access(WorthQueryInvariantAccessDenial),
    Stopped,
    ManagedIndex(Box<super::super::application_query::WorthQueryManagedIndexInvariantDenial>),
    Interrupted(super::CustomInvariantInterruption),
    Allocation,
    MissingRequiredField { field: &'static str },
    CheckedHookRequired,
}

impl<Cause> From<WorthQueryInvariantAccessDenial> for WorthQueryApplicationInvariantError<Cause> {
    fn from(cause: WorthQueryInvariantAccessDenial) -> Self {
        Self::Access(cause)
    }
}
impl<Cause> From<std::collections::TryReserveError> for WorthQueryApplicationInvariantError<Cause> {
    fn from(_: std::collections::TryReserveError) -> Self {
        Self::Allocation
    }
}

pub(super) use worth_relational::facade::runtime::CustomInvariantFailureCause as SystemInvariantError;
impl<Cause> WorthQueryApplicationInvariantError<Cause> {
    pub(super) fn into_cause(self) -> Result<Cause, SystemInvariantError> {
        match self {
            Self::Access(cause) => Err(SystemInvariantError::Access(cause)),
            Self::Stopped => Err(SystemInvariantError::Stopped),
            Self::Interrupted(cause) => Err(SystemInvariantError::Interrupted(cause)),
            Self::ManagedIndex(cause) => Err(managed_index_cause(*cause)),
            Self::Allocation => Err(SystemInvariantError::Allocation),
            Self::MissingRequiredField { field } => Err(required_field_cause(field)),
            Self::CheckedHookRequired => Err(SystemInvariantError::CheckedHookRequired),
            Self::Cause(cause) => Ok(cause),
        }
    }
}

impl<Cause> From<super::CustomInvariantInterruption>
    for WorthQueryApplicationInvariantError<Cause>
{
    fn from(cause: super::CustomInvariantInterruption) -> Self {
        Self::Interrupted(cause)
    }
}
impl<Cause> From<super::super::application_query::WorthQueryManagedIndexInvariantDenial>
    for WorthQueryApplicationInvariantError<Cause>
{
    fn from(cause: super::super::application_query::WorthQueryManagedIndexInvariantDenial) -> Self {
        Self::ManagedIndex(Box::new(cause))
    }
}

impl<Cause> From<worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial>
    for WorthQueryApplicationInvariantError<Cause>
{
    fn from(cause: worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial) -> Self {
        Self::from(
            super::super::application_query::WorthQueryManagedIndexInvariantDenial::from(cause),
        )
    }
}

fn managed_index_cause(
    cause: super::super::application_query::WorthQueryManagedIndexInvariantDenial,
) -> SystemInvariantError {
    SystemInvariantError::ManagedIndex {
        detail: cause.to_string(),
    }
}
fn required_field_cause(field: &'static str) -> SystemInvariantError {
    SystemInvariantError::MissingRequiredField {
        field: field.to_owned(),
    }
}
