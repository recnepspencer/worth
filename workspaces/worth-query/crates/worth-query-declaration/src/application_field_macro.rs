#[macro_export]
macro_rules! worth_query_field {
    (
        $vis:vis $Field:ident for $Schema:ident : $Binding:path, $Entity:ty, $Aspect:ty:
        $Value:ty => $ValueBinding:path, $write:ident, $equality:ident
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Field;
        impl $crate::facade::application_schema::DeclaredApplicationFieldValue for $Field {
            type Value = $Value;
            type Binding = $ValueBinding;
            const PRESENCE: $crate::facade::application_schema::ApplicationFieldPresence =
                $crate::facade::application_schema::ApplicationFieldPresence::Required;
        }
        impl $crate::facade::application_schema::RequiredApplicationFieldValue for $Field {}
        impl<$Schema: $Binding> $crate::facade::application_schema::ApplicationFieldMarkerIdentity<$Schema, $Entity, $Aspect> for $Field {
            const IDENTIFIER: &'static str = stringify!($Field);
        }
        impl $Field {
            pub const fn reference<$Schema: $Binding>() -> $crate::facade::application_schema::ApplicationFieldRef<
                $Schema, $Entity, $Aspect, Self, $Value,
                $crate::worth_query_field!(@write $write),
                $crate::worth_query_field!(@equality $equality),
            > {
                $crate::facade::application_schema::ApplicationFieldRef::from_schema_types()
            }
        }
    };
    (
        $vis:vis $Field:ident for $Schema:ident : $Binding:path, $Entity:ty, $Aspect:ty:
        $Value:ty => $ValueBinding:path, unit $Unit:ty, $write:ident, $equality:ident
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Field;

        impl $crate::facade::application_schema::DeclaredApplicationFieldValue for $Field {
            type Value = $Value;
            type Binding = $ValueBinding;
            const PRESENCE: $crate::facade::application_schema::ApplicationFieldPresence =
                $crate::facade::application_schema::ApplicationFieldPresence::Required;
        }

        impl $crate::facade::application_schema::RequiredApplicationFieldValue for $Field {}

        impl<$Schema>
            $crate::facade::application_schema::ApplicationFieldMarkerIdentity<
                $Schema,
                $Entity,
                $Aspect,
            > for $Field
        where
            $Schema: $Binding,
        {
            const IDENTIFIER: &'static str = stringify!($Field);
        }

        impl $Field {
            pub const fn reference<$Schema>() -> $crate::facade::application_schema::ApplicationFieldRef<
                $Schema,
                $Entity,
                $Aspect,
                Self,
                $Value,
                $crate::worth_query_field!(@write $write),
                $crate::worth_query_field!(@equality $equality),
                $crate::facade::application_schema::DeclaredApplicationUnit<
                    $Unit,
                    <$ValueBinding as $crate::facade::application_schema::ApplicationScalarValueBinding>::Unit,
                >,
            >
            where
                $Schema: $Binding,
            {
                $crate::facade::application_schema::ApplicationFieldRef::from_schema_types()
            }
        }
    };
    (
        $vis:vis $Field:ident for $Schema:ty, $Entity:ty, $Aspect:ty:
        optional $Value:ty => $ValueBinding:path, unit $Unit:ty, $write:ident, $equality:ident
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Field;

        impl $crate::facade::application_schema::DeclaredApplicationFieldValue for $Field {
            type Value = $Value;
            type Binding = $ValueBinding;
            const PRESENCE: $crate::facade::application_schema::ApplicationFieldPresence =
                $crate::facade::application_schema::ApplicationFieldPresence::Optional;
        }

        impl $crate::facade::application_schema::OptionalApplicationFieldValue for $Field {}

        impl $crate::facade::application_schema::ApplicationFieldMarkerIdentity<$Schema, $Entity, $Aspect> for $Field {
            const IDENTIFIER: &'static str = stringify!($Field);
        }

        impl $Field {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationFieldRef<
                $Schema,
                $Entity,
                $Aspect,
                Self,
                $Value,
                $crate::worth_query_field!(@write $write),
                $crate::worth_query_field!(@equality $equality),
                $crate::facade::application_schema::DeclaredApplicationUnit<
                    $Unit,
                    <$ValueBinding as $crate::facade::application_schema::ApplicationScalarValueBinding>::Unit,
                >,
            > {
                $crate::facade::application_schema::ApplicationFieldRef::from_schema_types()
            }
        }
    };
    (
        $vis:vis $Field:ident for $Schema:ty, $Entity:ty, $Aspect:ty:
        optional $Value:ty => $ValueBinding:path, $write:ident, $equality:ident
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Field;

        impl $crate::facade::application_schema::DeclaredApplicationFieldValue for $Field {
            type Value = $Value;
            type Binding = $ValueBinding;
            const PRESENCE: $crate::facade::application_schema::ApplicationFieldPresence =
                $crate::facade::application_schema::ApplicationFieldPresence::Optional;
        }

        impl $crate::facade::application_schema::OptionalApplicationFieldValue for $Field {}

        impl $crate::facade::application_schema::ApplicationFieldMarkerIdentity<$Schema, $Entity, $Aspect> for $Field {
            const IDENTIFIER: &'static str = stringify!($Field);
        }

        impl $Field {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationFieldRef<
                $Schema,
                $Entity,
                $Aspect,
                Self,
                $Value,
                $crate::worth_query_field!(@write $write),
                $crate::worth_query_field!(@equality $equality),
            > {
                $crate::facade::application_schema::ApplicationFieldRef::from_schema_types()
            }
        }
    };
    (
        $vis:vis $Field:ident for $Schema:ty, $Entity:ty, $Aspect:ty:
        $Value:ty => $ValueBinding:path, unit $Unit:ty, $write:ident, $equality:ident
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Field;

        impl $crate::facade::application_schema::DeclaredApplicationFieldValue for $Field {
            type Value = $Value;
            type Binding = $ValueBinding;
            const PRESENCE: $crate::facade::application_schema::ApplicationFieldPresence =
                $crate::facade::application_schema::ApplicationFieldPresence::Required;
        }

        impl $crate::facade::application_schema::RequiredApplicationFieldValue for $Field {}

        impl $crate::facade::application_schema::ApplicationFieldMarkerIdentity<$Schema, $Entity, $Aspect> for $Field {
            const IDENTIFIER: &'static str = stringify!($Field);
        }

        impl $Field {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationFieldRef<
                $Schema,
                $Entity,
                $Aspect,
                Self,
                $Value,
                $crate::worth_query_field!(@write $write),
                $crate::worth_query_field!(@equality $equality),
                $crate::facade::application_schema::DeclaredApplicationUnit<
                    $Unit,
                    <$ValueBinding as $crate::facade::application_schema::ApplicationScalarValueBinding>::Unit,
                >,
            > {
                $crate::facade::application_schema::ApplicationFieldRef::from_schema_types()
            }
        }
    };
    (
        $vis:vis $Field:ident for $Schema:ty, $Entity:ty, $Aspect:ty:
        $Value:ty => $ValueBinding:path, $write:ident, $equality:ident
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Field;

        impl $crate::facade::application_schema::DeclaredApplicationFieldValue for $Field {
            type Value = $Value;
            type Binding = $ValueBinding;
            const PRESENCE: $crate::facade::application_schema::ApplicationFieldPresence =
                $crate::facade::application_schema::ApplicationFieldPresence::Required;
        }

        impl $crate::facade::application_schema::RequiredApplicationFieldValue for $Field {}

        impl $crate::facade::application_schema::ApplicationFieldMarkerIdentity<$Schema, $Entity, $Aspect> for $Field {
            const IDENTIFIER: &'static str = stringify!($Field);
        }

        impl $Field {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationFieldRef<
                $Schema,
                $Entity,
                $Aspect,
                Self,
                $Value,
                $crate::worth_query_field!(@write $write),
                $crate::worth_query_field!(@equality $equality),
            > {
                $crate::facade::application_schema::ApplicationFieldRef::from_schema_types()
            }

        }
    };
    (@write read_only) => { $crate::facade::application_schema::ReadOnly };
    (@write read_write) => { $crate::facade::application_schema::ReadWrite };
    (@equality no_equality) => { $crate::facade::application_schema::NoEqualityPredicate };
    (@equality equality) => { $crate::facade::application_schema::EqualityPredicate };
}
