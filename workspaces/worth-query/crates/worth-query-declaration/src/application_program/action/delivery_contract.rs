//! One installed delivery contract across owning operation lanes.
use super::ApplicationFixedActionDelivery;
use super::ApplicationOutputActionDeliveryDeclaration;
use crate::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};
use std::{any::TypeId, marker::PhantomData};

/// Declares stable delivery meaning and the concrete implementation installed for an action.
pub trait ApplicationActionDelivery<S: ApplicationSchema, B: ApplicationMutationBinding<S>>:
    Sized + 'static
{
    type Mode: ApplicationActionDeliveryMode<S, B, Self>;
    const IDENTITY: &'static str;
}
mod sealed {
    pub trait Mode {}
}
/// Selects fixed-output settlement with its declared root and projection shape.
pub struct ApplicationFixedDeliveryMode<R, Q>(PhantomData<fn() -> (R, Q)>);
/// Discovered output meaning; runtime construction belongs to its owning adapter.
pub struct ApplicationDiscoveredDeliveryMode<R, Q>(PhantomData<fn() -> (R, Q)>);
/// Declares receipt-only completion for a source with no required output obligation.
pub struct ApplicationSourceCompletion;
/// Declares completion with the authentic observation retained by native commit.
pub struct ApplicationRetainedCommit;
impl<R, Q> sealed::Mode for ApplicationFixedDeliveryMode<R, Q> {}
impl<R, Q> sealed::Mode for ApplicationDiscoveredDeliveryMode<R, Q> {}
impl sealed::Mode for ApplicationSourceCompletion {}
impl sealed::Mode for ApplicationRetainedCommit {}
/// Sealed delivery modes derive installed meaning and the concrete implementation stamp.
pub trait ApplicationActionDeliveryMode<
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    D: 'static,
>: sealed::Mode + 'static
{
    fn declaration() -> ApplicationActionDeliveryDeclaration;
}
impl<S, B, D, R, Q> ApplicationActionDeliveryMode<S, B, D> for ApplicationFixedDeliveryMode<R, Q>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    D: ApplicationFixedActionDelivery<S, B, Root = R, Projection = Q>,
    R: 'static,
    Q: 'static,
{
    fn declaration() -> ApplicationActionDeliveryDeclaration {
        ApplicationActionDeliveryDeclaration::Fixed(
            ApplicationOutputActionDeliveryDeclaration::fixed::<S, B, D>(),
        )
    }
}
impl<S, B, D, R, Q> ApplicationActionDeliveryMode<S, B, D>
    for ApplicationDiscoveredDeliveryMode<R, Q>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    D: super::ApplicationDiscoveredActionDelivery<S, B, Root = R, Projection = Q>,
    R: 'static,
    Q: 'static,
{
    fn declaration() -> ApplicationActionDeliveryDeclaration {
        ApplicationActionDeliveryDeclaration::Discovered(
            ApplicationOutputActionDeliveryDeclaration::discovered::<S, B, D>(),
        )
    }
}
impl<S, B, D> ApplicationActionDeliveryMode<S, B, D> for ApplicationSourceCompletion
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    D: ApplicationActionDelivery<S, B>,
{
    fn declaration() -> ApplicationActionDeliveryDeclaration {
        ApplicationActionDeliveryDeclaration::SourceCompletion {
            identity: D::IDENTITY,
            implementation: TypeId::of::<D>(),
        }
    }
}
impl<S, B, D> ApplicationActionDeliveryMode<S, B, D> for ApplicationRetainedCommit
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    D: ApplicationActionDelivery<S, B>,
{
    fn declaration() -> ApplicationActionDeliveryDeclaration {
        ApplicationActionDeliveryDeclaration::RetainedCommit {
            identity: D::IDENTITY,
            implementation: TypeId::of::<D>(),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
/// Installed delivery meaning; implementation stamps are runtime-only, outside canonical identity.
pub enum ApplicationActionDeliveryDeclaration {
    Fixed(ApplicationOutputActionDeliveryDeclaration),
    Discovered(ApplicationOutputActionDeliveryDeclaration),
    SourceCompletion {
        identity: &'static str,
        implementation: TypeId,
    },
    RetainedCommit {
        identity: &'static str,
        implementation: TypeId,
    },
}
impl ApplicationActionDeliveryDeclaration {
    pub fn fixed<S, B, D>() -> Self
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
        D: ApplicationFixedActionDelivery<S, B>,
    {
        Self::Fixed(ApplicationOutputActionDeliveryDeclaration::fixed::<S, B, D>())
    }
    /// Declares the concrete discovered root and closed projection shape.
    pub fn discovered<S, B, D>() -> Self
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
        D: super::ApplicationDiscoveredActionDelivery<S, B>,
    {
        Self::Discovered(ApplicationOutputActionDeliveryDeclaration::discovered::<
            S,
            B,
            D,
        >())
    }
    pub fn identity(&self) -> &'static str {
        match self {
            Self::Fixed(d) | Self::Discovered(d) => d.identity(),
            Self::SourceCompletion { identity, .. } | Self::RetainedCommit { identity, .. } => {
                identity
            }
        }
    }
    pub fn fixed_output(&self) -> Option<&ApplicationOutputActionDeliveryDeclaration> {
        match self {
            Self::Fixed(d) => Some(d),
            _ => None,
        }
    }
    /// The installed output payload for either native output mode.
    pub fn output(&self) -> Option<&ApplicationOutputActionDeliveryDeclaration> {
        match self {
            Self::Fixed(d) | Self::Discovered(d) => Some(d),
            _ => None,
        }
    }
    /// Only the explicit discovered mode supplies this payload.
    pub fn discovered_output(&self) -> Option<&ApplicationOutputActionDeliveryDeclaration> {
        match self {
            Self::Discovered(d) => Some(d),
            _ => None,
        }
    }
    pub fn canonical_meaning(&self) -> String {
        match self {
            Self::Fixed(d) | Self::Discovered(d) => d.canonical_meaning(),
            Self::SourceCompletion { identity, .. } => {
                super::super::semantic_encoding::framed_parts(&["source-completion", identity])
            }
            Self::RetainedCommit { identity, .. } => {
                super::super::semantic_encoding::framed_parts(&["retained-commit", identity])
            }
        }
    }
}
