//! A sparse preserved output beside live-owner members: one image, two exact captures.
use super::anchored::AnchoredFields;
use super::output_fields::capture_output_fields;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

/// Installation partitions the demand by entity: the output's own loci are
/// captured sparsely at the exact output record, and every other locus belongs
/// to a member family and is captured completely on each member the mutation
/// moves. Both share one byte total, and member fields are charged the same
/// per-field metadata as output fields, so cold readmission recomputes it.
///
/// Work is the sum of the two captures over disjoint loci:
/// O(L_out * (F + M)) + O(F + A * L_member * C) for output loci L_out, facts F,
/// footprint membership M, anchors A, member loci L_member and candidates C.
pub(in super::super) fn retain_output_fields_with_members<'a>(
    demand: &InstalledPreImageDemand,
    facts: impl Iterator<Item = &'a WorthQueryPrimaryGraphApplicationDecisionFact> + Clone,
    mutation: &ValidatedMutationFootprint,
    target: EntityId,
    layout: &WorthQueryPrimaryGraphLayout,
    members: &super::super::authored_anchors::Selection<'_>,
) -> Result<WorthQueryRetainedPreImageSelection, WorthQueryPreImageRetentionDenial> {
    let output = demand
        .output_fields()
        .ok_or(WorthQueryPreImageRetentionDenial::MissingDemandedField)?;
    let mut captured = capture_output_fields(demand, facts.clone(), mutation, target, layout)?;
    let anchored = AnchoredFields {
        demand,
        mutation,
        anchors: &members.anchors,
        own_changes: members.own_changes,
        admits: &|locus| locus.entity() != output.entity(),
        seal: seal_member_field,
    };
    let (fields, candidates) = anchored.retain(facts, &mut captured.total)?;
    captured.seal(demand, fields, candidates)
}

/// A member field carries its record and locus beside its value.
fn seal_member_field(
    demand: &InstalledPreImageDemand,
    total: &mut usize,
    locus: &InstalledPreImageLocus,
    candidate: &WorthQueryObservedPreImageCandidate,
) -> Result<WorthQueryRetainedPreImageFieldSeal, WorthQueryPreImageRetentionDenial> {
    use WorthQueryPreImageRetentionDenial as Denial;
    let metadata = 21usize
        .checked_add(locus.entity().len())
        .and_then(|n| n.checked_add(locus.aspect().len()))
        .and_then(|n| n.checked_add(locus.field().len()))
        .ok_or(Denial::ExceedsByteBound)?;
    *total = total
        .checked_add(metadata)
        .ok_or(Denial::ExceedsByteBound)?;
    seal_field(demand, total, locus, candidate)
}
