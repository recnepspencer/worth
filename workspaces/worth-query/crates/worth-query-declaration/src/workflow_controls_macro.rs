//! `worth_query_workflow_controls!`: a workflow's control capabilities and
//! their `WORKFLOW_CONTROL` bindings from one declaration.

/// Declares the control steps of one workflow (authoring, instance start,
/// advance and each approval) that share an input, scope and grant target.
///
/// ```text
/// worth_query_workflow_controls! {
///     schema: BankSchema,
///     binding_context: BankBindingContext,
///     input: ApprovePayment => ApprovePaymentInputBinding,
///     result: PaymentDecisionResult => PaymentDecisionResultBinding,
///     denial: BankProposalDenial => PaymentDecisionDenialBinding,
///     scope: PaymentIntent, field PaymentIdentityField = PaymentIdentityField::at("PaymentIdentityField"), input.payment,
///     capability_context: pub ApprovedBusinessPaymentControlContext = "worth.bank.approved-business-payment-control-context.v1",
///     grant: "manage-approved-business-payment-workflow" for "business-payment-approval",
///     controls: [
///         pub ApprovedBusinessPaymentApproval = "worth.bank.approved-business-payment-approval" {
///             ApprovedBusinessPaymentApprovalOperation,
///             ApprovedBusinessPaymentApprovalBinding,
///             ApprovedBusinessPaymentApprovalIntent,
///         },
///     ],
/// }
/// ```
///
/// For each control `Capability = "identity"` it declares the capability
/// marker (portable identity `"identity.v1"`), the control operation, a
/// mutation binding with `WORKFLOW_CONTROL = true` and identities
/// `"identity.binding.v1"`, `"identity.handler.v1"` and `"identity.command.v1"`,
/// its intent, and the input's capability request. Principal and idempotency
/// types come from `binding_context`. The capability request names the same
/// action and purpose the grant target installs, so a contract and its
/// request cannot disagree. The capability contracts themselves remain
/// explicit: install each with `schema.capability(..)`.
///
/// The grant may instead be typed, in the grant record's own field types
/// (a `WorthQueryVocabulary` or any scalar), and two optional sections follow
/// it:
///
/// ```text
///     grant: {
///         action: ControlGrantFacts::action() => ControlAction::Manage,
///         purpose: ControlGrantFacts::purpose() => ControlPurpose::Approval,
///     },
///     context_slots: [ControlSite = SiteFacts::site_id(), input.site],
///     reads: [PaymentFacts::payment_id(), PaymentFacts::amount()],
/// ```
///
/// A typed grant also gives the context `grant_action()` and
/// `grant_purpose()`, the contract target values built from the same typed
/// values the requests encode. Each `context_slots` entry selects the slot's
/// entity by an input field; `reads` gives the context
/// `declare_control_reads(schema)`, declaring every control operation's read
/// fields in one call.
#[macro_export]
macro_rules! worth_query_workflow_controls {
    (
        schema: $Schema:ty,
        binding_context: $Context:ty,
        input: $Input:ty => $InputBinding:ty,
        result: $Result:ty => $ResultBinding:ty,
        denial: $Denial:ty => $DenialBinding:ty,
        scope: $Scope:ty, field $ScopeField:ty = $scope_field:expr, input.$scope_value:ident,
        capability_context: $context_vis:vis $CapabilityContext:ident = $context_identity:literal,
        grant: $action:literal for $purpose:literal,
        $(context_slots: $slots:tt,)?
        $(reads: $reads:tt,)?
        controls: $controls:tt $(,)?
    ) => {
        $crate::__worth_query_workflow_controls_items! {
            $Schema,
            ($Schema, $Context, $Input, $InputBinding, $Result, $ResultBinding, $Denial,
                $DenialBinding, $Scope, $ScopeField, ($scope_field), $scope_value),
            $context_vis $CapabilityContext = $context_identity,
            action: ($crate::__worth_query_control_text!($action, "action")),
            purpose: ($crate::__worth_query_control_text!($purpose, "purpose")),
            slots: [$($slots)?], reads: [$($reads)?], controls: $controls
        }
    };
    (
        schema: $Schema:ty,
        binding_context: $Context:ty,
        input: $Input:ty => $InputBinding:ty,
        result: $Result:ty => $ResultBinding:ty,
        denial: $Denial:ty => $DenialBinding:ty,
        scope: $Scope:ty, field $ScopeField:ty = $scope_field:expr, input.$scope_value:ident,
        capability_context: $context_vis:vis $CapabilityContext:ident = $context_identity:literal,
        grant: {
            action: $action_field:expr => $action:expr,
            purpose: $purpose_field:expr => $purpose:expr $(,)?
        },
        $(context_slots: $slots:tt,)?
        $(reads: $reads:tt,)?
        controls: $controls:tt $(,)?
    ) => {
        impl $CapabilityContext {
            /// The contract's action target: the value every control requests.
            pub fn grant_action() -> ::core::result::Result<
                $crate::facade::application_capability::ApplicationCapabilityValueBinding,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                $crate::facade::application_capability::application_capability_grant_value(
                    $action_field, $action,
                )
            }

            /// The contract's purpose target: the value every control requests.
            pub fn grant_purpose() -> ::core::result::Result<
                $crate::facade::application_capability::ApplicationCapabilityValueBinding,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                $crate::facade::application_capability::application_capability_grant_value(
                    $purpose_field, $purpose,
                )
            }
        }
        $crate::__worth_query_workflow_controls_items! {
            $Schema,
            ($Schema, $Context, $Input, $InputBinding, $Result, $ResultBinding, $Denial,
                $DenialBinding, $Scope, $ScopeField, ($scope_field), $scope_value),
            $context_vis $CapabilityContext = $context_identity,
            action: ($crate::facade::application_capability::application_capability_request_value(
                &$action_field, $action,
            )),
            purpose: ($crate::facade::application_capability::application_capability_request_value(
                &$purpose_field, $purpose,
            )),
            slots: [$($slots)?], reads: [$($reads)?], controls: $controls
        }
    };
}

/// Text grant values, encoded as the literal form always has.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_control_text {
    ($value:literal, $subject:literal) => {
        $crate::facade::application_schema::ApplicationEncodedScalarValue::<
            $crate::facade::application_schema::StringApplicationValueBinding,
        >::try_new($value.to_owned())
        .map_err(|_| {
            $crate::facade::application_capability::ApplicationCapabilityRequestProjectionDenial::input_variant(
                $subject,
            )
        })
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_workflow_controls_items {
    (
        $Schema:ty,
        $shared:tt,
        $context_vis:vis $CapabilityContext:ident = $context_identity:literal,
        action: $action:tt, purpose: $purpose:tt,
        slots: $slots:tt, reads: $reads:tt,
        controls: [$(
            $vis:vis $Capability:ident = $identity:literal {
                $Operation:ident, $Binding:ident, $Intent:ident $(,)?
            }
        ),+ $(,)?]
    ) => {
        $crate::worth_query_capability_context!(
            $context_vis $CapabilityContext in $Schema, identity $context_identity
        );
        $(
            $crate::worth_query_capability!($vis $Capability in $Schema, identity concat!($identity, ".v1"));
            $crate::__worth_query_workflow_control! {
                $shared, $CapabilityContext, $action, $purpose, $slots, $reads,
                $vis, $Capability, $identity, $Operation, $Binding, $Intent
            }
        )+
        impl $CapabilityContext {
            /// Declares every control operation's read fields.
            pub fn declare_control_reads(
                schema: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                $(let schema = $Binding::declare_reads(schema);)+
                schema
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_workflow_control {
    (
        ($Schema:ty, $Context:ty, $Input:ty, $InputBinding:ty, $Result:ty, $ResultBinding:ty,
            $Denial:ty, $DenialBinding:ty, $Scope:ty, $ScopeField:ty, ($scope_field:expr),
            $scope_value:ident),
        $CapabilityContext:ident, ($action:expr), ($purpose:expr),
        [$([$($Slot:ident = $slot_field:expr, input.$slot_value:ident);* $(;)?])?],
        [$([$($read:expr),* $(,)?])?],
        $vis:vis, $Capability:ident, $identity:literal, $Operation:ident, $Binding:ident,
        $Intent:ident
    ) => {
        $crate::worth_query_operation!($vis $Operation for $Schema, input $InputBinding);

        impl $Binding {
            /// Declares this control operation's read fields.
            pub fn declare_reads(
                schema: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                schema $($(.operation_read_field($Operation::reference(), $read))*)?
            }
        }

        $vis struct $Binding;

        impl $crate::facade::application_operation::ApplicationMutationBinding<$Schema> for $Binding {
            type Input = $Input;
            type InputBinding = $InputBinding;
            type Result = $Result;
            type ResultBinding = $ResultBinding;
            type IdempotencyKey =
                <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::IdempotencyKey;
            type Operation = $Operation;
            type Decision = ();
            type Denial = $Denial;
            type DenialBinding = $DenialBinding;
            type Output = $crate::facade::application_operation::NoApplicationMutationOutputs;
            type ScopeBinding = $crate::facade::application_operation::ApplicationMutationFieldScope<$ScopeField>;
            type PrincipalBinding =
                <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::PrincipalBinding;
            type Mapping =
                <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::Mapping;
            type Principal =
                <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::Principal;
            type PrincipalIdentity =
                <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::PrincipalIdentity;
            type PrincipalIdentityBinding =
                <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::PrincipalIdentityBinding;
            type SourceExpectation = $crate::facade::application_operation::NoApplicationMutationSource;

            const IDENTITY: &'static str = concat!($identity, ".binding.v1");
            const HANDLER_IDENTITY: &'static str = concat!($identity, ".handler.v1");
            const IDEMPOTENCY_IDENTITY: &'static str = concat!($identity, ".command.v1");
            const REQUIRES_APPLICATION_PROGRAM: bool = true;
            const WORKFLOW_CONTROL: bool = true;

            fn workflow_execution_requirements(
            ) -> $crate::facade::application_operation::ApplicationWorkflowExecutionRequirements {
                $crate::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
            }

            fn scope_field() -> $ScopeField {
                $scope_field
            }

            fn principal_binding() -> $crate::facade::application_schema::ApplicationPrincipalBindingRef<
                $Schema,
                Self::PrincipalBinding,
                Self::Mapping,
                Self::Principal,
                Self::PrincipalIdentity,
                Self::PrincipalIdentityBinding,
            > {
                <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::principal_binding()
            }
        }

        impl $crate::facade::application_operation::ApplicationMutationAdmissionBinding<$Schema> for $Binding {
            type Admission = $crate::facade::application_operation::ApplicationCapabilityAdmission<$Capability>;
        }

        impl $crate::facade::application_operation::ApplicationCapabilityMutationBinding<$Schema> for $Binding {
            type Capability = $Capability;
        }

        #[derive(Clone, Debug, Eq, PartialEq)]
        $vis struct $Intent {
            pub input: $Input,
        }

        impl $crate::facade::application_operation::ApplicationMutationIntent<$Schema> for $Intent {
            type Binding = $Binding;

            fn input(&self) -> &$Input {
                &self.input
            }

            fn scope_binding(
                &self,
            ) -> $crate::facade::application_operation::ApplicationMutationFieldScope<$ScopeField> {
                $crate::facade::application_operation::ApplicationMutationFieldScope::new(
                    $scope_field,
                    ::core::clone::Clone::clone(&self.input.$scope_value),
                )
            }
        }

        impl $crate::facade::application_capability::ApplicationCapabilityRequest<$Schema, $Capability>
            for $Input
        {
            type Scope = $Scope;
            type Context = $CapabilityContext;

            fn capability_request(
                &self,
            ) -> ::core::result::Result<
                $crate::facade::application_capability::ApplicationCapabilityRequestProjection<
                    $Schema,
                    Self::Scope,
                    Self::Context,
                >,
                $crate::facade::application_capability::ApplicationCapabilityRequestProjectionDenial,
            > {
                use $crate::facade::application_capability::{
                    ApplicationCapabilityEntitySelector as Selector,
                    ApplicationCapabilityRequestContext as RequestContext,
                    ApplicationCapabilityRequestProjection as Projection,
                    ApplicationCapabilityRequestProjectionDenial as Refusal,
                };
                use $crate::facade::application_schema::{
                    fragment::RecordSelection, ApplicationEncodedScalarValue as Encoded,
                };
                let refused = |subject: &'static str| move |_| Refusal::input_variant(subject);
                let scope = Encoded::<<$ScopeField as RecordSelection>::Binding>::try_new(
                    ::core::clone::Clone::clone(&self.$scope_value),
                )
                .map_err(refused(stringify!($scope_value)))?;
                let context = RequestContext::new($CapabilityContext::reference());
                $($(
                    let context = context.entity(
                        $Slot::reference(),
                        $crate::facade::application_capability::application_capability_selector(
                            $slot_field,
                            ::core::clone::Clone::clone(&self.$slot_value),
                        )?,
                    );
                )*)?
                Ok(Projection::new(Selector::new($scope_field, scope)?, $action?, $purpose?, context))
            }
        }
    };
}
