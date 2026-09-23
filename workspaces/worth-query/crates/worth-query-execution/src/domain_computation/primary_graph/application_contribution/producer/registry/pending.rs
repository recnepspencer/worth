use super::*;

pub(in crate::domain_computation::primary_graph::application_contribution) struct PendingProducerRegistry<
    Schema,
> {
    declared: BTreeMap<String, DeclaredProducerBinding>,
    providers: BTreeMap<
        String,
        (
            Arc<dyn Any + Send + Sync>,
            Arc<dyn InstalledProducerExecutor<Schema>>,
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
            providers: BTreeMap::new(),
            marker: PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn register<
        Binding,
    >(
        &mut self,
        owner: &str,
        provider: Binding::Provider,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
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
                Arc::new(TypedInstalledProducer::<Schema, Binding>::new(provider)),
            ),
        );
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn seal(
        self,
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
                let (value, executor) = self
                    .providers
                    .get(&identity)
                    .expect("complete provider inventory checked")
                    .clone();
                (
                    identity,
                    InstalledProducerProvider {
                        declaration,
                        value,
                        executor,
                    },
                )
            })
            .collect();
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
