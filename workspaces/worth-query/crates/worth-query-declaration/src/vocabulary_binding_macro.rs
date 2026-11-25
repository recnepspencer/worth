//! Identity scalar bindings of closed platform vocabularies: an enum is
//! stored as its variant name, so a foreign or misspelled value never decodes.

/// Declares `Binding`, the identity scalar binding of enum `Value` whose
/// listed variants encode as their names.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_vocabulary_binding {
    ($(#[$meta:meta])* $Binding:ident for $Value:ident = $identity:literal { $($Variant:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $Binding;

        impl $crate::facade::application_schema::ApplicationScalarValueBinding for $Binding {
            type Value = $Value;
            type Unit = ();
            type Decode = $crate::facade::application_schema::ApplicationValueDecodeAvailable;
            type Identity = $crate::facade::application_schema::ApplicationValueIsIdentity;
            type SignedAggregate =
                $crate::facade::application_schema::ApplicationValueSignedAggregateUnavailable;

            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: $crate::facade::typed::ScalarAspectType =
                $crate::facade::typed::ScalarAspectType::String;

            fn validate(
                _: &Self::Value,
            ) -> Result<(), $crate::facade::application_schema::ApplicationValueValidationDenial> {
                Ok(())
            }

            fn encode(
                value: &Self::Value,
            ) -> Result<
                $crate::facade::application_schema::ApplicationValue,
                $crate::facade::application_schema::ApplicationValueEncodeDenial,
            > {
                let name = match value {
                    $($Value::$Variant => stringify!($Variant),)+
                };
                Ok($crate::facade::application_schema::ApplicationValue::String(
                    $crate::facade::application_schema::InternedString::from(name),
                ))
            }
        }

        impl $crate::facade::application_schema::ApplicationReadableScalarValueBinding
            for $Binding
        {
            fn decode(
                value: &$crate::facade::application_schema::ApplicationValue,
            ) -> Result<Self::Value, $crate::facade::application_schema::ApplicationValueDecodeDenial> {
                use $crate::facade::application_schema::{
                    ApplicationScalarValueBinding, ApplicationValueDecodeDenial,
                };
                let Some(name) = value.raw_text() else {
                    return Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                        binding_identity: Self::IDENTITY,
                        expected: Self::SCALAR_FAMILY,
                        observed: value.value_family(),
                    });
                };
                match name {
                    $(stringify!($Variant) => Ok($Value::$Variant),)+
                    _ => Err(ApplicationValueDecodeDenial::CodecRejected {
                        binding_identity: Self::IDENTITY,
                    }),
                }
            }
        }

        impl $crate::facade::application_schema::ApplicationIdentityScalarValueBinding
            for $Binding
        {
        }
    };
}
