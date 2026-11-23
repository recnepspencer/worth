//! `context Context,` binding clauses resolve to the clauses they abbreviate.
//!
//! Each omitted clause becomes the context's associated item; each stated
//! clause wins. The resolved tokens enter the same `@binding` lowering as a
//! fully written binding, so both spellings produce one canonical binding.

/// Resolves `worth_query_mutation_binding!` with `context Context,`.
///
/// Optional overrides, in their usual positions: `idempotency Key, identity
/// "..",`, `denial Binding,` and the complete principal group `principal ..,
/// mapping .., principal_entity .., principal_identity .., identity_binding
/// ..,`. Scope is `field F, value V,` or `principal_field F,`.
///
/// `intent Intent => input_of,` directly after the context binds a distinct
/// intent type to a binding whose input is shared by several intents: the
/// intent's `input()` is `input_of(&intent)` and its scope `value V` reads the
/// intent. Without it the input is its own intent.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_contextual_mutation_binding {
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty, context $Context:ty,
        $(intent $Intent:ty => $intent_input:path,)?
        identity $identity:expr,
        input $InputBinding:path,
        operation $Operation:path,
        result $ResultBinding:path,
        $(idempotency $IdempotencyKey:ty, identity $idempotency_identity:expr,)?
        decision $Decision:ty,
        $(denial $DenialBinding:ty,)?
        handler identity $handler_identity:expr,
        $(admission $Admission:ty,)?
        $(program $program_requirement:ident,)?
        $(workflow_authority $workflow_authority:ident,)?
        $(workflow_requirements $workflow_requirements:expr_2021,)?
        $(inspection_disclosure $InspectionDisclosure:ty,)?
        $(coupled_outputs |$coupled_visitor:ident| $coupled_outputs:block,)?
        outputs $Output:ty,
        $(principal $PrincipalBinding:ty, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:ty,)?
        scope $Selected:ty,
        $($scope:tt)*
    ) => {
        $crate::worth_query_mutation_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, operation $Operation, result $ResultBinding,
            idempotency $crate::__worth_query_binding_context!(@type [$($IdempotencyKey)?]
                $Context, $Schema, IdempotencyKey),
            identity $crate::__worth_query_binding_context!(@identity [$($idempotency_identity)?]
                $Context, $Schema),
            decision $Decision,
            denial $crate::__worth_query_binding_context!(@type [$($DenialBinding)?]
                $Context, $Schema, DenialBinding),
            handler identity $handler_identity,
            $(admission $Admission,)?
            program $($program_requirement)?,
            workflow_authority $($workflow_authority)?,
            workflow_requirements $($workflow_requirements)?,
            inspection_disclosure $($InspectionDisclosure)?,
            coupled_outputs $(|$coupled_visitor| $coupled_outputs)?,
            outputs $Output,
            principal $crate::__worth_query_binding_context!(@type [$($PrincipalBinding)?]
                $Context, $Schema, PrincipalBinding),
            mapping $crate::__worth_query_binding_context!(@type [$($Mapping)?]
                $Context, $Schema, Mapping),
            principal_entity $crate::__worth_query_binding_context!(@type [$($Principal)?]
                $Context, $Schema, Principal),
            principal_identity $crate::__worth_query_binding_context!(@type [$($PrincipalIdentity)?]
                $Context, $Schema, PrincipalIdentity),
            identity_binding $crate::__worth_query_binding_context!(
                @type [$($PrincipalIdentityBinding)?] $Context, $Schema, PrincipalIdentityBinding),
            scope_binding $crate::__worth_query_binding_context!(@mutation_scope [$Selected] $($scope)*),
            scope_field_type $Selected,
            scope_field $crate::__worth_query_binding_context!(@scope_field $($scope)*),
            principal_reference $crate::__worth_query_binding_context!(
                @principal_reference [$($PrincipalBinding)?] $Context, $Schema),

        );
        $crate::__worth_query_binding_context!(
            @mutation_intent $Binding, $Input, $Schema, [$($Intent => $intent_input)?]; $($scope)*
        );
    };
}

/// Resolves `worth_query_query_binding!` with `context Context,`.
///
/// The complete principal group may be stated as an override. Scope is
/// `field F, value V, limits results N` or `principal_field F, limits results N`.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_contextual_query_binding {
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty, context $Context:ty,
        identity $identity:expr,
        input $InputBinding:path,
        query $Query:path,
        parameters $ParameterBinding:path => $parameters:expr,
        result $ResultBinding:path,
        $(principal $PrincipalBinding:ty, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:ty,)?
        scope $Selected:ty,
        $($scope:tt)*
    ) => {
        $crate::worth_query_query_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, query $Query,
            parameters $ParameterBinding, result $ResultBinding,
            principal $crate::__worth_query_binding_context!(@type [$($PrincipalBinding)?]
                $Context, $Schema, PrincipalBinding),
            mapping $crate::__worth_query_binding_context!(@type [$($Mapping)?]
                $Context, $Schema, Mapping),
            principal_entity $crate::__worth_query_binding_context!(@type [$($Principal)?]
                $Context, $Schema, Principal),
            principal_identity $crate::__worth_query_binding_context!(@type [$($PrincipalIdentity)?]
                $Context, $Schema, PrincipalIdentity),
            identity_binding $crate::__worth_query_binding_context!(
                @type [$($PrincipalIdentityBinding)?] $Context, $Schema, PrincipalIdentityBinding),
            scope_binding $crate::__worth_query_binding_context!(@query_scope [$Selected] $($scope)*),
            scope_field_type $Selected,
            scope_field $crate::__worth_query_binding_context!(@scope_field $($scope)*),
            limits results $crate::__worth_query_binding_context!(@query_limit $($scope)*),
            principal_reference $crate::__worth_query_binding_context!(
                @principal_reference [$($PrincipalBinding)?] $Context, $Schema)
        );
        $crate::__worth_query_binding_context!(
            @query_intent $Binding, $Input, $Schema, $Query, $parameters; $($scope)*
        );
    };
}

/// Clause resolution and scope lowering shared by both contextual bindings.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_binding_context {
    (@type [$Stated:ty] $Context:ty, $Schema:ty, $Item:ident) => { $Stated };
    (@type [] $Context:ty, $Schema:ty, $Item:ident) => {
        <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::$Item
    };
    (@identity [$stated:expr] $Context:ty, $Schema:ty) => { $stated };
    (@identity [] $Context:ty, $Schema:ty) => {
        <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::IDEMPOTENCY_IDENTITY
    };
    (@principal_reference [$Stated:ty] $Context:ty, $Schema:ty) => { <$Stated>::reference() };
    (@principal_reference [] $Context:ty, $Schema:ty) => {
        <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::principal_binding()
    };
    (@scope_field field $field:expr, value $value:path $(, $($rest:tt)*)?) => { $field };
    (@scope_field principal_field $field:expr $(, $($rest:tt)*)?) => { $field };
    (@mutation_scope [$Selected:ty] field $($rest:tt)*) => {
        $crate::facade::application_operation::ApplicationMutationFieldScope<$Selected>
    };
    (@mutation_scope [$Selected:ty] principal_field $($rest:tt)*) => {
        $crate::facade::application_operation::ApplicationMutationPrincipalScope<$Selected>
    };
    (@query_scope [$Selected:ty] field $($rest:tt)*) => {
        $crate::facade::application_query::ApplicationQueryFieldScope<$Selected>
    };
    (@query_scope [$Selected:ty] principal_field $($rest:tt)*) => {
        $crate::facade::application_query::ApplicationQueryPrincipalScope<$Selected>
    };
    (@query_limit field $field:expr, value $value:path, limits results $results:expr) => { $results };
    (@query_limit principal_field $field:expr, limits results $results:expr) => { $results };
    (@mutation_intent $Binding:ident, $Input:ty, $Schema:ty, [$($intent:tt)*];
        field $field:expr, value $value:path $(,)?
    ) => {
        impl $crate::facade::application_operation::ApplicationMutationIntent<$Schema>
            for $crate::__worth_query_binding_context!(@intent_type [$($intent)*] $Input)
        {
            type Binding = $Binding;
            $crate::__worth_query_binding_context!(@intent_input [$($intent)*] $Input);
            fn scope_binding(
                &self,
            ) -> <$Binding as $crate::facade::application_operation::ApplicationMutationBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_operation::ApplicationMutationFieldScope::new(
                    $field,
                    ($value)(self),
                )
            }
        }
    };
    (@mutation_intent $Binding:ident, $Input:ty, $Schema:ty, [$($intent:tt)*];
        principal_field $field:expr $(,)?
    ) => {
        impl $crate::facade::application_operation::ApplicationMutationIntent<$Schema>
            for $crate::__worth_query_binding_context!(@intent_type [$($intent)*] $Input)
        {
            type Binding = $Binding;
            $crate::__worth_query_binding_context!(@intent_input [$($intent)*] $Input);
            fn scope_binding(
                &self,
            ) -> <$Binding as $crate::facade::application_operation::ApplicationMutationBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_operation::ApplicationMutationPrincipalScope::new($field)
            }
        }
    };
    (@intent_type [] $Input:ty) => { $Input };
    (@intent_type [$Intent:ty => $intent_input:path] $Input:ty) => { $Intent };
    (@intent_input [] $Input:ty) => {
        fn input(&self) -> &Self {
            self
        }
    };
    (@intent_input [$Intent:ty => $intent_input:path] $Input:ty) => {
        fn input(&self) -> &$Input {
            ($intent_input)(self)
        }
    };
    (@query_intent $Binding:ident, $Input:ty, $Schema:ty, $Query:path, $parameters:expr;
        field $field:expr, value $value:path, limits results $results:expr
    ) => {
        impl $crate::facade::application_query::ApplicationQueryIntent<$Schema> for $Input {
            type Binding = $Binding;
            fn parameters(&self) -> $crate::facade::application_query::ApplicationQueryParameterSet<$Query> {
                ($parameters)(self)
            }
            fn into_scope(self) -> <$Binding as $crate::facade::application_query::ApplicationQueryBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_query::ApplicationQueryFieldScope::new($field, $value(self))
            }
        }
    };
    (@query_intent $Binding:ident, $Input:ty, $Schema:ty, $Query:path, $parameters:expr;
        principal_field $field:expr, limits results $results:expr
    ) => {
        impl $crate::facade::application_query::ApplicationQueryIntent<$Schema> for $Input {
            type Binding = $Binding;
            fn parameters(&self) -> $crate::facade::application_query::ApplicationQueryParameterSet<$Query> {
                ($parameters)(self)
            }
            fn into_scope(self) -> <$Binding as $crate::facade::application_query::ApplicationQueryBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_query::ApplicationQueryPrincipalScope::new($field)
            }
        }
    };
}
