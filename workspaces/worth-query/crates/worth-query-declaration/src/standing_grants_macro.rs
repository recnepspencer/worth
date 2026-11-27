//! `worth_query_standing_grants!`: a schema's standing role policy, written
//! as one compact table over its existing typed grant record.

/// Declares standing roles over one scope entity in module `module`.
///
/// ```text
/// worth_query_standing_grants! {
///     pub mod site_roles for QmsSchema {
///         principal: Member,
///         request: SiteRequest => SiteRequestInput, input.site,
///         scope: Site = SiteFacts::site_id(),
///         context: SiteContext,
///         provenance: SiteRoleProvenance,
///         grant: RoleGrant {
///             holder: RoleHolder,
///             resource: GrantSite,
///             role: RoleGrantFacts::role(),
///             purpose: RoleGrantFacts::purpose() => Purpose::Quality,
///             active: RoleGrantFacts::grant_status() => GrantStatus::Active,
///         },
///         require_active: [SiteFacts::site_status() => SiteStatus::Active],
///         work_budget: 64,
///         roles: [
///             pub Review / ReviewOp = Role::Review, utc(RoleGrantFacts::valid_from(), RoleGrantFacts::valid_until());
///             pub Approve / ApproveOp = Role::Approve, local_date(RoleGrantFacts::first_day(), RoleGrantFacts::last_day());
///             pub Observe / ObserveOp = Role::Observe, unbounded(), allow through_assignment();
///         ],
///     }
/// }
/// ```
///
/// Each role names its capability marker, its operation (input `request`),
/// the role value its grant's `role` field carries (a typed vocabulary or
/// text) and its validity window, any constructor of
/// `ApplicationCapabilityValidityDefinition` (`utc`, `local_date`, `from`,
/// `until`, `unbounded`). A role is admitted when the principal holds an
/// active grant of that role over the scope (`holder` then `resource`), inside
/// its window, every `require_active` extent on the way is active, and its
/// composition allows: by default the grant path itself (`through_grant()`),
/// or the `allow` path given. Grants are standing: no workflow, lineage,
/// grantor or delegation limit, delegation forbidden, and no operation may
/// link its own principal as a holder.
///
/// The module holds the context and provenance markers, every role's
/// capability and operation markers, the request projection each role reads
/// from `request` (the same typed values its contract installs) and
/// `members(schema)`, which installs every operation (pure, `work_budget`,
/// reading the scope field) and standing contract. Grant rows are written by
/// the application's own handlers or seeds, onto the record named here;
/// revocation is a status write or a deactivated extent, never a deletion.
#[macro_export]
macro_rules! worth_query_standing_grants {
    (
        $vis:vis mod $module:ident for $Schema:ty {
            principal: $Principal:ident,
            request: $Request:ty => $RequestBinding:ty, input.$scope_value:ident,
            scope: $Scope:ident = $scope_field:expr,
            context: $Context:ident,
            provenance: $Provenance:ident,
            grant: $Grant:ident {
                holder: $Holder:ident,
                resource: $Resource:ident,
                role: $role_field:expr,
                purpose: $purpose_field:expr => $purpose:expr,
                active: $active_field:expr => $active:expr $(,)?
            },
            require_active: [$($extent_field:expr => $extent:expr),* $(,)?],
            work_budget: $budget:expr,
            roles: [$(
                $role_vis:vis $Capability:ident / $Operation:ident = $role:expr,
                $validity:ident($($bound:expr),* $(,)?) $(, allow $allow:expr)?
            );+ $(;)?] $(,)?
        }
    ) => {
        $vis mod $module {
            use super::*;
            use $crate::facade::application_capability as capability;
            use $crate::facade::application_schema as schema;

            $crate::worth_query_capability_context!(pub $Context in $Schema);
            $crate::worth_query_capability_provenance!(pub $Provenance in $Schema);
            $(
                $crate::worth_query_capability!($role_vis $Capability in $Schema);
                $crate::worth_query_operation!($role_vis $Operation for $Schema, input $RequestBinding);

                impl capability::ApplicationCapabilityRequest<$Schema, $Capability> for $Request {
                    type Scope = $Scope;
                    type Context = $Context;
                    fn capability_request(
                        &self,
                    ) -> ::core::result::Result<
                        $crate::facade::application_capability::ApplicationCapabilityRequestProjection<$Schema, $Scope, $Context>,
                        $crate::facade::application_capability::ApplicationCapabilityRequestProjectionDenial,
                    > {
                        ::core::result::Result::Ok(capability::ApplicationCapabilityRequestProjection::new(
                            capability::application_capability_selector(
                                $scope_field,
                                ::core::clone::Clone::clone(&self.$scope_value),
                            )?,
                            capability::application_capability_request_value(&$role_field, $role)?,
                            capability::application_capability_request_value(&$purpose_field, $purpose)?,
                            $crate::facade::application_capability::ApplicationCapabilityRequestContext::new($Context::reference()),
                        ))
                    }
                }
            )+

            /// The principal reaches the scope through a grant it holds.
            pub fn through_grant() -> schema::ApplicationAuthorizationPath {
                schema::ApplicationAuthorizationPathBuilder::from_principal($Principal::reference())
                    .forward($Holder::reference())
                    .forward($Resource::reference())
                    .allow($Scope::reference())
            }

            /// Every role's operation and standing contract.
            pub fn members(
                schema: schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> ::core::result::Result<
                schema::ApplicationSchemaDeclarationBuilder<$Schema>,
                schema::ApplicationSchemaDeclarationDenial,
            > {
                let mut schema = schema
                    .capability_context($Context::reference())
                    .capability_provenance($Provenance::reference());
                $(
                    let operation = $Operation::reference();
                    let validity = capability::ApplicationCapabilityValidityDeclaration::into_validity(
                        $crate::facade::application_capability::ApplicationCapabilityValidityDefinition::$validity($($bound),*),
                    )?;
                    let role = capability::application_capability_grant_value($role_field, $role)?;
                    let allow = $crate::__worth_query_standing_allow!($($allow)?);
                    schema = schema
                        .operation(operation.definition().no_external_effect().no_aftermath().finish())
                        .operation_projection_work_budget(operation, $budget)
                        .operation_read_field(operation, $scope_field)
                        .capability(standing_contract($Capability::reference(), operation, role, validity, allow)?);
                )+
                ::core::result::Result::Ok(schema)
            }

            fn standing_contract<Cap, Op>(
                capability: capability::ApplicationCapabilityRef<$Schema, Cap>,
                operation: schema::ApplicationOperationRef<$Schema, Op, $Request>,
                role: capability::ApplicationCapabilityValueBinding,
                validity: capability::ApplicationCapabilityValidityDefinition,
                allow: schema::ApplicationAuthorizationPath,
            ) -> ::core::result::Result<
                $crate::facade::application_capability::ApplicationCapabilityContract<$Schema, Cap, Op, $Request>,
                schema::ApplicationSchemaDeclarationDenial,
            >
            where
                Cap: capability::ApplicationCapabilityMarkerIdentity<Schema = $Schema> + 'static,
                Op: schema::ApplicationOperationMarkerIdentity<$Schema, InputBinding = $RequestBinding>
                    + 'static,
                $Request: capability::ApplicationCapabilityRequest<$Schema, Cap>,
            {
                let active = capability::application_capability_grant_value($active_field, $active)?;
                let currentness = capability::ApplicationCapabilityCurrentnessDefinition::standing(active, validity);
                $(
                    let currentness = currentness.require_active(
                        capability::application_capability_grant_value($extent_field, $extent)?,
                    );
                )*
                let target = capability::ApplicationCapabilityTargetDefinition::new(
                    role,
                    $crate::facade::application_capability::ApplicationCapabilityRelationBinding::from_reference($Resource::reference()),
                    $crate::facade::application_capability::ApplicationCapabilityRelationDimension::not_applicable(),
                    $crate::facade::application_capability::ApplicationCapabilityFieldDimension::not_applicable(),
                    capability::application_capability_grant_value($purpose_field, $purpose)?,
                );
                ::core::result::Result::Ok(
                    $crate::facade::application_capability::ApplicationCapabilityContractBuilder::new(capability, operation, $Grant::reference())
                        .target(target)
                        .constraints(capability::ApplicationCapabilityConstraintDefinition::new(
                            $crate::facade::application_capability::ApplicationCapabilityMagnitudeDimension::not_applicable(),
                            $crate::facade::application_capability::ApplicationCapabilityCardinalityDimension::One,
                            currentness,
                            $Context::reference(),
                        ))
                        .delegation(
                            $crate::facade::application_capability::ApplicationCapabilityDelegationDefinition::standing(
                                $crate::facade::application_capability::ApplicationCapabilityRelationBinding::from_reference(
                                    $Holder::reference(),
                                ),
                                $Provenance::reference(),
                            )
                            .forbid_self_grant(),
                        )
                        .composition($crate::__worth_query_standing_composition!(allow))
                        .elevation(capability::ApplicationCapabilityElevationRule::not_applicable())
                        .build(),
                )
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_standing_allow {
    () => {
        through_grant()
    };
    ($allow:expr) => {
        $allow
    };
}

/// The standing composition: allow along one path; no deny, conflict,
/// actor or disclosure rule; delegation forbidden.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_standing_composition {
    ($allow:expr) => {
        $crate::facade::application_capability::ApplicationCapabilityComposition::new(
            $crate::facade::application_capability::ApplicationCapabilityDecisionComposition::new(
                $crate::facade::application_capability::ApplicationCapabilityAllowRule::new(
                    $crate::facade::application_capability::ApplicationCapabilityGraphRule::any([
                        $crate::facade::application_capability::ApplicationCapabilityGraphClause::new($allow),
                    ]),
                ),
                $crate::facade::application_capability::ApplicationCapabilityDenyRule::not_applicable(),
                $crate::facade::application_capability::ApplicationCapabilityConflictRule::not_applicable(),
            ),
            $crate::facade::application_capability::ApplicationCapabilityActorComposition::new(
                $crate::facade::application_capability::ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
                $crate::facade::application_capability::ApplicationCapabilityDistinctActorRule::not_applicable(),
                $crate::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable(),
            ),
            $crate::facade::application_capability::ApplicationCapabilityPropagationComposition::new(
                $crate::facade::application_capability::ApplicationCapabilityDelegationRule::forbidden(),
                $crate::facade::application_capability::ApplicationCapabilityDisclosureRule::not_applicable(),
            ),
        )
    };
}
