/// Declare a scalar field owned by a real application relation. The endpoint
/// types are explicit; the relation is never given entity-marker identity.
#[macro_export]
macro_rules! worth_query_relation_field {
    (
        $vis:vis $Field:ident for $Schema:ty, $Relation:ty, $From:ty, $To:ty, $Aspect:ty:
        optional $Value:ty => $Binding:path, $write:ident $(,)?
    ) => {
        $crate::worth_query_relation_field!(@declare $vis $Field, $Schema, $Relation, $From, $To,
            $Aspect, $Value, $Binding, $write, Optional, OptionalApplicationFieldValue);
    };
    (
        $vis:vis $Field:ident for $Schema:ty, $Relation:ty, $From:ty, $To:ty, $Aspect:ty:
        $Value:ty => $Binding:path, $write:ident $(,)?
    ) => {
        $crate::worth_query_relation_field!(@declare $vis $Field, $Schema, $Relation, $From, $To,
            $Aspect, $Value, $Binding, $write, Required, RequiredApplicationFieldValue);
    };
    (@declare $vis:vis $Field:ident, $Schema:ty, $Relation:ty, $From:ty, $To:ty,
        $Aspect:ty, $Value:ty, $Binding:path, $write:ident, $presence:ident, $PresenceTrait:ident
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Field;

        impl $crate::facade::application_schema::DeclaredApplicationFieldValue for $Field {
            type Value = $Value;
            type Binding = $Binding;
            const PRESENCE: $crate::facade::application_schema::ApplicationFieldPresence =
                $crate::facade::application_schema::ApplicationFieldPresence::$presence;
        }
        impl $crate::facade::application_schema::$PresenceTrait for $Field {}
        impl $crate::facade::application_schema::ApplicationFieldMarkerIdentity<$Schema, $Relation, $Aspect> for $Field {
            const IDENTIFIER: &'static str = stringify!($Field);
        }
        impl $Field {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationRelationFieldRef<
                $Schema, $Relation, $From, $To, $Aspect, Self,
                $crate::worth_query_field!(@write $write),
            > {
                $crate::facade::application_schema::ApplicationRelationFieldRef::from_relation(
                    <$Relation>::reference(),
                )
            }
        }
    };
}
