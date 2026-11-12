use super::super::WorthQueryProducerDenialDeclaration;
use super::pending_factory::{PendingProducerExecutor, TypedPendingProducer};
use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

pub(in crate::domain_computation::primary_graph) struct PendingProducerRegistry<Schema> {
    pub(super) declared: BTreeMap<String, DeclaredProducerBinding>,
    /// The operations whose partitioned computation a producer's runs retain.
    retaining: std::collections::HashSet<TypeId>,
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
            retaining: std::collections::HashSet::new(),
            providers: BTreeMap::new(),
            marker: PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.declared.is_empty() && self.providers.is_empty()
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
        let operation = TypeId::of::<Operation>();
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
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
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
            .get_mut(Binding::IDENTITY)
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
        if !<Binding::Provider as super::super::WorthQueryApplicationProducerProvider<
            Schema,
            Binding,
        >>::DenialValue::admits_installation()
        {
            return Err(denial(
                DenialKind::ProducerDenialValueUnbounded,
                Binding::IDENTITY,
            ));
        }
        if self.providers.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateProducerBinding,
                Binding::IDENTITY,
            ));
        }
        if let Some(contract) = Binding::PREDECESSOR_READS {
            contract.validate(installed, Binding::OWNERSHIP)?;
        }
        declared.owned_scope_entity =
            super::scope_ownership::validate::<Schema, Binding>(&mutation, &source_query)?;
        declared.installed_output_equivalence = declared
            .output_equivalence
            .as_ref()
            .map(|field| {
                super::super::output_equivalence_contract::seal(
                    Binding::IDENTITY,
                    &declared.output_role,
                    field,
                    installed,
                )
            })
            .transpose()?;
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
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
        support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
        packages: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
    ) -> Result<
        WorthQueryInstalledApplicationProducerRegistry<Schema>,
        super::seal_refusal::RefusedProducerSeal<Schema>,
    > {
        let mut entries = BTreeMap::new();
        let validation = (|| {
            super::scope_ownership::require_family_agreement(&self.declared)?;
            if let Some((left, right)) = duplicate_operation_binding(&self.declared) {
                return Err(denial(
                    DenialKind::DuplicateProducerOperationBinding,
                    format!("{left} / {right}"),
                ));
            }
            self.validate_complete()
        })();
        if let Err(cause) = validation {
            return Err(super::seal_refusal::RefusedProducerSeal::new(
                self, entries, cause,
            ));
        }
        while let Some((identity, declaration)) = self.declared.pop_first() {
            let edition =
                match InstalledProducerEdition::from_declaration(&declaration, build_identity) {
                    Ok(edition) => edition.retaining_computation(
                        self.retaining.contains(&declaration.operation_type),
                    ),
                    Err(cause) => {
                        self.declared.insert(identity, declaration);
                        return Err(super::seal_refusal::RefusedProducerSeal::new(
                            self, entries, cause,
                        ));
                    }
                };
            let (value, executor) = self
                .providers
                .remove(&identity)
                .expect("complete original provider inventory");
            let executor = match executor.prepare(support, packages) {
                Ok(executor) => executor,
                Err(refused) => {
                    self.providers
                        .insert(identity.clone(), (value, refused.pending));
                    self.declared.insert(identity, declaration);
                    return Err(super::seal_refusal::RefusedProducerSeal::new(
                        self,
                        entries,
                        refused.cause,
                    ));
                }
            };
            entries.insert(
                identity,
                Arc::new(InstalledProducerProvider {
                    declaration,
                    edition,
                    value,
                    executor,
                }),
            );
        }
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
