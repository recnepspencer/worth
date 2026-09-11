mod operation_compilation;
mod reinstallation_match;

pub(in crate::application_operation) use operation_compilation::WorthQuerySealedOperationContractCompilation;

#[cfg(test)]
pub(crate) mod aftermath_install_fixture;

#[cfg(test)]
mod operation_compilation_tests;

use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationSchemaBindingIdentity,
    ApplicationStructuredValueBinding,
};

use super::contract_resolution::ability_requirements;
use super::installed_contract_support::{
    authority_identity, graph_obligation_denial, operation_capability_requirements,
};
use super::operation_declaration_resolution::{
    resolve_operation_declaration, ResolvedApplicationOperationDeclaration,
};
use super::{
    WorthQueryApplicationOperationInstallationDenial,
    WorthQueryCompiledApplicationOperationContracts,
};
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use crate::authority_cryptography::AuthoritySeal;
use crate::graph_obligation::{
    bind_capability_operation_obligations, bind_operation_obligations,
    WorthQueryApplicationOperationObligationSource, WorthQueryInstalledGraphCapabilityRequirement,
    WorthQueryInstalledGraphObligationInspection, WorthQueryInstalledGraphObligationSet,
};
use crate::package::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableInstalledReconciliationProcedureRecord,
    WorthQueryPortableNativeAspectContractRecord,
};

mod aftermath_installation_source_seal {
    pub trait Sealed {}
}

/// Read-only view of one whole candidate operation for aftermath installation.
///
/// The private supertrait keeps candidate construction with this installed-
/// operation owner. The aftermath owner may read the already-resolved axes,
/// but no other module can implement or recombine this source.
pub(crate) trait WorthQueryOperationAftermathInstallationSource:
    aftermath_installation_source_seal::Sealed
{
    fn binding(&self) -> &ApplicationSchemaBindingIdentity;

    fn operation(&self) -> &str;

    fn portable_decision_reads(
        &self,
    ) -> &[worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget];

    fn external_effect(&self) -> &crate::application_aftermath::InstalledExternalEffectContract;

    fn portable_aftermath(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_aftermath::PortableApplicationAftermathContract,
    >;

    fn portable_reconciliation(
        &self,
    ) -> Option<&WorthQueryPortableInstalledReconciliationProcedureRecord>;
}

pub struct WorthQueryInstalledApplicationOperation<Schema, Operation, Input> {
    binding_identity: ApplicationSchemaBindingIdentity,
    owner: String,
    schema_name: String,
    operation: String,
    input_type: String,
    contracts: WorthQueryCompiledApplicationOperationContracts,
    native_contracts:
        Arc<crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog>,
    portable_native_contracts: Arc<Vec<WorthQueryPortableNativeAspectContractRecord>>,
    portable_contract: WorthQueryPortableApplicationOperationContractRecord,
    obligations: WorthQueryInstalledGraphObligationSet,
    authority_identity: AuthoritySeal,
    _marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

/// Installation-owned graph authority for an operation named by a capability.
///
/// This view intentionally does not grant executable operation authority. A
/// capability may name an operation that is used only as an authorization
/// target, such as a governed application query.
pub struct WorthQueryInstalledApplicationOperationGraphAuthority<Schema, Operation, Input> {
    binding_identity: ApplicationSchemaBindingIdentity,
    operation: String,
    obligations: WorthQueryInstalledGraphObligationSet,
    authority_identity: AuthoritySeal,
    _marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

impl<Schema, Operation, Input>
    WorthQueryInstalledApplicationOperationGraphAuthority<Schema, Operation, Input>
{
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn authority_identity(&self) -> &str {
        self.authority_identity.as_str()
    }

    pub const fn graph_obligations(&self) -> WorthQueryInstalledGraphObligationInspection<'_> {
        self.obligations.inspect()
    }

    #[doc(hidden)]
    pub fn retain_graph_obligations_for_admission(&self) -> WorthQueryInstalledGraphObligationSet {
        self.obligations.clone()
    }
}

impl<Schema, Operation, Input> WorthQueryInstalledApplicationOperation<Schema, Operation, Input> {
    pub(crate) fn from_installed_schema(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        operation: &str,
    ) -> Result<Self, WorthQueryApplicationOperationInstallationDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        let declaration =
            resolve_operation_declaration::<Schema, Operation, Input>(schema, operation)?;
        Self::install_executable_operation(schema, &declaration)
    }

    pub(crate) fn graph_authority_from_installed_schema<Capability>(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        capability: &crate::application_capability::WorthQueryInstalledApplicationCapability<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<
        WorthQueryInstalledApplicationOperationGraphAuthority<Schema, Operation, Input>,
        WorthQueryApplicationOperationInstallationDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        let declaration = resolve_operation_declaration::<Schema, Operation, Input>(
            schema,
            capability.contract().operation(),
        )?;
        let binding_identity = schema.binding_identity();
        let requirement = WorthQueryInstalledGraphCapabilityRequirement::new(
            capability.identity().clone(),
            capability.contract().clone(),
        );
        let obligations = bind_capability_operation_obligations(
            &binding_identity,
            declaration.operation(),
            declaration.input_type(),
            requirement,
        )
        .map_err(|denial| graph_obligation_denial(declaration.operation(), denial))?;
        let authority_identity = authority_identity(
            &schema.package_authority.authority_key,
            &binding_identity,
            declaration.operation(),
            declaration.input_type(),
            obligations.identity(),
        );
        Ok(WorthQueryInstalledApplicationOperationGraphAuthority {
            binding_identity,
            operation: declaration.operation().to_owned(),
            obligations,
            authority_identity,
            _marker: PhantomData,
        })
    }

    fn install_executable_operation(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        declaration: &ResolvedApplicationOperationDeclaration,
    ) -> Result<Self, WorthQueryApplicationOperationInstallationDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        let operation = declaration.operation();
        let input_type = declaration.input_type();
        let abilities = ability_requirements(schema, operation)?;
        let binding_identity = schema.binding_identity();
        let portable_contract = schema
            .portable_operation_contracts()
            .iter()
            .find(|record| {
                record.operation() == operation && record.input_type().as_str() == input_type
            })
            .ok_or_else(|| {
                super::installed_contract_support::operation_denial(
                    super::WorthQueryApplicationOperationInstallationDenialKind::InvalidGraphObligationContract,
                    operation,
                )
            })?;
        let compilation =
            operation_compilation::WorthQueryApplicationOperationCompilation::resolve(
                binding_identity.clone(),
                schema.installed_declaration().members(),
                portable_contract,
                operation,
                input_type,
            )?;
        let contracts = compilation.compile_contracts(abilities, schema.native_contracts())?;
        let authorization = contracts.authorization();
        let capability_requirements =
            operation_capability_requirements(schema, operation, input_type);
        let obligations = bind_operation_obligations(
            &binding_identity,
            operation,
            input_type,
            WorthQueryApplicationOperationObligationSource {
                authorization,
                ability_requirements: contracts.ability_requirements(),
                capability_requirements: &capability_requirements,
                graph_reads: contracts.graph_reads(),
                touches: contracts.touches(),
                effects: contracts.effects(),
                invariants: contracts.invariants(),
                invariant_execution: contracts.invariant_execution(),
                resources: contracts.resources(),
            },
        )
        .map_err(|denial| graph_obligation_denial(operation, denial))?;
        let authority_identity = authority_identity(
            &schema.package_authority.authority_key,
            &binding_identity,
            operation,
            input_type,
            obligations.identity(),
        );
        Ok(Self {
            binding_identity,
            owner: schema.owner().to_string(),
            schema_name: schema.schema_name().to_string(),
            operation: operation.to_string(),
            input_type: input_type.to_string(),
            contracts,
            native_contracts: schema.retain_native_contracts(),
            portable_native_contracts: Arc::new(schema.portable_native_contracts().to_vec()),
            portable_contract: portable_contract.clone(),
            obligations,
            authority_identity,
            _marker: PhantomData,
        })
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn contracts(&self) -> &WorthQueryCompiledApplicationOperationContracts {
        &self.contracts
    }

    pub fn input_type(&self) -> &str {
        &self.input_type
    }

    pub const fn execution_posture(
        &self,
    ) -> super::WorthQueryInstalledApplicationOperationExecutionPosture {
        self.contracts.execution_posture()
    }

    pub fn authority_identity(&self) -> &str {
        self.authority_identity.as_str()
    }

    #[doc(hidden)]
    pub fn authority_identity_bytes(&self) -> [u8; 32] {
        *self.authority_identity.bytes()
    }

    pub const fn graph_obligations(&self) -> WorthQueryInstalledGraphObligationInspection<'_> {
        self.obligations.inspect()
    }

    #[doc(hidden)]
    pub fn retain_graph_obligations_for_admission(&self) -> WorthQueryInstalledGraphObligationSet {
        self.obligations.clone()
    }
}
