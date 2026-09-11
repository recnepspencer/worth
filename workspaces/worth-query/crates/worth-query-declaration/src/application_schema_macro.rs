#[macro_export]
macro_rules! worth_query_application_schema {
    (
        $vis:vis schema $Schema:ident {
            owner: $owner:literal,
            version: ($major:expr, $minor:expr),
            members: |$builder:ident| $body:block
        }
    ) => {
        $crate::worth_query_application_schema!(
            @define
            $vis schema $Schema {
                owner: $owner,
                version: ($major, $minor),
                members: |$builder| $body
            }
        );
    };
    (
        $vis:vis schema $Schema:ident {
            owner: $owner:ident,
            version: ($major:expr, $minor:expr),
            members: |$builder:ident| $body:block
        }
    ) => {
        $crate::worth_query_application_schema!(
            @define
            $vis schema $Schema {
                owner: stringify!($owner),
                version: ($major, $minor),
                members: |$builder| $body
            }
        );
    };
    (
        @define
        $vis:vis schema $Schema:ident {
            owner: $owner:expr,
            version: ($major:expr, $minor:expr),
            members: |$builder:ident| $body:block
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Schema;

        impl $crate::facade::application_schema::ApplicationSchema for $Schema {
            const OWNER: &'static str = $owner;
            const NAME: &'static str = stringify!($Schema);
            const MAJOR: u32 = $major;
            const MINOR: u32 = $minor;

            fn declaration() -> Result<
                $crate::facade::application_schema::ApplicationSchemaDeclaration<Self>,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                let $builder =
                    $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder::<Self>::for_schema();
                let $builder = $body;
                $builder.build()
            }
        }

        impl $Schema {
            pub fn declaration() -> Result<
                $crate::facade::application_schema::ApplicationSchemaDeclaration<Self>,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                <Self as $crate::facade::application_schema::ApplicationSchema>::declaration()
            }
        }
    };
}

#[macro_export]
macro_rules! worth_query_entity {
    ($vis:vis $Entity:ident for $Schema:ident : $Binding:path) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Entity;

        impl<$Schema> $crate::facade::application_schema::ApplicationEntityMarkerIdentity<$Schema>
            for $Entity
        where
            $Schema: $Binding,
        {
            const IDENTIFIER: &'static str = stringify!($Entity);
        }

        impl $Entity {
            pub const fn reference<$Schema>() ->
                $crate::facade::application_schema::ApplicationEntityRef<$Schema, Self>
            where
                $Schema: $Binding,
            {
                $crate::facade::application_schema::ApplicationEntityRef::from_schema_identifier(
                    stringify!($Entity),
                )
            }
        }
    };
    ($vis:vis $Entity:ident for $Schema:ty) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Entity;

        impl $crate::facade::application_schema::ApplicationEntityMarkerIdentity<$Schema> for $Entity {
            const IDENTIFIER: &'static str = stringify!($Entity);
        }

        impl $Entity {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationEntityRef<$Schema, Self> {
                $crate::facade::application_schema::ApplicationEntityRef::from_schema_identifier(
                    stringify!($Entity),
                )
            }
        }
    };
}

#[macro_export]
macro_rules! worth_query_field {
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

#[macro_export]
macro_rules! worth_query_principal_binding {
    (
        $vis:vis $Binding:ident in $Schema:ty,
        mapping $Mapping:ty {
            identity: $IdentityField:ty,
            status: $StatusField:ty,
            target: $TargetRelation:ty => $Principal:ty,
            principal_identity: $PrincipalIdentityField:ty
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;

        impl $Binding {
            pub fn reference() -> $crate::facade::application_schema::ApplicationPrincipalBindingRef<
                $Schema,
                Self,
                $Mapping,
                $Principal,
                <$PrincipalIdentityField as $crate::facade::application_schema::DeclaredApplicationFieldValue>::Value,
                <$PrincipalIdentityField as $crate::facade::application_schema::DeclaredApplicationFieldValue>::Binding,
            > {
                let identity: $crate::facade::application_schema::ApplicationFieldRef<
                    $Schema,
                    $Mapping,
                    _,
                    $IdentityField,
                    $crate::facade::authentication::WorthQueryExternalPrincipalIdentity,
                    $crate::facade::application_schema::ReadOnly,
                    $crate::facade::application_schema::EqualityPredicate,
                    _,
                > = <$IdentityField>::reference();
                let status: $crate::facade::application_schema::ApplicationFieldRef<
                    $Schema,
                    $Mapping,
                    _,
                    $StatusField,
                    $crate::facade::authentication::WorthQueryPrincipalMappingStatus,
                    $crate::facade::application_schema::ReadWrite,
                    _,
                    _,
                > = <$StatusField>::reference();
                let target: $crate::facade::application_schema::ApplicationRelationRef<
                    $Schema,
                    $TargetRelation,
                    $Mapping,
                    $Principal,
                > = <$TargetRelation>::reference();
                let principal_identity: $crate::facade::application_schema::ApplicationFieldRef<
                    $Schema,
                    $Principal,
                    _,
                    $PrincipalIdentityField,
                    _,
                    $crate::facade::application_schema::ReadOnly,
                    $crate::facade::application_schema::EqualityPredicate,
                    _,
                > = <$PrincipalIdentityField>::reference();
                $crate::facade::application_schema::ApplicationPrincipalBindingRef::<
                    $Schema,
                    Self,
                    $Mapping,
                    $Principal,
                    <$PrincipalIdentityField as $crate::facade::application_schema::DeclaredApplicationFieldValue>::Value,
                    <$PrincipalIdentityField as $crate::facade::application_schema::DeclaredApplicationFieldValue>::Binding,
                >::from_requirements(
                    stringify!($Binding),
                    $crate::facade::application_schema::ApplicationPrincipalBindingRequirements {
                        mapping_identity: $crate::facade::application_schema::ApplicationPrincipalMappingIdentityRequirement::from_field(identity),
                        mapping_status: $crate::facade::application_schema::ApplicationPrincipalMappingStatusRequirement::from_field(status),
                        target: $crate::facade::application_schema::ApplicationPrincipalTargetRequirement::from_relation(target),
                        principal_identity: $crate::facade::application_schema::ApplicationPrincipalIdentityRequirement::from_field(principal_identity),
                    },
                )
            }
        }
    };
}
