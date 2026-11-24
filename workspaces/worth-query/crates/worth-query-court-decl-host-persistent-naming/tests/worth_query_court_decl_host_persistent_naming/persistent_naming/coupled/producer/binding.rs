//! One producer phase's installed binding: its input (the source model's
//! row), mutation binding, producer binding and provider. Each phase adds its
//! own operation, output roles, decision and handler. Publications are
//! independent unless the phase names another ownership.
macro_rules! producer_binding {
    ($identity:literal, $posture:ident, $role:ident) => {
        producer_binding!($identity, $posture, $role, Independent);
    };
    ($identity:literal, $posture:ident, $role:ident, $ownership:ident) => {
        #[derive(Clone, Debug, WorthValue)]
        #[worth_value(crate = "worth_query_decl::facade::application_schema")]
        pub struct Input {
            pub key: String,
            pub value: u64,
        }
        worth_query_structured_value_binding!(pub InputBinding for Input { identity: $identity });
        type Scope = ApplicationMutationFieldScope<ModelKey>;
        fn model_key() -> ModelKey {
            ModelKey::at("ModelKey")
        }
        pub struct Binding;
        impl ApplicationMutationBinding<Schema> for Binding {
            type Input = Input;
            type InputBinding = InputBinding;
            type Result = edit::Edited;
            type ResultBinding = edit::ResultBinding;
            type IdempotencyKey = u64;
            type Operation = Operation;
            type Decision = Decision;
            type Denial = edit::Denial;
            type DenialBinding = edit::DenialBinding;
            type Output = Outputs;
            type ScopeBinding = Scope;
            type PrincipalBinding = Actor;
            type Mapping = Mapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = ApplicationQueryMutationSource<query::model::Query>;
            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str = $identity;
            const IDEMPOTENCY_IDENTITY: &'static str = $identity;
            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
            }
            fn scope_field() -> ModelKey {
                model_key()
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                Schema,
                Actor,
                Mapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                Actor::reference()
            }
        }
        impl ApplicationMutationIntent<Schema> for Input {
            type Binding = Binding;
            fn input(&self) -> &Self {
                self
            }
            fn scope_binding(&self) -> Scope {
                Scope::new(model_key(), self.key.clone())
            }
        }
        pub struct Producer;
        pub struct Provider;
        impl WorthQueryApplicationProducerBinding<Schema> for Producer {
            type Operation = Binding;
            type OutputFamily = super::Family;
            type Provider = Provider;
            type OutputRole = $role;
            const IDENTITY: &'static str = $identity;
            const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
                &[WorthQueryProducerApplicability::new(
                    super::PROFILE,
                    WorthQueryProducerLifecyclePosture::$posture,
                )];
            const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
            const OWNERSHIP: ProducerPublicationOwnership =
                ProducerPublicationOwnership::$ownership;
            const REUSE_POLICY: &'static str = "exact-source";
        }
        impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
            type DenialValue = WorthQueryUndeclaredProducerDenial;
            const SEMANTIC_IDENTITY: &'static str = $identity;
            fn operation_input(&self, row: &query::Row) -> Input {
                Input {
                    key: row.key.clone(),
                    value: row.value,
                }
            }
            fn idempotency_key(&self, _: &query::Row, identity: &[u8; 32]) -> u64 {
                u64::from_le_bytes(identity[..8].try_into().unwrap())
            }
        }
        pub fn declare(
            schema: ApplicationSchemaDeclarationBuilder<Schema>,
        ) -> ApplicationSchemaDeclarationBuilder<Schema> {
            schema
                .declared_operation::<Operation>(
                    Operation::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .application_mutation_binding::<Binding>()
        }
        readiness!($identity);
    };
}
