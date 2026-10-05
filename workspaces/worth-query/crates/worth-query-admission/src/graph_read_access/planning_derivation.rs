use std::{convert::Infallible, mem::size_of};

use worth_foundational::facade::{CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;

use super::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryCanonicalGraphReadPlanningInput,
    WorthQueryGraphReadAccessRequirementRow, WorthQueryGraphReadAccessRequirementSet,
    WorthQueryGraphReadOrderingPosture, WorthQueryGraphReadPlanningRelation,
    WorthQueryGraphReadPredicateFamily, WorthQueryGraphReadTraversalOperator,
};

mod row_construction;

use row_construction::build_row;

#[derive(Clone, Copy)]
enum RelationRequirementBasis {
    DirectionalAdjacency,
    ReverseAdjacency,
    TraversalWorkset,
}

#[derive(Clone)]
enum RowSpec<'a> {
    Relation {
        relation: &'a WorthQueryGraphReadPlanningRelation,
        operator: &'a WorthQueryGraphReadTraversalOperator,
        kind: super::WorthQueryGraphReadAccessRequirementKind,
        basis: RelationRequirementBasis,
    },
    ResultBuffer,
    Predicate,
    Ordering,
    RootUnionDedup,
    Proof,
    Lifecycle,
    LiveMaintenance,
}

trait RowSink<'a> {
    type Stop;

    fn visit(&mut self) -> Result<(), Self::Stop>;
    fn row(&mut self, spec: RowSpec<'a>) -> Result<(), Self::Stop>;
}

fn visit_specs<'a, S: RowSink<'a>>(
    input: &'a WorthQueryCanonicalGraphReadPlanningInput,
    sink: &mut S,
) -> Result<(), S::Stop> {
    sink.visit()?;
    for relation in input.shape().relations() {
        sink.visit()?;
        for operator in relation.operators() {
            sink.visit()?;
            let forward = RowSpec::Relation {
                relation,
                operator,
                kind: super::WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
                basis: RelationRequirementBasis::DirectionalAdjacency,
            };
            let reverse = RowSpec::Relation {
                relation,
                operator,
                kind: super::WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency,
                basis: RelationRequirementBasis::ReverseAdjacency,
            };
            match operator {
                WorthQueryGraphReadTraversalOperator::BoundedAncestor
                | WorthQueryGraphReadTraversalOperator::SharedEndpoint
                | WorthQueryGraphReadTraversalOperator::SharedAttachment => sink.row(reverse)?,
                WorthQueryGraphReadTraversalOperator::FrontierSearch => {
                    sink.row(forward)?;
                    sink.row(reverse)?;
                }
                WorthQueryGraphReadTraversalOperator::DeclarationTraversal
                | WorthQueryGraphReadTraversalOperator::DirectEdge => match relation.direction() {
                    WorthQueryAdmittedGraphReadRelationDirection::Ancestor => sink.row(reverse)?,
                    WorthQueryAdmittedGraphReadRelationDirection::Forward
                    | WorthQueryAdmittedGraphReadRelationDirection::Descendant => {
                        sink.row(forward)?
                    }
                },
                WorthQueryGraphReadTraversalOperator::SuccessorWalk
                | WorthQueryGraphReadTraversalOperator::BoundedDescendant
                | WorthQueryGraphReadTraversalOperator::AnchoredFrontier => sink.row(forward)?,
            }
            if !matches!(operator, WorthQueryGraphReadTraversalOperator::DirectEdge) {
                for kind in [
                    super::WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
                    super::WorthQueryGraphReadAccessRequirementKind::VisitedSet,
                ] {
                    sink.row(RowSpec::Relation {
                        relation,
                        operator,
                        kind,
                        basis: RelationRequirementBasis::TraversalWorkset,
                    })?;
                }
            }
            if matches!(
                operator,
                WorthQueryGraphReadTraversalOperator::AnchoredFrontier
                    | WorthQueryGraphReadTraversalOperator::SharedEndpoint
                    | WorthQueryGraphReadTraversalOperator::SharedAttachment
                    | WorthQueryGraphReadTraversalOperator::FrontierSearch
            ) {
                sink.row(RowSpec::Relation {
                    relation,
                    operator,
                    kind: super::WorthQueryGraphReadAccessRequirementKind::DedupSet,
                    basis: RelationRequirementBasis::TraversalWorkset,
                })?;
            }
        }
    }
    sink.row(RowSpec::ResultBuffer)?;
    if input.shape().predicate_family() != &WorthQueryGraphReadPredicateFamily::None
        || !input.shape().predicate_fields().is_empty()
    {
        sink.row(RowSpec::Predicate)?;
    }
    if input.shape().ordering_posture() != &WorthQueryGraphReadOrderingPosture::Unordered {
        sink.row(RowSpec::Ordering)?;
    }
    if input.shape().root_union_dedup_required() {
        sink.row(RowSpec::RootUnionDedup)?;
    }
    if input.shape().relationship_proof_required() {
        sink.row(RowSpec::Proof)?;
    }
    sink.row(RowSpec::Lifecycle)?;
    if input.live_maintenance_required() {
        sink.row(RowSpec::LiveMaintenance)?;
    }
    Ok(())
}

struct CountRows<'a, Stop> {
    count: usize,
    admit: &'a mut dyn FnMut(u64, u64) -> Result<(), Stop>,
}

impl<'input, 'meter, Stop> RowSink<'input> for CountRows<'meter, Stop> {
    type Stop = WorthQueryCanonicalIdentityStop<Stop>;

    fn visit(&mut self) -> Result<(), Self::Stop> {
        (self.admit)(1, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)
    }

    fn row(&mut self, _spec: RowSpec<'input>) -> Result<(), Self::Stop> {
        self.visit()?;
        self.count = self
            .count
            .checked_add(1)
            .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
        Ok(())
    }
}

struct BuildRows<'a, Stop> {
    input: &'a WorthQueryCanonicalGraphReadPlanningInput,
    rows: Vec<WorthQueryGraphReadAccessRequirementRow>,
    admit: &'a mut dyn FnMut(u64, u64) -> Result<(), Stop>,
}

impl<'a, Stop> RowSink<'a> for BuildRows<'a, Stop> {
    type Stop = WorthQueryCanonicalIdentityStop<Stop>;

    fn visit(&mut self) -> Result<(), Self::Stop> {
        (self.admit)(1, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)
    }

    fn row(&mut self, spec: RowSpec<'a>) -> Result<(), Self::Stop> {
        self.visit()?;
        self.rows.push(build_row(self.input, spec, self.admit)?);
        Ok(())
    }
}

fn collect_rows<Stop>(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<Vec<WorthQueryGraphReadAccessRequirementRow>, WorthQueryCanonicalIdentityStop<Stop>> {
    let mut count = CountRows { count: 0, admit };
    visit_specs(input, &mut count)?;
    let count = count.count;
    let bytes = count
        .checked_mul(size_of::<WorthQueryGraphReadAccessRequirementRow>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    admit(bytes, bytes).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| WorthQueryCanonicalIdentityStop::AllocationUnavailable)?;
    let mut build = BuildRows { input, rows, admit };
    visit_specs(input, &mut build)?;
    Ok(build.rows)
}

pub fn derive_canonical_graph_read_access_requirements(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    budget: CanonicalDigestWorkBudget,
    prior_work: WorthQueryCanonicalWorkEvidence,
) -> Result<WorthQueryGraphReadAccessRequirementSet, CanonicalDigestDerivationDenial> {
    let mut ordinary = |_, _| Ok::<(), Infallible>(());
    let rows = collect_rows(input, &mut ordinary).expect("infallible ordinary row construction");
    let identity = input.identity();
    WorthQueryGraphReadAccessRequirementSet::new(
        *identity.read_graph_digest(),
        *identity.access_shape_digest(),
        *identity.selectivity_shape_digest(),
        rows,
        budget,
        prior_work,
    )
}

pub(crate) fn derive_canonical_graph_read_access_requirements_admitted<Stop>(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    budget: CanonicalDigestWorkBudget,
    prior_work: WorthQueryCanonicalWorkEvidence,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQueryGraphReadAccessRequirementSet, WorthQueryCanonicalIdentityStop<Stop>> {
    let rows = collect_rows(input, admit)?;
    let identity = input.identity();
    WorthQueryGraphReadAccessRequirementSet::new_admitted(
        *identity.read_graph_digest(),
        *identity.access_shape_digest(),
        *identity.selectivity_shape_digest(),
        rows,
        budget,
        prior_work,
        admit,
    )
}
