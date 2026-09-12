/// Associates a domain input with one installed, fixed-shape mutation binding.
#[macro_export]
macro_rules! worth_query_mutation_binding {
    (
        $vis:vis $Binding:ident for $Input:ty, schema $Schema:ty,
        identity $identity:literal,
        input $InputBinding:path,
        operation $Operation:path,
        result $ResultBinding:path,
        idempotency $IdempotencyKey:ty, identity $idempotency_identity:literal,
            key_identity $key_identity:path, input_identity $input_identity:path,
        decision $Decision:ty, denial $DenialBinding:path,
        handler identity $handler_identity:literal,
        outputs $Output:ty,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        field $scope_field:expr,
        value $scope_value:path,
        candidates creates $maximum_creates:expr, deletes $maximum_deletes:expr,
            links $maximum_links:expr, unlinks $maximum_unlinks:expr, writes $maximum_writes:expr,
            emits $maximum_emits:expr,
        resources retained_representation_bytes $maximum_retained_representation_bytes:expr, validator_work $maximum_validator_work:expr
    ) => {
        $crate::worth_query_mutation_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, operation $Operation, result $ResultBinding,
            idempotency $IdempotencyKey, identity $idempotency_identity,
                key_identity $key_identity, input_identity $input_identity,
            decision $Decision, denial $DenialBinding, handler identity $handler_identity,
            outputs $Output,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_operation::ApplicationMutationFieldScope<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write, $Unit
            >,
            scope_field_type $Scope, $Aspect, $Field, $Value, $Write, $Unit,
            scope_field $scope_field,
            candidates creates $maximum_creates, deletes $maximum_deletes,
                links $maximum_links, unlinks $maximum_unlinks, writes $maximum_writes,
                emits $maximum_emits,
            resources retained_representation_bytes $maximum_retained_representation_bytes, validator_work $maximum_validator_work
        );

        impl $crate::facade::application_operation::ApplicationMutationIntent<$Schema> for $Input {
            type Binding = $Binding;

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
            key_identity $key_identity:path, input_identity $input_identity:path,
        decision $Decision:ty, denial $DenialBinding:path,
        handler identity $handler_identity:literal,
        outputs $Output:ty,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
            principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        principal_field $scope_field:expr,
        candidates creates $maximum_creates:expr, deletes $maximum_deletes:expr,
            links $maximum_links:expr, unlinks $maximum_unlinks:expr, writes $maximum_writes:expr,
            emits $maximum_emits:expr,
        resources retained_representation_bytes $maximum_retained_representation_bytes:expr, validator_work $maximum_validator_work:expr
    ) => {
        $crate::worth_query_mutation_binding!(@binding
            $vis $Binding for $Input, schema $Schema,
            identity $identity, input $InputBinding, operation $Operation, result $ResultBinding,
            idempotency $IdempotencyKey, identity $idempotency_identity,
                key_identity $key_identity, input_identity $input_identity,
            decision $Decision, denial $DenialBinding, handler identity $handler_identity,
            outputs $Output,
            principal $PrincipalBinding, mapping $Mapping, principal_entity $Principal,
            principal_identity $PrincipalIdentity, identity_binding $PrincipalIdentityBinding,
            scope_binding $crate::facade::application_operation::ApplicationMutationPrincipalScope<
                $Schema, $Scope, $Aspect, $Field, $Value, $Write, $Unit
            >,
            scope_field_type $Scope, $Aspect, $Field, $Value, $Write, $Unit,
            scope_field $scope_field,
            candidates creates $maximum_creates, deletes $maximum_deletes,
                links $maximum_links, unlinks $maximum_unlinks, writes $maximum_writes,
                emits $maximum_emits,
            resources retained_representation_bytes $maximum_retained_representation_bytes, validator_work $maximum_validator_work
        );

        impl $crate::facade::application_operation::ApplicationMutationIntent<$Schema> for $Input {
            type Binding = $Binding;

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
            key_identity $key_identity:path, input_identity $input_identity:path,
        decision $Decision:ty, denial $DenialBinding:path, handler identity $handler_identity:literal,
        outputs $Output:ty,
        principal $PrincipalBinding:path, mapping $Mapping:ty, principal_entity $Principal:ty,
        principal_identity $PrincipalIdentity:ty, identity_binding $PrincipalIdentityBinding:path,
        scope_binding $ScopeBinding:ty,
        scope_field_type $Scope:ty, $Aspect:ty, $Field:ty, $Value:ty, $Write:ty, $Unit:ty,
        scope_field $scope_field:expr,
        candidates creates $maximum_creates:expr, deletes $maximum_deletes:expr,
            links $maximum_links:expr, unlinks $maximum_unlinks:expr, writes $maximum_writes:expr,
            emits $maximum_emits:expr,
        resources retained_representation_bytes $maximum_retained_representation_bytes:expr, validator_work $maximum_validator_work:expr
    ) => {
        $vis struct $Binding;

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

            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str = $handler_identity;
            const IDEMPOTENCY_IDENTITY: &'static str = $idempotency_identity;
            const CANDIDATES: $crate::facade::application_operation::ApplicationCandidateRequirements =
                $crate::facade::application_operation::ApplicationCandidateRequirements::fixed_shape(
                    $crate::facade::application_operation::ApplicationCandidateCardinalityCeiling::fixed(
                        $maximum_creates,
                        $maximum_deletes,
                        $maximum_links,
                        $maximum_unlinks,
                        $maximum_writes,
                        $maximum_emits,
                    ),
                    $crate::facade::application_operation::ApplicationCandidateResourceCeiling::bounded(
                        $maximum_retained_representation_bytes,
                        $maximum_validator_work,
                    ),
                );

            fn idempotency_key_identity(key: &Self::IdempotencyKey) -> [u8; 32] {
                ($key_identity)(key)
            }

            fn input_identity(input: &Self::Input) -> [u8; 32] {
                ($input_identity)(input)
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
}
