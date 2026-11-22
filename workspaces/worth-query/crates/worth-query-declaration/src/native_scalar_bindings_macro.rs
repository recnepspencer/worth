/// Declares a fragment binding set that registers every native scalar
/// carrier with its stock codec.
///
/// Input: `vis Bindings`. It generates the unit struct, its `BindingSet`
/// (decoding under the caller's `ValueDecodeAdmission`) and one writable
/// `Scalar<T>` per native carrier: `bool`, the signed and unsigned integers,
/// `String`, `InternedString`, `CanonicalDate`, `CanonicalTime`,
/// `CanonicalTimestamp`, `CanonicalTimestampTz`, `[u8; 16]` (UUID),
/// `CanonicalF32`, `CanonicalF64` and `InlineBytes`.
///
/// Records and further scalars stay explicit on the generated type:
/// principal identity and mapping status, content references, entity
/// references, unit-bearing and structured codecs each add their own
/// `Scalar<T>` impl. A fragment that needs a different codec for a native
/// carrier hand-writes its whole binding set instead. With no further
/// impls, use `fragment::NativeScalars`.
///
/// ```
/// use worth_query_declaration::facade::application_schema::{fragment::*, *};
/// use worth_query_declaration::facade::authentication::*;
/// worth_query_declaration::worth_query_native_scalar_bindings!(pub Codecs);
/// impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
///     type Binding = WorthQueryExternalPrincipalIdentityBinding;
///     const WRITABLE: bool = false;
/// }
/// fn registered<T: 'static, B: Scalar<T>>() {}
/// registered::<u64, Codecs>();
/// registered::<String, Codecs>();
/// registered::<worth_foundational::facade::CanonicalDate, Codecs>();
/// registered::<WorthQueryExternalPrincipalIdentity, Codecs>();
/// ```
///
/// The stock set never registers a principal, content or entity-reference
/// codec on its own:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_schema::fragment::*;
/// use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
/// fn registered<T: 'static, B: Scalar<T>>() {}
/// registered::<WorthQueryExternalPrincipalIdentity, NativeScalars>();
/// ```
#[macro_export]
macro_rules! worth_query_native_scalar_bindings {
    ($(#[$meta:meta])* $vis:vis $Bindings:ident) => {
        $(#[$meta])*
        $vis struct $Bindings;
        impl $crate::facade::application_schema::fragment::BindingSet for $Bindings {
            type Work<'a> = $crate::facade::application_schema::ValueDecodeAdmission<'a>;
            fn with_admission<Output>(
                work: &mut Self::Work<'_>,
                use_admission: impl FnOnce(
                    &mut $crate::facade::application_schema::ValueDecodeAdmission<'_>,
                ) -> Output,
            ) -> Output {
                use_admission(work)
            }
        }
        $crate::__worth_query_native_scalars!($Bindings:
            bool => BoolApplicationValueBinding,
            i8 => I8ApplicationValueBinding,
            i16 => I16ApplicationValueBinding,
            i32 => I32ApplicationValueBinding,
            i64 => I64ApplicationValueBinding,
            u8 => U8ApplicationValueBinding,
            u16 => U16ApplicationValueBinding,
            u32 => U32ApplicationValueBinding,
            u64 => U64ApplicationValueBinding,
            ::std::string::String => StringApplicationValueBinding,
            $crate::__native_scalar_carriers::InternedString => InternedStringApplicationValueBinding,
            $crate::__native_scalar_carriers::CanonicalDate => CanonicalDateApplicationValueBinding,
            $crate::__native_scalar_carriers::CanonicalTime => CanonicalTimeApplicationValueBinding,
            $crate::__native_scalar_carriers::CanonicalTimestamp => CanonicalTimestampApplicationValueBinding,
            $crate::__native_scalar_carriers::CanonicalTimestampTz => CanonicalTimestampTzApplicationValueBinding,
            [u8; 16] => UuidApplicationValueBinding,
            $crate::__native_scalar_carriers::CanonicalF32 => CanonicalF32ApplicationValueBinding,
            $crate::__native_scalar_carriers::CanonicalF64 => CanonicalF64ApplicationValueBinding,
            $crate::__native_scalar_carriers::InlineBytes => InlineBytesApplicationValueBinding
        );
    };
}

/// One `Scalar<T>` per native carrier. Carriers are named concretely so
/// coherence keeps further `Scalar<T>` and `Record<T>` impls on the set apart.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_native_scalars {
    ($Bindings:ident: $($Value:ty => $Binding:ident),+) => {
        $(
            impl $crate::facade::application_schema::fragment::Scalar<$Value> for $Bindings {
                type Binding = $crate::facade::application_schema::$Binding;
            }
        )+
    };
}
