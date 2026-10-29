use worth_foundational::facade::{
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
};
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_installation::facade::{
    WorthQueryAdmittedReadGraphPlanningInventory, WorthQueryCanonicalWorkEvidence,
    WorthQueryInstalledGraphReadContract, WorthQueryPlanningInventoryStop,
    WorthQueryReadGraphPlanningContract, WorthQueryReadGraphRelationDirection,
    WorthQueryReadGraphRelationView,
};

use crate::application_query::WorthQueryApplicationQueryLane;
use crate::canonical_identity_derivation::{
    WorthQueryCanonicalIdentityBasis, WorthQueryCanonicalIdentityStop,
};
use crate::graph_read_access::digest_text::{admitted_digest_text, admitted_text_clone};

const ACCESS_DOMAIN: &str = "worth-query.application-query-access-shape";
const ACCESS_VERSION: &str = "worth-query-application-query-access-shape-v3";
const SELECTIVITY_DOMAIN: &str = "worth-query.application-query-selectivity-shape";
const SELECTIVITY_VERSION: &str = "worth-query-application-query-selectivity-shape-v3";

enum AccessEntry<'a> {
    Digest(&'static str, CanonicalDigestId),
    Text(&'static str, &'a str),
    Unsigned(&'static str, usize),
    RelationText(usize, &'static str, &'a str),
    RelationUnsigned(usize, &'static str, usize),
}

fn for_each_access_entry<'a, Error>(
    schema_basis: CanonicalDigestId,
    root: &'a str,
    cardinality: ApplicationQueryCardinality,
    planning_graph_identity: CanonicalDigestId,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    relation_count: usize,
    relations: impl Iterator<Item = WorthQueryReadGraphRelationView<'a>>,
    mut visit: impl FnMut(AccessEntry<'a>) -> Result<(), Error>,
) -> Result<(), Error> {
    let mut stop = None;
    let mut step = |entry: AccessEntry<'a>| -> bool {
        match visit(entry) {
            Ok(()) => true,
            Err(error) => {
                stop = Some(error);
                false
            }
        }
    };
    let mut relations = relations;
    if drive_access_entries(
        schema_basis,
        root,
        cardinality,
        planning_graph_identity,
        lane,
        maximum_result_count,
        relation_count,
        &mut relations,
        &mut step,
    ) {
        Ok(())
    } else {
        Err(stop.expect("a refused access visit stores its error"))
    }
}

#[inline(never)]
fn drive_access_entries<'a>(
    schema_basis: CanonicalDigestId,
    root: &'a str,
    cardinality: ApplicationQueryCardinality,
    planning_graph_identity: CanonicalDigestId,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    relation_count: usize,
    relations: &mut dyn Iterator<Item = WorthQueryReadGraphRelationView<'a>>,
    visit: &mut dyn FnMut(AccessEntry<'a>) -> bool,
) -> bool {
    if !visit(AccessEntry::Digest("graph", planning_graph_identity)) {
        return false;
    }
    if !visit(AccessEntry::Digest("schema-basis", schema_basis)) {
        return false;
    }
    if !visit(AccessEntry::Text("root", root)) {
        return false;
    }
    if !visit(AccessEntry::Text(
        "cardinality",
        cardinality_name(cardinality),
    )) {
        return false;
    }
    if !visit(AccessEntry::Text("lane", lane.as_str())) {
        return false;
    }
    if !visit(AccessEntry::Unsigned(
        "maximum-result-count",
        maximum_result_count,
    )) {
        return false;
    }
    if !visit(AccessEntry::Unsigned("relation-count", relation_count)) {
        return false;
    }
    for (index, relation) in relations.enumerate() {
        if !visit(AccessEntry::RelationText(index, "name", relation.relation)) {
            return false;
        }
        if !visit(AccessEntry::RelationText(
            index,
            "direction",
            relation_direction_name(relation.direction),
        )) {
            return false;
        }
        if !visit(AccessEntry::RelationUnsigned(
            index,
            "depth",
            relation.depth,
        )) {
            return false;
        }
    }
    true
}

pub(super) fn access_shape_digest(
    graph: &impl WorthQueryReadGraphPlanningContract,
    planning_graph_identity: CanonicalDigestId,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    budget: CanonicalDigestWorkBudget,
) -> Result<(CanonicalDigestId, WorthQueryCanonicalWorkEvidence), CanonicalDigestDerivationDenial> {
    let mut basis = WorthQueryCanonicalIdentityBasis::new(ACCESS_DOMAIN, ACCESS_VERSION, budget);
    for_each_access_entry(
        *graph.schema_basis_digest(),
        graph.root_entity(),
        graph.cardinality(),
        planning_graph_identity,
        lane,
        maximum_result_count,
        graph.relation_count(),
        (0..graph.relation_count()).map(|index| {
            graph
                .relation(index)
                .expect("planning relation count must be exact")
        }),
        |entry| match entry {
            AccessEntry::Digest(locus, value) => basis.digest(locus, value),
            AccessEntry::Text(locus, value) => basis.text(locus, value),
            AccessEntry::Unsigned(locus, value) => basis.unsigned(locus, value),
            AccessEntry::RelationText(index, suffix, value) => {
                basis.text(format!("relation[{index}].{suffix}"), value)
            }
            AccessEntry::RelationUnsigned(index, suffix, value) => {
                basis.unsigned(format!("relation[{index}].{suffix}"), value)
            }
        },
    )?;
    basis.derive()
}

pub(super) fn access_shape_digest_admitted<'a, Stop>(
    graph: &'a WorthQueryInstalledGraphReadContract,
    inventory: &WorthQueryAdmittedReadGraphPlanningInventory<'a>,
    planning_graph_identity: CanonicalDigestId,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    budget: CanonicalDigestWorkBudget,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<
    (CanonicalDigestId, WorthQueryCanonicalWorkEvidence),
    WorthQueryCanonicalIdentityStop<Stop>,
> {
    let mut basis = WorthQueryCanonicalIdentityBasis::new_admitted(
        ACCESS_DOMAIN,
        ACCESS_VERSION,
        budget,
        admit,
    )?;
    let relations = inventory
        .relations_admitted(admit)
        .map_err(inventory_stop)?;
    let direction_visits = u64::try_from(inventory.relation_count())
        .map_err(|_| WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    admit(direction_visits, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    for_each_access_entry(
        *graph.schema_basis_digest(),
        graph.root_entity(),
        graph.cardinality(),
        planning_graph_identity,
        lane,
        maximum_result_count,
        inventory.relation_count(),
        relations,
        |entry| match entry {
            AccessEntry::Digest(locus, value) => basis.digest_admitted(locus, value, admit),
            AccessEntry::Text(locus, value) => basis.text_admitted(locus, value, admit),
            AccessEntry::Unsigned(locus, value) => basis.unsigned_admitted(locus, value, admit),
            AccessEntry::RelationText(index, suffix, value) => {
                let locus = relation_locus(index, suffix, &mut *admit)?;
                let value = admitted_text_clone(value, &mut *admit)?;
                basis.text_owned_admitted(locus, value, admit)
            }
            AccessEntry::RelationUnsigned(index, suffix, value) => {
                let locus = relation_locus(index, suffix, &mut *admit)?;
                basis.unsigned_owned_admitted(locus, value, admit)
            }
        },
    )?;
    basis.derive_admitted(admit)
}

fn relation_locus<Stop>(
    index: usize,
    suffix: &str,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<String, WorthQueryCanonicalIdentityStop<Stop>> {
    let decimal_digits = if index == 0 {
        1usize
    } else {
        usize::try_from(index.ilog10())
            .map_err(|_| WorthQueryCanonicalIdentityStop::AccountingOverflow)?
            .checked_add(1)
            .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?
    };
    let count_work = "relation["
        .len()
        .checked_add(decimal_digits)
        .and_then(|work| work.checked_add("].".len()))
        .and_then(|work| work.checked_add(suffix.len()))
        .and_then(|work| work.checked_add(2))
        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    let count_work = u64::try_from(count_work)
        .map_err(|_| WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    admitted_digest_text(
        count_work,
        |output| write!(output, "relation[{index}].{suffix}"),
        admit,
    )
    .map_err(Into::into)
}

pub(super) fn selectivity_shape_digest(
    planning_graph_identity: CanonicalDigestId,
    access_shape_identity: CanonicalDigestId,
    binding_identity: CanonicalDigestId,
    budget: CanonicalDigestWorkBudget,
) -> Result<(CanonicalDigestId, WorthQueryCanonicalWorkEvidence), CanonicalDigestDerivationDenial> {
    let mut basis =
        WorthQueryCanonicalIdentityBasis::new(SELECTIVITY_DOMAIN, SELECTIVITY_VERSION, budget);
    for (locus, value) in selectivity_entries(
        planning_graph_identity,
        access_shape_identity,
        binding_identity,
    ) {
        basis.digest(locus, value)?;
    }
    basis.derive()
}

pub(super) fn selectivity_shape_digest_admitted<Stop>(
    planning_graph_identity: CanonicalDigestId,
    access_shape_identity: CanonicalDigestId,
    binding_identity: CanonicalDigestId,
    budget: CanonicalDigestWorkBudget,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<
    (CanonicalDigestId, WorthQueryCanonicalWorkEvidence),
    WorthQueryCanonicalIdentityStop<Stop>,
> {
    let mut basis = WorthQueryCanonicalIdentityBasis::new_admitted(
        SELECTIVITY_DOMAIN,
        SELECTIVITY_VERSION,
        budget,
        admit,
    )?;
    for (locus, value) in selectivity_entries(
        planning_graph_identity,
        access_shape_identity,
        binding_identity,
    ) {
        basis.digest_admitted(locus, value, admit)?;
    }
    basis.derive_admitted(admit)
}

fn selectivity_entries(
    planning_graph_identity: CanonicalDigestId,
    access_shape_identity: CanonicalDigestId,
    binding_identity: CanonicalDigestId,
) -> [(&'static str, CanonicalDigestId); 3] {
    [
        ("graph", planning_graph_identity),
        ("access-shape", access_shape_identity),
        ("bindings", binding_identity),
    ]
}

fn inventory_stop<Stop>(
    stop: WorthQueryPlanningInventoryStop<Stop>,
) -> WorthQueryCanonicalIdentityStop<Stop> {
    match stop {
        WorthQueryPlanningInventoryStop::Admission(stop) => {
            WorthQueryCanonicalIdentityStop::Admission(stop)
        }
        WorthQueryPlanningInventoryStop::AccountingOverflow => {
            WorthQueryCanonicalIdentityStop::AccountingOverflow
        }
    }
}

const fn cardinality_name(cardinality: ApplicationQueryCardinality) -> &'static str {
    match cardinality {
        ApplicationQueryCardinality::OptionalOne => "optional-one",
        ApplicationQueryCardinality::ExactlyOne => "exactly-one",
        ApplicationQueryCardinality::Many => "many",
    }
}

const fn relation_direction_name(direction: WorthQueryReadGraphRelationDirection) -> &'static str {
    match direction {
        WorthQueryReadGraphRelationDirection::Forward => "forward",
        WorthQueryReadGraphRelationDirection::Reverse => "reverse",
    }
}
