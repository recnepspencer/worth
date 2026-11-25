//! Grant and revoke actions lowered by `worth_query_scope_access!`.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_scope_access_actions {
    ($Schema:ident, $Context:ident, $Scope:ident, $ScopeKey:ident, $ScopeFacts:ident :: $scope_key:ident,
     $Principal:ident, $PrincipalFacts:ident :: $principal_id:ident,
     $Mapping:ident, $IdentityFacts:ident :: $mapping_identity:ident,
     $StatusFacts:ident :: $mapping_status:ident, $MappingTarget:ident) => {
        const GRANT_NAMESPACE: $crate::facade::declaration::application_identity::ApplicationIdentityNamespace =
            $crate::facade::worth_query_identity_namespace!("worth.scope_access.grant.v1");
        const REVOKE_NAMESPACE: $crate::facade::declaration::application_identity::ApplicationIdentityNamespace =
            $crate::facade::worth_query_identity_namespace!("worth.scope_access.revoke.v1");

        /// Grants `grantee` typed `access` to one scope.
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize, $crate::facade::WorthQueryAction)]
        #[worth_query(
            schema = $Schema, context = $Context, namespace = GRANT_NAMESPACE,
            input = GrantScopeAccessInput, operation = GrantScopeAccessOperation,
            binding = GrantScopeAccessBinding, delivery = ScopeAccessGranted,
            result = UnitApplicationValueBinding, decision = ScopeGrantDecision, program,
            access(
                read_entities: [$Principal, $Mapping, $Scope];
                read_fields: [$PrincipalFacts::$principal_id(), $IdentityFacts::$mapping_identity(),
                    $StatusFacts::$mapping_status(), $ScopeFacts::$scope_key()];
                read_relations: [$MappingTarget];
                writes: [ScopeGrantFacts::grant_id(), ScopeGrantFacts::action(),
                    ScopeGrantFacts::purpose(), ScopeGrantFacts::disclosure(),
                    ScopeGrantFacts::access(), ScopeGrantFacts::active(),
                    ScopeGrantFacts::workflow(), ScopeGrantFacts::not_before(),
                    ScopeGrantFacts::not_after(), ScopeGrantFacts::delegation_limit()];
                creates: [ScopeGrant];
                links: [ScopeGrantor, ScopeGrantee, ScopeGrantResource];
            )
        )]
        pub struct GrantScopeAccess {
            #[worth_query(scope = $ScopeKey)]
            pub scope: ::std::string::String,
            pub grant_id: ::std::string::String,
            pub grantee: $crate::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity,
            pub access: ScopeAccess,
        }

        /// The grantor, grantee and scope a grant links.
        pub struct ScopeGrantDecision {
            grantor: $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, $Principal>,
            grantee: $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, $Principal>,
            scope: $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, $Scope>,
        }

        /// Registers a grant from the requesting principal to an enabled,
        /// mapped grantee.
        pub struct ScopeGrantHandler;
        impl $crate::facade::primary_graph::OperationHandler<$Schema, GrantScopeAccessBinding> for ScopeGrantHandler {
            fn decide(
                &self,
                input: &GrantScopeAccess,
                reader: &mut $crate::facade::primary_graph::DecisionReader<'_, '_, '_, $Schema, GrantScopeAccessBinding>,
            ) -> $crate::facade::primary_graph::HandlerResult<
                ScopeGrantDecision,
                $crate::facade::primary_graph::WorthQueryOperationBodyDenial,
            > {
                use $crate::facade::{
                    declaration::authentication::WorthQueryPrincipalMappingStatus,
                    primary_graph::HandlerExecutionDenial,
                    shared_access::WorthQueryScopeGrantRefusal as Refusal,
                };
                (|| -> ::core::result::Result<ScopeGrantDecision, HandlerExecutionDenial> {
                    let requester = reader.principal_identity().clone();
                    let grantor = reader.resolve_entity($PrincipalFacts::$principal_id(), requester)?;
                    let mapping = reader.resolve_entity($IdentityFacts::$mapping_identity(), input.grantee.clone())?;
                    if reader.field(&mapping, $StatusFacts::$mapping_status())?
                        != ::core::option::Option::Some(WorthQueryPrincipalMappingStatus::Enabled)
                    {
                        return ::core::result::Result::Err(HandlerExecutionDenial::new(Refusal::GranteeMappingDisabled));
                    }
                    let grantee = reader
                        .relations_from($MappingTarget::reference(), &mapping)?
                        .pop()
                        .ok_or_else(|| HandlerExecutionDenial::new(Refusal::GranteeUnmapped))?
                        .into_to();
                    let scope = reader.resolve_entity($ScopeFacts::$scope_key(), input.scope.clone())?;
                    ::core::result::Result::Ok(ScopeGrantDecision {
                        grantor: reader.mutation_target(&grantor)?,
                        grantee: reader.mutation_target(&grantee)?,
                        scope: reader.mutation_target(&scope)?,
                    })
                })()
                .into()
            }

            fn build_candidate(
                &self,
                input: &GrantScopeAccess,
                decision: ScopeGrantDecision,
                writer: &mut $crate::facade::primary_graph::CandidateWriter<'_, $Schema, GrantScopeAccessBinding>,
            ) -> $crate::facade::primary_graph::HandlerResult<(), $crate::facade::primary_graph::WorthQueryOperationBodyDenial> {
                use $crate::facade::{
                    primary_graph::{HandlerExecutionDenial, WorthQueryApplicationEntityKey},
                    shared_access::WorthQueryScopeGrantRefusal as Refusal,
                };
                (|| -> ::core::result::Result<(), HandlerExecutionDenial> {
                    let grantor = writer.projected_entity(&decision.grantor)?;
                    let grantee = writer.projected_entity(&decision.grantee)?;
                    let scope = writer.projected_entity(&decision.scope)?;
                    let key = WorthQueryApplicationEntityKey::new(input.grant_id.clone())
                        .map_err(|_| HandlerExecutionDenial::new(Refusal::InvalidGrantKey))?;
                    let grant = writer.create_entity_in_context(&scope, ScopeGrant::reference(), key)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::grant_id(), input.grant_id.clone())?;
                    writer.initialize_field(&grant, ScopeGrantFacts::action(), ScopeGrantAction::Access)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::purpose(), ScopeGrantPurpose::Shared)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::disclosure(), ScopeGrantDisclosure::ScopeView)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::access(), input.access)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::active(), true)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::workflow(), input.scope.clone())?;
                    writer.initialize_field(&grant, ScopeGrantFacts::not_before(), 0_u64)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::not_after(), u64::MAX)?;
                    writer.initialize_field(&grant, ScopeGrantFacts::delegation_limit(), 0_u64)?;
                    let id = &input.grant_id;
                    writer.link(ScopeGrantor::reference(), ::std::format!("grantor-{id}"), &grantor, &grant)?;
                    writer.link(ScopeGrantee::reference(), ::std::format!("grantee-{id}"), &grantee, &grant)?;
                    writer.link(ScopeGrantResource::reference(), ::std::format!("resource-{id}"), &grant, &scope)?;
                    ::core::result::Result::Ok(())
                })()
                .into()
            }
        }

        /// Deactivates one grant immediately and durably.
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize, $crate::facade::WorthQueryAction)]
        #[worth_query(
            schema = $Schema, context = $Context, namespace = REVOKE_NAMESPACE,
            input = RevokeScopeAccessInput, operation = RevokeScopeAccessOperation,
            binding = RevokeScopeAccessBinding, delivery = ScopeAccessRevoked,
            result = UnitApplicationValueBinding,
            decision = $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, ScopeGrant>,
            program,
            access(
                read_entities: [ScopeGrant];
                read_fields: [ScopeGrantFacts::grant_id(), ScopeGrantFacts::active()];
                writes: [ScopeGrantFacts::active()];
            )
        )]
        pub struct RevokeScopeAccess {
            #[worth_query(scope = GrantId)]
            pub grant_id: ::std::string::String,
        }

        /// Assigns one grant inactive.
        pub struct ScopeRevokeHandler;
        impl $crate::facade::primary_graph::OperationHandler<$Schema, RevokeScopeAccessBinding> for ScopeRevokeHandler {
            fn decide(
                &self,
                input: &RevokeScopeAccess,
                reader: &mut $crate::facade::primary_graph::DecisionReader<'_, '_, '_, $Schema, RevokeScopeAccessBinding>,
            ) -> $crate::facade::primary_graph::HandlerResult<
                $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, ScopeGrant>,
                $crate::facade::primary_graph::WorthQueryOperationBodyDenial,
            > {
                (|| {
                    let grant = reader.resolve_entity(ScopeGrantFacts::grant_id(), input.grant_id.clone())?;
                    reader.field(&grant, ScopeGrantFacts::active())?;
                    reader.mutation_target(&grant)
                })()
                .into()
            }

            fn build_candidate(
                &self,
                _: &RevokeScopeAccess,
                grant: $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, ScopeGrant>,
                writer: &mut $crate::facade::primary_graph::CandidateWriter<'_, $Schema, RevokeScopeAccessBinding>,
            ) -> $crate::facade::primary_graph::HandlerResult<(), $crate::facade::primary_graph::WorthQueryOperationBodyDenial> {
                (|| -> ::core::result::Result<(), $crate::facade::primary_graph::HandlerExecutionDenial> {
                    let grant = writer.projected_entity(&grant)?;
                    writer.write_field(&grant, ScopeGrantFacts::active(), false)?;
                    ::core::result::Result::Ok(())
                })()
                .into()
            }
        }
    };
}
