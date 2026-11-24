/// Declares a fragment binding set that registers every native scalar
/// carrier with its stock codec.
///
/// Input: `vis Bindings`. It generates the unit struct, its `BindingSet`
/// (decoding under the caller's `ValueDecodeAdmission`) and one writable
/// `Scalar<T>` per native carrier: `bool`, the signed and unsigned integers,
/// `String`, `InternedString`, `CanonicalDate`, `CanonicalTime`,
/// `CanonicalTimestamp`, `CanonicalTimestampTz`, `[u8; 16]` (UUID),
/// `CanonicalF32`, `CanonicalF64` and `InlineBytes`. It also stores a held
/// persistent name (`WorthQueryName<K>`) and a held target (`NamedTarget<K>`)
/// of every `WorthQueryNamedKind`, each as one string column.
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
/// A record column holds a persistent name or target directly:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     EntityNameSubject, NamedTarget, Unowned, WorthQueryName, WorthQueryNamedKind,
/// };
/// use worth_query_declaration::facade::application_schema::fragment::*;
/// struct Edge;
/// impl Entity for Edge {
///     const NAME: &'static str = "Edge";
///     type Aspects = ();
///     type Relations = ();
/// }
/// impl WorthQueryNamedKind for Edge {
///     type Subject = EntityNameSubject;
///     type OwnerScope = Unowned;
/// }
/// fn registered<T: 'static, B: Scalar<T>>() {}
/// registered::<WorthQueryName<Edge>, NativeScalars>();
/// registered::<NamedTarget<Edge>, NativeScalars>();
/// ```
///
/// A kind that declares no persistent-name subject has no stored form:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
/// use worth_query_declaration::facade::application_schema::fragment::*;
/// struct Edge;
/// fn registered<T: 'static, B: Scalar<T>>() {}
/// registered::<WorthQueryName<Edge>, NativeScalars>();
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
            $crate::facade::application_schema::InternedString => InternedStringApplicationValueBinding,
            $crate::facade::application_schema::CanonicalDate => CanonicalDateApplicationValueBinding,
            $crate::facade::application_schema::CanonicalTime => CanonicalTimeApplicationValueBinding,
            $crate::facade::application_schema::CanonicalTimestamp => CanonicalTimestampApplicationValueBinding,
            $crate::facade::application_schema::CanonicalTimestampTz => CanonicalTimestampTzApplicationValueBinding,
            [u8; 16] => UuidApplicationValueBinding,
            $crate::facade::application_schema::CanonicalF32 => CanonicalF32ApplicationValueBinding,
            $crate::facade::application_schema::CanonicalF64 => CanonicalF64ApplicationValueBinding,
            $crate::facade::application_schema::InlineBytes => InlineBytesApplicationValueBinding
        );
        impl<K: $crate::facade::application_naming::WorthQueryNamedKind>
            $crate::facade::application_schema::fragment::Scalar<
                $crate::facade::application_naming::WorthQueryName<K>,
            > for $Bindings
        {
            type Binding =
                $crate::facade::application_schema::WorthQueryNameApplicationValueBinding<K>;
        }
        impl<K: $crate::facade::application_naming::WorthQueryNamedKind>
            $crate::facade::application_schema::fragment::Scalar<
                $crate::facade::application_naming::NamedTarget<K>,
            > for $Bindings
        {
            type Binding =
                $crate::facade::application_schema::NamedTargetApplicationValueBinding<K>;
        }
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
