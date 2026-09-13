use std::marker::PhantomData;
use std::sync::Arc;

use super::compilation::CompiledApplicationSchema;
use super::contribution::WorthQueryInstalledApplicationContributionCatalog;
use super::invariant::{
    resolve_installed_invariant, WorthQueryInstalledApplicationInvariant,
    WorthQueryInstalledApplicationInvariantCatalog,
};
use super::native_contract::WorthQueryInstalledApplicationSchemaContractCatalog;
use super::value_binding::WorthQueryInstalledApplicationValueBindingCatalog;
use crate::application_ability::{
    WorthQueryAbilityInstallationDenial, WorthQueryAbilityInstallationDenialKind,
    WorthQueryInstalledAbility,
};
use crate::application_capability::{
    ApplicationCapabilityRegistry, WorthQueryInstalledApplicationCapability,
};
use crate::application_operation::{
    binding::{
        compile_application_mutation_catalog, WorthQueryInstalledApplicationMutationCatalog,
    },
    ApplicationAuthorizationPolicyRegistry, WorthQueryApplicationOperationInstallationDenial,
    WorthQueryInstalledAbilityRequirement, WorthQueryInstalledApplicationOperation,
};
use crate::application_query::{
    binding::{compile_application_query_catalog, WorthQueryInstalledApplicationQueryCatalog},
    WorthQueryApplicationQueryInstallationDenial, WorthQueryInstalledApplicationQuery,
};
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::installed_index::WorthQueryInstalledPackageAuthority;
use crate::package::WorthQueryPortableDomainPackageIdentity;
use crate::package::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableNativeAspectContractRecord,
};
use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_operation::ApplicationMutationBindingDescriptor;
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationEntityRef, ApplicationOperationMarkerIdentity,
    ApplicationOperationRef, ApplicationSchema, ApplicationSchemaAuthoringContext,
    ApplicationSchemaBindingIdentity, ApplicationSchemaMember, ApplicationSchemaMemberProvenance,
    ApplicationStructuredValueBinding, ErasedApplicationSchemaDeclaration,
    TypedEffectIntentBuilder, TypedOperationBuilder, TypedReadDeclarationBuilder,
};

/// Opaque proof that one typed schema declaration belongs to an exact
/// installed package, runtime, and generation.
pub struct WorthQueryInstalledApplicationSchema<Schema> {
    pub(crate) package_authority: WorthQueryInstalledPackageAuthority,
    pub(crate) schema_name: String,
    pub(crate) schema_identity: CanonicalDigestId,
    pub(crate) schema: ErasedApplicationSchemaDeclaration,
    pub(crate) member_provenance: ApplicationSchemaMemberProvenance,
    pub(crate) capability_registry: ApplicationCapabilityRegistry,
    authorization_policy_registry: ApplicationAuthorizationPolicyRegistry,
    value_binding_catalog: WorthQueryInstalledApplicationValueBindingCatalog,
    native_contract_catalog: Arc<WorthQueryInstalledApplicationSchemaContractCatalog>,
    portable_native_contracts: Arc<Vec<WorthQueryPortableNativeAspectContractRecord>>,
    portable_operation_contracts: Arc<Vec<WorthQueryPortableApplicationOperationContractRecord>>,
    installation_canonical_work: WorthQueryCanonicalWorkEvidence,
    pub(crate) query_catalog: WorthQueryInstalledApplicationQueryCatalog,
    pub(crate) mutation_catalog: WorthQueryInstalledApplicationMutationCatalog,
    invariant_catalog: WorthQueryInstalledApplicationInvariantCatalog,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> std::fmt::Debug for WorthQueryInstalledApplicationSchema<Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryInstalledApplicationSchema")
            .field("owner", &self.package_authority.owner())
            .field("schema_name", &self.schema_name)
            .field("schema_identity", &self.schema_identity)
            .finish_non_exhaustive()
    }
}

impl<Schema> WorthQueryInstalledApplicationSchema<Schema>
where
    Schema: ApplicationSchema,
{
    pub(crate) fn from_compilation(
        compiled: CompiledApplicationSchema<Schema>,
    ) -> Result<Self, super::ApplicationSchemaCompilationDenial> {
        let invariant_catalog =
            WorthQueryInstalledApplicationInvariantCatalog::compile(&compiled.schema);
        let mut installed = Self {
            package_authority: compiled.package_authority,
            schema_name: compiled.schema_name,
            schema_identity: compiled.schema_identity,
            schema: compiled.schema,
            member_provenance: compiled.member_provenance,
            capability_registry: compiled.capability_registry,
            authorization_policy_registry: compiled.authorization_policy_registry,
            value_binding_catalog: compiled.value_binding_catalog,
            native_contract_catalog: compiled.native_contract_catalog,
            portable_native_contracts: compiled.portable_native_contracts,
            portable_operation_contracts: compiled.portable_operation_contracts,
            installation_canonical_work: compiled.installation_canonical_work,
            query_catalog: WorthQueryInstalledApplicationQueryCatalog::default(),
            mutation_catalog: WorthQueryInstalledApplicationMutationCatalog::default(),
            invariant_catalog,
            _schema: compiled.marker,
        };
        installed.query_catalog = compile_application_query_catalog(&installed)
            .map_err(super::ApplicationSchemaCompilationDenial::Query)?;
        installed.mutation_catalog = compile_application_mutation_catalog(&installed)
            .map_err(super::ApplicationSchemaCompilationDenial::Operation)?;
        Ok(installed)
    }

    fn authoring_context(&self) -> ApplicationSchemaAuthoringContext {
        ApplicationSchemaAuthoringContext::from_installed_declaration(
            self.binding_identity(),
            &self.schema,
            &self.member_provenance,
        )
    }

    pub fn owner(&self) -> &str {
        self.package_authority.owner()
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn package_identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        self.package_authority.package_identity()
    }

    pub fn binding_identity(&self) -> ApplicationSchemaBindingIdentity {
        ApplicationSchemaBindingIdentity::from_installed_parts(
            self.package_authority.runtime_ordinal,
            self.package_authority.generation.ordinal(),
            *self.package_authority.package_identity.digest(),
            self.schema_identity,
        )
    }

    /// Returns the descriptive schema meaning retained by this installed proof.
    ///
    /// The declaration itself carries no installation authority. Callers must
    /// retain this handle when an operation requires proof of the installed
    /// runtime and generation.
    pub fn installed_declaration(&self) -> &ErasedApplicationSchemaDeclaration {
        &self.schema
    }

    pub fn invariants(&self) -> &WorthQueryInstalledApplicationInvariantCatalog {
        &self.invariant_catalog
    }

    pub fn installed_invariant<Invariant>(
        &self,
        reference: worth_query_declaration::facade::application_schema::ApplicationInvariantRef<
            Schema,
            Invariant,
        >,
        execution_point: worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint,
    ) -> Option<WorthQueryInstalledApplicationInvariant<Schema, Invariant>>
    where
        Invariant:
            worth_query_declaration::facade::application_schema::ApplicationInvariantMarkerIdentity<
                Schema,
            >,
    {
        resolve_installed_invariant(
            &self.invariant_catalog,
            self.binding_identity(),
            reference,
            execution_point,
        )
    }

    /// Returns descriptive contribution ownership resolved against this exact
    /// installed declaration.
    pub fn contributions(&self) -> WorthQueryInstalledApplicationContributionCatalog<'_> {
        WorthQueryInstalledApplicationContributionCatalog::from_installed_declaration(&self.schema)
    }

    pub fn native_contracts(&self) -> &WorthQueryInstalledApplicationSchemaContractCatalog {
        self.native_contract_catalog.as_ref()
    }

    /// Returns the immutable entry-local scalar bindings validated for this
    /// exact installed schema generation.
    pub fn value_bindings(&self) -> &WorthQueryInstalledApplicationValueBindingCatalog {
        &self.value_binding_catalog
    }

    /// Returns the exact mutation-binding inventory retained at installation.
    #[doc(hidden)]
    pub fn installed_mutation_binding_inventory(
        &self,
    ) -> impl ExactSizeIterator<Item = &ApplicationMutationBindingDescriptor> {
        self.mutation_catalog.descriptors()
    }

    pub(crate) fn retain_native_contracts(
        &self,
    ) -> Arc<WorthQueryInstalledApplicationSchemaContractCatalog> {
        Arc::clone(&self.native_contract_catalog)
    }

    /// Descriptive bindings available for fresh typed query admission.
    pub fn installed_query_binding_inventory(
        &self,
    ) -> impl ExactSizeIterator<Item = &worth_query_declaration::facade::application_query::ApplicationQueryBindingDescriptor>{
        self.query_catalog.descriptors()
    }

    pub(crate) fn portable_native_contracts(
        &self,
    ) -> &[WorthQueryPortableNativeAspectContractRecord] {
        self.portable_native_contracts.as_ref()
    }

    pub(crate) fn portable_operation_contracts(
        &self,
    ) -> &[WorthQueryPortableApplicationOperationContractRecord] {
        self.portable_operation_contracts.as_ref()
    }

    pub fn installed_ability_requirement(
        &self,
        ability: &str,
        scope_entity: &str,
    ) -> Option<&WorthQueryInstalledAbilityRequirement> {
        self.authorization_policy_registry
            .get(ability)
            .and_then(|policies| policies.get(scope_entity))
    }

    pub const fn installation_canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.installation_canonical_work
    }

    pub fn query<Entity>(
        &self,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> TypedReadDeclarationBuilder<Schema, Entity> {
        TypedReadDeclarationBuilder::new(entity).with_installed_context(self.authoring_context())
    }

    pub fn operation<Operation: 'static, Input>(
        &self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
    ) -> TypedOperationBuilder<Schema, Operation, Input>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        TypedOperationBuilder::new(operation).with_installed_context(self.authoring_context())
    }

    pub fn effects<Operation: 'static, Input>(
        &self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
    ) -> TypedEffectIntentBuilder<Schema, Operation, Input>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        TypedEffectIntentBuilder::new(operation).with_installed_context(self.authoring_context())
    }

    pub fn ability<Ability, Scope>(
        &self,
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
    ) -> Result<
        WorthQueryInstalledAbility<Schema, Ability, Scope>,
        WorthQueryAbilityInstallationDenial,
    > {
        let installed = self
            .schema
            .members()
            .iter()
            .find(|member| {
                matches!(
                    member,
                    ApplicationSchemaMember::Ability {
                        ability: installed,
                        ..
                    } if installed == ability.name()
                )
            })
            .ok_or_else(|| {
                WorthQueryAbilityInstallationDenial::new(
                    WorthQueryAbilityInstallationDenialKind::AbilityNotInstalled,
                    ability.name(),
                )
            })?;
        let ApplicationSchemaMember::Ability {
            ability: installed_name,
            scope_entity,
        } = installed
        else {
            unreachable!("ability lookup returned a non-ability member")
        };
        if installed_name != ability.name() || scope_entity != ability.scope() {
            return Err(WorthQueryAbilityInstallationDenial::new(
                WorthQueryAbilityInstallationDenialKind::AbilityMeaningChanged,
                ability.name(),
            ));
        }
        Ok(WorthQueryInstalledAbility::from_installed_schema(
            self,
            installed_name,
            scope_entity,
        ))
    }

    pub fn installed_operation<Operation, Input>(
        &self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
    ) -> Result<
        WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        WorthQueryApplicationOperationInstallationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        WorthQueryInstalledApplicationOperation::from_installed_schema(self, operation.name())
    }

    #[doc(hidden)]
    pub fn installed_operation_for_capability<Capability, Operation, Input>(
        &self,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    ) -> Result<
        crate::application_operation::WorthQueryInstalledApplicationOperationGraphAuthority<
            Schema,
            Operation,
            Input,
        >,
        WorthQueryApplicationOperationInstallationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        WorthQueryInstalledApplicationOperation::graph_authority_from_installed_schema(
            self, capability,
        )
    }

    pub fn validate_installed_query<Query, Parameters, QueryResult, Scope>(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
        let expected = self.binding_identity();
        let actual = query.binding_identity();
        let kind = if actual.runtime_ordinal() != expected.runtime_ordinal() {
            Some(
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::ForeignRuntime,
            )
        } else if actual.generation() != expected.generation() {
            Some(
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
            )
        } else if actual.package_identity() != expected.package_identity() {
            Some(
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::PackageIdentityChanged,
            )
        } else if actual.schema_identity() != expected.schema_identity() {
            Some(
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::SchemaMeaningChanged,
            )
        } else {
            None
        };
        if let Some(kind) = kind {
            return Err(WorthQueryApplicationQueryInstallationDenial::new(
                kind,
                query.name(),
            ));
        }
        if !query.authority_matches(&self.package_authority) {
            return Err(WorthQueryApplicationQueryInstallationDenial::new(
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::AuthorityMismatch,
                query.name(),
            ));
        }
        Ok(())
    }
}
