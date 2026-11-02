/// Implementation arm for `worth_query_value_binding!` binary atoms.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_binary_value_binding {
    ($vis:vis $Binding:ident for $Value:ty {
        identity: $identity:literal, binary: $Delegate:ty,
        $(unit: $unit:literal,)? $(frame: $frame:literal,)?
        $(validate: $validate:path $(,)?)?
    }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;
        impl $crate::facade::application_schema::ApplicationScalarValueBinding for $Binding {
            type Value = $Value;
            type Unit = ();
            type Decode = $crate::facade::application_schema::ApplicationValueDecodeAvailable;
            type Identity = $crate::facade::application_schema::ApplicationValueIsNotIdentity;
            type SignedAggregate = $crate::facade::application_schema::ApplicationValueSignedAggregateUnavailable;
            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: $crate::facade::typed::ScalarAspectType = $crate::facade::typed::ScalarAspectType::InlineBytes;
            const DOMAIN_VALUE: bool = true;
            const SUPPORTS_ADMITTED_DECODE: bool = true;
            const UNIT: Option<$crate::facade::application_schema::ApplicationUnitIdentity> = $crate::worth_query_value_binding!(@unit $($unit)?);
            const FRAME: Option<$crate::facade::application_schema::ApplicationFrameIdentity> = $crate::worth_query_value_binding!(@frame $($frame)?);
            fn canonical_contract() -> Option<$crate::facade::application_schema::ApplicationCanonicalValueContract> {
                Some($crate::facade::application_schema::ApplicationCanonicalValueContract::binary::<$Delegate>())
            }
            fn validate(value: &Self::Value) -> Result<(), $crate::facade::application_schema::ApplicationValueValidationDenial> {
                $crate::worth_query_value_binding!(@validate Self::IDENTITY, value $(, $validate)?)
            }
            fn encode(value: &Self::Value) -> Result<$crate::facade::application_schema::ApplicationValue, $crate::facade::application_schema::ApplicationValueEncodeDenial> {
                Self::validate(value)?;
                $crate::facade::application_schema::encode_application_binary_value::<$Delegate>(value, Self::IDENTITY)
            }
        }
        impl $crate::facade::application_schema::ApplicationReadableScalarValueBinding for $Binding {
            fn decode(raw: &$crate::facade::application_schema::ApplicationValue) -> Result<Self::Value, $crate::facade::application_schema::ApplicationValueDecodeDenial> {
                let value = $crate::facade::application_schema::decode_application_binary_value::<$Delegate>(raw,
                    <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY, None)?;
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(&value)?;
                Ok(value)
            }
            fn decode_admitted(raw: &$crate::facade::application_schema::ApplicationValue, admission: &mut $crate::facade::application_schema::ValueDecodeAdmission<'_>) -> Result<Self::Value, $crate::facade::application_schema::ApplicationValueDecodeDenial> {
                let value = $crate::facade::application_schema::decode_application_binary_value::<$Delegate>(raw,
                    <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::IDENTITY, Some(admission))?;
                <Self as $crate::facade::application_schema::ApplicationScalarValueBinding>::validate(&value)?;
                Ok(value)
            }
        }
    };
}
