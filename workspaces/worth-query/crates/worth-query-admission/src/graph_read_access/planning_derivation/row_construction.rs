use std::mem::size_of;

use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;

use super::{RelationRequirementBasis, RowSpec};
use crate::graph_read_access::{
    digest_order::sort_digest_keys_admitted, digest_text::admitted_digest_text,
    AdmittedDigestTextStop, WorthQueryCanonicalGraphReadPlanningInput,
    WorthQueryGraphReadAccessComplexityContract, WorthQueryGraphReadAccessInvalidationBasis,
    WorthQueryGraphReadAccessMemoryEstimateBasis, WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadOrderingFieldAuthority, WorthQueryGraphReadPredicateFieldAuthority,
    WorthQueryGraphReadRelationAuthority,
};

type Stop<Admission> = WorthQueryCanonicalIdentityStop<Admission>;

fn amount<Admission>(value: usize) -> Result<u64, Stop<Admission>> {
    u64::try_from(value).map_err(|_| Stop::AccountingOverflow)
}

fn claim_text<Admission>(
    text: &str,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<(), Stop<Admission>> {
    let bytes = amount(text.len())?;
    admit(bytes, bytes).map_err(Stop::Admission)
}

fn claim_backing<T, Admission>(
    count: usize,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<(), Stop<Admission>> {
    let bytes = count
        .checked_mul(size_of::<T>())
        .ok_or(Stop::AccountingOverflow)
        .and_then(amount)?;
    admit(bytes, bytes).map_err(Stop::Admission)
}

fn map_text_stop<Admission>(stop: AdmittedDigestTextStop<Admission>) -> Stop<Admission> {
    match stop {
        AdmittedDigestTextStop::Admission(stop) => Stop::Admission(stop),
        AdmittedDigestTextStop::AccountingOverflow => Stop::AccountingOverflow,
    }
}

fn bound_row(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    row: WorthQueryGraphReadAccessRequirementRow,
) -> WorthQueryGraphReadAccessRequirementRow {
    match input.maximum_cardinality() {
        Some(maximum) => row.with_maximum_cardinality(maximum),
        None => row,
    }
}

fn relation_dimensions(
    basis: RelationRequirementBasis,
) -> (
    WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessInvalidationBasis,
    WorthQueryGraphReadAccessComplexityContract,
    WorthQueryGraphReadAccessMemoryEstimateBasis,
) {
    use WorthQueryGraphReadAccessComplexityContract as Complexity;
    use WorthQueryGraphReadAccessInvalidationBasis as Invalidation;
    use WorthQueryGraphReadAccessMemoryEstimateBasis as Memory;
    use WorthQueryGraphReadAccessRebuildBasis as Rebuild;
    match basis {
        RelationRequirementBasis::DirectionalAdjacency => (
            Rebuild::AuthoritativeRelationTruth,
            Invalidation::AuthoritativeRelationDelta,
            Complexity::DirectionalRelationLookup,
            Memory::RelationDegreeBound,
        ),
        RelationRequirementBasis::ReverseAdjacency => (
            Rebuild::AuthoritativeRelationTruth,
            Invalidation::AuthoritativeRelationDelta,
            Complexity::ReverseRelationLookup,
            Memory::RelationDegreeBound,
        ),
        RelationRequirementBasis::TraversalWorkset => (
            Rebuild::OperationResolutionProof,
            Invalidation::ReadGraphProofDelta,
            Complexity::BoundedTraversalWorkset,
            Memory::FrontierDepthBound,
        ),
    }
}

fn distinct_values<T: Eq, Admission>(
    mut keyed: Vec<(String, T)>,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<Vec<T>, Stop<Admission>> {
    keyed = sort_digest_keys_admitted(keyed, admit)?;
    admit(1, 0).map_err(Stop::Admission)?;
    admit(amount(keyed.len())?, 0).map_err(Stop::Admission)?;
    let maximum = keyed.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
    let equality_work = maximum
        .checked_add(32)
        .and_then(|width| width.checked_mul(2))
        .and_then(|width| width.checked_mul(keyed.len()))
        .ok_or(Stop::AccountingOverflow)?;
    let relocation_work = keyed
        .len()
        .checked_mul(size_of::<(String, T)>())
        .and_then(|bytes| bytes.checked_mul(3))
        .ok_or(Stop::AccountingOverflow)?;
    admit(
        amount(
            equality_work
                .checked_add(relocation_work)
                .ok_or(Stop::AccountingOverflow)?,
        )?,
        0,
    )
    .map_err(Stop::Admission)?;
    keyed.dedup_by(|left, right| left.1 == right.1);
    claim_backing::<T, _>(keyed.len(), admit)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(keyed.len())
        .map_err(|_| Stop::AllocationUnavailable)?;
    for (_, value) in keyed {
        values.push(value);
    }
    Ok(values)
}

fn predicate_authorities<Admission>(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<Vec<WorthQueryGraphReadPredicateFieldAuthority>, Stop<Admission>> {
    // Shape, predicate-slice header and cardinality precede backing selection.
    admit(3, 0).map_err(Stop::Admission)?;
    let fields = input.shape().predicate_fields();
    claim_backing::<(String, WorthQueryGraphReadPredicateFieldAuthority), _>(fields.len(), admit)?;
    let mut keyed = Vec::new();
    keyed
        .try_reserve_exact(fields.len())
        .map_err(|_| Stop::AllocationUnavailable)?;
    for field in fields {
        // Aspect, field, family and schema authority metadata.
        admit(4, 0).map_err(Stop::Admission)?;
        admit(32, 0).map_err(Stop::Admission)?;
        claim_text(field.aspect().as_str(), admit)?;
        claim_text(field.field().as_str(), admit)?;
        claim_text(field.native_family(), admit)?;
        let authority = WorthQueryGraphReadPredicateFieldAuthority::new(
            *input.identity().schema_basis_digest(),
            field.aspect().clone(),
            field.field().clone(),
            field.native_family(),
        );
        let count_work = "predicate_authority:::"
            .len()
            .checked_add(field.aspect().as_str().len())
            .and_then(|work| work.checked_add(field.field().as_str().len()))
            .and_then(|work| work.checked_add(field.native_family().len()))
            .ok_or(Stop::AccountingOverflow)
            .and_then(amount)?;
        let key = admitted_digest_text(
            count_work,
            |out| authority.write_digest_part(out),
            &mut *admit,
        )
        .map_err(map_text_stop)?;
        keyed.push((key, authority));
    }
    distinct_values(keyed, admit)
}

fn ordering_authorities<Admission>(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<Vec<WorthQueryGraphReadOrderingFieldAuthority>, Stop<Admission>> {
    // Shape, ordering-slice header and cardinality precede backing selection.
    admit(3, 0).map_err(Stop::Admission)?;
    let fields = input.shape().ordering_fields();
    claim_backing::<(String, WorthQueryGraphReadOrderingFieldAuthority), _>(fields.len(), admit)?;
    let mut keyed = Vec::new();
    keyed
        .try_reserve_exact(fields.len())
        .map_err(|_| Stop::AllocationUnavailable)?;
    for field in fields {
        // Path, aspect, field, direction, family and schema metadata.
        admit(6, 0).map_err(Stop::Admission)?;
        admit(32, 0).map_err(Stop::Admission)?;
        for text in [
            field.collection_path(),
            field.aspect().as_str(),
            field.field().as_str(),
            field.direction(),
            field.native_family(),
        ] {
            claim_text(text, admit)?;
        }
        let authority = WorthQueryGraphReadOrderingFieldAuthority::new(
            *input.identity().schema_basis_digest(),
            field.collection_path(),
            field.aspect().clone(),
            field.field().clone(),
            field.direction(),
            field.native_family(),
        );
        let count_work = [
            field.collection_path(),
            field.aspect().as_str(),
            field.field().as_str(),
            field.direction(),
            field.native_family(),
        ]
        .into_iter()
        .try_fold("ordering_authority:::::".len(), |total, part| {
            total.checked_add(part.len())
        })
        .ok_or(Stop::AccountingOverflow)
        .and_then(amount)?;
        let key = admitted_digest_text(
            count_work,
            |out| authority.write_digest_part(out),
            &mut *admit,
        )
        .map_err(map_text_stop)?;
        keyed.push((key, authority));
    }
    distinct_values(keyed, admit)
}

pub(super) fn build_row<Admission>(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    spec: RowSpec<'_>,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<WorthQueryGraphReadAccessRequirementRow, Stop<Admission>> {
    use WorthQueryGraphReadAccessComplexityContract as Complexity;
    use WorthQueryGraphReadAccessInvalidationBasis as Invalidation;
    use WorthQueryGraphReadAccessMemoryEstimateBasis as Memory;
    use WorthQueryGraphReadAccessRebuildBasis as Rebuild;
    use WorthQueryGraphReadAccessRequirementKind as Kind;
    // Spec tag, kind, four dimension axes and maximum-cardinality metadata.
    admit(7, 0).map_err(Stop::Admission)?;
    let row = match spec {
        RowSpec::Relation {
            relation,
            operator,
            kind,
            basis,
        } => {
            let (rebuild, invalidation, complexity, memory) = relation_dimensions(basis);
            // Relation name, direction, depth and schema authority metadata.
            admit(4, 0).map_err(Stop::Admission)?;
            let name = relation.relation_name();
            admit(32, 0).map_err(Stop::Admission)?;
            claim_text(name, admit)?;
            claim_text(name, admit)?;
            WorthQueryGraphReadAccessRequirementRow::new(
                kind,
                rebuild,
                invalidation,
                complexity,
                memory,
            )
            .with_relation(
                name,
                WorthQueryGraphReadRelationAuthority::new(
                    *input.identity().schema_basis_digest(),
                    name,
                ),
                relation.direction().clone(),
                relation.depth(),
            )
            .with_fanout_posture(input.shape().fanout_posture().clone())
            .with_traversal_operator(operator.clone())
        }
        RowSpec::ResultBuffer => WorthQueryGraphReadAccessRequirementRow::new(
            Kind::ResultBuffer,
            Rebuild::OperationResolutionProof,
            Invalidation::ReadGraphProofDelta,
            Complexity::ResultPressureBuffer,
            Memory::ResultPressureBound,
        )
        .with_result_pressure(input.shape().result_pressure().clone()),
        RowSpec::Predicate => WorthQueryGraphReadAccessRequirementRow::new(
            Kind::PredicateSupport,
            Rebuild::SelectivityProof,
            Invalidation::AuthoritativeFieldDelta,
            Complexity::CandidatePredicateSupport,
            Memory::PredicateCandidateSet,
        )
        .with_predicate_family(input.shape().predicate_family().clone())
        .with_predicate_field_authorities_sorted(predicate_authorities(input, admit)?),
        RowSpec::Ordering => WorthQueryGraphReadAccessRequirementRow::new(
            Kind::OrderingSupport,
            Rebuild::AuthoritativeFieldTruth,
            Invalidation::AuthoritativeFieldDelta,
            Complexity::CandidateOrderingSupport,
            Memory::OrderedCandidateSet,
        )
        .with_ordering_posture(input.shape().ordering_posture().clone())
        .with_ordering_field_authorities_sorted(ordering_authorities(input, admit)?),
        RowSpec::RootUnionDedup => WorthQueryGraphReadAccessRequirementRow::new(
            Kind::DedupSet,
            Rebuild::OperationResolutionProof,
            Invalidation::ReadGraphProofDelta,
            Complexity::BoundedTraversalWorkset,
            Memory::FrontierDepthBound,
        ),
        RowSpec::Proof => WorthQueryGraphReadAccessRequirementRow::new(
            Kind::ProofSupport,
            Rebuild::ReadGraphProof,
            Invalidation::ReadGraphProofDelta,
            Complexity::ProofEvidenceSupport,
            Memory::ProofEvidenceSet,
        ),
        kind @ (RowSpec::Lifecycle | RowSpec::LiveMaintenance) => {
            let kind = if matches!(kind, RowSpec::Lifecycle) {
                Kind::MaterializationLifecycle
            } else {
                Kind::LiveMaintenanceSupport
            };
            WorthQueryGraphReadAccessRequirementRow::new(
                kind,
                Rebuild::RuntimeSupportRequired,
                Invalidation::RuntimeLifecycleDelta,
                Complexity::LifecycleSupportAdmission,
                Memory::LifecycleManagedSupport,
            )
            .with_lifecycle_class(input.shape().lifecycle_class().clone())
        }
    };
    Ok(bound_row(input, row))
}
