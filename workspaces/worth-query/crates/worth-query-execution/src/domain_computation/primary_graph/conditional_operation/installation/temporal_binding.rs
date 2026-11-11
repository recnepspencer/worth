//! Temporal operation installation retains its existing reconstruction and execution admission.
use super::*;
impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>
where
    Schema: ApplicationSchema + 'static,
{
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
        ScopeField,
        ScopeValue,
        PrincipalSource,
        QueryAuthorization,
        Invoker,
        IntentEntity,
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
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
            IdentityField,
            IdentityValue,
            RevisionField,
            RevisionValue,
            LifecycleField,
            LifecycleValue,
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
            ScopeField,
            ScopeValue,
            PrincipalSource,
            QueryAuthorization,
        >,
    ) -> Result<
        WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        WorthQueryConditionalRuntimeInstallationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema + 'static,
        ApplicationOperation: 'static,
        Input: Clone + Send + Sync + 'static,
        D: 'static,
        O: 'static,
        F: 'static,
        Node: 'static,
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
        Query: 'static,
        Parameters: 'static,
        QueryResult: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<Schema, Query>
            + 'static,
        Scope: 'static,
        Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
        PrincipalBinding: 'static,
        PrincipalMapping: 'static,
        Principal: 'static,
        PrincipalIdentity: 'static,
        PrincipalIdentityBinding:
            worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
                    Value = PrincipalIdentity,
                > + 'static,
        ScopeField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                Entity = Scope,
                Value = ScopeValue,
            > + 'static,
        ScopeField::Binding:
            worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
        ScopeValue: Clone + Send + Sync + 'static,
        PrincipalSource:
            super::super::reconstruction_authority::WorthQueryTemporalPrincipalSource<Schema>,
        QueryAuthorization: super::super::WorthQueryTemporalQueryAuthorization<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            > + 'static,
        Invoker: super::super::operation_invocation::WorthQueryTemporalOperationInvoker<
            Schema,
            ApplicationOperation,
            Input,
            Scope,
        >,
        IntentEntity: 'static,
        IdentityField:
            worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                    Entity = IntentEntity,
                    Value = IdentityValue,
                > + 'static,
        IdentityField::Binding:
            worth_query_installation::facade::ApplicationReadableScalarValueBinding<
                Value = IdentityValue,
            >,
        IdentityValue: Clone + Send + 'static,
        RevisionField:
            worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                    Entity = IntentEntity,
                    Value = RevisionValue,
                > + 'static,
        RevisionField::Binding:
            worth_query_installation::facade::ApplicationReadableScalarValueBinding<
                    Value = RevisionValue,
                > + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
        RevisionValue: Clone + Send + 'static,
        LifecycleField:
            worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                    Entity = IntentEntity,
                    Value = LifecycleValue,
                > + 'static,
        LifecycleField::Binding:
            worth_query_installation::facade::ApplicationReadableScalarValueBinding<
                Value = LifecycleValue,
            >,
        LifecycleValue: Clone + Send + Sync + 'static,
        Authorization: super::super::WorthQueryTemporalOperationAuthorization<
                Schema,
                ApplicationOperation,
                Input,
                Scope,
            > + 'static,
    {
        self.validate_application_binding_scope(&binding)?;
        self.validate_temporal_binding(&binding)?;
        super::super::access_validation::validate_reconstruction_access(
            &self.publication,
            &reconstruction,
        )?;
        execution
            .validate_publication(&self.publication)
            .map_err(foreign_binding_denial)?;
        let identity = super::super::pending_binding::temporal_binding_identity(
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
            super::super::pending_binding::PendingTemporalOperation::new(
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
}
