/// Declares an [`ApplicationNameDiscriminator`](crate::facade::application_naming::ApplicationNameDiscriminator)
/// implementation for a struct or enum composed exclusively of Foundational
/// discriminator values and fixed-size [`WorthQueryName`](crate::facade::application_naming::WorthQueryName)
/// references.
///
/// Attempting to include a float (`f32`/`f64`) or non-admitted type fails at
/// compile time because only sealed [`ApplicationNameDiscriminatorValue`](crate::facade::application_naming::ApplicationNameDiscriminatorValue)
/// types compile against `encoder.field(...)`.
#[macro_export]
macro_rules! worth_query_name_discriminator {
    (
        enum $Type:ty {
            identity: $identity:literal,
            $(referenced_roles: [$($ref_role:literal),* $(,)?],)?
            variants: {
                $(
                    $Variant:ident { $($field:ident),* $(,)? }
                ),* $(,)?
            }
        }
    ) => {
        impl $crate::facade::application_naming::ApplicationNameDiscriminator for $Type {
            const DISCRIMINATOR_IDENTITY: &'static str = $identity;

            $(
                fn referenced_roles() -> &'static [&'static str] {
                    &[$($ref_role),*]
                }
            )?

            fn encode_canonical(
                &self,
                encoder: &mut $crate::facade::application_naming::ApplicationNameDiscriminatorEncoder,
            ) {
                match self {
                    $(
                        Self::$Variant { $($field),* } => {
                            encoder.variant(stringify!($Variant));
                            $(
                                encoder.field(stringify!($field), $field);
                            )*
                        }
                    )*
                }
            }
        }
    };
    (
        struct $Type:ty {
            identity: $identity:literal,
            $(referenced_roles: [$($ref_role:literal),* $(,)?],)?
            fields: [$($field:ident),* $(,)?]
        }
    ) => {
        impl $crate::facade::application_naming::ApplicationNameDiscriminator for $Type {
            const DISCRIMINATOR_IDENTITY: &'static str = $identity;

            $(
                fn referenced_roles() -> &'static [&'static str] {
                    &[$($ref_role),*]
                }
            )?

            fn encode_canonical(
                &self,
                encoder: &mut $crate::facade::application_naming::ApplicationNameDiscriminatorEncoder,
            ) {
                $(
                    encoder.field(stringify!($field), &self.$field);
                )*
            }
        }
    };
}
