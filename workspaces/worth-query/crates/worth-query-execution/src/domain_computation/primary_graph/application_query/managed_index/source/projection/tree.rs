//! Original-funded Native forest assembly guards partial and completed deep rows.
use super::*;
use crate::domain_computation::primary_graph::application_query::projection::{
    OwnedProjectionRows, ProjectionDisposal,
};
use worth_query_installation::facade::WorthQueryInstalledGraphRelation;

struct Origin {
    read: Arc<PreparedManagedIndexProjectionRead>,
    occurrence: ManagedIndexSourceProjectionOccurrence,
}
struct NodeFrame {
    row: Row,
    route: Option<usize>,
    origin: Option<Origin>,
    next: usize,
    payload: usize,
}
struct RelationFrame<'a> {
    relation: &'a WorthQueryInstalledGraphRelation,
    index: usize,
    read: Arc<PreparedManagedIndexProjectionRead>,
    occurrences: std::vec::IntoIter<ManagedIndexSourceProjectionOccurrence>,
    rows: Vec<Row>,
    payload: usize,
}
enum Frame<'a> {
    Node(NodeFrame),
    Relation(RelationFrame<'a>),
}
struct Assembly<'a> {
    frames: Vec<Frame<'a>>,
    roots: Vec<Row>,
    disposal: Option<ProjectionDisposal>,
    payload: usize,
    allocation: Arc<SourceFactsAllocation>,
}
impl<'a> Assembly<'a> {
    fn admitted(
        execution: &ExecutionRequest<'_, '_>,
        allocation: Arc<SourceFactsAllocation>,
        depth: usize,
    ) -> Result<Self, MaterialDenial> {
        let capacity = depth.checked_mul(2).ok_or(Storage::ChargedBytesOverflow)?;
        allocation.claim_slice::<Frame<'a>>(execution, capacity)?;
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(capacity)
            .map_err(|_| Storage::AllocationFailed)?;
        let disposal = ProjectionDisposal::admitted(execution, &allocation, depth)?;
        allocation.claim_slice::<Row>(execution, 1)?;
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(1)
            .map_err(|_| Storage::AllocationFailed)?;
        let payload = roots
            .capacity()
            .checked_mul(size_of::<Row>())
            .ok_or(Storage::ChargedBytesOverflow)?;
        Ok(Self {
            frames,
            roots,
            disposal: Some(disposal),
            payload,
            allocation,
        })
    }
    fn push(&mut self, frame: Frame<'a>) {
        assert!(
            self.frames.len() < self.frames.capacity(),
            "compiled forest bounds assembly continuations"
        );
        self.frames.push(frame);
    }
    fn complete_node(&mut self) -> Result<(), MaterialDenial> {
        let Some(Frame::Node(node)) = self.frames.last() else {
            unreachable!()
        };
        let payload = node.payload;
        let count = self.frames.len();
        if count == 1 {
            self.payload = self
                .payload
                .checked_add(payload)
                .ok_or(Storage::ChargedBytesOverflow)?;
        } else {
            let Frame::Relation(parent) = &mut self.frames[count - 2] else {
                unreachable!()
            };
            parent.payload = parent
                .payload
                .checked_add(payload)
                .ok_or(Storage::ChargedBytesOverflow)?;
        }
        let Frame::Node(node) = self.frames.pop().expect("completed node") else {
            unreachable!()
        };
        if count == 1 {
            self.roots.push(node.row);
        } else {
            let Some(Frame::Relation(parent)) = self.frames.last_mut() else {
                unreachable!()
            };
            parent.rows.push(node.row);
        }
        Ok(())
    }
    fn complete_relation(&mut self) -> Result<(), MaterialDenial> {
        let Some(Frame::Relation(relation)) = self.frames.last() else {
            unreachable!()
        };
        let payload = relation.payload;
        let slot = relation.relation.slot_type();
        let count = self.frames.len();
        let Frame::Node(parent) = &mut self.frames[count - 2] else {
            unreachable!()
        };
        if parent.row.relation(slot).is_some() {
            return Err(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::InvalidExpression,
            )
            .into());
        }
        parent.payload = parent
            .payload
            .checked_add(payload)
            .ok_or(Storage::ChargedBytesOverflow)?;
        let Frame::Relation(relation) = self.frames.pop().expect("completed relation") else {
            unreachable!()
        };
        let Some(Frame::Node(parent)) = self.frames.last_mut() else {
            unreachable!()
        };
        assert!(
            parent
                .row
                .insert_relation(Relation::new(relation.relation, Vec::new(), relation.rows)),
            "unique compiled relation slot"
        );
        Ok(())
    }
    fn seal(&mut self) -> Tree {
        let rows = OwnedProjectionRows::managed(
            std::mem::take(&mut self.roots),
            self.disposal.take().expect("admitted disposal"),
            self.payload,
            Arc::clone(&self.allocation),
        );
        Tree::from_managed(rows)
    }
}
impl Drop for Assembly<'_> {
    fn drop(&mut self) {
        if let Some(disposal) = &mut self.disposal {
            for frame in &mut self.frames {
                match frame {
                    Frame::Node(node) => disposal.clear_node(&mut node.row),
                    Frame::Relation(relation) => disposal.clear_rows(&mut relation.rows),
                }
            }
            disposal.clear_rows(&mut self.roots);
        }
        // Partial row backing and continuation backing precede the original payer.
    }
}
pub(super) fn project(
    projector: &mut Projector<'_, '_, '_>,
    program: &ManagedIndexSourceProgram,
    root: &ManagedIndexSourceOccurrence,
    fields: Vec<Field>,
    depth: usize,
) -> Result<Tree, MaterialDenial> {
    let mut assembly = Assembly::admitted(
        projector.execution,
        Arc::clone(&projector.allocation),
        depth,
    )?;
    let row = node(projector, program, None, root.entity(), None, fields)?;
    let payload = row.owned_shallow_bytes();
    assembly.push(Frame::Node(NodeFrame {
        row,
        route: None,
        origin: None,
        next: 0,
        payload,
    }));
    while let Some(frame) = assembly.frames.last_mut() {
        SourceFactsAllocation::work(projector.execution, 1)?;
        match frame {
            Frame::Node(node) => {
                let next = next_route(projector, program, node.route, node.next)?;
                if let Some(index) = next {
                    node.next = index + 1;
                    let parent = node
                        .origin
                        .as_ref()
                        .map(|origin| (origin.read.as_ref(), &origin.occurrence));
                    let acquired =
                        projector.acquire(program, root, index, node.row.result_path(), parent)?;
                    projector
                        .allocation
                        .claim_arc::<PreparedManagedIndexProjectionRead>(projector.execution)?;
                    projector
                        .allocation
                        .claim_slice::<Row>(projector.execution, acquired.occurrences.len())?;
                    let mut rows = Vec::new();
                    rows.try_reserve_exact(acquired.occurrences.len())
                        .map_err(|_| Storage::AllocationFailed)?;
                    let payload = rows
                        .capacity()
                        .checked_mul(size_of::<Row>())
                        .ok_or(Storage::ChargedBytesOverflow)?;
                    assembly.push(Frame::Relation(RelationFrame {
                        relation: acquired.relation,
                        index,
                        read: Arc::new(acquired.read),
                        occurrences: acquired.occurrences,
                        rows,
                        payload,
                    }));
                } else {
                    assembly.complete_node()?;
                }
            }
            Frame::Relation(relation) => {
                if let Some(occurrence) = relation.occurrences.next() {
                    let fields = projector.fields(
                        FieldSource::Endpoint(&relation.read, &occurrence),
                        occurrence.entity(),
                        relation.relation.result_path(),
                        program.projection_routes()[relation.index].fields(),
                    )?;
                    let mut row = node(
                        projector,
                        program,
                        Some(relation.index),
                        occurrence.entity(),
                        Some(relation.relation.result_path_identity()),
                        fields,
                    )?;
                    assert!(
                        row.attach_incoming_relation(occurrence.relation().relation_id),
                        "one actual incoming occurrence"
                    );
                    let payload = row.owned_shallow_bytes();
                    let child = NodeFrame {
                        row,
                        route: Some(relation.index),
                        origin: Some(Origin {
                            read: Arc::clone(&relation.read),
                            occurrence,
                        }),
                        next: 0,
                        payload,
                    };
                    assembly.push(Frame::Node(child));
                } else {
                    assembly.complete_relation()?;
                }
            }
        }
    }
    Ok(assembly.seal())
}
fn node(
    projector: &Projector<'_, '_, '_>,
    program: &ManagedIndexSourceProgram,
    parent: Option<usize>,
    entity: EntityId,
    path: Option<Arc<str>>,
    fields: Vec<Field>,
) -> Result<Row, MaterialDenial> {
    let mut count = 0;
    for route in program.projection_routes() {
        SourceFactsAllocation::work(projector.execution, 1)?;
        if route.parent_route() == parent {
            count += 1;
        }
    }
    projector
        .allocation
        .claim_slice::<Relation>(projector.execution, count)?;
    let mut relations = Vec::new();
    relations
        .try_reserve_exact(count)
        .map_err(|_| Storage::AllocationFailed)?;
    Ok(Row::new(entity, path, true, fields, relations))
}

fn next_route(
    projector: &Projector<'_, '_, '_>,
    program: &ManagedIndexSourceProgram,
    parent: Option<usize>,
    start: usize,
) -> Result<Option<usize>, MaterialDenial> {
    for (index, route) in program.projection_routes().iter().enumerate().skip(start) {
        SourceFactsAllocation::work(projector.execution, 1)?;
        if route.parent_route() == parent {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
