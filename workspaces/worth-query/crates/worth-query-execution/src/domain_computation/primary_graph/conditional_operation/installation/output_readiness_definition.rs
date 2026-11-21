//! Definition authority is issued by the same binding that installs readiness.
use super::super::canonical_identity::CanonicalIdentityMaterial;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    TypedPendingOutputReadiness, WorthQueryApplicationProducerBinding,
};
use worth_query_execution_query::facade::ports::ConditionalClockHandleInternalPort as _;
use worth_query_installation::facade::{
    WorthQueryAuthoredHostConditionalPredicateProvider, WorthQueryAuthoredHostPredicateBinding,
    WorthQueryInstalledApplicationConditionalNode,
};

impl<Schema: ApplicationSchema + 'static>
    WorthQueryConditionalApplicationRuntimeInstallation<Schema>
{
    /// Install output readiness and retain its definition-authoring authority.
    /// The unit clock parameter denotes a non-temporal definition: this issues
    /// no clock source, temporal operation, wake queue or additional Signal node.
    /// `prepare_conditional_definition` consumes this same installed binding.
    pub fn bind_output_readiness_definition<
        Producer,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
    >(
        &mut self,
        node: WorthQueryInstalledApplicationConditionalNode<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
        >,
        delivery_dependency_ordinal: usize,
        provider: Arc<Provider>,
    ) -> Result<
        WorthQueryConditionalClockHandle<Schema, Node, ()>,
        WorthQueryConditionalRuntimeInstallationDenial,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
        ApplicationOperation: 'static,
        Input: 'static,
        D: 'static,
        O: 'static,
        F: 'static,
        Node: 'static,
        Provider: WorthQueryAuthoredHostConditionalPredicateProvider<Node>,
    {
        self.validate_output_readiness_node::<Producer, _, _, _, _, _, _>(
            &node,
            delivery_dependency_ordinal,
        )?;
        let mut material = CanonicalIdentityMaterial::new(
            "worth-query.output-readiness-definition",
            "worth-query-output-readiness-definition-v1",
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
        material.text("producer", Producer::IDENTITY);
        material.unsigned_u64(
            "delivery-dependency-ordinal",
            delivery_dependency_ordinal as u64,
        );
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
        let providers = super::super::predicate_admission::output_readiness_providers::<Node, _>(
            Arc::from(node.declaration().identity()),
            super::super::QueryAuthoredPredicateProvider::new(
                WorthQueryAuthoredHostPredicateBinding::from_provider(provider),
                Arc::from(node.declaration().identity()),
                Arc::clone(&node_authority),
            ),
        );
        let lease = WorthQueryConditionalClockHandle::<Schema, (), ()>::new_lease();
        self.output_readiness
            .push(Box::new(TypedPendingOutputReadiness::new(
                Producer::IDENTITY,
                delivery_dependency_ordinal,
                node,
                Some((identity.clone(), lease.clone(), providers)),
            )));
        Ok(WorthQueryConditionalClockHandle::from_parts(
            identity,
            *digest.bytes(),
            node_authority,
            work.with_digest_text_materializations(1),
            lease,
        ))
    }
}
