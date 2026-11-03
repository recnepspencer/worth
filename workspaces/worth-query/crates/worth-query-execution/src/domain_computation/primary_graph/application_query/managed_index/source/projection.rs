//! Materialize the compiled Source shape through the current Native loan.
mod acquisition;
mod tree;
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
        PreparedManagedIndexProjectionRead, PreparedManagedIndexRelationRead,
        PreparedManagedIndexSourceOccurrence, PreparedManagedIndexSourceRowRead,
    },
    schema::DerivedRelationDenial,
    transactions::RelationalExecutionDenialCause as Storage,
};

pub(in crate::domain_computation::primary_graph::application_query::managed_index) struct Entry {
    pub(super) occurrence: ManagedIndexSourceOccurrence,
    pub(super) row: Tree,
    pub(super) dependencies: BTreeSet<ViewDependency>,
    pub(super) content: std::sync::OnceLock<[u8; 32]>,
    pub(super) comparison_depth: usize,
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
enum FieldSource<'a, 'read, 'source> {
    Root(&'a PreparedManagedIndexSourceRowRead<'read, 'source>),
    Endpoint(
        &'a PreparedManagedIndexProjectionRead,
        &'a ManagedIndexSourceProjectionOccurrence,
    ),
}

pub(super) enum ProjectionMembership<'a> {
    Forward(&'a PreparedManagedIndexRelationRead),
    Inverse(&'a PreparedManagedIndexSourceOccurrence),
}

pub(super) fn project(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    rows: &ManagedIndexSourceRows<'_>,
    occurrence: ManagedIndexSourceOccurrence,
    graph: &WorthQueryInstalledGraphReadContract,
    support: Option<ProjectionMembership<'_>>,
) -> Result<Entry, MaterialDenial> {
    let row_read = match support {
        Some(ProjectionMembership::Forward(read)) => {
            rows.prepare_row_from_read(read, &occurrence)?
        }
        Some(ProjectionMembership::Inverse(read)) => {
            rows.prepare_row_from_occurrence(read, &occurrence)?
        }
        None => rows.prepare_row(&occurrence)?,
    };
    let mut projector = Projector {
        execution,
        rows,
        graph,
        allocation: SourceFactsAllocation::open(execution)?,
        dependencies: BTreeSet::new(),
    };
    let fields = projector.fields(
        FieldSource::Root(&row_read),
        occurrence.entity(),
        "root",
        program.fields(),
    )?;
    let comparison_depth = if program
        .projection_routes()
        .iter()
        .any(|route| route.parent_route().is_some())
    {
        program
            .projection_routes()
            .len()
            .checked_add(1)
            .ok_or(Storage::ChargedBytesOverflow)?
    } else {
        2
    };
    let row = if comparison_depth > 2 {
        tree::project(
            &mut projector,
            program,
            &occurrence,
            fields,
            comparison_depth,
        )?
    } else {
        projector
            .allocation
            .claim_slice::<Relation>(execution, program.projection_routes().len())?;
        let mut relations = Vec::new();
        relations
            .try_reserve_exact(program.projection_routes().len())
            .map_err(|_| Storage::AllocationFailed)?;
        for (index, route) in program.projection_routes().iter().enumerate() {
            let acquired = projector.acquire(program, &occurrence, index, "root", None)?;
            let relation = acquired.relation;
            let read = acquired.read;
            let occurrences = acquired.occurrences;
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
        WorkingTree::new(vec![row]).into_disclosed(&WorthQueryApplicationQueryGovernance::Public)
    };
    Ok(Entry {
        occurrence,
        row,
        dependencies: projector.dependencies,
        content: std::sync::OnceLock::new(),
        comparison_depth,
        _allocation: projector.allocation,
    })
}

impl Projector<'_, '_, '_> {
    fn fields(
        &mut self,
        source: FieldSource<'_, '_, '_>,
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
            let retain = |value: Option<&worth_foundational::facade::AspectValue>,
                          content|
             -> Result<_, MaterialDenial> {
                match value {
                    Some(value) => {
                        self.allocation.claim(self.execution, value.owned_allocation_capacity_bytes() as u64)?;
                        Ok((value.clone(), content))
                    }
                    None if projection.presence().schema_presence() == worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Optional => Ok((worth_foundational::facade::AspectValue::Null, content)),
                    None => Err(DerivedRelationReadDenial::Definition(DerivedRelationDenial::RecordUnavailable).into()),
                }
            };
            let (value, content) = match source {
                FieldSource::Root(read) => read.with_field_content(column, retain)??,
                FieldSource::Endpoint(read, occurrence) => self
                    .rows
                    .with_projected_field_content(read, occurrence, column, retain)??,
            };
            self.dependency(ViewDependency::Field(entity, field.locator.clone()))?;
            fields.push(Field::source(projection, value, content).map_err(
                worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::Content,
            )?);
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
    let entry = project(execution, program, rows, occurrence, graph, None)?;
    Ok((entry.row, entry._allocation))
}
