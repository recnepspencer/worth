use std::{convert::Infallible, mem::size_of};

use worth_foundational::facade::{CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;

use super::{
    WorthQueryCanonicalGraphReadPlanningInput, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadPlanningRelation,
    WorthQueryGraphReadTraversalOperator,
};

mod row_construction;
mod spec_visit;

use row_construction::build_row;

#[derive(Clone, Copy)]
enum RelationRequirementBasis {
    DirectionalAdjacency,
    ReverseAdjacency,
    AggregateSelection,
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
    ComponentSelection,
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
    let mut stop = None;
    let mut step = |action: spec_visit::SpecStep<'a>| -> bool {
        let visited = match action {
            spec_visit::SpecStep::Visit => sink.visit(),
            spec_visit::SpecStep::Row(spec) => sink.row(spec),
        };
        match visited {
            Ok(()) => true,
            Err(error) => {
                stop = Some(error);
                false
            }
        }
    };
    if spec_visit::drive(input, &mut step) {
        Ok(())
    } else {
        Err(stop.expect("a refused planning-spec visit stores its stop"))
    }
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
