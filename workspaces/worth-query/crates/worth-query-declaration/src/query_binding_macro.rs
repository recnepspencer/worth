/// Associates an application-owned request value with one installed query binding.
///
/// The principal group may be written out, or `context Context,` after the
/// schema supplies it from one `ApplicationBindingContext`; see
/// `__worth_query_contextual_query_binding`. The binding identity accepts any
/// `&'static str` constant expression, including `worth_query_identity!`.
#[macro_export]
macro_rules! worth_query_query_binding {
    ($vis:vis $Binding:ident for $Input:ty, schema $Schema:ty, context $($rest:tt)*) => {
        $crate::__worth_query_contextual_query_binding!(
            $vis $Binding for $Input, schema $Schema, context $($rest)*
        );
    };
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:expr,
        input $InputBinding:path,
        query $Query:path,
        parameters $ParameterBinding:path => $parameters:expr,
        result $ResultBinding:path,
        principal $PrincipalBinding:ty, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:ty,
        scope $Selected:ty,
        field $scope_field:expr,
        value $scope_value:path,
        limits results $maximum_results:expr
    ) => {
        $crate::worth_query_query_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, query $Query,
            parameters $ParameterBinding, result $ResultBinding,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_query::ApplicationQueryFieldScope<
                $Selected
            >,
            scope_field_type $Selected,
            scope_field $scope_field,
            limits results $maximum_results
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
        identity $identity:expr,
        input $InputBinding:path,
        query $Query:path,
        parameters $ParameterBinding:path => $parameters:expr,
        result $ResultBinding:path,
        principal $PrincipalBinding:ty, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:ty,
        scope $Selected:ty,
        principal_field $scope_field:expr,
        limits results $maximum_results:expr
    ) => {
        $crate::worth_query_query_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, query $Query,
            parameters $ParameterBinding, result $ResultBinding,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_query::ApplicationQueryPrincipalScope<
                $Selected
            >,
            scope_field_type $Selected,
            scope_field $scope_field,
            limits results $maximum_results
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
        identity $identity:expr, input $InputBinding:path, query $Query:path,
        parameters $ParameterBinding:path, result $ResultBinding:path,
        principal $PrincipalBinding:ty, mapping $Mapping:ty, principal_entity $Principal:ty,
        principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:ty,
        scope_binding $ScopeBinding:ty,
        scope_field_type $Selected:ty,
        scope_field $scope_field:expr,
        limits results $maximum_results:expr
        $(, principal_reference $principal_reference:expr)?
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
                $crate::worth_query_query_binding!(@limits $maximum_results);

            fn scope_field() -> $Selected {
                $scope_field
            }

            fn principal_binding() -> $crate::facade::application_schema::ApplicationPrincipalBindingRef<
                $Schema, Self::PrincipalBinding, Self::Mapping, Self::Principal,
                Self::PrincipalIdentity, Self::PrincipalIdentityBinding,
            > {
                $crate::worth_query_query_binding!(
                    @principal_reference [$($principal_reference)?] $PrincipalBinding
                )
            }
        }
    };
    (@principal_reference [$reference:expr] $PrincipalBinding:ty) => { $reference };
    (@principal_reference [] $PrincipalBinding:ty) => { <$PrincipalBinding>::reference() };
    (@limits $results:expr) => {
        $crate::facade::application_query::ApplicationQueryBindingLimits::results($results)
    };
}
