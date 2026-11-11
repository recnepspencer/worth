//! Declare one conditional definition without installing its temporal runtime.
use super::super::canonical_identity::CanonicalIdentityMaterial;
use super::*;
use worth_query_installation::facade::{
    WorthQueryAuthoredHostConditionalPredicateProvider,
    WorthQueryInstalledNamedClockConditionalNode,
};

impl<Schema: ApplicationSchema + 'static>
    WorthQueryConditionalApplicationRuntimeInstallation<Schema>
{
    pub fn bind_conditional_definition<
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
    >(
        &mut self,
        clocked: WorthQueryInstalledNamedClockConditionalNode<
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
        >,
    ) -> Result<
        WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        WorthQueryConditionalRuntimeInstallationDenial,
    >
    where
        ApplicationOperation: 'static,
        Input: 'static,
        D: 'static,
        O: 'static,
        F: 'static,
        Node: 'static,
        Provider: WorthQueryAuthoredHostConditionalPredicateProvider<Node>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
    {
        let node = clocked.provider().node();
        self.publication
            .runtime
            .installed_packages()
            .validate_conditional_application_node(node)
            .map_err(|cause| foreign_binding_denial(cause.subject()))?;
        if let Some(scope) = &self.application_binding_scope {
            if scope.binding != *node.operation().binding()
                || scope.target != *node.location()
                || scope.operation_type != TypeId::of::<ApplicationOperation>()
            {
                return Err(foreign_binding_denial(node.declaration().identity()));
            }
        }
        if clocked
            .provider()
            .retain_authored_predicate_for_runtime()
            .is_none()
        {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::AuthoredProviderSourceRequired,
                node.declaration().identity(),
            ));
        }
        let mut material = CanonicalIdentityMaterial::new(
            "worth-query.authored-conditional-definition",
            "worth-query-authored-conditional-definition-v1",
        );
        material.digest(
            "application-definition",
            worth_foundational::facade::CanonicalDigestId::new(
                node.operation()
                    .application_operation()
                    .definition_identity_bytes(),
            ),
        );
        material.text(
            "domain-operation",
            node.operation()
                .binding()
                .domain_operation_canonical_identity(),
        );
        material.text("node", node.declaration().identity());
        material.text("stage", node.location().stage_identity().unwrap_or(""));
        material.text("clock", clocked.clock_identity());
        material.text("source", clocked.source_identity().as_str());
        material.text("timeline", clocked.timeline_identity().as_str());
        let (digest, work) = material.derive().map_err(|cause| {
            WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                format!("{cause:?}"),
            )
        })?;
        let identity: Arc<str> = format!("conditional-definition:{}", digest.render_hex()).into();
        if !self.binding_identities.insert(identity.clone()) {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::DuplicateBinding,
                identity.as_ref(),
            ));
        }
        let node_authority = Arc::from(node.authority_identity());
        let lease = Arc::new(ConditionalClockLease);
        self.definitions.push(Box::new(
            super::super::pending_definition::PendingTypedDefinition {
                provider: clocked.into_definition_provider(),
                identity: identity.clone(),
                lease: lease.clone(),
            },
        ));
        Ok(WorthQueryConditionalClockHandle {
            binding_identity: identity,
            binding_identity_digest: *digest.bytes(),
            node_authority,
            binding_canonical_work: work.with_digest_text_materializations(1),
            lease,
            marker: PhantomData,
        })
    }
}
