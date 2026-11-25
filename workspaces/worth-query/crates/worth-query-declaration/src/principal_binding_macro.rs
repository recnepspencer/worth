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
///
/// The record form takes `#[derive(WorthQueryRecord)]` field selections and
/// also registers the binding as a schema member, optionally declaring the
/// action binding context that names it:
///
/// ```text
/// worth_query_principal_binding! {
///     pub PrincipalBinding in Schema, mapping Mapping => Owner {
///         identity: MappingFacts::external_identity(),
///         status: MappingFacts::mapping_status(),
///         target: MappingOwner,
///         principal: OwnerId = OwnerFacts::owner_id(),
///     }
///     context pub DesktopContext {
///         idempotency: u64 = worth_query_identity!(DAYMARK, idempotency),
///         denial: CommandDenial,
///     }
/// }
/// ```
///
/// `principal` names the identity field's selection type and value.
#[macro_export]
macro_rules! worth_query_principal_binding {
    (
        $vis:vis $Binding:ident in $Schema:ty, mapping $Mapping:ty => $Principal:ty {
            identity: $identity:expr,
            status: $status:expr,
            target: $Target:ty,
            principal: $PrincipalField:ty = $principal:expr $(,)?
        }
        $(context $context_vis:vis $Context:ident {
            idempotency: $Key:ty = $key_identity:expr,
            denial: $Denial:ty $(,)?
        })?
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;

        impl $Binding {
            pub fn reference() -> $crate::worth_query_principal_binding!(@ref $Schema, $Binding, $Mapping, $Principal, $PrincipalField) {
                $crate::facade::application_schema::ApplicationPrincipalBindingRef::from_requirements(
                    stringify!($Binding),
                    $crate::facade::application_schema::ApplicationPrincipalBindingRequirements {
                        mapping_identity: $crate::facade::application_schema::ApplicationPrincipalMappingIdentityRequirement::from_field($identity),
                        mapping_status: $crate::facade::application_schema::ApplicationPrincipalMappingStatusRequirement::from_field($status),
                        target: $crate::facade::application_schema::ApplicationPrincipalTargetRequirement::from_relation(<$Target>::reference()),
                        principal_identity: $crate::facade::application_schema::ApplicationPrincipalIdentityRequirement::from_field($principal),
                    },
                )
            }
        }

        impl $crate::facade::application_schema::fragment::Registration<$Schema> for $Binding {
            fn register(
                builder: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> ::core::result::Result<
                $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                ::core::result::Result::Ok(builder.principal_binding(Self::reference()))
            }
        }

        $(
            $context_vis struct $Context;

            impl $crate::facade::application_schema::ApplicationBindingContext<$Schema> for $Context {
                type PrincipalBinding = $Binding;
                type Mapping = $Mapping;
                type Principal = $Principal;
                type PrincipalIdentity =
                    <$PrincipalField as $crate::facade::application_schema::fragment::RecordSelection>::Value;
                type PrincipalIdentityBinding =
                    <$PrincipalField as $crate::facade::application_schema::fragment::RecordSelection>::Binding;
                type IdempotencyKey = $Key;
                type DenialBinding = $Denial;
                const IDEMPOTENCY_IDENTITY: &'static str = $key_identity;

                fn principal_binding() -> $crate::worth_query_principal_binding!(@ref $Schema, $Binding, $Mapping, $Principal, $PrincipalField) {
                    $Binding::reference()
                }
            }
        )?
    };
    (@ref $Schema:ty, $Binding:ty, $Mapping:ty, $Principal:ty, $PrincipalField:ty) => {
        $crate::facade::application_schema::ApplicationPrincipalBindingRef<
            $Schema,
            $Binding,
            $Mapping,
            $Principal,
            <$PrincipalField as $crate::facade::application_schema::fragment::RecordSelection>::Value,
            <$PrincipalField as $crate::facade::application_schema::fragment::RecordSelection>::Binding,
        >
    };
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
