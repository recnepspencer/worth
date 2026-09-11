/// Declares an entry-local binding for a domain value without implementing a
/// Query trait on that value.
///
/// The encoder returns the native carrier for the declared scalar variant. The
/// decoder returns `Option<Value>` so malformed carrier values fail explicitly.
/// An optional validator returns `Result<(), &'static str>`.
#[macro_export]
macro_rules! worth_query_value_binding {
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
