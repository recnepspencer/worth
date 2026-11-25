//! Capability contracts, read/watch operations and governed disclosure
//! lowered by `worth_query_scope_access!`.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_scope_access_contracts {
    ($Schema:ident, $Scope:ident, $Principal:ident, $ScopeFacts:ident :: $scope_key:ident,
     [$($ReadFacts:ident :: $read:ident),+]) => {
        $crate::facade::worth_query_capability_context!(pub ScopeAccessContext in $Schema);
        $crate::facade::worth_query_capability_provenance!(pub ScopeAccessProvenance in $Schema);
        $crate::facade::worth_query_capability!(pub ScopeReadCapability in $Schema);
        $crate::facade::worth_query_capability!(pub ScopeWatchCapability in $Schema);
        $crate::facade::worth_query_structured_value_binding!(pub ScopeCapabilityInput for ::std::string::String {
            identity: "worth.scope_access.capability_input.v1"
        });
        $crate::facade::worth_query_operation!(pub ScopeReadOperation for $Schema, input ScopeCapabilityInput);
        $crate::facade::worth_query_operation!(pub ScopeWatchOperation for $Schema, input ScopeCapabilityInput);
        $crate::facade::worth_query_disclosure_classification!(
            /// The classification of every query a stock scope grant discloses.
            pub ScopeAccessDisclosure
        );

        /// The governed disclosure of a shared scope query: add each disclosed
        /// field or relation with [`scope_view`].
        pub fn disclosure() -> $crate::facade::declaration::application_query::ApplicationQueryDisclosureContract {
            $crate::facade::declaration::application_query::ApplicationQueryDisclosureContract::governed_by(
                ScopeAccessDisclosure,
                ScopeReadCapability::reference(),
            )
        }

        /// The disclosure value every stock read grant carries.
        pub fn scope_view() -> ApplicationEncodedScalarValue<ScopeGrantDisclosureBinding> {
            $crate::facade::shared_access::scope_grant_value(ScopeGrantDisclosure::ScopeView)
        }

        fn scope_request(
            scope_key: &::std::string::String,
            dimension: ::core::option::Option<ScopeAccess>,
        ) -> ::core::result::Result<
            ApplicationCapabilityRequestProjection<$Schema, $Scope, ScopeAccessContext>,
            ApplicationCapabilityRequestProjectionDenial,
        > {
            let key = ApplicationEncodedScalarValue::try_new(scope_key.clone()).map_err(|_| {
                ApplicationCapabilityRequestProjectionDenial::input_variant("scope key")
            })?;
            let request = ApplicationCapabilityRequestProjection::new(
                ApplicationCapabilityEntitySelector::new($ScopeFacts::$scope_key(), key).map_err(|_| {
                    ApplicationCapabilityRequestProjectionDenial::input_variant("scope key")
                })?,
                $crate::facade::shared_access::scope_grant_value(ScopeGrantAction::Access),
                $crate::facade::shared_access::scope_grant_value(ScopeGrantPurpose::Shared),
                ApplicationCapabilityRequestContext::new(ScopeAccessContext::reference()),
            );
            ::core::result::Result::Ok(match dimension {
                ::core::option::Option::Some(access) => {
                    request.field($crate::facade::shared_access::scope_grant_value(access))
                }
                ::core::option::Option::None => request.field(scope_view()),
            })
        }

        impl ApplicationCapabilityRequest<$Schema, ScopeReadCapability> for ::std::string::String {
            type Scope = $Scope;
            type Context = ScopeAccessContext;
            fn capability_request(&self) -> ::core::result::Result<
                ApplicationCapabilityRequestProjection<$Schema, $Scope, ScopeAccessContext>,
                ApplicationCapabilityRequestProjectionDenial,
            > {
                scope_request(self, ::core::option::Option::None)
            }
        }

        impl ApplicationCapabilityRequest<$Schema, ScopeWatchCapability> for ::std::string::String {
            type Scope = $Scope;
            type Context = ScopeAccessContext;
            fn capability_request(&self) -> ::core::result::Result<
                ApplicationCapabilityRequestProjection<$Schema, $Scope, ScopeAccessContext>,
                ApplicationCapabilityRequestProjectionDenial,
            > {
                scope_request(self, ::core::option::Option::Some(ScopeAccess::ReadAndWatch))
            }
        }

        fn scope_capability<Cap, Op, Dimension>(
            capability: ApplicationCapabilityRef<$Schema, Cap>,
            operation: ApplicationOperationRef<$Schema, Op, ::std::string::String>,
            dimension: Dimension,
            accepted: ApplicationEncodedScalarValue<Dimension::Binding>,
        ) -> ::core::result::Result<
            ApplicationCapabilityContract<$Schema, Cap, Op, ::std::string::String>,
            ApplicationSchemaDeclarationDenial,
        >
        where
            Cap: ApplicationCapabilityMarkerIdentity + 'static,
            Op: ApplicationOperationMarkerIdentity<$Schema> + 'static,
            Op::InputBinding: ApplicationStructuredValueBinding<Value = ::std::string::String>,
            ::std::string::String: ApplicationCapabilityRequest<$Schema, Cap>,
            Dimension: RecordSelection<Entity = ScopeGrant> + Copy + 'static,
        {
            use $crate::facade::shared_access::scope_grant_value as value;
            let allow = ApplicationAuthorizationPathBuilder::from_principal($Principal::reference())
                .forward(ScopeGrantee::reference())
                .forward(ScopeGrantResource::reference())
                .allow($Scope::reference());
            fn field<Field: RecordSelection>(
                field: Field,
            ) -> ::core::result::Result<ApplicationCapabilityFieldBinding, ApplicationSchemaDeclarationDenial> {
                ApplicationCapabilityFieldBinding::from_reference(field)
            }
            fn relation<Relation, From, To>(
                relation: ApplicationRelationRef<$Schema, Relation, From, To>,
            ) -> ApplicationCapabilityRelationBinding {
                ApplicationCapabilityRelationBinding::from_reference(relation)
            }
            ::core::result::Result::Ok(ApplicationCapabilityContractBuilder::new(capability, operation, ScopeGrant::reference())
                .target(ApplicationCapabilityTargetDefinition::new(
                    ApplicationCapabilityValueBinding::new(ScopeGrantFacts::action(), value(ScopeGrantAction::Access))?,
                    relation(ScopeGrantResource::reference()),
                    ApplicationCapabilityRelationDimension::not_applicable(),
                    ApplicationCapabilityFieldDimension::bound(dimension)?,
                    ApplicationCapabilityValueBinding::new(ScopeGrantFacts::purpose(), value(ScopeGrantPurpose::Shared))?,
                ))
                .constraints(ApplicationCapabilityConstraintDefinition::new(
                    ApplicationCapabilityMagnitudeDimension::not_applicable(),
                    ApplicationCapabilityCardinalityDimension::One,
                    ApplicationCapabilityCurrentnessDefinition::new(
                        ApplicationCapabilityValueBinding::new(ScopeGrantFacts::active(), value(true))?,
                        ApplicationCapabilityWorkflowDefinition::new(
                            field(ScopeGrantFacts::workflow())?,
                            field($ScopeFacts::$scope_key())?,
                        ),
                        ApplicationCapabilityValidityDefinition::new(
                            ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                            field(ScopeGrantFacts::not_before())?,
                            field(ScopeGrantFacts::not_after())?,
                        ),
                    ),
                    ScopeAccessContext::reference(),
                ))
                .delegation(ApplicationCapabilityDelegationDefinition::new(
                    relation(ScopeGrantParent::reference()),
                    relation(ScopeGrantor::reference()),
                    relation(ScopeGrantee::reference()),
                    field(ScopeGrantFacts::delegation_limit())?,
                    ScopeAccessProvenance::reference(),
                ))
                .composition(ApplicationCapabilityComposition::new(
                    ApplicationCapabilityDecisionComposition::new(
                        ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                            ApplicationCapabilityGraphClause::new(allow),
                        ])),
                        ApplicationCapabilityDenyRule::not_applicable(),
                        ApplicationCapabilityConflictRule::not_applicable(),
                    ),
                    ApplicationCapabilityActorComposition::new(
                        ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
                        ApplicationCapabilityDistinctActorRule::not_applicable(),
                        ApplicationCapabilityRecordedActorRule::not_applicable(),
                    ),
                    ApplicationCapabilityPropagationComposition::new(
                        ApplicationCapabilityDelegationRule::forbidden(),
                        ApplicationCapabilityDisclosureRule::permit([
                            ApplicationCapabilityScopeGuard::requiring([
                                ApplicationCapabilityAcceptedValues::one_of(dimension, [accepted])?,
                            ]),
                        ]),
                    ),
                ))
                .elevation(ApplicationCapabilityElevationRule::not_applicable())
                .build())
        }

        /// Capability contexts, read/watch operations and their contracts.
        pub struct ScopeAccessRegistrations;
        impl Registration<$Schema> for ScopeAccessRegistrations {
            fn register(
                schema: ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> ::core::result::Result<ApplicationSchemaDeclarationBuilder<$Schema>, ApplicationSchemaDeclarationDenial> {
                fn pure<Op>(
                    operation: ApplicationOperationRef<$Schema, Op, ::std::string::String>,
                ) -> ApplicationOperationDefinition<$Schema, Op, ::std::string::String>
                where
                    Op: ApplicationOperationMarkerIdentity<$Schema>,
                    Op::InputBinding: ApplicationStructuredValueBinding<Value = ::std::string::String>,
                {
                    operation.definition().no_external_effect().no_aftermath().finish()
                }
                let read = ScopeReadOperation::reference();
                let watch = ScopeWatchOperation::reference();
                ::core::result::Result::Ok(schema
                    .capability_context(ScopeAccessContext::reference())
                    .capability_provenance(ScopeAccessProvenance::reference())
                    .operation(pure(read))
                    .operation_projection_work_budget(read, 64)
                    $(.operation_read_field(read, $ReadFacts::$read()))+
                    .operation(pure(watch))
                    .operation_projection_work_budget(watch, 64)
                    $(.operation_read_field(watch, $ReadFacts::$read()))+
                    .capability(scope_capability(
                        ScopeReadCapability::reference(),
                        read,
                        ScopeGrantFacts::disclosure(),
                        scope_view(),
                    )?)
                    .capability(scope_capability(
                        ScopeWatchCapability::reference(),
                        watch,
                        ScopeGrantFacts::access(),
                        $crate::facade::shared_access::scope_grant_value(ScopeAccess::ReadAndWatch),
                    )?))
            }
        }
    };
}
