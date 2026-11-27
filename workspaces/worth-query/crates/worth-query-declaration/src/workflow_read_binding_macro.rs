//! `worth_query_workflow_read_binding!`: one workflow read binding from the
//! capability, operation and binding context it admits through.

/// Declares a workflow read binding: the capability admission a workflow
/// subject, audit or definition read runs for the reading principal.
///
/// ```text
/// worth_query_workflow_read_binding!(pub SubjectRead in Schema {
///     identity: "app.workflow.subject-read.v1",
///     capability: SubjectReadCapability,
///     operation: SubjectReadOp,
///     binding_context: Desktop,
/// });
/// ```
///
/// It generates a unit struct `SubjectRead` implementing
/// `ApplicationWorkflowReadBinding<Schema>`. The read's input and input
/// binding are the operation's own (`worth_query_operation!(.., input
/// Codec)`), so the request a read admits cannot drift from the operation it
/// names; that input must implement `ApplicationCapabilityRequest<Schema,
/// Capability>`. Principal binding, mapping, principal and identity types
/// come from `binding_context`, the same context the application's actions
/// admit through.
#[macro_export]
macro_rules! worth_query_workflow_read_binding {
    (
        $vis:vis $Binding:ident in $Schema:ty {
            identity: $identity:expr,
            capability: $Capability:ty,
            operation: $Operation:ty,
            binding_context: $Context:ty $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Binding;

        impl $crate::facade::application_program::ApplicationWorkflowReadBinding<$Schema>
            for $Binding
        {
            const IDENTITY: &'static str = $identity;
            type Capability = $Capability;
            type Operation = $Operation;
            type InputBinding = <$Operation as $crate::facade::application_schema::ApplicationOperationMarkerIdentity<$Schema>>::InputBinding;
            type Input = <<$Operation as $crate::facade::application_schema::ApplicationOperationMarkerIdentity<$Schema>>::InputBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value;
            type PrincipalBinding = <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::PrincipalBinding;
            type Mapping = <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::Mapping;
            type Principal = <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::Principal;
            type PrincipalIdentity = <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::PrincipalIdentity;
            type PrincipalIdentityBinding = <$Context as $crate::facade::application_schema::ApplicationBindingContext<$Schema>>::PrincipalIdentityBinding;

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
    };
}
