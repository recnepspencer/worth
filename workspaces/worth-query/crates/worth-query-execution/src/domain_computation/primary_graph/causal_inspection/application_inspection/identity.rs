//! Additive admitted identities describe actual originals and their fresh session.
use super::WorthQueryApplicationInspectionTarget;
use crate::domain_computation::{
    primary_graph::WorthQueryApplicationBasisSelectionIdentity,
    provider_session::WorthQueryManagedGraphWorkSession,
};
use std::{
    alloc::Layout,
    fmt::Write,
    sync::{atomic::AtomicUsize, Arc},
};
use worth_foundational::facade::{
    prepare_owned_canonical_basis_sequence_admitted, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisPreparationStop,
    CanonicalBasisValue, CanonicalDigestAdmissionStop, CanonicalDigestAlgorithmId,
    CanonicalDigestFrontDoor, CanonicalDigestId, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};
use worth_runtime_bridge::facade::{bridge_truth_external_identity_token, BridgeIdentityEvidence};

const SCHEME: &str = "worth.query.application-inspection.v1";

pub(super) struct InspectionReferenceIdentities {
    pub(super) observation: BridgeIdentityEvidence,
    pub(super) admission: BridgeIdentityEvidence,
    pub(super) anchor: BridgeIdentityEvidence,
}

pub(super) enum InspectionIdentityStop<Stop> {
    Resource(Stop),
    Basis(CanonicalBasisPreparationStop<Stop>),
    Digest(CanonicalDigestAdmissionStop<Stop>),
    Overflow,
    ForeignSession,
    ProductRequired,
}

enum Part<'a> {
    Text(&'a str),
    Digest(CanonicalDigestId),
    Number(u64),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum InspectionOriginalReference {
    Before,
    After,
}

pub(super) fn identities<Stop>(
    target: &Arc<WorthQueryApplicationInspectionTarget>,
    session: &WorthQueryManagedGraphWorkSession,
    reference: InspectionOriginalReference,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<InspectionReferenceIdentities, InspectionIdentityStop<Stop>> {
    let (actual, current) = session
        .inspection_binding()
        .ok_or(InspectionIdentityStop::ForeignSession)?;
    if !Arc::ptr_eq(actual, target) {
        return Err(InspectionIdentityStop::ForeignSession);
    }
    let original = match reference {
        InspectionOriginalReference::After => target.original(),
        InspectionOriginalReference::Before => target
            .before()
            .ok_or(InspectionIdentityStop::ForeignSession)?,
    };
    let source = &original.source;
    let source_product = product(&source.selection)?;
    let receipt_product = product(original.basis.selection())?;
    let schema = &source.schema_binding;
    let count =
        u64::try_from(original.result_count).map_err(|_| InspectionIdentityStop::Overflow)?;
    let observation = compose(
        "query.application-inspection.original-observation.v1",
        &[
            Part::Number(source.runtime_authority),
            Part::Number(schema.runtime_ordinal()),
            Part::Number(schema.generation()),
            Part::Digest(*schema.package_identity()),
            Part::Digest(*schema.schema_identity()),
            Part::Digest(*target.contract().graph_obligations().identity().digest()),
            Part::Digest(CanonicalDigestId::new(*source.query_identity.as_bytes())),
            Part::Digest(source.parameter_binding_identity),
            Part::Digest(CanonicalDigestId::new(
                source.idempotency_identity().bytes(),
            )),
            Part::Number(original.basis.runtime_instance_id()),
            Part::Text(original.basis.branch_id().0.as_str()),
            Part::Number(original.basis.snapshot_id().0),
            Part::Number(count),
            Part::Number(source_product.branch_identity().owner_identity().get()),
            Part::Text(source_product.branch_identity().name().as_str()),
            Part::Number(source_product.lifecycle_incarnation().ordinal()),
            Part::Number(source_product.reference_generation().get()),
            Part::Number(source_product.selected_commit().owner_identity().get()),
            Part::Number(source_product.selected_commit().ordinal()),
            Part::Number(receipt_product.branch_identity().owner_identity().get()),
            Part::Text(receipt_product.branch_identity().name().as_str()),
            Part::Number(receipt_product.lifecycle_incarnation().ordinal()),
            Part::Number(receipt_product.reference_generation().get()),
            Part::Number(receipt_product.selected_commit().owner_identity().get()),
            Part::Number(receipt_product.selected_commit().ordinal()),
            Part::Text("observed-result"),
        ],
        admit,
    )?;
    let current_product = product(current.selection())?;
    let admission = compose(
        "query.application-inspection.fresh-admission.v1",
        &[
            Part::Number(session.runtime_authority().as_u64()),
            Part::Number(session.identity().as_u64()),
            Part::Number(session.managed_run_identity().as_u64()),
            Part::Number(session.plan_identity().as_u64()),
            Part::Number(current.runtime_instance_id()),
            Part::Text(current.branch_id().0.as_str()),
            Part::Number(current.snapshot_id().0),
            Part::Number(current_product.branch_identity().owner_identity().get()),
            Part::Text(current_product.branch_identity().name().as_str()),
            Part::Number(current_product.lifecycle_incarnation().ordinal()),
            Part::Number(current_product.reference_generation().get()),
            Part::Number(current_product.selected_commit().owner_identity().get()),
            Part::Number(current_product.selected_commit().ordinal()),
        ],
        admit,
    )?;
    let anchor = compose(
        "query.application-inspection.observation-anchor.v1",
        &[
            Part::Text(observation.terminal_projection_for_reporting()),
            Part::Text(admission.terminal_projection_for_reporting()),
            Part::Text("why-observed"),
            Part::Text("reference-only"),
        ],
        admit,
    )?;
    Ok(InspectionReferenceIdentities {
        observation,
        admission,
        anchor,
    })
}

pub(super) fn comparison_identity<Stop>(
    before: &super::report::InspectionReportPayload,
    after: &super::report::InspectionReportPayload,
    reason: super::super::CausalInspectionReason,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, InspectionIdentityStop<Stop>> {
    compose(
        "query.application-inspection.original-comparison.v1",
        &[
            Part::Text(before.observation_identity()),
            Part::Text(before.report_identity()),
            Part::Text(after.observation_identity()),
            Part::Text(after.report_identity()),
            Part::Text(reason.as_str()),
            Part::Text("reference-only"),
        ],
        admit,
    )
}

fn product<Stop>(
    selection: &WorthQueryApplicationBasisSelectionIdentity,
) -> Result<&crate::basis::WorthQueryProductBranchReadIdentity, InspectionIdentityStop<Stop>> {
    match selection {
        WorthQueryApplicationBasisSelectionIdentity::Product(product) => Ok(product),
        WorthQueryApplicationBasisSelectionIdentity::Relational => {
            Err(InspectionIdentityStop::ProductRequired)
        }
    }
}

fn compose<Stop>(
    domain: &'static str,
    parts: &[Part<'_>],
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, InspectionIdentityStop<Stop>> {
    use InspectionIdentityStop as Failure;
    let count = u32::try_from(parts.len()).map_err(|_| Failure::Overflow)?;
    let strings = parts
        .iter()
        .try_fold(SCHEME.len() + "sha256".len(), |bytes, part| {
            bytes.checked_add(match part {
                Part::Text(text) => text.len(),
                _ => 0,
            })
        })
        .ok_or(Failure::Overflow)?;
    let bytes = Layout::array::<CanonicalBasisEntry>(parts.len())
        .map_err(|_| Failure::Overflow)?
        .size()
        .checked_add(strings)
        .ok_or(Failure::Overflow)?;
    let work = bytes.checked_add(parts.len()).ok_or(Failure::Overflow)?;
    admit(work, bytes).map_err(Failure::Resource)?;
    let domain = CanonicalBasisDomain::Future(domain);
    let mut entries = Vec::with_capacity(parts.len());
    for (ordinal, part) in (0..count).zip(parts) {
        let value = match part {
            Part::Text(text) => CanonicalBasisValue::ExactText((*text).to_owned().into()),
            Part::Digest(value) => CanonicalBasisValue::BytesDigest(*value),
            Part::Number(value) => CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: (*value).into(),
            },
        };
        entries.push(CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::EntryOrdinal(ordinal),
            CanonicalBasisEntryKind::Identity,
            value,
        ));
    }
    let version =
        CanonicalizationRuleVersion::new(SCHEME).expect("fixed inspection scheme is canonical");
    let sequence =
        prepare_owned_canonical_basis_sequence_admitted(version, domain, entries, &mut *admit)
            .map_err(Failure::Basis)?;
    let ready = CanonicalDigestFrontDoor
        .for_sequence_with_admission(
            sequence,
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalDigestWorkBudget::standard(),
            admit,
        )
        .map_err(Failure::Digest)?;
    let digest = CanonicalDigestFrontDoor
        .derive_with_admission(ready, admit)
        .map_err(Failure::Digest)?;
    let length = SCHEME
        .len()
        .checked_add(1 + "sha256".len() + 1 + 64)
        .ok_or(Failure::Overflow)?;
    let arc = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::array::<u8>(length).map_err(|_| Failure::Overflow)?)
        .map_err(|_| Failure::Overflow)?
        .0
        .pad_to_align()
        .size();
    admit(length, length.checked_add(arc).ok_or(Failure::Overflow)?).map_err(Failure::Resource)?;
    let mut token = String::with_capacity(length);
    token.push_str(SCHEME);
    token.push_str(":sha256:");
    for byte in digest.value().bytes() {
        write!(&mut token, "{byte:02x}").expect("fixed admitted hex String");
    }
    // This is descriptive Query-owned evidence. The actual Inspection session,
    // target proof and terminal, not this externalized label, admit publication.
    Ok(BridgeIdentityEvidence::from_external_authority(
        bridge_truth_external_identity_token(token),
    ))
}
