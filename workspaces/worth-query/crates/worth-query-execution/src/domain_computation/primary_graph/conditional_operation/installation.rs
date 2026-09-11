use std::{collections::BTreeSet, marker::PhantomData, sync::Arc};

use crate::domain_computation::primary_graph::application_runtime::installation::ApplicationRuntimePublication;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, OperationReads, OperationWrites,
    WorthQueryHostConditionalPredicateProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentProjector,
    WritableCapability, WritePosture,
};

use super::operation_invocation::{
    WorthQueryTemporalOperationExecution, WorthQueryTemporalOperationInvoker,
};
use super::reconstruction_authority::{
    WorthQueryTemporalPrincipalSource, WorthQueryTemporalReconstructionAccess,
};

mod clock_handle;
pub(in crate::domain_computation::primary_graph) use clock_handle::ConditionalClockLease;
pub use clock_handle::WorthQueryConditionalClockHandle;
mod denial;
pub use denial::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
mod pending_operation;
pub(in crate::domain_computation::primary_graph) use pending_operation::WorthQueryPendingConditionalOperation;

pub struct WorthQueryConditionalApplicationRuntimeInstallation<Schema> {
    publication: ApplicationRuntimePublication<Schema>,
    binding_identities: BTreeSet<Arc<str>>,
    bindings: Vec<Box<dyn WorthQueryPendingConditionalOperation<Schema>>>,
}

impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn new(
        publication: ApplicationRuntimePublication<Schema>,
    ) -> Result<Self, WorthQueryConditionalRuntimeInstallationDenial> {
        publication
            .runtime
            .installed_packages()
            .validate_application_schema(&publication.installed_schema)
            .map_err(|denial| {
                WorthQueryConditionalRuntimeInstallationDenial::new(
                    WorthQueryConditionalRuntimeInstallationDenialKind::PrimaryGraphPublication,
                    denial.subject(),
                )
            })?;
        Ok(Self {
            publication,
            binding_identities: BTreeSet::new(),
            bindings: Vec::new(),
        })
    }

    pub fn bind_temporal_operation<
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
        QueryAuthorization,
        Invoker,
        IntentEntity,
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        IdentityUnit,
        RevisionAspect,
        RevisionField,
        RevisionValue,
        RevisionWrite,
        RevisionEquality,
        RevisionUnit,
        LifecycleAspect,
        LifecycleField,
        LifecycleValue,
        LifecycleWrite,
        LifecycleEquality,
        LifecycleUnit,
        Authorization,
    >(
        &mut self,
        binding: WorthQueryInstalledTemporalConditionalOperation<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
            Provider,
            Clock,
            Source,
            Query,
            Parameters,
            QueryResult,
            Scope,
            Projector,
        >,
        execution: WorthQueryTemporalOperationExecution<
            Schema,
            ApplicationOperation,
            Input,
            Scope,
            Invoker,
            IntentEntity,
            IdentityAspect,
            IdentityField,
            IdentityValue,
            IdentityWrite,
            IdentityUnit,
            RevisionAspect,
            RevisionField,
            RevisionValue,
            RevisionWrite,
            RevisionEquality,
            RevisionUnit,
            LifecycleAspect,
            LifecycleField,
            LifecycleValue,
            LifecycleWrite,
            LifecycleEquality,
            LifecycleUnit,
            Authorization,
        >,
        reconstruction: WorthQueryTemporalReconstructionAccess<
            Schema,
            PrincipalBinding,
            PrincipalMapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
            Scope,
            ScopeAspect,
            ScopeField,
            ScopeValue,
            ScopeWrite,
            ScopeUnit,
            PrincipalSource,
            QueryAuthorization,
        >,
    ) -> Result<
        WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        WorthQueryConditionalRuntimeInstallationDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
        Query: 'static,
        Parameters: 'static,
        QueryResult: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<Schema, Query>
            + 'static,
        Scope: 'static,
        Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
        PrincipalIdentityBinding:
            worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
                    Value = PrincipalIdentity,
                > + 'static,
        ScopeField:
            worth_query_installation::facade::DeclaredApplicationFieldValue<Value = ScopeValue>,
        ScopeField::Binding:
            worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
        ScopeValue: Clone + Send + Sync + 'static,
        ScopeWrite: WritePosture + 'static,
        ScopeUnit: ApplicationFieldUnit + 'static,
        PrincipalBinding: 'static,
        PrincipalMapping: 'static,
        Principal: 'static,
        PrincipalIdentity: 'static,
        ScopeAspect: 'static,
        ScopeField: 'static,
        PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
        QueryAuthorization: super::WorthQueryTemporalQueryAuthorization<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            > + 'static,
        Invoker: WorthQueryTemporalOperationInvoker<Schema, ApplicationOperation, Input, Scope>,
        IntentEntity: 'static,
        IdentityAspect: 'static,
        IdentityField: OperationReads<ApplicationOperation>
            + worth_query_installation::facade::DeclaredApplicationFieldValue<Value = IdentityValue>
            + 'static,
        IdentityField::Binding:
            worth_query_installation::facade::ApplicationReadableScalarValueBinding<
                Value = IdentityValue,
            >,
        IdentityValue: Clone + Send + 'static,
        IdentityWrite: WritePosture + 'static,
        IdentityUnit: ApplicationFieldUnit + 'static,
        RevisionAspect: 'static,
        RevisionField: OperationReads<ApplicationOperation>
            + OperationWrites<ApplicationOperation>
            + worth_query_installation::facade::DeclaredApplicationFieldValue<Value = RevisionValue>
            + 'static,
        RevisionField::Binding:
            worth_query_installation::facade::ApplicationReadableScalarValueBinding<
                    Value = RevisionValue,
                > + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
        RevisionValue: Clone + Send + 'static,
        RevisionWrite: WritableCapability + 'static,
        RevisionEquality: 'static,
        RevisionUnit: ApplicationFieldUnit + 'static,
        LifecycleAspect: 'static,
        LifecycleField: OperationReads<ApplicationOperation>
            + OperationWrites<ApplicationOperation>
            + worth_query_installation::facade::DeclaredApplicationFieldValue<Value = LifecycleValue>
            + 'static,
        LifecycleField::Binding:
            worth_query_installation::facade::ApplicationReadableScalarValueBinding<
                Value = LifecycleValue,
            >,
        LifecycleValue: Clone + Send + Sync + 'static,
        LifecycleWrite: WritableCapability + 'static,
        LifecycleEquality: 'static,
        LifecycleUnit: ApplicationFieldUnit + 'static,
        Authorization: super::WorthQueryTemporalOperationAuthorization<
                Schema,
                ApplicationOperation,
                Input,
                Scope,
            > + 'static,
        ApplicationOperation: 'static,
        D: 'static,
        O: 'static,
        F: 'static,
        Node: 'static,
    {
        self.validate_temporal_binding(&binding)?;
        super::access_validation::validate_reconstruction_access(
            &self.publication,
            &reconstruction,
        )?;
        execution
            .validate_publication(&self.publication)
            .map_err(foreign_binding_denial)?;
        let identity = super::pending_binding::temporal_binding_identity(
            &binding,
            reconstruction.principal_source_identity(),
            execution.invoker_identity(),
        )
        .map_err(|denial| {
            WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::ForeignBinding,
                format!("conditional binding identity was denied: {denial:?}"),
            )
        })?;
        let support_identity: Arc<str> = Arc::from(identity.support_identity());
        if !self
            .binding_identities
            .insert(Arc::clone(&support_identity))
        {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::DuplicateBinding,
                support_identity.as_ref(),
            ));
        }
        let binding_canonical_work = identity.canonical_work();
        let lease = Arc::new(ConditionalClockLease);
        let node_authority = Arc::from(
            binding
                .clocked_node()
                .provider()
                .node()
                .authority_identity(),
        );
        self.bindings.push(Box::new(
            super::pending_binding::PendingTemporalOperation::new(
                Arc::clone(&identity),
                Arc::clone(&lease),
                binding,
                reconstruction,
                execution,
            ),
        ));
        Ok(WorthQueryConditionalClockHandle {
            binding_identity: support_identity,
            binding_identity_digest: *identity.digest().bytes(),
            node_authority,
            binding_canonical_work,
            lease,
            marker: PhantomData,
        })
    }

    pub fn publish(
        self,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        super::super::application_runtime::installation::publish_application_runtime_with_conditionals(
            self.publication,
            self.bindings,
        )
    }

    fn validate_temporal_binding<
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >(
        &self,
        binding: &WorthQueryInstalledTemporalConditionalOperation<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
            Provider,
            Clock,
            Source,
            Query,
            Parameters,
            QueryResult,
            Scope,
            Projector,
        >,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>
    where
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
        Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
    {
        let index = self.publication.runtime.installed_packages();
        index
            .validate_conditional_application_node(binding.clocked_node().provider().node())
            .map_err(|denial| foreign_binding_denial(denial.subject()))?;
        self.publication
            .installed_schema
            .validate_installed_query(binding.query())
            .map_err(|denial| foreign_binding_denial(denial.subject()))
    }
}

fn foreign_binding_denial(
    subject: impl Into<String>,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::ForeignBinding,
        subject,
    )
}
