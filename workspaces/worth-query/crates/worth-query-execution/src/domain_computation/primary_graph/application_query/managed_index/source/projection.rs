//! Materialize the compiled Source shape through the current Native loan.
use super::occurrences::MaterialDenial;
use crate::domain_computation::primary_graph::{
    application_query::{
        derived_view::ViewDependency,
        disclosure::WorthQueryApplicationQueryGovernance,
        projection::{
            WorthQueryApplicationDisclosedProjectionTree as Tree,
            WorthQueryApplicationProjectedField as Field,
            WorthQueryApplicationProjectedRelation as Relation,
            WorthQueryApplicationProjectionNode as Row,
            WorthQueryApplicationWorkingProjectionTree as WorkingTree,
        },
        read_execution::validate_relation_cardinality,
    },
    source_facts::SourceFactsAllocation,
};
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;
use worth_relational::facade::{
    identity::EntityId,
    runtime::{
        DerivedRelationReadDenial, ManagedIndexSourceField, ManagedIndexSourceOccurrence,
        ManagedIndexSourceProgram, ManagedIndexSourceProjectionOccurrence, ManagedIndexSourceRows,
        PreparedManagedIndexProjectionRead,
    },
    schema::DerivedRelationDenial,
    transactions::RelationalExecutionDenialCause as Storage,
};

pub(super) struct Entry {
    pub(super) occurrence: ManagedIndexSourceOccurrence,
    pub(super) row: Tree,
    pub(super) dependencies: BTreeSet<ViewDependency>,
    _allocation: Arc<SourceFactsAllocation>,
}
impl Entry {
    pub(super) fn into_arc(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, MaterialDenial> {
        super::super::super::one_shot::outcome::material::MaterialCompletion::new(
            execution,
            &self._allocation,
        )
        .arc::<Self>()?;
        Ok(Arc::new(self))
    }
}

struct Projector<'borrow, 'scope, 'authority> {
    execution: &'borrow ExecutionRequest<'scope, 'authority>,
    rows: &'borrow ManagedIndexSourceRows<'borrow>,
    graph: &'borrow WorthQueryInstalledGraphReadContract,
    allocation: Arc<SourceFactsAllocation>,
    dependencies: BTreeSet<ViewDependency>,
}
#[derive(Clone, Copy)]
enum FieldSource<'a> {
    Root(&'a ManagedIndexSourceOccurrence),
    Endpoint(
        &'a PreparedManagedIndexProjectionRead,
        &'a ManagedIndexSourceProjectionOccurrence,
    ),
}

pub(super) fn project(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    rows: &ManagedIndexSourceRows<'_>,
    occurrence: ManagedIndexSourceOccurrence,
    graph: &WorthQueryInstalledGraphReadContract,
) -> Result<Entry, MaterialDenial> {
    let mut projector = Projector {
        execution,
        rows,
        graph,
        allocation: SourceFactsAllocation::open(execution)?,
        dependencies: BTreeSet::new(),
    };
    let fields = projector.fields(
        FieldSource::Root(&occurrence),
        occurrence.entity(),
        "root",
        program.fields(),
    )?;
    projector
        .allocation
        .claim_slice::<Relation>(execution, program.projection_routes().len())?;
    let mut relations = Vec::new();
    relations
        .try_reserve_exact(program.projection_routes().len())
        .map_err(|_| Storage::AllocationFailed)?;
    for (index, route) in program.projection_routes().iter().enumerate() {
        let relation = graph
            .relations()
            .iter()
            .find(|relation| {
                relation.slot_type() == route.name() && relation.parent_path() == "root"
            })
            .ok_or(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::InvalidExpression,
            ))?;
        let read = rows.prepare_projection(&occurrence, index)?;
        let support = rows.projection_support(&read)?;
        for &(entity, _) in support.entity_dependencies() {
            projector.dependency(ViewDependency::Entity(entity))?;
        }
        for adjacency in support.adjacency_dependencies() {
            projector.dependency(ViewDependency::Adjacency(
                adjacency.anchor,
                adjacency.relation,
                u8::from(!adjacency.outgoing),
            ))?;
        }
        let occurrences = rows.projection_occurrences(&read)?;
        validate_relation_cardinality(relation, occurrences.len()).map_err(|cause| {
            let cause = crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjectionDenial::new(
                crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjectionDenialKind::RelationCardinalityMismatch,
                cause.into_subject(),
            );
            MaterialDenial::Native(super::service::retain_failure(execution, cause))
        })?;
        projector
            .allocation
            .claim_slice::<Row>(execution, occurrences.len())?;
        let mut children = Vec::new();
        children
            .try_reserve_exact(occurrences.len())
            .map_err(|_| Storage::AllocationFailed)?;
        for child in occurrences {
            let fields = projector.fields(
                FieldSource::Endpoint(&read, &child),
                child.entity(),
                relation.result_path(),
                route.fields(),
            )?;
            let mut row = Row::new(
                child.entity(),
                Some(relation.result_path_identity()),
                true,
                fields,
                Vec::new(),
            );
            assert!(
                row.attach_incoming_relation(child.relation().relation_id),
                "a new projected child has one genuine incoming occurrence"
            );
            children.push(row);
        }
        relations.push(Relation::new(relation, Vec::new(), children));
    }
    projector.allocation.claim_slice::<Row>(execution, 1)?;
    let row = Row::new(occurrence.entity(), None, true, fields, relations);
    Ok(Entry {
        occurrence,
        row: WorkingTree::new(vec![row])
            .into_disclosed(&WorthQueryApplicationQueryGovernance::Public),
        dependencies: projector.dependencies,
        _allocation: projector.allocation,
    })
}

impl Projector<'_, '_, '_> {
    fn fields(
        &mut self,
        source: FieldSource<'_>,
        entity: EntityId,
        path: &str,
        columns: &[ManagedIndexSourceField],
    ) -> Result<Vec<Field>, MaterialDenial> {
        let count = self
            .graph
            .projections()
            .iter()
            .filter(|projection| projection.parent_path() == path)
            .count();
        self.allocation
            .claim_slice::<Field>(self.execution, count)?;
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(count)
            .map_err(|_| Storage::AllocationFailed)?;
        self.dependency(ViewDependency::Entity(entity))?;
        let graph = self.graph;
        for projection in graph
            .projections()
            .iter()
            .filter(|projection| projection.parent_path() == path)
        {
            let (column, field) = columns
                .iter()
                .enumerate()
                .find(|(_, field)| {
                    field.locator.aspect().aspect_key() == projection.aspect_key()
                        && field.locator.field_path().fields()
                            == std::slice::from_ref(projection.field_key())
                })
                .ok_or(DerivedRelationReadDenial::Definition(
                    DerivedRelationDenial::InvalidExpression,
                ))?;
            let retain = |value: Option<&worth_foundational::facade::AspectValue>| -> Result<_, MaterialDenial> {
                match value {
                    Some(value) => {
                        self.allocation.claim(self.execution, value.owned_allocation_capacity_bytes() as u64)?;
                        Ok(value.clone())
                    }
                    None if projection.presence() == worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Optional => Ok(worth_foundational::facade::AspectValue::Null),
                    None => Err(DerivedRelationReadDenial::Definition(DerivedRelationDenial::RecordUnavailable).into()),
                }
            };
            let value = match source {
                FieldSource::Root(occurrence) => {
                    self.rows.with_field(occurrence, column, retain)??
                }
                FieldSource::Endpoint(read, occurrence) => self
                    .rows
                    .with_projected_field(read, occurrence, column, retain)??,
            };
            self.dependency(ViewDependency::Field(entity, field.locator.clone()))?;
            fields.push(Field::new(projection, value));
        }
        Ok(fields)
    }

    fn dependency(&mut self, dependency: ViewDependency) -> Result<(), MaterialDenial> {
        if !self.dependencies.contains(&dependency) {
            self.allocation
                .claim_tree::<ViewDependency>(self.execution, self.dependencies.len())?;
            self.allocation
                .claim(self.execution, dependency.retained_bytes() as u64)?;
            self.dependencies.insert(dependency);
        }
        Ok(())
    }
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn project_row(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    rows: &ManagedIndexSourceRows<'_>,
    occurrence: ManagedIndexSourceOccurrence,
    graph: &WorthQueryInstalledGraphReadContract,
) -> Result<(Tree, Arc<SourceFactsAllocation>), MaterialDenial> {
    let entry = project(execution, program, rows, occurrence, graph)?;
    Ok((entry.row, entry._allocation))
}
