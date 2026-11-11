/// Declares how an authenticated external principal maps to a principal
/// entity in an application schema.
///
/// Input: `vis Binding in Schema, mapping Mapping { identity: IdentityField,
/// status: StatusField, target: TargetRelation => Principal,
/// principal_identity: PrincipalIdentityField }`. The mapping entity carries a
/// read-only, equality-comparable external principal identity field and a
/// read-write mapping status field; the target relation leads from the
/// mapping to the principal entity, whose identity field is read-only and
/// equality-comparable. The field and relation types are checked at compile
/// time. It generates a unit struct `Binding` with a `fn reference()`
/// returning its typed `ApplicationPrincipalBindingRef`.
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
                <$PrincipalIdentityField as $crate::facade::application_schema::fragment::RecordSelection>::Value,
                <$PrincipalIdentityField as $crate::facade::application_schema::fragment::RecordSelection>::Binding,
            > {
                let identity = <$IdentityField>::at(stringify!($IdentityField));
                let status = <$StatusField>::at(stringify!($StatusField));
                let target: $crate::facade::application_schema::ApplicationRelationRef<
                    $Schema,
                    $TargetRelation,
                    $Mapping,
                    $Principal,
                > = <$TargetRelation>::reference();
                let principal_identity = <$PrincipalIdentityField>::at(stringify!($PrincipalIdentityField));
                $crate::facade::application_schema::ApplicationPrincipalBindingRef::<
                    $Schema,
                    Self,
                    $Mapping,
                    $Principal,
                    <$PrincipalIdentityField as $crate::facade::application_schema::fragment::RecordSelection>::Value,
                    <$PrincipalIdentityField as $crate::facade::application_schema::fragment::RecordSelection>::Binding,
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
