use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;

use super::super::{
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};
use super::conditional::{
    validate_required_producer, DeclaredConditionalBinding, PendingConditionalRegistry,
};
use super::producer::{DeclaredProducerBinding, PendingProducerRegistry};
use super::{WorthQueryApplicationConditionalBinding, WorthQueryApplicationProducerBinding};

pub struct WorthQueryApplicationContributionContracts<Schema> {
    owner: String,
    producers: BTreeMap<String, DeclaredProducerBinding>,
    conditionals: BTreeMap<String, DeclaredConditionalBinding>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryApplicationContributionContracts<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn for_contribution(owner: &str) -> Self {
        Self {
            owner: owner.to_owned(),
            producers: BTreeMap::new(),
            conditionals: BTreeMap::new(),
            marker: PhantomData,
        }
    }

    pub fn conditional<Binding>(
        &mut self,
    ) -> Result<&mut Self, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: 'static,
        Binding: WorthQueryApplicationConditionalBinding<Schema>,
    {
        if self.conditionals.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateConditionalBinding,
                Binding::IDENTITY,
            ));
        }
        self.conditionals.insert(
            Binding::IDENTITY.to_owned(),
            DeclaredConditionalBinding {
                owner: self.owner.clone(),
                contract: Binding::package_contract(),
                required_producers: Binding::REQUIRED_PRODUCERS
                    .iter()
                    .map(|identity| (*identity).to_owned())
                    .collect(),
                binding_type: std::any::TypeId::of::<Binding>(),
                installed_type: std::any::TypeId::of::<Binding::Installed>(),
            },
        );
        Ok(self)
    }

    pub fn producer<Binding>(
        &mut self,
    ) -> Result<&mut Self, WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        if self.producers.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateProducerBinding,
                Binding::IDENTITY,
            ));
        }
        self.producers.insert(
            Binding::IDENTITY.to_owned(),
            DeclaredProducerBinding::of::<Schema, Binding>(&self.owner),
        );
        Ok(self)
    }

    pub(super) fn append_to(
        self,
        destination: &mut WorthQueryApplicationContractCatalog<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        for (identity, binding) in self.producers {
            if destination
                .producers
                .insert(identity.clone(), binding)
                .is_some()
            {
                return Err(denial(DenialKind::DuplicateProducerBinding, identity));
            }
        }
        for (identity, binding) in self.conditionals {
            if destination
                .conditionals
                .insert(identity.clone(), binding)
                .is_some()
            {
                return Err(denial(DenialKind::DuplicateConditionalBinding, identity));
            }
        }
        Ok(())
    }
}

#[doc(hidden)]
pub struct WorthQueryApplicationContractCatalog<Schema> {
    producers: BTreeMap<String, DeclaredProducerBinding>,
    conditionals: BTreeMap<String, DeclaredConditionalBinding>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> Default for WorthQueryApplicationContractCatalog<Schema> {
    fn default() -> Self {
        Self {
            producers: BTreeMap::new(),
            conditionals: BTreeMap::new(),
            marker: PhantomData,
        }
    }
}

impl<Schema> WorthQueryApplicationContractCatalog<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn validate(&self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let mut families: BTreeMap<
            (&str, &str),
            (
                &[super::WorthQueryProducerApplicability],
                BTreeSet<super::WorthQueryProducerApplicability>,
            ),
        > = BTreeMap::new();
        for binding in self.producers.values() {
            validate_identity(&binding.identity)?;
            validate_identity(&binding.source_selector)?;
            validate_identity(&binding.output_family)?;
            validate_identity(&binding.output_role)?;
            validate_identity(&binding.provider_identity)?;
            validate_identity(&binding.resource_policy)?;
            validate_identity(&binding.reuse_policy)?;
            if binding.applicability.is_empty()
                || binding.supported.is_empty()
                || binding.output_roles.is_empty()
                || !binding
                    .output_roles
                    .iter()
                    .any(|role| role == &binding.output_role)
            {
                return Err(denial(
                    DenialKind::ProducerBindingMeaningMismatch,
                    &binding.identity,
                ));
            }
            let mut roles = BTreeSet::new();
            for role in &binding.output_roles {
                validate_identity(role)?;
                if !roles.insert(role) {
                    return Err(denial(
                        DenialKind::ProducerBindingMeaningMismatch,
                        &binding.output_family,
                    ));
                }
            }
            let supported_set = binding.supported.iter().copied().collect::<BTreeSet<_>>();
            if supported_set.len() != binding.supported.len() {
                return Err(denial(
                    DenialKind::ProducerBindingMeaningMismatch,
                    &binding.output_family,
                ));
            }
            let family = families
                .entry((&binding.output_family, &binding.output_role))
                .or_insert((&binding.supported, BTreeSet::new()));
            if family.0.iter().copied().collect::<BTreeSet<_>>() != supported_set {
                return Err(denial(
                    DenialKind::ProducerBindingMeaningMismatch,
                    &binding.output_family,
                ));
            }
            for requirement in &binding.required_invariants {
                validate_identity(requirement.identifier())?;
            }
            for applicability in &binding.applicability {
                if !binding.supported.contains(applicability) {
                    return Err(denial(
                        DenialKind::ProducerBindingMeaningMismatch,
                        &binding.identity,
                    ));
                }
                if !family.1.insert(*applicability) {
                    return Err(denial(
                        DenialKind::AmbiguousApplicableProducer,
                        &binding.output_family,
                    ));
                }
            }
        }
        for ((family, role), (supported, installed)) in families {
            if supported.iter().any(|case| !installed.contains(case)) {
                return Err(denial(
                    DenialKind::MissingApplicableProducer,
                    format!("{family}/{role}"),
                ));
            }
        }
        for (identity, conditional) in &self.conditionals {
            validate_identity(identity)?;
            validate_identity(conditional.contract.node_identity())?;
            for producer in &conditional.required_producers {
                validate_required_producer(
                    producer,
                    &conditional.owner,
                    self.producers
                        .get(producer)
                        .map(|binding| binding.owner.as_str()),
                )?;
            }
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn compose_package(
        &self,
        mut package: WorthQueryPortableDomainPackage,
    ) -> WorthQueryPortableDomainPackage {
        for binding in self.conditionals.values() {
            package = package
                .domain_operation(binding.contract.domain_operation())
                .conditional_application_operation_erased(binding.contract.operation_binding());
        }
        package
    }

    pub(super) fn into_pending(
        self,
    ) -> (
        PendingProducerRegistry<Schema>,
        PendingConditionalRegistry<Schema>,
    )
    where
        Schema: 'static,
    {
        (
            PendingProducerRegistry::new(self.producers),
            PendingConditionalRegistry::new(self.conditionals),
        )
    }
}

fn validate_identity(identity: &str) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(denial(DenialKind::ProducerBindingMeaningMismatch, identity))
    } else {
        Ok(())
    }
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::{DeclaredProducerBinding, DenialKind, WorthQueryApplicationContractCatalog};
    use crate::domain_computation::primary_graph::application_contribution::{
        WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
    };

    const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
        "rectangle",
        WorthQueryProducerLifecyclePosture::Initial,
    );
    const PRESERVE: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
        "rectangle",
        WorthQueryProducerLifecyclePosture::Preserve,
    );

    #[test]
    fn output_family_meaning_cannot_change_with_source_binding() {
        let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
        catalog.producers.insert(
            "initial".to_owned(),
            producer("initial", "create-source", &[INITIAL]),
        );
        catalog.producers.insert(
            "preserve".to_owned(),
            producer("preserve", "edit-source", &[PRESERVE]),
        );

        let denial = catalog.validate().unwrap_err();
        assert_eq!(denial.kind(), DenialKind::ProducerBindingMeaningMismatch);
        assert_eq!(denial.subject(), "family");
    }

    fn producer(
        identity: &str,
        source: &str,
        supported: &[WorthQueryProducerApplicability],
    ) -> DeclaredProducerBinding {
        DeclaredProducerBinding {
            owner: "owner".to_owned(),
            identity: identity.to_owned(),
            source_selector: source.to_owned(),
            output_family: "family".to_owned(),
            output_roles: vec!["output".to_owned()],
            output_role: "output".to_owned(),
            operation: "operation".to_owned(),
            provider_identity: format!("{identity}-provider"),
            applicability: supported.to_vec(),
            supported: supported.to_vec(),
            required_invariants: Vec::new(),
            resource_policy: "bounded".to_owned(),
            reuse_policy: "exact-source".to_owned(),
            binding_type: TypeId::of::<()>(),
            source_type: TypeId::of::<()>(),
            provider_type: TypeId::of::<()>(),
        }
    }

    struct TestSchema;

    impl worth_query_installation::facade::ApplicationSchema for TestSchema {
        const OWNER: &'static str = "owner";
        const NAME: &'static str = "schema";
        const MAJOR: u32 = 1;
        const MINOR: u32 = 0;

        fn declaration() -> Result<
            worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration<Self>,
            worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
        > {
            unreachable!()
        }
    }
}
