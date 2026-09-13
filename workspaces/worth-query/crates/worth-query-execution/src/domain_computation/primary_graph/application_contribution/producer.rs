use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationOutputContract,
};
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryProducerLifecyclePosture {
    Initial,
    Preserve,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProducerApplicability {
    profile_kind: &'static str,
    lifecycle: WorthQueryProducerLifecyclePosture,
}

impl WorthQueryProducerApplicability {
    pub const fn new(
        profile_kind: &'static str,
        lifecycle: WorthQueryProducerLifecyclePosture,
    ) -> Self {
        Self {
            profile_kind,
            lifecycle,
        }
    }

    pub const fn profile_kind(self) -> &'static str {
        self.profile_kind
    }

    pub const fn lifecycle(self) -> WorthQueryProducerLifecyclePosture {
        self.lifecycle
    }
}

pub trait WorthQueryProducerOutputFamily: Sized + 'static {
    const IDENTITY: &'static str;
    const SUPPORTED: &'static [WorthQueryProducerApplicability];
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProducerInvariantRequirement {
    identifier: &'static str,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
}

impl WorthQueryProducerInvariantRequirement {
    pub const fn new(
        identifier: &'static str,
        major: u16,
        minor: u16,
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> Self {
        Self {
            identifier,
            major,
            minor,
            execution_point,
        }
    }

    pub const fn identifier(self) -> &'static str {
        self.identifier
    }

    pub const fn major(self) -> u16 {
        self.major
    }

    pub const fn minor(self) -> u16 {
        self.minor
    }

    pub const fn execution_point(self) -> ApplicationInvariantExecutionPoint {
        self.execution_point
    }
}

pub trait WorthQueryApplicationProducerProvider: Send + Sync + 'static {
    const SEMANTIC_IDENTITY: &'static str;
}

pub trait WorthQueryApplicationProducerBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Operation: ApplicationMutationBinding<Schema>;
    type Source: ApplicationMutationBinding<Schema>;
    type OutputFamily: WorthQueryProducerOutputFamily;
    type Provider: WorthQueryApplicationProducerProvider;

    const IDENTITY: &'static str;
    const OUTPUT_ROLE: &'static str;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement];
    const RESOURCE_POLICY: &'static str;
    const REUSE_POLICY: &'static str;
}

#[derive(Clone)]
pub(super) struct DeclaredProducerBinding {
    pub(super) owner: String,
    pub(super) identity: String,
    pub(super) source_selector: String,
    pub(super) output_family: String,
    pub(super) output_roles: Vec<String>,
    pub(super) output_role: String,
    pub(super) operation: String,
    pub(super) provider_identity: String,
    pub(super) applicability: Vec<WorthQueryProducerApplicability>,
    pub(super) supported: Vec<WorthQueryProducerApplicability>,
    pub(super) required_invariants: Vec<WorthQueryProducerInvariantRequirement>,
    pub(super) resource_policy: String,
    pub(super) reuse_policy: String,
    pub(super) binding_type: TypeId,
    pub(super) source_type: TypeId,
    pub(super) provider_type: TypeId,
}

impl DeclaredProducerBinding {
    pub(super) fn of<Schema, Binding>(owner: &str) -> Self
    where
        Schema: ApplicationSchema,
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        Self {
            owner: owner.to_owned(),
            identity: Binding::IDENTITY.to_owned(),
            source_selector: Binding::Source::IDENTITY.to_owned(),
            output_family: Binding::OutputFamily::IDENTITY.to_owned(),
            output_roles: <Binding::Operation as ApplicationMutationBinding<Schema>>::Output::ROLES
                .iter()
                .map(|role| role.name().to_owned())
                .collect(),
            output_role: Binding::OUTPUT_ROLE.to_owned(),
            operation: Binding::Operation::IDENTITY.to_owned(),
            provider_identity: Binding::Provider::SEMANTIC_IDENTITY.to_owned(),
            applicability: Binding::APPLICABILITY.to_vec(),
            supported: Binding::OutputFamily::SUPPORTED.to_vec(),
            required_invariants: Binding::REQUIRED_INVARIANTS.to_vec(),
            resource_policy: Binding::RESOURCE_POLICY.to_owned(),
            reuse_policy: Binding::REUSE_POLICY.to_owned(),
            binding_type: TypeId::of::<Binding>(),
            source_type: TypeId::of::<Binding::Source>(),
            provider_type: TypeId::of::<Binding::Provider>(),
        }
    }

    fn meaning_matches<Schema, Binding>(&self) -> bool
    where
        Schema: ApplicationSchema,
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        let expected = Self::of::<Schema, Binding>(&self.owner);
        self.identity == expected.identity
            && self.source_selector == expected.source_selector
            && self.output_family == expected.output_family
            && self.output_roles == expected.output_roles
            && self.output_role == expected.output_role
            && self.operation == expected.operation
            && self.provider_identity == expected.provider_identity
            && self.applicability == expected.applicability
            && self.supported == expected.supported
            && self.required_invariants == expected.required_invariants
            && self.resource_policy == expected.resource_policy
            && self.reuse_policy == expected.reuse_policy
            && self.binding_type == expected.binding_type
            && self.source_type == expected.source_type
            && self.provider_type == expected.provider_type
    }
}

#[derive(Clone)]
struct InstalledProducerProvider {
    declaration: DeclaredProducerBinding,
    value: Arc<dyn Any + Send + Sync>,
}

pub struct WorthQueryInstalledApplicationProducerRegistry<Schema> {
    entries: BTreeMap<String, InstalledProducerProvider>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> Clone for WorthQueryInstalledApplicationProducerRegistry<Schema> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            marker: PhantomData,
        }
    }
}

impl<Schema> Default for WorthQueryInstalledApplicationProducerRegistry<Schema> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            marker: PhantomData,
        }
    }
}

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn provider<Binding>(&self) -> Option<Arc<Binding::Provider>>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        self.entries
            .get(Binding::IDENTITY)
            .filter(|entry| entry.declaration.meaning_matches::<Schema, Binding>())
            .and_then(|entry| {
                Arc::clone(&entry.value)
                    .downcast::<Binding::Provider>()
                    .ok()
            })
    }
}

pub(super) struct PendingProducerRegistry<Schema> {
    declared: BTreeMap<String, DeclaredProducerBinding>,
    providers: BTreeMap<String, Arc<dyn Any + Send + Sync>>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> PendingProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn new(declared: BTreeMap<String, DeclaredProducerBinding>) -> Self {
        Self {
            declared,
            providers: BTreeMap::new(),
            marker: PhantomData,
        }
    }

    pub(super) fn register<Binding>(
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
        self.providers
            .insert(Binding::IDENTITY.to_owned(), Arc::new(provider));
        Ok(())
    }

    pub(super) fn seal(
        self,
    ) -> Result<
        WorthQueryInstalledApplicationProducerRegistry<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
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
                let value = self
                    .providers
                    .get(&identity)
                    .expect("complete provider inventory checked")
                    .clone();
                (identity, InstalledProducerProvider { declaration, value })
            })
            .collect();
        Ok(WorthQueryInstalledApplicationProducerRegistry {
            entries,
            marker: PhantomData,
        })
    }

    pub(super) fn validate_complete(&self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
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

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
