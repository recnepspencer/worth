//! Lower the installed rule contract into native invariant metadata.
use super::{
    denial, CustomInvariantRegistration, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
fn lower_access_contract(
    installed: &worth_query_installation::facade::WorthQueryInstalledApplicationInvariantDescriptor,
    layout: &super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    reads_program_activation: bool,
) -> Result<
    worth_relational::facade::runtime::CustomInvariantAccessContract,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    use worth_query_declaration::facade::application_schema::ApplicationInvariantScopeTarget;
    let mut contract = worth_relational::facade::runtime::CustomInvariantAccessContract {
        read_entity_kinds: Vec::new(),
        read_relation_kinds: Vec::new(),
        affected_entity_kinds: Vec::new(),
        affected_relation_kinds: Vec::new(),
        include_relation_endpoint_entity_touches: true,
    };
    if reads_program_activation {
        contract
            .read_entity_kinds
            .push(layout.program_activation().entity_kind);
    }
    for target in installed.read_closure() {
        match target {
            ApplicationInvariantScopeTarget::Entity(name) => contract.read_entity_kinds.push(
                layout.entity_kind(name).ok_or_else(|| denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryMeaningMismatch, name))?),
            ApplicationInvariantScopeTarget::Relation(name) => contract.read_relation_kinds.push(
                layout.relation(name).map(|relation| relation.kind).ok_or_else(|| denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryMeaningMismatch, name))?),
        }
    }
    for target in installed.applicability() {
        match target {
            ApplicationInvariantScopeTarget::Entity(name) => contract.affected_entity_kinds.push(
                layout.entity_kind(name).ok_or_else(|| denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryMeaningMismatch, name))?),
            ApplicationInvariantScopeTarget::Relation(name) => contract.affected_relation_kinds.push(
                layout.relation(name).map(|relation| relation.kind).ok_or_else(|| denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryMeaningMismatch, name))?),
        }
    }
    Ok(contract.canonicalize())
}

pub(super) fn lower_descriptor(
    installed: &worth_query_installation::facade::WorthQueryInstalledApplicationInvariantDescriptor,
    layout: &super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    reads_program_activation: bool,
) -> Result<
    worth_relational::facade::runtime::CustomInvariantDescriptor,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    use worth_relational::facade::runtime::*;
    if installed.provider() != "primary-relational-provider" {
        return Err(denial(
            WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryMeaningMismatch,
            installed.identifier(),
        ));
    }
    let execution_point = match installed.execution_point() {
        ApplicationInvariantExecutionPoint::CommitBoundary => {
            InvariantExecutionPoint::CommitBoundary
        }
        ApplicationInvariantExecutionPoint::MutationSensitive => {
            InvariantExecutionPoint::MutationSensitive
        }
        ApplicationInvariantExecutionPoint::SnapshotPublication => {
            InvariantExecutionPoint::SnapshotPublication
        }
    };
    Ok(CustomInvariantDescriptor {
        identity: CustomInvariantSemanticIdentity {
            rule_id: CustomInvariantRuleId::new(installed.identifier()),
            semantic_version: CustomInvariantSemanticVersion::new(
                installed.major(),
                installed.minor(),
            ),
        },
        display_name: installed.identifier().into(),
        operational: CustomInvariantOperationalMetadata {
            maximum_work_units: super::program_activation_work_reserve::lowered_work_budget(
                installed.maximum_work_units(),
                installed.identifier(),
                reads_program_activation,
            )?,
            execution_point,
            groups: InvariantGroupSet::from_mask(installed_group_mask(installed.required_groups())),
            cost_class: installed_cost(installed.cost_posture()),
            failure_effect: installed_enforcement(installed.enforcement()),
            access: lower_access_contract(installed, layout, reads_program_activation)?,
        },
    })
}

pub(super) fn is_mandatory_candidate_registration(
    registration: &CustomInvariantRegistration,
) -> bool {
    use worth_relational::facade::runtime::{
        InvariantExecutionPoint as Point, InvariantFailureEffect as Effect, InvariantGroup as Group,
    };
    let required_effect = match registration.execution_point() {
        Point::CommitBoundary | Point::MutationSensitive => Effect::BlockCommit,
        Point::SnapshotPublication => Effect::BlockPublication,
        _ => return false,
    };
    if registration.failure_effect() != required_effect {
        return false;
    }
    let groups = registration.groups();
    match registration.execution_point() {
        Point::CommitBoundary => {
            groups.contains(Group::StorageCoherence)
                || groups.contains(Group::IdentityCoherence)
                || groups.contains(Group::SchemaCompliance)
                || groups.contains(Group::RelationIntegrity)
                || groups.contains(Group::LineageIntegrity)
                || groups.contains(Group::PublicationCoherence)
        }
        Point::MutationSensitive => {
            groups.contains(Group::StorageCoherence)
                || groups.contains(Group::IdentityCoherence)
                || groups.contains(Group::SchemaCompliance)
                || groups.contains(Group::RelationIntegrity)
                || groups.contains(Group::AdjacencyIntegrity)
                || groups.contains(Group::LineageIntegrity)
        }
        Point::SnapshotPublication => {
            groups.contains(Group::VersionVisibility)
                || groups.contains(Group::PublicationCoherence)
        }
        _ => false,
    }
}

fn installed_group_mask(
    groups: &[worth_query_declaration::facade::application_schema::ApplicationInvariantGroup],
) -> u32 {
    use worth_query_declaration::facade::application_schema::ApplicationInvariantGroup as App;
    use worth_relational::facade::runtime::InvariantGroup as Rel;
    groups.iter().fold(0, |mask, group| {
        mask | match group {
            App::StorageCoherence => Rel::StorageCoherence.mask(),
            App::VersionVisibility => Rel::VersionVisibility.mask(),
            App::AdjacencyIntegrity => Rel::AdjacencyIntegrity.mask(),
            App::IdentityCoherence => Rel::IdentityCoherence.mask(),
            App::SchemaCompliance => Rel::SchemaCompliance.mask(),
            App::LineageIntegrity => Rel::LineageIntegrity.mask(),
            App::PublicationCoherence => Rel::PublicationCoherence.mask(),
            App::RelationIntegrity => Rel::RelationIntegrity.mask(),
        }
    })
}

fn installed_cost(
    cost: worth_query_declaration::facade::application_schema::ApplicationInvariantCostPosture,
) -> worth_relational::facade::runtime::InvariantCostClass {
    use worth_query_declaration::facade::application_schema::ApplicationInvariantCostPosture as App;
    use worth_relational::facade::runtime::InvariantCostClass as Rel;
    match cost {
        App::Touched => Rel::Touched,
        App::Partition => Rel::Partition,
        App::Global => Rel::Global,
    }
}

fn installed_enforcement(
    effect: worth_query_declaration::facade::application_schema::ApplicationInvariantEnforcement,
) -> worth_relational::facade::runtime::InvariantFailureEffect {
    use worth_query_declaration::facade::application_schema::ApplicationInvariantEnforcement as App;
    use worth_relational::facade::runtime::InvariantFailureEffect as Rel;
    match effect {
        App::BlockCommit => Rel::BlockCommit,
        App::BlockPublication => Rel::BlockPublication,
    }
}
