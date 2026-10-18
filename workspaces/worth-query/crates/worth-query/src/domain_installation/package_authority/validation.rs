use super::{
    WorthQueryDomainDeclarationFamilyDefinition, WorthQueryDomainGraphReadOperationDefinition,
    WorthQueryDomainIdentityDeclaration, WorthQueryDomainInvariantDefinition,
    WorthQueryDomainOperationDefinitionRecord, WorthQueryDomainOperationGraphParticipationRecord,
    WorthQueryDomainOperationRequiredDomainRecord, WorthQueryDomainPackage,
    WorthQueryDomainPackageIdentity,
};

mod operation_graph_participation;
mod operation_required_domain;
mod portable_denial_mapping;
use operation_graph_participation::validate_operation_graph_participations;
use operation_required_domain::validate_operation_required_domains;
use portable_denial_mapping::map_portable_validation_denial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainPackageValidationDenialKind {
    MarkerIdentityMismatch,
    MissingMarkerCapability,
    DuplicateInvariant,
    ConflictingInvariant,
    DuplicateGraphReadOperation,
    ConflictingGraphReadOperation,
    DuplicateDeclarationFamily,
    ConflictingDeclarationFamily,
    DuplicateContributionCategory,
    EmptyGraphReadRelationSet,
    InvalidInvariantPredicate,
    InvalidPortablePackage,
    InvalidDomainOperation,
    MissingDomainOperationCapability,
    MissingDomainOperationInvariant,
    DuplicateDomainOperation,
    ConflictingDomainOperation,
    ConflictingDomainOperationMarker,
    InvalidDomainOperationGraphParticipation,
    InvalidDomainOperationRequiredDomain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainPackageValidationDenial {
    kind: WorthQueryDomainPackageValidationDenialKind,
    detail: String,
}

impl WorthQueryDomainPackageValidationDenial {
    pub(crate) fn new(
        kind: WorthQueryDomainPackageValidationDenialKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryDomainPackageValidationDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for WorthQueryDomainPackageValidationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for WorthQueryDomainPackageValidationDenial {}
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily,
    WorthQueryDeclarationEntryContributionCategoryFamily, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingRequirement,
};

pub(crate) struct WorthQueryValidatedDomainPackage<D: WorthQueryDomainEntryMarker> {
    pub(crate) marker: D,
    pub(crate) identity: WorthQueryDomainIdentityDeclaration<D>,
    pub(crate) package_identity: WorthQueryDomainPackageIdentity,
    pub(crate) required_capabilities: Vec<WorthQueryCapabilityFamily>,
    pub(crate) required_configuration: Vec<WorthQueryConfigSectionFamily>,
    pub(crate) operating_requirements: Vec<WorthQueryDomainOperatingRequirement>,
    pub(crate) invariant_definitions: Vec<WorthQueryDomainInvariantDefinition>,
    pub(crate) graph_read_operations: Vec<WorthQueryDomainGraphReadOperationDefinition>,
    pub(crate) declaration_families: Vec<WorthQueryDomainDeclarationFamilyDefinition>,
    pub(crate) domain_operations: Vec<WorthQueryDomainOperationDefinitionRecord>,
    pub(crate) operation_graph_participations:
        Vec<WorthQueryDomainOperationGraphParticipationRecord>,
    pub(crate) operation_required_domains: Vec<WorthQueryDomainOperationRequiredDomainRecord>,
    pub(crate) contribution_policy: Vec<WorthQueryDeclarationEntryContributionCategoryFamily>,
    pub(crate) portable_package:
        worth_query_installation::facade::WorthQueryValidatedPortableDomainPackage,
}

impl<D: WorthQueryDomainEntryMarker> WorthQueryValidatedDomainPackage<D> {
    #[cfg(test)]
    pub fn identity(&self) -> &WorthQueryDomainPackageIdentity {
        &self.package_identity
    }
    #[cfg(test)]
    pub fn invariant_count(&self) -> usize {
        self.invariant_definitions.len()
    }
    #[cfg(test)]
    pub fn graph_read_operation_count(&self) -> usize {
        self.graph_read_operations.len()
    }
    #[cfg(test)]
    pub fn declaration_family_count(&self) -> usize {
        self.declaration_families.len()
    }
    #[cfg(test)]
    pub fn contribution_category_count(&self) -> usize {
        self.contribution_policy.len()
    }
}

pub(super) fn validate_domain_package<D: WorthQueryDomainEntryMarker>(
    package: WorthQueryDomainPackage<D>,
) -> Result<WorthQueryValidatedDomainPackage<D>, WorthQueryDomainPackageValidationDenial> {
    let marker_owner = package.marker.domain_key();
    let identity_owner = package.identity.canonical_owner();
    if marker_owner != identity_owner {
        return Err(WorthQueryDomainPackageValidationDenial::new(
            WorthQueryDomainPackageValidationDenialKind::MarkerIdentityMismatch,
            format!(
                "domain marker `{marker_owner}` does not match package identity `{identity_owner}`"
            ),
        ));
    }
    if let Some(missing) = package
        .marker
        .required_capability_families()
        .iter()
        .find(|family| !package.required_capabilities.contains(family))
    {
        return Err(WorthQueryDomainPackageValidationDenial::new(
            WorthQueryDomainPackageValidationDenialKind::MissingMarkerCapability,
            format!(
                "domain marker `{marker_owner}` requires package capability `{}`",
                missing.as_str()
            ),
        ));
    }
    let required_capabilities = canonicalize(package.required_capabilities);
    let required_configuration = canonicalize(package.required_configuration);
    let operating_requirements = canonicalize(package.operating_requirements);
    let mut invariant_definitions = package.invariant_definitions;
    let mut graph_read_operations = package.graph_read_operations;
    let mut declaration_families = package.declaration_families;
    let mut domain_operations = package.domain_operations;
    let operation_graph_participations = package.operation_graph_participations;
    let operation_required_domains = package.operation_required_domains;
    let artifact_contracts = package.artifact_contracts;
    let application_schemas = package.application_schemas;
    let conditional_application_operations = package.conditional_application_operations;
    let mut contribution_policy = package.contribution_policy;

    validate_invariant_predicates(&invariant_definitions)?;
    validate_operation_relations(&graph_read_operations)?;
    validate_domain_operation_markers(&domain_operations)?;
    validate_operation_graph_participations(&domain_operations, &operation_graph_participations)?;
    validate_operation_required_domains(&domain_operations, &operation_required_domains)?;
    validate_domain_operation_dependencies(
        &domain_operations,
        &required_capabilities,
        &invariant_definitions,
    )?;

    let portable_package = super::portable_validation::validate_portable_package(
        super::portable_validation::WorthQueryPortablePackageDeclaration {
            identity: &package.identity,
            required_capabilities: &required_capabilities,
            required_configuration: &required_configuration,
            operating_requirements: &operating_requirements,
            invariant_definitions: &invariant_definitions,
            graph_read_operations: &graph_read_operations,
            declaration_families: &declaration_families,
            domain_operations: &domain_operations,
            artifact_contracts: &artifact_contracts,
            application_schemas: &application_schemas,
            conditional_application_operations: &conditional_application_operations,
            contribution_policy: &contribution_policy,
        },
    )
    .map_err(map_portable_validation_denial)?;

    invariant_definitions.sort_by_key(WorthQueryDomainInvariantDefinition::canonical_part);
    graph_read_operations.sort_by_key(WorthQueryDomainGraphReadOperationDefinition::canonical_part);
    declaration_families.sort_by_key(WorthQueryDomainDeclarationFamilyDefinition::canonical_part);
    domain_operations.sort_by(|left, right| {
        left.definition()
            .identity()
            .cmp(right.definition().identity())
    });
    contribution_policy.sort_by_key(|category| category.as_str());
    let package_identity = WorthQueryDomainPackageIdentity::new(
        crate::evidence_identity::worth_query_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceScope::DomainPackageIdentity,
        )
        .field_digest(
            crate::evidence_identity::WorthQueryEvidenceTag::new("portable_package"),
            portable_package.identity().digest(),
        )
        .seal(),
    );
    Ok(WorthQueryValidatedDomainPackage {
        marker: package.marker,
        identity: package.identity,
        package_identity,
        required_capabilities,
        required_configuration,
        operating_requirements,
        invariant_definitions,
        graph_read_operations,
        declaration_families,
        domain_operations,
        operation_graph_participations,
        operation_required_domains,
        contribution_policy,
        portable_package,
    })
}

fn canonicalize<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values.dedup();
    values
}

fn validate_invariant_predicates(
    definitions: &[WorthQueryDomainInvariantDefinition],
) -> Result<(), WorthQueryDomainPackageValidationDenial> {
    if let Some(invalid) = definitions.iter().find(|definition| {
        matches!(
            definition.predicate(),
            super::WorthQueryDomainInvariantPredicate::RequiresOutgoingRelations {
                relevant_entity_kinds,
                required_relation_kinds,
                traversal_depth,
            } if relevant_entity_kinds.is_empty()
                || required_relation_kinds.is_empty()
                || *traversal_depth == 0
        )
    }) {
        return Err(WorthQueryDomainPackageValidationDenial::new(
            WorthQueryDomainPackageValidationDenialKind::InvalidInvariantPredicate,
            invalid.name().as_str(),
        ));
    }
    Ok(())
}

fn validate_operation_relations(
    definitions: &[WorthQueryDomainGraphReadOperationDefinition],
) -> Result<(), WorthQueryDomainPackageValidationDenial> {
    if let Some(empty) = definitions
        .iter()
        .find(|definition| definition.accepted_relations().is_empty())
    {
        return Err(WorthQueryDomainPackageValidationDenial::new(
            WorthQueryDomainPackageValidationDenialKind::EmptyGraphReadRelationSet,
            empty.name().as_str(),
        ));
    }
    Ok(())
}

fn validate_domain_operation_markers(
    definitions: &[WorthQueryDomainOperationDefinitionRecord],
) -> Result<(), WorthQueryDomainPackageValidationDenial> {
    for (index, definition) in definitions.iter().enumerate() {
        if definitions[..index].iter().any(|prior| {
            prior.operation_marker() == definition.operation_marker()
                && prior.family_marker() == definition.family_marker()
                && prior.definition().identity() != definition.definition().identity()
        }) {
            return Err(WorthQueryDomainPackageValidationDenial::new(
                WorthQueryDomainPackageValidationDenialKind::ConflictingDomainOperationMarker,
                definition.definition().identity().slot(),
            ));
        }
    }
    Ok(())
}

fn validate_domain_operation_dependencies(
    operations: &[WorthQueryDomainOperationDefinitionRecord],
    package_capabilities: &[WorthQueryCapabilityFamily],
    invariants: &[WorthQueryDomainInvariantDefinition],
) -> Result<(), WorthQueryDomainPackageValidationDenial> {
    for operation in operations {
        let definition = operation.definition();
        let direct_missing = definition
            .semantics()
            .required_capabilities
            .iter()
            .find(|required| {
                !package_capabilities
                    .iter()
                    .any(|installed| installed.satisfies_operation_requirement(**required))
            });
        let workflow_missing = match &definition.semantics().workflow {
            worth_query_installation::facade::WorthQueryOperationWorkflowContract::Declared(
                workflow,
            ) => workflow
                .stages()
                .iter()
                .flat_map(|stage| stage.required_capabilities())
                .find(|required| {
                    !package_capabilities
                        .iter()
                        .any(|installed| installed.satisfies_operation_requirement(**required))
                }),
            worth_query_installation::facade::WorthQueryOperationWorkflowContract::NotRequired => {
                None
            }
        };
        if let Some(missing) = direct_missing.or(workflow_missing) {
            return Err(WorthQueryDomainPackageValidationDenial::new(
                WorthQueryDomainPackageValidationDenialKind::MissingDomainOperationCapability,
                format!(
                    "{} requires `{}`",
                    definition.identity().slot(),
                    missing.as_str()
                ),
            ));
        }
        if let worth_query_installation::facade::WorthQueryOperationInvariantContract::Declared {
            invariant_slots,
        } = &definition.semantics().invariants
            && let Some(missing) = invariant_slots.iter().find(|required| {
                !invariants
                    .iter()
                    .any(|item| item.slot_key() == required.as_str())
            })
        {
            return Err(WorthQueryDomainPackageValidationDenial::new(
                WorthQueryDomainPackageValidationDenialKind::MissingDomainOperationInvariant,
                format!("{} requires `{missing}`", definition.identity().slot()),
            ));
        }
    }
    Ok(())
}
