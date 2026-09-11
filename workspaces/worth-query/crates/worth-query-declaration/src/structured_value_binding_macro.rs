/// Declares an entry-local binding for a structured application value.
#[macro_export]
macro_rules! worth_query_structured_value_binding {
    (
        $vis:vis $Binding:ident for $Value:ty {
            identity: $identity:expr
            $(, validate: $validate:path)? $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;

        impl $crate::facade::application_schema::ApplicationStructuredValueBinding for $Binding {
            type Value = $Value;
            const IDENTITY_NAME: &'static str = $identity;

            fn validate(
                value: &Self::Value,
            ) -> ::core::result::Result<
                (),
                $crate::facade::application_schema::ApplicationValueValidationDenial,
            > {
                $crate::worth_query_structured_value_binding!(
                    @validate <Self as $crate::facade::application_schema::ApplicationStructuredValueBinding>::IDENTITY.clone(), value $(, $validate)?
                )
            }
        }
    };
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
