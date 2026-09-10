use super::{BridgeConditionalDenial, BridgeConditionalDenialKind};

#[derive(Clone)]
struct BridgeConditionalSemanticObservationRead {
    ordinal: usize,
    record: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    managed_record: bool,
    contract: worth_foundational::facade::AspectContract,
    projection_mask:
        worth_foundational::facade::AspectMask<worth_foundational::facade::ProjectionMask>,
}

#[derive(Clone)]
pub(super) struct BridgeConditionalSemanticObservationPlan {
    reads: Vec<BridgeConditionalSemanticObservationRead>,
}

impl BridgeConditionalSemanticObservationPlan {
    pub(super) fn retained_heap_bytes(
        &self,
    ) -> Result<u64, super::retention::BridgeRetentionDenial> {
        super::retention::array_charge::<BridgeConditionalSemanticObservationRead>(
            self.reads.capacity(),
        )
    }

    #[cfg(test)]
    pub(super) fn managed_test_plan() -> Self {
        let contract = crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("balance").unwrap(),
            worth_foundational::facade::ScalarAspectType::UInt64,
        );
        Self::managed_test_plan_for(
            contract.aspect_contract().clone(),
            worth_foundational::facade::AspectMask::new([
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("available").unwrap(),
                ),
            ]),
        )
    }

    #[cfg(test)]
    pub(super) fn managed_test_plan_for(
        contract: worth_foundational::facade::AspectContract,
        projection_mask: worth_foundational::facade::AspectMask<
            worth_foundational::facade::ProjectionMask,
        >,
    ) -> Self {
        Self {
            reads: vec![BridgeConditionalSemanticObservationRead {
                ordinal: 0,
                record: None,
                managed_record: true,
                contract,
                projection_mask,
            }],
        }
    }

    pub(super) fn ordinals(&self) -> impl Iterator<Item = usize> + '_ {
        self.reads.iter().map(|read| read.ordinal)
    }

    pub(super) fn projection_mask(
        &self,
        ordinal: usize,
    ) -> Option<&worth_foundational::facade::AspectMask<worth_foundational::facade::ProjectionMask>>
    {
        self.reads
            .iter()
            .find(|read| read.ordinal == ordinal)
            .map(|read| &read.projection_mask)
    }

    pub(super) fn packet(
        &self,
        managed_record: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    ) -> Result<crate::snapshot::SnapshotReadPacket, BridgeConditionalDenial> {
        let reads = self
            .reads
            .iter()
            .map(|read| read.snapshot_request(managed_record))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(crate::snapshot::SnapshotReadPacket::new(reads))
    }
}

impl BridgeConditionalSemanticObservationRead {
    fn snapshot_request(
        &self,
        managed_record: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    ) -> Result<crate::snapshot::SnapshotReadRequest, BridgeConditionalDenial> {
        let record = self
            .record
            .or(managed_record.filter(|_| self.managed_record))
            .ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SnapshotAdmission,
                    "semantic condition execution lacked its exact source-record binding",
                )
            })?;
        Ok(
            crate::snapshot::SnapshotReadRequest::from_native_subscription_slice_relational_record(
                record.bridge_entity_identity(),
                record,
                crate::snapshot::SnapshotReadContract::new(self.contract.clone()),
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    self.contract.key().clone(),
                ),
                None,
                self.projection_mask.clone(),
                crate::mapping::SubscriptionSliceKind::SignalField,
            )
            .with_correlation_discriminator(format!("semantic-dependency:{}", self.ordinal)),
        )
    }
}

pub(super) fn compile_semantic_observation_plan(
    contract: &super::BridgeConditionalContract,
    registrations: &[crate::correspondence::BridgeSemanticCorrespondenceRegistration],
) -> Result<Option<BridgeConditionalSemanticObservationPlan>, BridgeConditionalDenial> {
    compile_semantic_observation_plan_from_dependencies(
        contract,
        registrations
            .iter()
            .map(crate::correspondence::BridgeSemanticCorrespondenceRegistration::dependency),
    )
}

pub(super) fn compile_semantic_observation_plan_from_dependencies<'a>(
    contract: &super::BridgeConditionalContract,
    dependencies: impl Iterator<Item = &'a crate::correspondence::BridgeSemanticDependencyCandidate>,
) -> Result<Option<BridgeConditionalSemanticObservationPlan>, BridgeConditionalDenial> {
    if !matches!(
        contract.condition(),
        super::BridgeConditionalCondition::DeltaThreshold(_)
            | super::BridgeConditionalCondition::RuntimePredicate
            | super::BridgeConditionalCondition::TemporalWake
    ) {
        return Ok(None);
    }
    let condition_dependencies = contract.condition_dependency_ordinals();
    let reads = dependencies
        .filter(|dependency| {
            condition_dependencies.is_empty()
                || condition_dependencies
                    .iter()
                    .any(|ordinal| dependency.dependency_ordinal() == *ordinal)
        })
        .map(|dependency| compile_read(dependency, contract.condition()))
        .collect::<Result<Vec<_>, _>>()?;
    if reads.is_empty() {
        return Err(BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::SnapshotAdmission,
            "semantic condition lowering produced no authoritative observation reads",
        ));
    }
    Ok(Some(BridgeConditionalSemanticObservationPlan { reads }))
}

fn compile_read(
    dependency: &crate::correspondence::BridgeSemanticDependencyCandidate,
    condition: &super::BridgeConditionalCondition,
) -> Result<BridgeConditionalSemanticObservationRead, BridgeConditionalDenial> {
    let managed_record = matches!(condition, super::BridgeConditionalCondition::TemporalWake)
        || matches!(
            dependency.locality(),
            crate::correspondence::BridgeSemanticLocality::ManagedSourceRecord
        );
    if dependency.observation_record_identity().is_none() && !managed_record {
        return Err(BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::SnapshotAdmission,
            "semantic condition observations require an exact source-record dependency",
        ));
    }
    Ok(BridgeConditionalSemanticObservationRead {
        ordinal: dependency.dependency_ordinal(),
        record: dependency.observation_record_identity(),
        managed_record,
        contract: dependency.contract().clone(),
        projection_mask: dependency.projection_mask().clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn managed_plan() -> BridgeConditionalSemanticObservationPlan {
        BridgeConditionalSemanticObservationPlan::managed_test_plan()
    }

    #[test]
    fn managed_temporal_plan_binds_each_exact_source_record_at_execution() {
        let plan = managed_plan();
        let first =
            crate::relational_identity::RelationalBridgeRecordIdentityParts::entity(1, 7, 2);
        let second =
            crate::relational_identity::RelationalBridgeRecordIdentityParts::entity(1, 9, 3);

        let first_packet = plan.packet(Some(first)).unwrap();
        let second_packet = plan.packet(Some(second)).unwrap();

        assert_eq!(
            first_packet.reads()[0].relational_record_identity_parts(),
            Some(first)
        );
        assert_eq!(
            second_packet.reads()[0].relational_record_identity_parts(),
            Some(second)
        );
    }

    #[test]
    fn managed_temporal_plan_fails_closed_without_retained_record_authority() {
        let denial = managed_plan().packet(None).unwrap_err();

        assert_eq!(
            denial.kind(),
            BridgeConditionalDenialKind::SnapshotAdmission
        );
    }

    #[test]
    fn managed_temporal_plan_always_reads_current_authoritative_posture() {
        let record =
            crate::relational_identity::RelationalBridgeRecordIdentityParts::entity(1, 7, 2);
        let packet = managed_plan().packet(Some(record)).unwrap();

        assert_eq!(packet.reads().len(), 1);
    }

    #[test]
    fn semantic_dependency_ordinals_disambiguate_shared_record_observations() {
        let contract = crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("balance").unwrap(),
            worth_foundational::facade::ScalarAspectType::UInt64,
        );
        let projection_mask = worth_foundational::facade::AspectMask::new([
            worth_foundational::facade::CanonicalFieldPath::single(
                worth_foundational::facade::FieldKey::new("available").unwrap(),
            ),
        ]);
        let plan = BridgeConditionalSemanticObservationPlan {
            reads: [0, 1]
                .map(|ordinal| BridgeConditionalSemanticObservationRead {
                    ordinal,
                    record: None,
                    managed_record: true,
                    contract: contract.aspect_contract().clone(),
                    projection_mask: projection_mask.clone(),
                })
                .to_vec(),
        };
        let record =
            crate::relational_identity::RelationalBridgeRecordIdentityParts::entity(1, 7, 2);

        let packet = plan.packet(Some(record)).unwrap();

        assert_eq!(packet.reads().len(), 2);
        assert_ne!(
            packet.reads()[0].correlation_id(),
            packet.reads()[1].correlation_id()
        );
    }
}
