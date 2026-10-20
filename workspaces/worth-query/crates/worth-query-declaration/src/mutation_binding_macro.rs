/// Associates a domain input with one installed mutation binding.
#[macro_export]
macro_rules! worth_query_mutation_binding {
    (
        $vis:vis $Binding:ident for $Input:ty, schema generic $Schema:ident: $SchemaBound:path,
        identity $identity:literal,
        input $InputBinding:path,
        operation $Operation:path,
        result $ResultBinding:path,
        idempotency $IdempotencyKey:ty, identity $idempotency_identity:literal,
        decision $Decision:ty, denial $DenialBinding:path,
        handler identity $handler_identity:literal,
        $(admission $Admission:ty,)?
        $(program $program_requirement:ident,)?
        $(workflow_authority $workflow_authority:ident,)?
        $(workflow_requirements $workflow_requirements:expr_2021,)?
        outputs $Output:ty,
        source $SourceExpectation:ty,
        principal binding $principal_binding:expr_2021,
            binding_type $PrincipalBinding:ty, mapping $Mapping:ty,
            principal_entity $Principal:ty, principal_identity $PrincipalIdentity:ty,
            identity_binding $PrincipalIdentityBinding:ty,
        scope $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        field $scope_field:expr_2021,
        value_field $scope_value:ident,

    ) => {
        $vis struct $Binding<$Schema>(::std::marker::PhantomData<fn() -> $Schema>);
        impl<$Schema: $SchemaBound> $crate::facade::application_operation::ApplicationMutationAdmissionBinding<$Schema> for $Binding<$Schema> {
            type Admission = $crate::worth_query_mutation_binding!(@admission $($Admission)?);
        }


        impl<$Schema: $SchemaBound>
            $crate::facade::application_operation::ApplicationMutationBinding<$Schema>
            for $Binding<$Schema>
        {
            type Input = $Input;
            type InputBinding = $InputBinding;
            type Result = <$ResultBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value;
            type ResultBinding = $ResultBinding;
            type IdempotencyKey = $IdempotencyKey;
            type Operation = $Operation;
            type Decision = $Decision;
            type Denial = <$DenialBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value;
            type DenialBinding = $DenialBinding;
            type Output = $Output;
            type ScopeBinding = $crate::facade::application_operation::ApplicationMutationFieldScope<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write, $Unit
            >;
            type PrincipalBinding = $PrincipalBinding;
            type Mapping = $Mapping;
            type Principal = $Principal;
            type PrincipalIdentity = $PrincipalIdentity;
            type PrincipalIdentityBinding = $PrincipalIdentityBinding;
            type SourceExpectation = $SourceExpectation;

            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str = $handler_identity;
            const IDEMPOTENCY_IDENTITY: &'static str = $idempotency_identity;
            const REQUIRES_APPLICATION_PROGRAM: bool =
                $crate::worth_query_mutation_binding!(@program_requirement $($program_requirement)?);
            const REQUIRES_WORKFLOW_AUTHORITY: bool =
                $crate::worth_query_mutation_binding!(@workflow_authority $($workflow_authority)?);
            fn workflow_execution_requirements() -> $crate::facade::application_operation::ApplicationWorkflowExecutionRequirements {
                $crate::worth_query_mutation_binding!(@workflow_requirements [$($workflow_authority)?] [$($workflow_requirements)?])
            }

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
                $principal_binding
            }
        }

        impl<$Schema: $SchemaBound>
            $crate::facade::application_operation::ApplicationMutationIntent<$Schema> for $Input
        {
            type Binding = $Binding<$Schema>;

            fn input(&self) -> &Self {
                self
            }

            fn scope_binding(&self) -> <$Binding<$Schema> as $crate::facade::application_operation::ApplicationMutationBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_operation::ApplicationMutationFieldScope::new(
                    $scope_field,
                    self.$scope_value.clone(),
                )
            }
        }
    };
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:literal,
        input $InputBinding:path,
        operation $Operation:path,
        result $ResultBinding:path,
        idempotency $IdempotencyKey:ty, identity $idempotency_identity:literal,
        decision $Decision:ty, denial $DenialBinding:path,
        handler identity $handler_identity:literal,
        $(admission $Admission:ty,)?
        $(program $program_requirement:ident,)?
        $(workflow_authority $workflow_authority:ident,)?
        $(workflow_requirements $workflow_requirements:expr_2021,)?
        outputs $Output:ty,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        field $scope_field:expr_2021,
        value $scope_value:path,

    ) => {
        $crate::worth_query_mutation_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, operation $Operation, result $ResultBinding,
            idempotency $IdempotencyKey, identity $idempotency_identity,
            decision $Decision, denial $DenialBinding, handler identity $handler_identity,
            $(admission $Admission,)?
            program $($program_requirement)?,
            workflow_authority $($workflow_authority)?,
            workflow_requirements $($workflow_requirements)?,
            outputs $Output,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_operation::ApplicationMutationFieldScope<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write, $Unit
            >,
            scope_field_type $Scope, $Aspect, $Field, $Value, $Write, $Unit,
            scope_field $scope_field,

        );

        impl $crate::facade::application_operation::ApplicationMutationIntent<$Schema> for $Input {
            type Binding = $Binding;

            fn input(&self) -> &Self {
                self
            }

            fn scope_binding(
                &self,
            ) -> <$Binding as $crate::facade::application_operation::ApplicationMutationBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_operation::ApplicationMutationFieldScope::new(
                    $scope_field,
                    ($scope_value)(self),
                )
            }
        }
    };
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:literal,
        input $InputBinding:path,
        operation $Operation:path,
        result $ResultBinding:path,
        idempotency $IdempotencyKey:ty, identity $idempotency_identity:literal,
        decision $Decision:ty, denial $DenialBinding:path,
        handler identity $handler_identity:literal,
        $(admission $Admission:ty,)?
        $(program $program_requirement:ident,)?
        $(workflow_authority $workflow_authority:ident,)?
        $(workflow_requirements $workflow_requirements:expr_2021,)?
        outputs $Output:ty,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        principal_field $scope_field:expr_2021,

    ) => {
        $crate::worth_query_mutation_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, operation $Operation, result $ResultBinding,
            idempotency $IdempotencyKey, identity $idempotency_identity,
            decision $Decision, denial $DenialBinding, handler identity $handler_identity,
            $(admission $Admission,)?
            program $($program_requirement)?,
            workflow_authority $($workflow_authority)?,
            workflow_requirements $($workflow_requirements)?,
            outputs $Output,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_operation::ApplicationMutationPrincipalScope<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write, $Unit
            >,
            scope_field_type $Scope, $Aspect, $Field, $Value, $Write, $Unit,
            scope_field $scope_field,

        );

        impl $crate::facade::application_operation::ApplicationMutationIntent<$Schema> for $Input {
            type Binding = $Binding;

            fn input(&self) -> &Self {
                self
            }

            fn scope_binding(
                &self,
            ) -> <$Binding as $crate::facade::application_operation::ApplicationMutationBinding<$Schema>>::ScopeBinding {
                $crate::facade::application_operation::ApplicationMutationPrincipalScope::new(
                    $scope_field,
                )
            }
        }
    };
    (@binding
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:literal, input $InputBinding:path, operation $Operation:path,
        result $ResultBinding:path,
        idempotency $IdempotencyKey:ty, identity $idempotency_identity:literal,
        decision $Decision:ty, denial $DenialBinding:path, handler identity $handler_identity:literal,
        $(admission $Admission:ty,)?
        program $($program_requirement:ident)?,
        workflow_authority $($workflow_authority:ident)?,
        workflow_requirements $($workflow_requirements:expr_2021)?,
        outputs $Output:ty,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
        principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope_binding $ScopeBinding:ty,
        scope_field_type $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        scope_field $scope_field:expr_2021,

    ) => {
        $vis struct $Binding;

        impl $crate::facade::application_operation::ApplicationMutationAdmissionBinding<$Schema> for $Binding {
            type Admission = $crate::worth_query_mutation_binding!(@admission $($Admission)?);
        }
        impl $crate::facade::application_operation::ApplicationMutationBinding<$Schema>
            for $Binding
        {
            type Input = $Input;
            type InputBinding = $InputBinding;
            type Result = <$ResultBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value;
            type ResultBinding = $ResultBinding;
            type IdempotencyKey = $IdempotencyKey;
            type Operation = $Operation;
            type Decision = $Decision;
            type Denial = <$DenialBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value;
            type DenialBinding = $DenialBinding;
            type Output = $Output;
            type ScopeBinding = $ScopeBinding;
            type PrincipalBinding = $PrincipalBinding;
            type Mapping = $Mapping;
            type Principal = $Principal;
            type PrincipalIdentity = $PrincipalIdentity;
            type PrincipalIdentityBinding = $PrincipalIdentityBinding;
            type SourceExpectation = $crate::facade::application_operation::NoApplicationMutationSource;

            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str = $handler_identity;
            const IDEMPOTENCY_IDENTITY: &'static str = $idempotency_identity;
            const REQUIRES_APPLICATION_PROGRAM: bool =
                $crate::worth_query_mutation_binding!(@program_requirement $($program_requirement)?);
            const REQUIRES_WORKFLOW_AUTHORITY: bool =
                $crate::worth_query_mutation_binding!(@workflow_authority $($workflow_authority)?);
            fn workflow_execution_requirements() -> $crate::facade::application_operation::ApplicationWorkflowExecutionRequirements {
                $crate::worth_query_mutation_binding!(@workflow_requirements [$($workflow_authority)?] [$($workflow_requirements)?])
            }

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
    (@admission $Admission:ty) => { $Admission };
    (@admission) => { $crate::facade::application_operation::ApplicationConventionalAdmission };
    (@workflow_requirements [required] []) => {
        compile_error!("guarded mutations must declare workflow_requirements explicitly")
    };
    (@workflow_requirements [$($authority:ident)?] [$requirements:expr_2021]) => { $requirements };
    (@workflow_requirements [] []) => {
        $crate::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    };
    (@program_requirement required) => { true };
    (@program_requirement) => { false };
    (@workflow_authority required) => { true };
    (@workflow_authority) => { false };
}
