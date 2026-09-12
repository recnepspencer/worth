/// Associates an application-owned request value with one installed query binding.
#[macro_export]
macro_rules! worth_query_query_binding {
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:literal,
        input $InputBinding:path,
        query $Query:path,
        parameters $ParameterBinding:path => $parameters:expr,
        result $ResultBinding:path,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        field $scope_field:expr,
        value $scope_value:path,
        limits results $maximum_results:expr, work $maximum_work:expr
    ) => {
        $crate::worth_query_query_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, query $Query,
            parameters $ParameterBinding, result $ResultBinding,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_query::ApplicationQueryFieldScope<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write, $Unit
            >,
            scope_field_type $Scope, $Aspect, $Field, $Value, $Write, $Unit,
            scope_field $scope_field,
            limits results $maximum_results, work $maximum_work
        );

        impl $crate::facade::application_query::ApplicationQueryIntent<$Schema> for $Input {
            type Binding = $Binding;

            fn parameters(&self) -> $crate::facade::application_query::ApplicationQueryParameterSet<$Query> {
                ($parameters)(self)
            }

            fn into_scope(self) -> <$Binding as $crate::facade::application_query::ApplicationQueryBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_query::ApplicationQueryFieldScope::new(
                    $scope_field,
                    $scope_value(self),
                )
            }
        }
    };
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:literal,
        input $InputBinding:path,
        query $Query:path,
        parameters $ParameterBinding:path => $parameters:expr,
        result $ResultBinding:path,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        principal_field $scope_field:expr,
        limits results $maximum_results:expr, work $maximum_work:expr
    ) => {
        $crate::worth_query_query_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, query $Query,
            parameters $ParameterBinding, result $ResultBinding,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_query::ApplicationQueryPrincipalScope<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write, $Unit
            >,
            scope_field_type $Scope, $Aspect, $Field, $Value, $Write, $Unit,
            scope_field $scope_field,
            limits results $maximum_results, work $maximum_work
        );

        impl $crate::facade::application_query::ApplicationQueryIntent<$Schema> for $Input {
            type Binding = $Binding;

            fn parameters(&self) -> $crate::facade::application_query::ApplicationQueryParameterSet<$Query> {
                ($parameters)(self)
            }

            fn into_scope(self) -> <$Binding as $crate::facade::application_query::ApplicationQueryBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_query::ApplicationQueryPrincipalScope::new($scope_field)
            }
        }
    };
    (@binding
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:literal, input $InputBinding:path, query $Query:path,
        parameters $ParameterBinding:path, result $ResultBinding:path,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
        principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope_binding $ScopeBinding:ty,
        scope_field_type $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        scope_field $scope_field:expr,
        limits results $maximum_results:expr, work $maximum_work:expr
    ) => {
        $vis struct $Binding;

        impl $crate::facade::application_query::ApplicationQueryBinding<$Schema> for $Binding {
            type Input = $Input;
            type InputBinding = $InputBinding;
            type Query = $Query;
            type ParameterBinding = $ParameterBinding;
            type ResultBinding = $ResultBinding;
            type ScopeBinding = $ScopeBinding;
            type PrincipalBinding = $PrincipalBinding;
            type Mapping = $Mapping;
            type Principal = $Principal;
            type PrincipalIdentity = $PrincipalIdentity;
            type PrincipalIdentityBinding = $PrincipalIdentityBinding;

            const IDENTITY: &'static str = $identity;
            const LIMITS: $crate::facade::application_query::ApplicationQueryBindingLimits =
                $crate::facade::application_query::ApplicationQueryBindingLimits::bounded(
                    $maximum_results, $maximum_work,
                );

            fn scope_field() -> $crate::facade::application_schema::ApplicationFieldRef<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write,
                $crate::facade::application_schema::EqualityPredicate, $Unit,
            > {
                $scope_field
            }

            fn principal_binding() -> $crate::facade::application_schema::ApplicationPrincipalBindingRef<
                $Schema, Self::PrincipalBinding, Self::Mapping, Self::Principal,
                Self::PrincipalIdentity, Self::PrincipalIdentityBinding,
            > {
                <$PrincipalBinding>::reference()
            }
        }
    };
}
