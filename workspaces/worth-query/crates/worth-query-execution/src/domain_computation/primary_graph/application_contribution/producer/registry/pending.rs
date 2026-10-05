use super::pending_factory::{PendingProducerExecutor, TypedPendingProducer};
use super::*;

pub(in crate::domain_computation::primary_graph) struct PendingProducerRegistry<Schema> {
    declared: BTreeMap<String, DeclaredProducerBinding>,
    /// The operations whose partitioned computation a producer's runs retain.
    retaining: std::collections::BTreeSet<std::any::TypeId>,
    providers: BTreeMap<
        String,
        (
            Arc<dyn Any + Send + Sync>,
            Box<dyn PendingProducerExecutor<Schema>>,
        ),
    >,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> PendingProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph::application_contribution) fn new(
        declared: BTreeMap<String, DeclaredProducerBinding>,
    ) -> Self {
        Self {
            declared,
            retaining: std::collections::BTreeSet::new(),
            providers: BTreeMap::new(),
            marker: PhantomData,
        }
    }

    /// Whether a declared producer runs `Operation`, whose partitioned
    /// computation is being installed; when one does, that producer's runs
    /// retain the computation's state. Every producer is declared before any
    /// contribution installs, so the answer is final.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn retain_computation_of<
        Operation: 'static,
    >(
        &mut self,
    ) -> bool {
        let operation = std::any::TypeId::of::<Operation>();
        let runs = self
            .declared
            .values()
            .any(|declared| declared.operation_type == operation);
        if runs {
            self.retaining.insert(operation);
        }
        runs
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn register<
        Binding,
    >(
        &mut self,
        owner: &str,
        provider: Binding::Provider,
        mutation: worth_query_installation::facade::WorthQueryInstalledApplicationMutationBinding<
            Schema,
            Binding::Operation,
        >,
        source_query: worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding<
            Schema,
            super::super::ProducerSourceBinding<Schema, Binding>,
        >,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
        super::super::ProducerSourceValue<Schema, Binding>:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                Schema,
                super::super::ProducerSourceQuery<Schema, Binding>,
            >,
    {
        let declared = self
            .declared
            .get(Binding::IDENTITY)
            .ok_or_else(|| denial(DenialKind::ForeignProducerBinding, Binding::IDENTITY))?;
        if declared.owner != owner {
            return Err(denial(
                DenialKind::ForeignProducerBinding,
                Binding::IDENTITY,
            ));
        }
        if !declared.meaning_matches::<Schema, Binding>() {
            return Err(denial(
                DenialKind::ProducerBindingMeaningMismatch,
                Binding::IDENTITY,
            ));
        }
        if self.providers.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateProducerBinding,
                Binding::IDENTITY,
            ));
        }
        let provider = Arc::new(provider);
        self.providers.insert(
            Binding::IDENTITY.to_owned(),
            (
                provider.clone(),
                Box::new(TypedPendingProducer::<Schema, Binding>::new(
                    provider,
                    mutation,
                    source_query,
                )),
            ),
        );
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn seal_with_support(
        mut self,
        support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
        packages: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
    ) -> Result<
        WorthQueryInstalledApplicationProducerRegistry<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        if let Some((left, right)) = duplicate_operation_binding(&self.declared) {
            return Err(denial(
                DenialKind::DuplicateProducerOperationBinding,
                format!("{left} / {right}"),
            ));
        }
        let missing = self
            .declared
            .keys()
            .find(|identity| !self.providers.contains_key(*identity));
        if let Some(identity) = missing {
            return Err(denial(DenialKind::MissingProducerProvider, identity));
        }
        let entries = self
            .declared
            .into_iter()
            .map(|(identity, declaration)| {
                let edition = InstalledProducerEdition::from_declaration(&declaration)?
                    .retaining_computation(self.retaining.contains(&declaration.operation_type));
                let (value, executor) = self
                    .providers
                    .remove(&identity)
                    .expect("complete provider inventory checked");
                let executor = executor.prepare(support, packages)?;
                Ok((
                    identity,
                    Arc::new(InstalledProducerProvider {
                        declaration,
                        edition,
                        value,
                        executor,
                    }),
                ))
            })
            .collect::<Result<_, WorthQueryPrimaryGraphInstallationDenial>>()?;
        Ok(WorthQueryInstalledApplicationProducerRegistry {
            entries,
            marker: PhantomData,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn validate_complete(
        &self,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if let Some(identity) = self
            .declared
            .keys()
            .find(|identity| !self.providers.contains_key(*identity))
        {
            Err(denial(DenialKind::MissingProducerProvider, identity))
        } else {
            Ok(())
        }
    }
}
