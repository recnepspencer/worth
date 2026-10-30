use crate::admission::{UiSupportPosture, UiSupportSnapshot};
use crate::declaration::{
    UiDeclarationArtifact, UiDeclarationSupportRowSchemaKind, UiDeclaredPostureApplicability,
};
use crate::evidence::UiEvidenceAuthorityGeneration;
use crate::graph::UiGraphSnapshot;
use crate::obligations::catalog::{UiObligationCheckKind, UiObligationFamilyCatalog};
use crate::obligations::inspection::{
    not_selected_obligation_evidence_record, selected_obligation_evidence_records,
    UiObligationEvidenceHandle, UiObligationEvidenceIndex, UiObligationEvidenceRecord,
    UiObligationNonSelectionReason,
};
use crate::obligations::prerequisites::UiObligationPrerequisiteEvidenceRef;
use crate::obligations::touch::UiGraphTouchDescriptor;

use super::{
    UiObligationSelectionMatrix, UiObligationSupportBasis, UiObligationSupportSelectionPosture,
    UiSelectedObligation, UiSelectedObligationIdentity, UiSelectedObligationSet,
};

pub struct UiObligationSelectionBoundary<'a> {
    support_artifacts: &'a [UiDeclarationArtifact],
    graph_snapshot: &'a UiGraphSnapshot,
    family_catalog: UiObligationFamilyCatalog,
}

struct UiSelectedObligationConstruction<'a> {
    touch: &'a UiGraphTouchDescriptor,
    family: crate::obligations::catalog::UiObligationFamily,
    row: super::UiObligationSelectionMatrixRow,
    support_posture: UiObligationSupportSelectionPosture,
    check_kind: UiObligationCheckKind,
    evidence_handle: UiObligationEvidenceHandle,
    selection_reasons: Box<[super::UiObligationSelectionReason]>,
    prerequisite_evidence_refs: Box<[UiObligationPrerequisiteEvidenceRef]>,
}

impl<'a> UiObligationSelectionBoundary<'a> {
    pub(crate) const fn new(
        support_artifacts: &'a [UiDeclarationArtifact],
        graph_snapshot: &'a UiGraphSnapshot,
    ) -> Self {
        Self {
            support_artifacts,
            graph_snapshot,
            family_catalog: UiObligationFamilyCatalog::closed(),
        }
    }

    pub fn select(
        &self,
        touch: &UiGraphTouchDescriptor,
        support_snapshot: UiSupportSnapshot,
        requested_target: crate::admission::UiAdmissionTarget,
    ) -> UiSelectedObligationSet {
        let selected_declaration_identity = self
            .graph_snapshot
            .lookup()
            .graph_node(touch.target().graph_node_identity())
            .map(|lookup| lookup.value().declaration_identity().clone());
        let (obligations, non_selected_records) = match support_snapshot.posture() {
            UiSupportPosture::WrongWorld { .. } => (Vec::new(), Vec::new()),
            UiSupportPosture::Supported { .. }
            | UiSupportPosture::Unsupported { .. }
            | UiSupportPosture::Deferred { .. }
            | UiSupportPosture::DiagnosticOnly { .. } => {
                self.select_supported_matrix(touch, &support_snapshot)
            }
        };

        let selected = UiSelectedObligationSet::new(
            UiEvidenceAuthorityGeneration::new(self.graph_snapshot.generation().as_u64()),
            touch.clone(),
            support_snapshot,
            requested_target,
            selected_declaration_identity,
            obligations.into_boxed_slice(),
            UiObligationEvidenceIndex::empty(),
        );
        let evidence_index =
            UiObligationEvidenceIndex::new(non_selected_records.into_boxed_slice())
                .with_appended(selected_obligation_evidence_records(&selected));

        selected.with_evidence_index(evidence_index)
    }

    fn select_supported_matrix(
        &self,
        touch: &UiGraphTouchDescriptor,
        support_snapshot: &UiSupportSnapshot,
    ) -> (Vec<UiSelectedObligation>, Vec<UiObligationEvidenceRecord>) {
        let selection_authority_digest =
            UiSelectedObligationSet::identity_digest_for(touch, support_snapshot);
        let node_record = self
            .graph_snapshot
            .lookup()
            .graph_node(touch.target().graph_node_identity())
            .map(|lookup| lookup.value().clone());
        let target = support_snapshot.target();
        let mut obligations = Vec::new();
        let mut evidence_records = Vec::new();
        for (ordinal, row) in UiObligationSelectionMatrix::starter()
            .rows()
            .iter()
            .copied()
            .enumerate()
        {
            let row_support_posture = self.row_support_posture(
                row.support_basis(),
                node_record.as_ref(),
                support_snapshot.posture(),
            );
            let base_reasons = row
                .selection_reasons(touch, row_support_posture)
                .into_boxed_slice();
            if !row.matches(touch, node_record.as_ref(), row_support_posture) {
                evidence_records.push(not_selected_obligation_evidence_record(
                    touch,
                    selection_authority_digest,
                    ordinal,
                    row.family(),
                    base_reasons,
                    &prerequisite_evidence_refs(row.support_basis(), target),
                    UiObligationNonSelectionReason::RuleDidNotMatch,
                ));
                continue;
            }

            let Some(family) = self.family_for_row(row, node_record.as_ref()) else {
                evidence_records.push(not_selected_obligation_evidence_record(
                    touch,
                    selection_authority_digest,
                    ordinal,
                    row.family(),
                    base_reasons,
                    &prerequisite_evidence_refs(row.support_basis(), target),
                    UiObligationNonSelectionReason::FamilyUnavailable,
                ));
                continue;
            };

            match row_support_posture {
                UiObligationSupportSelectionPosture::Supported
                | UiObligationSupportSelectionPosture::Unsupported
                | UiObligationSupportSelectionPosture::Deferred
                | UiObligationSupportSelectionPosture::DiagnosticOnly => {
                    let evidence_handle = UiObligationEvidenceHandle::new(
                        crate::obligations::inspection::UiObligationEvidenceHandleKind::Selected,
                        touch.identity_digest()
                            ^ (family as u64).rotate_left(11)
                            ^ (ordinal as u64).rotate_left(29),
                    );
                    let prerequisite_evidence_refs =
                        prerequisite_evidence_refs(row.support_basis(), target);
                    obligations.push(self.obligation(UiSelectedObligationConstruction {
                        touch,
                        family,
                        row,
                        support_posture: row_support_posture,
                        check_kind: row.check_kind(),
                        evidence_handle,
                        selection_reasons: base_reasons,
                        prerequisite_evidence_refs,
                    }));
                }
                UiObligationSupportSelectionPosture::WrongWorld => {
                    evidence_records.push(not_selected_obligation_evidence_record(
                        touch,
                        selection_authority_digest,
                        ordinal,
                        family,
                        base_reasons,
                        &prerequisite_evidence_refs(row.support_basis(), target),
                        UiObligationNonSelectionReason::WrongWorld,
                    ));
                }
            }
        }

        (obligations, evidence_records)
    }

    fn obligation(
        &self,
        construction: UiSelectedObligationConstruction<'_>,
    ) -> UiSelectedObligation {
        let UiSelectedObligationConstruction {
            touch,
            family,
            row,
            support_posture,
            check_kind,
            evidence_handle,
            selection_reasons,
            prerequisite_evidence_refs,
        } = construction;
        debug_assert!(self.family_catalog.contains(family));
        let identity = UiSelectedObligationIdentity::new(
            touch.identity_digest(),
            family,
            touch.target(),
            row.aspect_scope().to_vec().into_boxed_slice(),
            touch.world(),
            row.support_basis(),
        );
        UiSelectedObligation::new(
            identity,
            family,
            check_kind,
            support_posture,
            evidence_handle,
            selection_reasons,
            prerequisite_evidence_refs,
        )
    }

    fn row_support_posture(
        &self,
        support_basis: UiObligationSupportBasis,
        node_record: Option<&crate::graph::UiGraphNodeRecord>,
        touch_support_posture: &UiSupportPosture,
    ) -> UiObligationSupportSelectionPosture {
        if matches!(support_basis, UiObligationSupportBasis::TouchMeaning) {
            return UiObligationSupportSelectionPosture::from_support_posture(
                touch_support_posture,
            );
        }

        let Some(node_record) = node_record else {
            return UiObligationSupportSelectionPosture::Unsupported;
        };
        let Some(artifact) = self
            .support_artifacts
            .iter()
            .find(|artifact| artifact.identity() == node_record.declaration_identity())
        else {
            return UiObligationSupportSelectionPosture::Unsupported;
        };
        let Ok(snapshot) = artifact.support_snapshot() else {
            return UiObligationSupportSelectionPosture::Unsupported;
        };
        let Some(row) = snapshot.row(schema_kind_for_support_basis(support_basis)) else {
            return UiObligationSupportSelectionPosture::Unsupported;
        };

        if row.unsupported_posture().is_some() {
            UiObligationSupportSelectionPosture::Deferred
        } else {
            match row.applicability() {
                UiDeclaredPostureApplicability::Required
                | UiDeclaredPostureApplicability::Optional => {
                    UiObligationSupportSelectionPosture::Supported
                }
                UiDeclaredPostureApplicability::DiagnosticOnly => {
                    UiObligationSupportSelectionPosture::DiagnosticOnly
                }
                UiDeclaredPostureApplicability::NotApplicable => {
                    UiObligationSupportSelectionPosture::Unsupported
                }
                UiDeclaredPostureApplicability::ArchitecturallyOwnedButNotYetAdmitted => {
                    UiObligationSupportSelectionPosture::Deferred
                }
            }
        }
    }

    fn family_for_row(
        &self,
        row: super::UiObligationSelectionMatrixRow,
        node_record: Option<&crate::graph::UiGraphNodeRecord>,
    ) -> Option<crate::obligations::catalog::UiObligationFamily> {
        if row.support_basis() != UiObligationSupportBasis::ServiceUsage
            || row.family()
                != crate::obligations::catalog::UiObligationFamily::PortalHostRequirement
        {
            return Some(row.family());
        }

        let node_record = node_record?;
        let artifact = self
            .support_artifacts
            .iter()
            .find(|artifact| artifact.identity() == node_record.declaration_identity())?;
        let snapshot = artifact.support_snapshot().ok()?;
        let row = snapshot.row(UiDeclarationSupportRowSchemaKind::ServiceUsage)?;
        match row.declared_service_usage_posture()?.family() {
            crate::capability::UiRuntimeServiceFamily::Portal => {
                Some(crate::obligations::catalog::UiObligationFamily::PortalHostRequirement)
            }
            crate::capability::UiRuntimeServiceFamily::Focus => {
                Some(crate::obligations::catalog::UiObligationFamily::FocusRouteRequirement)
            }
            crate::capability::UiRuntimeServiceFamily::Motion => {
                Some(crate::obligations::catalog::UiObligationFamily::MotionSupportRequirement)
            }
            crate::capability::UiRuntimeServiceFamily::Scroll => {
                Some(crate::obligations::catalog::UiObligationFamily::ScrollRoutingRequirement)
            }
            crate::capability::UiRuntimeServiceFamily::Selection => {
                Some(crate::obligations::catalog::UiObligationFamily::SelectionStateRequirement)
            }
            crate::capability::UiRuntimeServiceFamily::CommandRouting => {
                Some(crate::obligations::catalog::UiObligationFamily::CommandRouteRequirement)
            }
        }
    }
}

const fn schema_kind_for_support_basis(
    support_basis: UiObligationSupportBasis,
) -> UiDeclarationSupportRowSchemaKind {
    match support_basis {
        UiObligationSupportBasis::TouchMeaning => UiDeclarationSupportRowSchemaKind::TouchMeaning,
        UiObligationSupportBasis::QueryBinding => UiDeclarationSupportRowSchemaKind::QueryBinding,
        UiObligationSupportBasis::ServiceUsage => UiDeclarationSupportRowSchemaKind::ServiceUsage,
        UiObligationSupportBasis::MeasurementPolicy => {
            UiDeclarationSupportRowSchemaKind::MeasurementPolicy
        }
        UiObligationSupportBasis::HostCapability => {
            UiDeclarationSupportRowSchemaKind::HostCapability
        }
    }
}

fn prerequisite_evidence_refs(
    support_basis: UiObligationSupportBasis,
    target: &crate::admission::UiAdmissionTarget,
) -> Box<[UiObligationPrerequisiteEvidenceRef]> {
    let mut refs = Vec::new();
    if matches!(
        support_basis,
        UiObligationSupportBasis::HostCapability | UiObligationSupportBasis::ServiceUsage
    ) && let Some(host_capability_report) = target.host_capability_report()
    {
        refs.push(UiObligationPrerequisiteEvidenceRef::Host(
            host_capability_report.clone(),
        ));
    }
    refs.into_boxed_slice()
}
