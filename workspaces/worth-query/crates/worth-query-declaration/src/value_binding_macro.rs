/// Declares an entry-local binding for a domain value without implementing a
/// Query trait on that value.
///
/// Use `scalar: String, canonical,` for a derived canonical value, optionally
/// followed by `validate: domain_validator`. A sealed original-text document
/// instead uses `native_envelope: DomainEnvelope` and its pure domain readers.
/// The manual scalar arm takes an encoder and an `Option<Value>` decoder.
/// Domain validators return `Result<(), &'static str>`.
#[macro_export]
macro_rules! worth_query_value_binding {
    (
        $vis:vis $Binding:ident for $Value:ty {
            identity: $identity:literal, native_envelope: $Codec:ty $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;
        impl $crate::facade::application_schema::ApplicationScalarValueBinding for $Binding {
            type Value = $Value;
            type Unit = ();
            type Decode = $crate::facade::application_schema::ApplicationValueDecodeAvailable;
            type Identity = $crate::facade::application_schema::ApplicationValueIsNotIdentity;
            type SignedAggregate = $crate::facade::application_schema::ApplicationValueSignedAggregateUnavailable;
            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: $crate::facade::typed::ScalarAspectType = $crate::facade::typed::ScalarAspectType::String;
            const SUPPORTS_ADMITTED_DECODE: bool = <$Codec as $crate::facade::application_schema::CanonicalValue>::SUPPORTS_ADMISSION;
            fn canonical_contract() -> Option<$crate::facade::application_schema::ApplicationCanonicalValueContract> {
                Some($crate::facade::application_schema::ApplicationCanonicalValueContract::of::<$Codec>(
                    <$Codec as $crate::facade::application_schema::CanonicalDocumentCodec>::BOUNDS))
            }
            fn validate(value: &Self::Value) -> Result<(), $crate::facade::application_schema::ApplicationValueValidationDenial> {
                // The domain owner seals the body/text pair; preserve its wire policy.
                if <$Codec as $crate::facade::application_schema::CanonicalDocumentCodec>::BOUNDS
                    .maximum_wire_bytes().is_some_and(|maximum|
                        <$Codec as $crate::facade::application_schema::CanonicalDocumentCodec>::encoded(value).len() > maximum)
                {
                    return Err($crate::facade::application_schema::ApplicationValueValidationDenial::rejected(
                        Self::IDENTITY, "native record exceeds its wire bound"));
                }
                Ok(())
            }
            fn encode(value: &Self::Value) -> Result<$crate::facade::application_schema::ApplicationValue, $crate::facade::application_schema::ApplicationValueEncodeDenial> {
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(value)?;
                $crate::facade::application_schema::encode_application_document::<$Codec>(value, Self::IDENTITY)
            }
        }
        impl $crate::facade::application_schema::ApplicationReadableScalarValueBinding for $Binding {
            fn decode(raw: &$crate::facade::application_schema::ApplicationValue) -> Result<Self::Value, $crate::facade::application_schema::ApplicationValueDecodeDenial> {
                $crate::facade::application_schema::decode_application_document::<$Codec>(raw, <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY, None)
            }
            fn decode_admitted(raw: &$crate::facade::application_schema::ApplicationValue, admission: &mut $crate::facade::application_schema::ValueDecodeAdmission<'_>) -> Result<Self::Value, $crate::facade::application_schema::ApplicationValueDecodeDenial> {
                $crate::facade::application_schema::decode_application_document::<$Codec>(raw, <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY, Some(admission))
            }
        }
    };
    (
        $vis:vis $Binding:ident for $Value:ty {
            identity: $identity:literal, scalar: String, canonical,
            $(validate: $validate:path $(,)?)?
        }
    ) => {
        $crate::worth_query_value_binding!($vis $Binding for $Value {
            identity: $identity, scalar: String,
            canonical: $crate::facade::application_schema::ValueDecodeBounds::unrestricted(),
            $(validate: $validate,)?
        });
    };
    (
        $vis:vis $Binding:ident for $Value:ty {
            identity: $identity:literal,
            scalar: String,
            canonical: $bounds:expr $(, validate: $validate:path)? $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;
        impl $Binding {
            const CANONICAL_BOUNDS: $crate::facade::application_schema::ValueDecodeBounds = $bounds;
        }
        impl $crate::facade::application_schema::ApplicationScalarValueBinding for $Binding {
            type Value = $Value;
            type Unit = ();
            type Decode = $crate::facade::application_schema::ApplicationValueDecodeAvailable;
            type Identity = $crate::facade::application_schema::ApplicationValueIsNotIdentity;
            type SignedAggregate = $crate::facade::application_schema::ApplicationValueSignedAggregateUnavailable;
            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: $crate::facade::typed::ScalarAspectType = $crate::facade::typed::ScalarAspectType::String;
            const SUPPORTS_ADMITTED_DECODE: bool = <$Value as $crate::facade::application_schema::CanonicalValue>::SUPPORTS_ADMISSION;
            fn canonical_contract() -> Option<$crate::facade::application_schema::ApplicationCanonicalValueContract> {
                Some($crate::facade::application_schema::ApplicationCanonicalValueContract::of::<$Value>(Self::CANONICAL_BOUNDS))
            }
            fn validate(value: &Self::Value) -> Result<(), $crate::facade::application_schema::ApplicationValueValidationDenial> {
                $crate::worth_query_value_binding!(@validate
                    <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY,
                    value $(, $validate)?)
            }
            fn encode(value: &Self::Value) -> Result<$crate::facade::application_schema::ApplicationValue, $crate::facade::application_schema::ApplicationValueEncodeDenial> {
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(value)?;
                $crate::facade::application_schema::encode_canonical_value(value, Self::CANONICAL_BOUNDS)
                    .map(|text| $crate::facade::application_schema::ApplicationValue::String(text.into()))
                    .map_err(|cause| $crate::facade::application_schema::ApplicationValueEncodeDenial::Canonical {
                        binding_identity: <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY,
                        cause,
                    })
            }
        }
        impl $crate::facade::application_schema::ApplicationReadableScalarValueBinding for $Binding {
            fn decode(raw: &$crate::facade::application_schema::ApplicationValue) -> Result<Self::Value, $crate::facade::application_schema::ApplicationValueDecodeDenial> {
                let value = $crate::facade::application_schema::decode_application_canonical_value::<Self::Value>(
                    raw, Self::CANONICAL_BOUNDS, <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY,
                )?;
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(&value)?;
                Ok(value)
            }
            fn decode_admitted(
                raw: &$crate::facade::application_schema::ApplicationValue,
                admission: &mut $crate::facade::application_schema::ValueDecodeAdmission<'_>,
            ) -> Result<Self::Value, $crate::facade::application_schema::ApplicationValueDecodeDenial> {
                let value = $crate::facade::application_schema::decode_application_canonical_value_with_admission::<Self::Value>(
                    raw, Self::CANONICAL_BOUNDS, <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY,
                    admission,
                )?;
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(&value)?;
                Ok(value)
            }
        }
    };
    (
        $vis:vis $Binding:ident for $Value:ty {
            identity: $identity:literal,
            scalar: $scalar:ident,
            $(unit: $unit:literal,)?
            $(frame: $frame:literal,)?
            $(validate: $validate:path,)?
            encode: $encode:path,
            decode: $decode:path $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;

        impl $crate::facade::application_schema::ApplicationScalarValueBinding for $Binding {
            type Value = $Value;
            type Unit = ();
            type Decode = $crate::facade::application_schema::ApplicationValueDecodeAvailable;
            type Identity = $crate::facade::application_schema::ApplicationValueIsNotIdentity;
            type SignedAggregate = $crate::facade::application_schema::ApplicationValueSignedAggregateUnavailable;

            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: $crate::facade::typed::ScalarAspectType =
                $crate::facade::typed::ScalarAspectType::$scalar;
            const UNIT: Option<
                $crate::facade::application_schema::ApplicationUnitIdentity,
            > = $crate::worth_query_value_binding!(@unit $($unit)?);
            const FRAME: Option<
                $crate::facade::application_schema::ApplicationFrameIdentity,
            > = $crate::worth_query_value_binding!(@frame $($frame)?);

            fn validate(
                value: &Self::Value,
            ) -> Result<
                (),
                $crate::facade::application_schema::ApplicationValueValidationDenial,
            > {
                $crate::worth_query_value_binding!(
                    @validate <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY.clone(), value $(, $validate)?
                )
            }

            fn encode(
                value: &Self::Value,
            ) -> Result<
                $crate::facade::application_schema::ApplicationValue,
                $crate::facade::application_schema::ApplicationValueEncodeDenial,
            > {
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(value)?;
                Ok($crate::facade::application_schema::ApplicationValue::$scalar(
                    ($encode)(value),
                ))
            }

        }

        impl $crate::facade::application_schema::ApplicationReadableScalarValueBinding
            for $Binding
        {
            fn decode(
                value: &$crate::facade::application_schema::ApplicationValue,
            ) -> Result<
                Self::Value,
                $crate::facade::application_schema::ApplicationValueDecodeDenial,
            > {
                let observed = value.value_family();
                let $crate::facade::application_schema::ApplicationValue::$scalar(carrier) = value
                else {
                    return Err(
                        $crate::facade::application_schema::ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                            binding_identity: <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY.clone(),
                            expected: <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::SCALAR_FAMILY,
                            observed,
                        },
                    );
                };
                let decoded = ($decode)(carrier.clone()).ok_or_else(|| {
                    $crate::facade::application_schema::ApplicationValueDecodeDenial::CodecRejected {
                        binding_identity: <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY.clone(),
                    }
                })?;
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(&decoded)?;
                Ok(decoded)
            }
        }
    };
    (@unit $unit:literal) => {
        Some($crate::facade::application_schema::ApplicationUnitIdentity::declared($unit))
    };
    (@unit) => { None };
    (@frame $frame:literal) => {
        Some($crate::facade::application_schema::ApplicationFrameIdentity::declared($frame))
    };
    (@frame) => { None };
    (@validate $identity:expr, $value:ident, $validate:path) => {
        ($validate)($value).map_err(|reason| {
            $crate::facade::application_schema::ApplicationValueValidationDenial::rejected(
                $identity,
                reason,
            )
        })
    };
    (@validate $identity:expr, $value:ident) => {{
        let _ = (&$identity, $value);
        Ok(())
    }};
}
