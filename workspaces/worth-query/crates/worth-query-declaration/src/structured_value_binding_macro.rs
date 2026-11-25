/// Declares an entry-local binding for a structured application value.
#[macro_export]
macro_rules! worth_query_structured_value_binding {
    (
        $vis:vis $Binding:ident for $Value:ty {
            identity: $identity:expr
            $(, capture_input: $capture:expr)?
            $(, validate: $validate:path)?
            $(, fields: [ $($kind:ident ( $wire:literal , $field:ident : $fty:ty )),* $(,)? ])? $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;

        impl $crate::facade::application_schema::ApplicationStructuredValueBinding for $Binding {
            type Value = $Value;
            const IDENTITY_NAME: &'static str = $identity;
            $(const RETAIN_CANONICAL_INPUT: bool = $capture;)?
            $(
                const DECLARED_INPUT_FIELDS: &'static [$crate::facade::application_schema::ApplicationDeclaredInputMember] = &[
                    $(
                        $crate::worth_query_structured_value_binding!(
                            @field $Value, $kind, $wire, $field, $fty
                        )
                    ),*
                ];
            )?

            fn validate(
                __worth_structured_value: &Self::Value,
            ) -> ::core::result::Result<
                (),
                $crate::facade::application_schema::ApplicationValueValidationDenial,
            > {
                $crate::worth_query_structured_value_binding!(
                    @validate <Self as $crate::facade::application_schema::ApplicationStructuredValueBinding>::IDENTITY.clone(), __worth_structured_value $(, $validate)?
                )
            }
        }
    };
    (@field $Value:ty, required, $wire:literal, $field:ident, $fty:ty) => {
        $crate::facade::application_schema::ApplicationDeclaredInputMember::required_from_binding_macro::<
            $Value,
            $fty,
        >($wire, |__worth_input: &$Value| &raw const __worth_input.$field)
    };
    (@field $Value:ty, optional, $wire:literal, $field:ident, $fty:ty) => {
        $crate::facade::application_schema::ApplicationDeclaredInputMember::optional_from_binding_macro::<
            $Value,
            $fty,
        >($wire, |__worth_input: &$Value| &raw const __worth_input.$field)
    };
    (@validate $identity:expr, $value:ident, $validate:path) => {
        ($validate)($value).map_err(|__worth_structured_reason| {
            $crate::facade::application_schema::ApplicationValueValidationDenial::rejected(
                $identity,
                __worth_structured_reason,
            )
        })
    };
    (@validate $identity:expr, $value:ident) => {{
        let _ = (&$identity, $value);
        ::core::result::Result::Ok(())
    }};
}
