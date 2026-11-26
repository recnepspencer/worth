//! Handler denials that may carry a structured domain refusal.
use super::WorthQueryDomainRefusal;
use crate::domain_computation::primary_graph::WorthQueryOperationBodyDenial;

/// A mutation binding's `Denial` that can carry a [`WorthQueryDomainRefusal`].
///
/// It lets a session denial expose `domain_refusal()` whatever denial type
/// the binding context declares. The stock body denial carries one in
/// `DomainRefused`; an application denial enum implements this trait,
/// returning its own refusal (the default returns `None`).
pub trait WorthQueryDomainRefusalCarrier {
    fn domain_refusal(&self) -> Option<&WorthQueryDomainRefusal> {
        None
    }
}

impl WorthQueryDomainRefusalCarrier for WorthQueryDomainRefusal {
    fn domain_refusal(&self) -> Option<&WorthQueryDomainRefusal> {
        Some(self)
    }
}

impl WorthQueryDomainRefusalCarrier for WorthQueryOperationBodyDenial {
    fn domain_refusal(&self) -> Option<&WorthQueryDomainRefusal> {
        match self {
            Self::DomainRefused(refusal) => Some(refusal),
            Self::SourceMismatch { .. }
            | Self::InputMismatch { .. }
            | Self::RequirementUnmet { .. }
            | Self::DeclaredRequirementUnmet { .. } => None,
        }
    }
}

impl WorthQueryDomainRefusalCarrier for () {}

impl WorthQueryDomainRefusalCarrier for std::convert::Infallible {}
