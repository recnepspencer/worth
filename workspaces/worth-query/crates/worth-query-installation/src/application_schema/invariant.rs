use std::marker::PhantomData;
use std::num::NonZeroU64;

use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantEnforcement,
    ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantRef, ApplicationInvariantScopeTarget,
    ApplicationSchemaBindingIdentity, ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationInvariantDescriptor {
    identifier: String,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
    maximum_work_units: NonZeroU64,
    enforcement: ApplicationInvariantEnforcement,
    required_groups: Vec<ApplicationInvariantGroup>,
    read_closure: Vec<ApplicationInvariantScopeTarget>,
    applicability: Vec<ApplicationInvariantScopeTarget>,
    provider: String,
    cost_posture: ApplicationInvariantCostPosture,
}

impl WorthQueryInstalledApplicationInvariantDescriptor {
    pub(crate) fn from_installed_parts(
        identifier: String,
        major: u16,
        minor: u16,
        execution_point: ApplicationInvariantExecutionPoint,
        maximum_work_units: NonZeroU64,
        enforcement: ApplicationInvariantEnforcement,
        required_groups: Vec<ApplicationInvariantGroup>,
        read_closure: Vec<ApplicationInvariantScopeTarget>,
        applicability: Vec<ApplicationInvariantScopeTarget>,
        provider: String,
        cost_posture: ApplicationInvariantCostPosture,
    ) -> Self {
        Self {
            identifier,
            major,
            minor,
            execution_point,
            maximum_work_units,
            enforcement,
            required_groups,
            read_closure,
            applicability,
            provider,
            cost_posture,
        }
    }
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    pub const fn major(&self) -> u16 {
        self.major
    }
    pub const fn minor(&self) -> u16 {
        self.minor
    }
    pub const fn execution_point(&self) -> ApplicationInvariantExecutionPoint {
        self.execution_point
    }
    pub const fn maximum_work_units(&self) -> NonZeroU64 {
        self.maximum_work_units
    }
    pub const fn enforcement(&self) -> ApplicationInvariantEnforcement {
        self.enforcement
    }
    pub fn required_groups(&self) -> &[ApplicationInvariantGroup] {
        &self.required_groups
    }
    pub fn read_closure(&self) -> &[ApplicationInvariantScopeTarget] {
        &self.read_closure
    }
    pub fn applicability(&self) -> &[ApplicationInvariantScopeTarget] {
        &self.applicability
    }
    pub fn provider(&self) -> &str {
        &self.provider
    }
    pub const fn cost_posture(&self) -> ApplicationInvariantCostPosture {
        self.cost_posture
    }
    pub(crate) fn canonical_parts(&self) -> Vec<String> {
        let mut parts = vec![
            self.identifier.clone(),
            self.major.to_string(),
            self.minor.to_string(),
            format!("{:?}", self.execution_point),
            self.maximum_work_units.get().to_string(),
            format!("{:?}", self.enforcement),
            self.provider.clone(),
            format!("{:?}", self.cost_posture),
            self.required_groups.len().to_string(),
        ];
        parts.extend(
            self.required_groups
                .iter()
                .map(|group| format!("{group:?}")),
        );
        parts.push(self.read_closure.len().to_string());
        parts.extend(self.read_closure.iter().map(|target| format!("{target:?}")));
        parts.push(self.applicability.len().to_string());
        parts.extend(
            self.applicability
                .iter()
                .map(|target| format!("{target:?}")),
        );
        parts
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationInvariantCatalog {
    descriptors: Vec<WorthQueryInstalledApplicationInvariantDescriptor>,
}

impl WorthQueryInstalledApplicationInvariantCatalog {
    pub(crate) fn compile(declaration: &ErasedApplicationSchemaDeclaration) -> Self {
        let descriptors = declaration
            .members()
            .iter()
            .filter_map(|member| {
                let ApplicationSchemaMember::ApplicationInvariant {
                    invariant,
                    major,
                    minor,
                    execution_point,
                    maximum_work_units,
                    enforcement,
                    required_groups,
                    read_closure,
                    applicability,
                    provider,
                    cost_posture,
                } = member
                else {
                    return None;
                };
                Some(WorthQueryInstalledApplicationInvariantDescriptor {
                    identifier: invariant.clone(),
                    major: *major,
                    minor: *minor,
                    execution_point: *execution_point,
                    maximum_work_units: *maximum_work_units,
                    enforcement: *enforcement,
                    required_groups: required_groups.clone(),
                    read_closure: read_closure.clone(),
                    applicability: applicability.clone(),
                    provider: provider.clone(),
                    cost_posture: *cost_posture,
                })
            })
            .collect();
        Self { descriptors }
    }

    pub fn descriptors(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryInstalledApplicationInvariantDescriptor> {
        self.descriptors.iter()
    }

    fn find(
        &self,
        identifier: &str,
        point: ApplicationInvariantExecutionPoint,
    ) -> Option<&WorthQueryInstalledApplicationInvariantDescriptor> {
        self.descriptors.iter().find(|descriptor| {
            descriptor.identifier == identifier && descriptor.execution_point == point
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationInvariant<Schema, Invariant> {
    binding_identity: ApplicationSchemaBindingIdentity,
    descriptor: WorthQueryInstalledApplicationInvariantDescriptor,
    _marker: PhantomData<fn() -> (Schema, Invariant)>,
}

impl<Schema, Invariant> WorthQueryInstalledApplicationInvariant<Schema, Invariant> {
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }
    pub fn descriptor(&self) -> &WorthQueryInstalledApplicationInvariantDescriptor {
        &self.descriptor
    }
}

pub(crate) fn resolve_installed_invariant<Schema, Invariant>(
    catalog: &WorthQueryInstalledApplicationInvariantCatalog,
    binding_identity: ApplicationSchemaBindingIdentity,
    reference: ApplicationInvariantRef<Schema, Invariant>,
    point: ApplicationInvariantExecutionPoint,
) -> Option<WorthQueryInstalledApplicationInvariant<Schema, Invariant>>
where
    Invariant: ApplicationInvariantMarkerIdentity<Schema>,
{
    catalog
        .find(reference.identifier(), point)
        .filter(|descriptor| {
            descriptor.major == reference.major() && descriptor.minor == reference.minor()
        })
        .cloned()
        .map(|descriptor| WorthQueryInstalledApplicationInvariant {
            binding_identity,
            descriptor,
            _marker: PhantomData,
        })
}
