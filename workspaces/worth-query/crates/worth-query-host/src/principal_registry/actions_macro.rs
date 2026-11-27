//! Onboarding and status actions lowered by `worth_query_principal_registry!`.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_principal_registry_actions {
    ($Schema:ident, $Context:ident,
     $Principal:ident, $PrincipalKey:ident, $PrincipalFacts:ident :: $principal_id:ident,
     $Directory:ident, $DirectoryKey:ident, $DirectoryFacts:ident :: $directory_id:ident) => {
        const ONBOARD_NAMESPACE: $crate::facade::declaration::application_identity::ApplicationIdentityNamespace =
            $crate::facade::worth_query_identity_namespace!("worth.principal_registry.onboard.v1");
        const STATUS_NAMESPACE: $crate::facade::declaration::application_identity::ApplicationIdentityNamespace =
            $crate::facade::worth_query_identity_namespace!("worth.principal_registry.status.v1");

        type RegistryError = $crate::facade::primary_graph::HandlerExecutionDenial;
        type RegistryResult<T> = $crate::facade::primary_graph::HandlerResult<
            T,
            $crate::facade::primary_graph::WorthQueryOperationBodyDenial,
        >;

        /// Onboards `identity` as a new principal `principal` in `directory`.
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize, $crate::facade::WorthQueryAction)]
        #[worth_query(
            schema = $Schema, context = $Context, namespace = ONBOARD_NAMESPACE,
            input = OnboardPrincipalInput, operation = OnboardPrincipalOperation,
            binding = OnboardPrincipalBinding, delivery = PrincipalOnboarded,
            result = UnitApplicationValueBinding, decision = OnboardDecision, program,
            access(
                read_entities: [$Directory, PrincipalAccount, $Principal];
                read_fields: [$DirectoryFacts::$directory_id(),
                    PrincipalAccountFacts::external_identity(), $PrincipalFacts::$principal_id()];
                writes: [PrincipalAccountFacts::external_identity(), PrincipalAccountFacts::status(),
                    $PrincipalFacts::$principal_id()];
                creates: [PrincipalAccount, $Principal];
                links: [PrincipalAccountTarget];
            )
        )]
        pub struct OnboardPrincipal {
            #[worth_query(scope = $DirectoryKey)]
            pub directory: <$DirectoryKey as RecordSelection>::Value,
            pub identity: WorthQueryExternalPrincipalIdentity,
            pub principal: <$PrincipalKey as RecordSelection>::Value,
        }

        /// The keys of a fresh account and principal; neither exists yet.
        pub struct OnboardDecision {
            key: ::std::string::String,
        }

        /// Refuses an identity or principal identity already present, then
        /// creates both records and their link in one commit.
        pub struct OnboardPrincipalHandler;
        impl $crate::facade::primary_graph::OperationHandler<$Schema, OnboardPrincipalBinding>
            for OnboardPrincipalHandler
        {
            fn decide(
                &self,
                input: &OnboardPrincipal,
                reader: &mut $crate::facade::primary_graph::DecisionReader<'_, '_, '_, $Schema, OnboardPrincipalBinding>,
            ) -> RegistryResult<OnboardDecision> {
                use $crate::facade::principal_registry::WorthQueryPrincipalRegistryRefusal as Refusal;
                let present = (|| -> ::core::result::Result<::core::option::Option<Refusal>, RegistryError> {
                    let account = reader.resolve_optional_entity(
                        PrincipalAccountFacts::external_identity(),
                        input.identity.clone(),
                    )?;
                    if account.is_some() {
                        return ::core::result::Result::Ok(::core::option::Option::Some(Refusal::IdentityAlreadyOnboarded));
                    }
                    let principal = reader.resolve_optional_entity(
                        $PrincipalFacts::$principal_id(),
                        input.principal.clone(),
                    )?;
                    ::core::result::Result::Ok(principal.map(|_| Refusal::PrincipalIdentityTaken))
                })();
                match present {
                    ::core::result::Result::Ok(::core::option::Option::None) => {
                        $crate::facade::primary_graph::HandlerResult::Completed(OnboardDecision {
                            key: input.identity.portable_text(),
                        })
                    }
                    ::core::result::Result::Ok(::core::option::Option::Some(refusal)) => {
                        refusal.into_handler_result()
                    }
                    ::core::result::Result::Err(cause) => {
                        $crate::facade::primary_graph::HandlerResult::ExecutionDenied(cause)
                    }
                }
            }

            fn build_candidate(
                &self,
                input: &OnboardPrincipal,
                decision: OnboardDecision,
                writer: &mut $crate::facade::primary_graph::CandidateWriter<'_, $Schema, OnboardPrincipalBinding>,
            ) -> RegistryResult<()> {
                use $crate::facade::primary_graph::WorthQueryApplicationEntityKey as Key;
                use $crate::facade::principal_registry::WorthQueryPrincipalRegistryRefusal as Refusal;
                let (::core::result::Result::Ok(account_key), ::core::result::Result::Ok(principal_key)) =
                    (Key::new(decision.key.clone()), Key::new(decision.key.clone()))
                else {
                    return Refusal::InvalidAccountKey.into_handler_result();
                };
                (|| -> ::core::result::Result<(), RegistryError> {
                    let principal = writer.create_entity($Principal::reference(), principal_key)?;
                    writer.initialize_field(&principal, $PrincipalFacts::$principal_id(), input.principal.clone())?;
                    let account = writer.create_entity(PrincipalAccount::reference(), account_key)?;
                    writer.initialize_field(
                        &account,
                        PrincipalAccountFacts::external_identity(),
                        input.identity.clone(),
                    )?;
                    writer.initialize_field(
                        &account,
                        PrincipalAccountFacts::status(),
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )?;
                    writer.link(PrincipalAccountTarget::reference(), decision.key, &account, &principal)?;
                    ::core::result::Result::Ok(())
                })()
                .into()
            }
        }

        /// Enables or disables the account of `identity` in `directory`.
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize, $crate::facade::WorthQueryAction)]
        #[worth_query(
            schema = $Schema, context = $Context, namespace = STATUS_NAMESPACE,
            input = SetPrincipalStatusInput, operation = SetPrincipalStatusOperation,
            binding = SetPrincipalStatusBinding, delivery = PrincipalStatusSet,
            result = UnitApplicationValueBinding,
            decision = $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, PrincipalAccount>,
            program,
            access(
                read_entities: [$Directory, PrincipalAccount];
                read_fields: [$DirectoryFacts::$directory_id(),
                    PrincipalAccountFacts::external_identity(), PrincipalAccountFacts::status()];
                writes: [PrincipalAccountFacts::status()];
            )
        )]
        pub struct SetPrincipalStatus {
            #[worth_query(scope = $DirectoryKey)]
            pub directory: <$DirectoryKey as RecordSelection>::Value,
            pub identity: WorthQueryExternalPrincipalIdentity,
            pub enabled: bool,
        }

        /// Assigns one account's status.
        pub struct SetPrincipalStatusHandler;
        impl $crate::facade::primary_graph::OperationHandler<$Schema, SetPrincipalStatusBinding>
            for SetPrincipalStatusHandler
        {
            fn decide(
                &self,
                input: &SetPrincipalStatus,
                reader: &mut $crate::facade::primary_graph::DecisionReader<'_, '_, '_, $Schema, SetPrincipalStatusBinding>,
            ) -> RegistryResult<$crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, PrincipalAccount>> {
                use $crate::facade::principal_registry::WorthQueryPrincipalRegistryRefusal as Refusal;
                let account = match reader.resolve_optional_entity(
                    PrincipalAccountFacts::external_identity(),
                    input.identity.clone(),
                ) {
                    ::core::result::Result::Ok(::core::option::Option::Some(account)) => account,
                    ::core::result::Result::Ok(::core::option::Option::None) => {
                        return Refusal::IdentityNotOnboarded.into_handler_result();
                    }
                    ::core::result::Result::Err(cause) => {
                        return $crate::facade::primary_graph::HandlerResult::ExecutionDenied(cause);
                    }
                };
                (|| {
                    reader.field(&account, PrincipalAccountFacts::status())?;
                    reader.mutation_target(&account)
                })()
                .into()
            }

            fn build_candidate(
                &self,
                input: &SetPrincipalStatus,
                account: $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, PrincipalAccount>,
                writer: &mut $crate::facade::primary_graph::CandidateWriter<'_, $Schema, SetPrincipalStatusBinding>,
            ) -> RegistryResult<()> {
                (|| -> ::core::result::Result<(), RegistryError> {
                    let account = writer.projected_entity(&account)?;
                    let status = if input.enabled {
                        WorthQueryPrincipalMappingStatus::Enabled
                    } else {
                        WorthQueryPrincipalMappingStatus::Disabled
                    };
                    writer.write_field(&account, PrincipalAccountFacts::status(), status)?;
                    ::core::result::Result::Ok(())
                })()
                .into()
            }
        }
    };
}
