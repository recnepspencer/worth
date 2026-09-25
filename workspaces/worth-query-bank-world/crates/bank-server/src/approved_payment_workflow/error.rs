use worth_query_host::facade::admission::authentication_event::WorthQueryAuthenticationEventDenial;
use worth_query_host::facade::application_entry::WorthQueryWorkflowOperationBindingDenial;

use crate::ApprovedBusinessPaymentDefinitionDenial;

#[derive(Debug)]
pub enum BankApprovedPaymentWorkflowError {
    Authentication(WorthQueryAuthenticationEventDenial),
    Definition(ApprovedBusinessPaymentDefinitionDenial),
    OperationBinding(WorthQueryWorkflowOperationBindingDenial),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl BankApprovedPaymentWorkflowError {
    pub fn denial<Denial: std::error::Error + 'static>(&self) -> Option<&Denial> {
        match self {
            Self::Other(denial) => denial.downcast_ref::<Denial>(),
            Self::Authentication(_) => None,
            Self::Definition(_) | Self::OperationBinding(_) => None,
        }
    }

    pub const fn authentication_denial(&self) -> Option<WorthQueryAuthenticationEventDenial> {
        match self {
            Self::Authentication(denial) => Some(*denial),
            Self::Other(_) => None,
            Self::Definition(_) | Self::OperationBinding(_) => None,
        }
    }
}

impl std::fmt::Display for BankApprovedPaymentWorkflowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authentication(denial) => {
                write!(formatter, "workflow authentication denied: {denial:?}")
            }
            Self::Definition(denial) => write!(formatter, "workflow definition denied: {denial:?}"),
            Self::OperationBinding(denial) => {
                write!(formatter, "workflow operation binding denied: {denial:?}")
            }
            Self::Other(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankApprovedPaymentWorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Authentication(_) => None,
            Self::Definition(_) | Self::OperationBinding(_) => None,
            Self::Other(denial) => Some(denial.as_ref()),
        }
    }
}

pub(super) fn other_denial<Denial: std::error::Error + Send + Sync + 'static>(
    denial: Denial,
) -> BankApprovedPaymentWorkflowError {
    BankApprovedPaymentWorkflowError::Other(Box::new(denial))
}
