use super::super::{
    WorthQueryCompiledSemanticAspectDependencyClosure, WorthQuerySemanticDependencyRole,
};
use super::{
    WorthQueryImpactAdmissionDenial, WorthQueryImpactAdmissionDenialKind, WorthQueryImpactClass,
    WorthQueryImpactCounters, WorthQueryInvalidationCandidateSet,
};

pub struct WorthQueryAdmittedInvalidationImpact {
    pub(crate) affinity:
        crate::domain_installation::operation_authority_chain::WorthQueryOperationAuthorityBasis,
    pub(crate) delivery: worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery,
    pub(crate) locality: WorthQueryAdmittedLocality,
    pub(crate) roles: Vec<WorthQuerySemanticDependencyRole>,
    pub(crate) class: WorthQueryImpactClass,
    pub(crate) consequence_classes: Vec<WorthQueryImpactClass>,
    pub(crate) phase: crate::domain_installation::operation_authority_chain::WorthQueryOperationPhaseProof<
        crate::domain_installation::operation_authority_chain::WorthQueryInvalidationAdmittedPhase,
    >,
}

/// Read-only observation of one impact actually admitted by Query.
///
/// This value retains the Bridge-delivered truth and Query-selected roles for
/// certification and diagnostics. It cannot authorize maintenance or
/// publication.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct WorthQueryAdmittedInvalidationObservation {
    truth: worth_runtime_bridge::facade::BridgeDeliveredTruthChange,
    roles: Vec<WorthQuerySemanticDependencyRole>,
    performed_signal_binding: Option<String>,
}

impl WorthQueryAdmittedInvalidationObservation {
    pub fn truth(&self) -> &worth_runtime_bridge::facade::BridgeDeliveredTruthChange {
        &self.truth
    }

    pub fn roles(&self) -> &[WorthQuerySemanticDependencyRole] {
        &self.roles
    }

    pub fn performed_signal_binding(&self) -> Option<&str> {
        self.performed_signal_binding.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryAdmittedLocality {
    ExactSourceRecord {
        partition_id: u32,
        local_slot: u64,
        generation: u32,
    },
    SourcePartition(String),
    WholeLogicalGraph,
}

impl WorthQueryAdmittedInvalidationImpact {
    pub fn truth(&self) -> &worth_runtime_bridge::facade::BridgeDeliveredTruthChange {
        self.delivery.truth()
    }

    pub fn roles(&self) -> &[WorthQuerySemanticDependencyRole] {
        &self.roles
    }

    pub const fn class(&self) -> WorthQueryImpactClass {
        self.class
    }

    pub fn consequence_classes(&self) -> &[WorthQueryImpactClass] {
        &self.consequence_classes
    }

    pub fn has_performed_signal_consequence(&self) -> bool {
        self.delivery.performed_signal().is_some()
    }

    pub(crate) fn observation(&self) -> WorthQueryAdmittedInvalidationObservation {
        WorthQueryAdmittedInvalidationObservation {
            truth: self.delivery.truth().clone(),
            roles: self.roles.clone(),
            performed_signal_binding: self
                .delivery
                .performed_signal()
                .map(|performed| performed.query_binding_identity().to_owned()),
        }
    }
}

pub fn admit_current_invalidation_impact<D, O, F, L: crate::basis_lifecycle::BasisOperationLane>(
    current: &crate::domain_installation::WorthQuerySettledDomainProjection<D, O, F, L>,
    candidates: WorthQueryInvalidationCandidateSet,
) -> Result<WorthQueryAdmittedInvalidationImpact, WorthQueryImpactAdmissionDenial> {
    let closure: &WorthQueryCompiledSemanticAspectDependencyClosure =
        current.semantic_aspect_dependency_closure();
    if candidates.affinity != closure.affinity {
        return Err(WorthQueryImpactAdmissionDenial::new(
            WorthQueryImpactAdmissionDenialKind::ForeignOperation,
            WorthQueryImpactCounters {
                operation_affinity_checks: 1,
                ..WorthQueryImpactCounters::default()
            },
        ));
    }
    if candidates
        .roles
        .contains(&WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness)
        && !candidates.has_performed_signal_consequence()
    {
        return Err(WorthQueryImpactAdmissionDenial::new(
            WorthQueryImpactAdmissionDenialKind::PerformedSignalRequired,
            WorthQueryImpactCounters {
                conditional_authority_checks: 1,
                ..Default::default()
            },
        ));
    }
    let (class, consequence_classes) = classify_roles(&candidates.roles);
    let locality = admitted_locality(
        candidates.truth().change_set().dependency(),
        candidates.truth().change_set().changes(),
    )?;
    let phase = crate::domain_installation::operation_authority_chain::mint_operation_phase_proof(
        format!(
            "invalidation-admitted:{}:{}",
            candidates.affinity.operation_identity,
            candidates
                .truth()
                .change_set()
                .dependency()
                .dependency_ordinal()
        ),
        None,
        candidates.affinity.clone(),
    );
    Ok(WorthQueryAdmittedInvalidationImpact {
        affinity: candidates.affinity,
        delivery: candidates.delivery,
        locality,
        roles: candidates.roles,
        class,
        consequence_classes,
        phase,
    })
}

fn admitted_locality(
    dependency: &worth_runtime_bridge::facade::BridgeSemanticDependencyCandidate,
    changes: &[worth_runtime_bridge::facade::BridgeDeliveredCorrespondenceChange],
) -> Result<WorthQueryAdmittedLocality, WorthQueryImpactAdmissionDenial> {
    use worth_runtime_bridge::facade::BridgeSemanticLocality;
    match dependency.locality() {
        BridgeSemanticLocality::SourceRecord => {
            let record = dependency.source_record_identity().ok_or_else(|| {
                WorthQueryImpactAdmissionDenial::new(
                    WorthQueryImpactAdmissionDenialKind::ConditionalDeliveryMismatch,
                    WorthQueryImpactCounters {
                        delivery_identity_checks: 1,
                        ..Default::default()
                    },
                )
            })?;
            Ok(WorthQueryAdmittedLocality::ExactSourceRecord {
                partition_id: record.partition_id(),
                local_slot: record.local_slot(),
                generation: record.generation(),
            })
        }
        BridgeSemanticLocality::ManagedSourceRecord => {
            let records = changes
                .iter()
                .filter_map(|change| change.relational_record_identity())
                .collect::<std::collections::BTreeSet<_>>();
            let mut records = records.into_iter();
            let record = records.next().ok_or_else(ambiguous_managed_record)?;
            if records.next().is_some() {
                return Err(ambiguous_managed_record());
            }
            Ok(WorthQueryAdmittedLocality::ExactSourceRecord {
                partition_id: record.partition_id(),
                local_slot: record.local_slot(),
                generation: record.generation(),
            })
        }
        BridgeSemanticLocality::SourcePartition(partition) => Ok(single_changed_record(changes)
            .map(exact_record_locality)
            .unwrap_or_else(|| {
                WorthQueryAdmittedLocality::SourcePartition(partition.as_str().to_owned())
            })),
        BridgeSemanticLocality::WholeLogicalGraph => Ok(single_changed_record(changes)
            .map(exact_record_locality)
            .unwrap_or(WorthQueryAdmittedLocality::WholeLogicalGraph)),
    }
}

fn single_changed_record(
    changes: &[worth_runtime_bridge::facade::BridgeDeliveredCorrespondenceChange],
) -> Option<worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts> {
    let records = changes
        .iter()
        .filter_map(|change| change.relational_record_identity())
        .collect::<std::collections::BTreeSet<_>>();
    (records.len() == 1).then(|| *records.first().expect("one changed record exists"))
}

fn exact_record_locality(
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> WorthQueryAdmittedLocality {
    WorthQueryAdmittedLocality::ExactSourceRecord {
        partition_id: record.partition_id(),
        local_slot: record.local_slot(),
        generation: record.generation(),
    }
}

fn ambiguous_managed_record() -> WorthQueryImpactAdmissionDenial {
    WorthQueryImpactAdmissionDenial::new(
        WorthQueryImpactAdmissionDenialKind::ConditionalDeliveryMismatch,
        WorthQueryImpactCounters {
            delivery_identity_checks: 1,
            ..Default::default()
        },
    )
}

fn classify_roles(
    roles: &[WorthQuerySemanticDependencyRole],
) -> (WorthQueryImpactClass, Vec<WorthQueryImpactClass>) {
    let primary = roles.iter().copied().fold(
        WorthQueryImpactClass::UnaffectedOrSuppressed,
        |current, role| widen(current, class_for_role(role)),
    );
    let mut consequences = Vec::new();
    for role in roles {
        let class = class_for_role(*role);
        if class != WorthQueryImpactClass::UnaffectedOrSuppressed && !consequences.contains(&class)
        {
            consequences.push(class);
        }
    }
    consequences.sort_by_key(|class| rank(*class));
    if consequences.is_empty() {
        consequences.push(WorthQueryImpactClass::UnaffectedOrSuppressed);
    }
    (primary, consequences)
}

fn class_for_role(role: WorthQuerySemanticDependencyRole) -> WorthQueryImpactClass {
    match role {
        WorthQuerySemanticDependencyRole::OperationalIdentity => WorthQueryImpactClass::Replacement,
        WorthQuerySemanticDependencyRole::SelectionOrMembership => {
            WorthQueryImpactClass::MembershipSplice
        }
        WorthQuerySemanticDependencyRole::Ordering | WorthQuerySemanticDependencyRole::Grouping => {
            WorthQueryImpactClass::ReorderOrRegroup
        }
        WorthQuerySemanticDependencyRole::ProjectedValue
        | WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness => {
            WorthQueryImpactClass::ValuePatch
        }
        WorthQuerySemanticDependencyRole::WindowBoundary => WorthQueryImpactClass::WindowShift,
        WorthQuerySemanticDependencyRole::SupportAndLifecycle => {
            WorthQueryImpactClass::ExplicitRebind
        }
        WorthQuerySemanticDependencyRole::InstalledDomainInvariant => {
            WorthQueryImpactClass::Reexecute
        }
        WorthQuerySemanticDependencyRole::AdvisoryOnlyContext => {
            WorthQueryImpactClass::UnaffectedOrSuppressed
        }
    }
}

fn widen(left: WorthQueryImpactClass, right: WorthQueryImpactClass) -> WorthQueryImpactClass {
    if rank(left) >= rank(right) {
        left
    } else {
        right
    }
}

const fn rank(class: WorthQueryImpactClass) -> u8 {
    match class {
        WorthQueryImpactClass::UnaffectedOrSuppressed => 0,
        WorthQueryImpactClass::ValuePatch => 1,
        WorthQueryImpactClass::MembershipSplice => 2,
        WorthQueryImpactClass::ReorderOrRegroup => 3,
        WorthQueryImpactClass::WindowShift => 4,
        WorthQueryImpactClass::Reexecute => 5,
        WorthQueryImpactClass::ExplicitRebind => 6,
        WorthQueryImpactClass::Replacement => 7,
        WorthQueryImpactClass::Retirement => 8,
        WorthQueryImpactClass::UnsupportedEscalation => 9,
    }
}
