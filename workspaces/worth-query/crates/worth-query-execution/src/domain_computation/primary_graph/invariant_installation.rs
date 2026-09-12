use std::collections::BTreeMap;
use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationSchema, ApplicationSchemaBindingIdentity,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationInvariant, WorthQueryInstalledApplicationSchema,
};
use worth_relational::facade::runtime::CustomInvariantRegistration;

use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

use super::application_invariant::{
    ErasedApplicationInvariantRule, WorthQueryApplicationInvariantRule,
};

type Factory<Schema> = Box<
    dyn for<'resolver> FnOnce(
            &WorthQueryApplicationInvariantSchemaResolver<'resolver, Schema>,
        ) -> Result<Box<dyn ErasedApplicationInvariantRule>, String>
        + Send,
>;

/// Borrowed, application-only view used while lowering one invariant factory.
pub struct WorthQueryApplicationInvariantSchemaResolver<'layout, Schema> {
    _schema: PhantomData<fn() -> Schema>,
    layout: &'layout super::schema_layout::WorthQueryPrimaryGraphLayout,
}

impl<Schema> WorthQueryApplicationInvariantSchemaResolver<'_, Schema> {
    pub fn entity_kind<Entity>(
        &self,
        entity: worth_query_declaration::facade::application_schema::ApplicationEntityRef<
            Schema,
            Entity,
        >,
    ) -> Option<worth_relational::facade::identity::KindId> {
        self.layout.entity_kind(entity.name())
    }
    pub fn relation<Relation, From, To>(
        &self,
        relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
            Schema,
            Relation,
            From,
            To,
        >,
    ) -> Option<(
        worth_relational::facade::identity::KindId,
        worth_relational::facade::identity::KindId,
        worth_relational::facade::identity::KindId,
    )> {
        let layout = self.layout.relation(relation.name())?;
        Some((layout.kind, layout.from, layout.to))
    }
    pub fn field<
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit: worth_query_declaration::facade::application_schema::ApplicationFieldUnit,
    >(
        &self,
        field: worth_query_declaration::facade::application_schema::ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Option<(
        worth_relational::facade::identity::KindId,
        worth_foundational::facade::AspectFieldLocator,
    )> {
        self.layout
            .field(field.entity(), field.aspect(), field.field())
            .map(|layout| (layout.entity_kind, layout.locator.clone()))
    }
}

pub struct WorthQueryApplicationInvariantFactories<Schema> {
    binding_identity: ApplicationSchemaBindingIdentity,
    expected: BTreeMap<
        (String, ApplicationInvariantExecutionPoint),
        worth_query_installation::facade::WorthQueryInstalledApplicationInvariantDescriptor,
    >,
    factories: BTreeMap<(String, ApplicationInvariantExecutionPoint), Factory<Schema>>,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryApplicationInvariantFactories<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn for_installed_schema(schema: &WorthQueryInstalledApplicationSchema<Schema>) -> Self {
        let expected = schema
            .invariants()
            .descriptors()
            .map(|descriptor| {
                (
                    (
                        descriptor.identifier().to_owned(),
                        descriptor.execution_point(),
                    ),
                    descriptor.clone(),
                )
            })
            .collect();
        Self {
            binding_identity: schema.binding_identity(),
            expected,
            factories: BTreeMap::new(),
            _schema: PhantomData,
        }
    }

    pub fn bind<Invariant, Rule: WorthQueryApplicationInvariantRule>(
        &mut self,
        installed: &WorthQueryInstalledApplicationInvariant<Schema, Invariant>,
        factory: impl for<'resolver> FnOnce(
                &WorthQueryApplicationInvariantSchemaResolver<'resolver, Schema>,
            ) -> Result<Rule, String>
            + Send
            + 'static,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if installed.binding_identity() != &self.binding_identity {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignInvariantFactory,
                installed.descriptor().identifier(),
            ));
        }
        let key = (
            installed.descriptor().identifier().to_owned(),
            installed.descriptor().execution_point(),
        );
        if self.factories.contains_key(&key) {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::DuplicateInvariantFactory,
                installed.descriptor().identifier(),
            ));
        }
        self.factories.insert(
            key,
            Box::new(move |resolver| {
                factory(resolver)
                    .map(|rule| Box::new(rule) as Box<dyn ErasedApplicationInvariantRule>)
            }),
        );
        Ok(())
    }

    pub(super) fn lower(
        mut self,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
    ) -> Result<Vec<CustomInvariantRegistration>, WorthQueryPrimaryGraphInstallationDenial> {
        if binding_identity != self.binding_identity {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignInvariantFactory,
                "factory inventory belongs to another installed schema",
            ));
        }
        if self.factories.len() != self.expected.len() {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::MissingInvariantFactory,
                "factory inventory does not match the installed invariant catalog",
            ));
        }
        let mut lowered = Vec::with_capacity(self.expected.len());
        for (key, installed) in self.expected {
            if !self.factories.contains_key(&key) {
                return Err(denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::MissingInvariantFactory,
                    &key.0,
                ));
            }
            let factory = self
                .factories
                .remove(&key)
                .expect("checked factory remains present");
            let resolver = WorthQueryApplicationInvariantSchemaResolver {
                layout,
                _schema: PhantomData,
            };
            let rule = factory(&resolver).map_err(|detail| {
                denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected,
                    detail,
                )
            })?;
            let descriptor = lower_descriptor(&installed, layout)?;
            let registration = rule.into_registration(descriptor).map_err(|detail| {
                denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected,
                    detail,
                )
            })?;
            if !is_mandatory_candidate_registration(&registration) {
                return Err(denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryMeaningMismatch,
                    &key.0,
                ));
            }
            lowered.push(registration);
        }
        Ok(lowered)
    }
}

fn lower_access_contract(
    installed: &worth_query_installation::facade::WorthQueryInstalledApplicationInvariantDescriptor,
    layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
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
    };
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

fn lower_descriptor(
    installed: &worth_query_installation::facade::WorthQueryInstalledApplicationInvariantDescriptor,
    layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
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
            maximum_work_units: installed.maximum_work_units(),
            execution_point,
            groups: InvariantGroupSet::from_mask(installed_group_mask(installed.required_groups())),
            cost_class: installed_cost(installed.cost_posture()),
            failure_effect: installed_enforcement(installed.enforcement()),
            access: lower_access_contract(installed, layout)?,
        },
    })
}

fn is_mandatory_candidate_registration(registration: &CustomInvariantRegistration) -> bool {
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

fn denial(
    kind: WorthQueryPrimaryGraphInstallationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
