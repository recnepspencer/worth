/// Declares an aspect marker: a stable, named surface of meaning on one
/// entity of an application schema, with its portable identity and contract
/// revision.
///
/// Input: `vis Aspect for Schema, Entity;` (or `for Schema: SchemaBinding,
/// Entity;` to declare it for every schema implementing that trait), then
/// `identity = AspectIdentity(..)` and `revision = AspectContractRevision(..)`.
/// It generates a unit struct `Aspect` that implements
/// `ApplicationAspectMarkerIdentity` and a `const fn reference()` returning its
/// typed `ApplicationAspectRef`.
#[macro_export]
macro_rules! worth_query_aspect {
    (
        $vis:vis $Aspect:ident for $Schema:ident : $Binding:path, $Entity:ty;
        identity = AspectIdentity($identity:expr),
        revision = AspectContractRevision($revision:expr) $(,)?
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Aspect;

        impl<$Schema>
            $crate::facade::application_schema::ApplicationAspectMarkerIdentity<$Schema, $Entity>
            for $Aspect
        where
            $Schema: $Binding,
        {
            const IDENTIFIER: &'static str = stringify!($Aspect);
            const ASPECT_IDENTITY: $crate::facade::application_schema::AspectIdentity =
                $crate::facade::application_schema::AspectIdentity($identity);
            const CONTRACT_REVISION: $crate::facade::application_schema::AspectContractRevision =
                $crate::facade::application_schema::AspectContractRevision($revision);
        }

        impl $Aspect {
            pub const fn reference<$Schema>() ->
                $crate::facade::application_schema::ApplicationAspectRef<$Schema, $Entity, Self>
            where
                $Schema: $Binding,
            {
                $crate::facade::application_schema::ApplicationAspectRef::from_schema_identifier(
                    stringify!($Aspect),
                )
            }
        }
    };
    (
        $vis:vis $Aspect:ident for $Schema:ty, $Entity:ty;
        identity = AspectIdentity($identity:expr),
        revision = AspectContractRevision($revision:expr) $(,)?
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Aspect;

        impl $crate::facade::application_schema::ApplicationAspectMarkerIdentity<$Schema, $Entity> for $Aspect {
            const IDENTIFIER: &'static str = stringify!($Aspect);
            const ASPECT_IDENTITY: $crate::facade::application_schema::AspectIdentity =
                $crate::facade::application_schema::AspectIdentity($identity);
            const CONTRACT_REVISION: $crate::facade::application_schema::AspectContractRevision =
                $crate::facade::application_schema::AspectContractRevision($revision);
        }

        impl $Aspect {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationAspectRef<$Schema, $Entity, Self> {
                $crate::facade::application_schema::ApplicationAspectRef::from_schema_identifier(
                    stringify!($Aspect),
                )
            }
        }
    };
}
