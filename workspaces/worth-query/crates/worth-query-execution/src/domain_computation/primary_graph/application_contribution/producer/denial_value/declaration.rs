//! How a provider declares the value its domain denials carry. The declared
//! binding is the sole encoder; nothing here substitutes or drops a value.
use super::{WorthQueryProducerDenialEncodingRefusal, WorthQueryProducerDenialValue};
use crate::domain_computation::primary_graph::application_invariant::refusal::canonical_wire_maximum;
use std::marker::PhantomData;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
};

mod sealed {
    pub trait Sealed<Denial> {}
}

/// A provider's declaration of its operation's domain denial value, in one of
/// three forms:
///
/// - a canonical readable binding whose `Value` is exactly the denial;
/// - [`WorthQueryProjectedProducerDenial`], a projection of the denial onto
///   another bound value, or an explicit absence for a denial without one;
/// - the explicit [`WorthQueryUndeclaredProducerDenial`].
///
/// A declared binding needs a finite canonical wire bound; installation refuses
/// one without it.
pub trait WorthQueryProducerDenialDeclaration<Denial>: sealed::Sealed<Denial> + 'static {
    #[doc(hidden)]
    fn admits_installation() -> bool;
    #[doc(hidden)]
    fn declared_value(denial: &Denial) -> WorthQueryProducerDenialValue;
}

/// An explicit declaration that this producer's denials carry no domain value.
/// Callers decoding one receive `WorthQueryProducerDenialDecodeDenial::Undeclared`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryUndeclaredProducerDenial;

impl<Denial> sealed::Sealed<Denial> for WorthQueryUndeclaredProducerDenial {}
impl<Denial> WorthQueryProducerDenialDeclaration<Denial> for WorthQueryUndeclaredProducerDenial {
    fn admits_installation() -> bool {
        true
    }
    fn declared_value(_: &Denial) -> WorthQueryProducerDenialValue {
        WorthQueryProducerDenialValue::Undeclared
    }
}

impl<Binding: ApplicationReadableScalarValueBinding> sealed::Sealed<Binding::Value> for Binding {}
impl<Binding: ApplicationReadableScalarValueBinding>
    WorthQueryProducerDenialDeclaration<Binding::Value> for Binding
{
    fn admits_installation() -> bool {
        canonical_wire_maximum::<Binding>().is_some()
    }
    fn declared_value(denial: &Binding::Value) -> WorthQueryProducerDenialValue {
        encode_declared::<Binding>(denial)
    }
}

/// Projects an operation's denial onto the value its producer declares.
/// `None` declares that this denial has no such value; callers receive
/// `WorthQueryProducerDenialDecodeDenial::Absent`, never an empty value.
pub trait WorthQueryProducerDenialProjection<Denial>: 'static {
    type Binding: ApplicationReadableScalarValueBinding;
    fn project(denial: &Denial) -> Option<<Self::Binding as ApplicationScalarValueBinding>::Value>;
}

/// Declares the producer's denial value through `Projection`.
pub struct WorthQueryProjectedProducerDenial<Projection>(PhantomData<fn() -> Projection>);

impl<Denial, Projection: WorthQueryProducerDenialProjection<Denial>> sealed::Sealed<Denial>
    for WorthQueryProjectedProducerDenial<Projection>
{
}
impl<Denial, Projection: WorthQueryProducerDenialProjection<Denial>>
    WorthQueryProducerDenialDeclaration<Denial> for WorthQueryProjectedProducerDenial<Projection>
{
    fn admits_installation() -> bool {
        canonical_wire_maximum::<Projection::Binding>().is_some()
    }
    fn declared_value(denial: &Denial) -> WorthQueryProducerDenialValue {
        match Projection::project(denial) {
            Some(value) => encode_declared::<Projection::Binding>(&value),
            None => WorthQueryProducerDenialValue::Absent {
                value_identity:
                    <Projection::Binding as ApplicationScalarValueBinding>::IDENTITY_NAME,
            },
        }
    }
}

fn encode_declared<Binding: ApplicationReadableScalarValueBinding>(
    value: &Binding::Value,
) -> WorthQueryProducerDenialValue {
    use WorthQueryProducerDenialEncodingRefusal as Refusal;
    let refused = |cause| WorthQueryProducerDenialValue::EncodingRefused {
        value_identity: Binding::IDENTITY_NAME,
        cause,
    };
    let Some(maximum) = canonical_wire_maximum::<Binding>() else {
        return refused(Refusal::NoFiniteWireBound);
    };
    let encoded = match Binding::encode(value) {
        Ok(encoded) => encoded,
        Err(denial) => return refused(Refusal::Codec(denial)),
    };
    let AspectValue::String(InternedString::Raw(text)) = encoded else {
        return refused(Refusal::NoncanonicalCarrier);
    };
    if text.len() > maximum {
        return refused(Refusal::ExceedsWireBound {
            maximum,
            observed: text.len(),
        });
    }
    WorthQueryProducerDenialValue::DeclaredValue {
        value_identity: Binding::IDENTITY_NAME,
        bytes: text.into_bytes(),
    }
}
