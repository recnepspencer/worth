//! Expectations use the generic item handler, which never performs a Query READ.
use super::super::mutation;
use super::*;
use worth_query_decl::facade::application_operation::*;
macro_rules! binding {
    ($binding:ident, $intent:ident, $query:ty, $identity:literal) => {
        pub struct $binding;
        impl ApplicationMutationAdmissionBinding<Catalog> for $binding {
            type Admission = ApplicationConventionalAdmission;
        }
        impl ApplicationMutationBinding<Catalog> for $binding {
            type Input = mutation::Rename;
            type InputBinding = mutation::InputBinding;
            type Result = mutation::Renamed;
            type ResultBinding = mutation::ResultBinding;
            type IdempotencyKey = u64;
            type Operation = mutation::RenameItem;
            type Decision = <mutation::Binding as ApplicationMutationBinding<Catalog>>::Decision;
            type Denial = mutation::Denial;
            type DenialBinding = mutation::DenialBinding;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding =
                <mutation::Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding;
            type PrincipalBinding = CatalogPrincipal;
            type Mapping = ExternalMapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = ApplicationQueryMutationSource<$query>;
            const IDENTITY: &'static str = concat!($identity, ".binding.v1");
            const HANDLER_IDENTITY: &'static str = concat!($identity, ".handler.v1");
            const IDEMPOTENCY_IDENTITY: &'static str = concat!($identity, ".key.v1");
            const REQUIRES_APPLICATION_PROGRAM: bool = true;
            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
            }
            fn scope_field() -> ApplicationFieldRef<
                Catalog,
                Directory,
                DirectoryFacts,
                DirectoryIdentity,
                String,
                ReadOnly,
                EqualityPredicate,
                NoApplicationUnit,
            > {
                DirectoryIdentity::reference()
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                Catalog,
                CatalogPrincipal,
                ExternalMapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                CatalogPrincipal::reference()
            }
            fn expected_source_parameters(
                input: &mutation::Rename,
            ) -> Result<Option<ApplicationQueryParameterSet<$query>>, ApplicationValueEncodeDenial>
            {
                Ok(Some(
                    ApplicationQueryParameterSet::new()
                        .bind(
                            probe::<$query>(),
                            ordinary::Probe {
                                prefix: input.prefix.clone(),
                            },
                        )
                        .unwrap(),
                ))
            }
        }
        #[derive(Clone, Debug)]
        pub struct $intent(pub mutation::Rename);
        impl ApplicationMutationIntent<Catalog> for $intent {
            type Binding = $binding;
            fn input(&self) -> &mutation::Rename {
                &self.0
            }
            fn scope_binding(
                &self,
            ) -> <$binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
                ApplicationMutationFieldScope::new(
                    DirectoryIdentity::reference(),
                    self.0.directory.clone(),
                )
            }
        }
    };
}
binding!(
    NativeBinding,
    RenameNative,
    NativeParent,
    "maintained-catalog.rename-native-source"
);
binding!(
    CompletedBinding,
    RenameCompleted,
    CompletedParent,
    "maintained-catalog.rename-completed-source"
);
