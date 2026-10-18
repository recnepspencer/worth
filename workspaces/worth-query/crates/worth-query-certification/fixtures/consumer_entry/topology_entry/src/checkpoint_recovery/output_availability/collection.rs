//! One checked owner, one complete population, pure count/available-value reduction.
use super::*;
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, Schema, binding::PublishQuantity>;
type InputDenial = WorthQueryComputationInputDenial<u32>;
pub(super) struct PopulationInput;
impl ApplicationComputationInput for PopulationInput {
    type Value = WorthQueryInvariantEntityIdentity<Schema, Body>;
    const IDENTITY: &'static str = "availability-population";
}
#[derive(serde::Serialize)]
pub(super) struct TotalKey(u32);
impl ApplicationComputationPartition for TotalKey {
    const IDENTITY: &'static str = "availability-total-group";
}
pub(super) struct NoWarmStart;
impl ApplicationComputationReuse for NoWarmStart {
    const IDENTITY: &'static str = "availability-no-approximate-reuse";
}
pub(super) struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "availability-owner-refusal";
}
pub(super) struct Totals;
impl ApplicationManagedComputation<Schema, program::QuantityFeature> for Totals {
    type Input = PopulationInput;
    type Output = program::QuantityArtifact;
    type Partition = TotalKey;
    type Reuse = NoWarmStart;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "availability-count-and-observed-total";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    // This bounds one pure gathered/reduced snapshot, not the producer's
    // native source/query custody. Each fixture population has one member.
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(1_000_000, 16 * 1024);
}
pub(super) struct Contributors;
impl ApplicationCollectionContributor for Contributors {
    const IDENTITY: &'static str = "availability-native-members";
}
pub(super) struct Groups;
impl ApplicationCollectionGrouping for Groups {
    const IDENTITY: &'static str = "availability-one-population-total";
}
pub(super) struct Measures;
impl ApplicationCollectionMeasures for Measures {
    const IDENTITY: &'static str = "availability-count-and-total";
}
pub(super) struct Lineage;
impl ApplicationCollectionLineage for Lineage {
    const IDENTITY: &'static str = "availability-original-native-source-and-output";
}
pub(super) struct Complete;
impl ApplicationCollectionCompleteness for Complete {
    type Value = Snapshot;
    type Stopped = u32;
    type Lineage = Lineage;
    const IDENTITY: &'static str = "availability-per-measure-population-completeness";
    fn check(value: &Snapshot) -> Result<(), u32> {
        if value.count != value.contributors.len() as u64 {
            return Err(91);
        }
        let mut total = Some(0u64);
        for member in &value.contributors {
            match member.measure {
                Measure::Complete(part) if member.output.is_some() => {
                    total = match total {
                        Some(sum) => Some(sum.checked_add(part).ok_or(92u32)?),
                        None => None,
                    };
                }
                Measure::Pending => total = None,
                _ => return Err(92),
            }
        }
        match (&value.measure, total) {
            (Measure::Complete(actual), Some(expected)) if *actual == expected => Ok(()),
            (Measure::Pending, None) => Ok(()),
            _ => Err(93),
        }
    }
}
pub(super) struct Quantities;
impl ApplicationDerivedCollection<Schema, program::QuantityFeature> for Quantities {
    type Computation = Totals;
    type Completeness = Complete;
    type Contributor = Contributors;
    type Grouping = Groups;
    type Measures = Measures;
    type Lineage = Lineage;
    const IDENTITY: &'static str = "availability-installed-quantities";
    const APPLICABILITY: &'static str = "ordinary-native-population";
    const INCOMPLETE: ApplicationCollectionIncompletePosture =
        ApplicationCollectionIncompletePosture::RetainPending;
    const INCREMENTAL_UPDATE: &'static str = "native-full-validation-when-carriage-unsupported";
}
pub(super) struct Owner {
    pub(super) enrollment: Arc<installation::EnrollmentWindow>,
}
impl WorthQueryPartitionedComputationOwner<Schema, program::QuantityFeature, Totals> for Owner {
    type Operation = binding::PublishQuantity;
    type Item = members::Member;
    type Gathered = Vec<Snapshot>;
    type PartitionResult = Snapshot;
    type Output = Snapshot;
    type Stopped = u32;
    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        root: &WorthQueryInvariantEntityIdentity<Schema, Body>,
    ) -> Result<WorthQueryComputationPartitionPlan<Self::Item>, InputDenial> {
        let mut items = Vec::new();
        for edge in reader.relations_from(members::PopulationMember::reference(), root)? {
            let entity = edge.into_to();
            let number = reader
                .field(&entity, members::MemberNumber::reference())?
                .ok_or(WorthQueryComputationInputDenial::Owner(1))?;
            items.push(members::Member { number, entity });
        }
        Ok(WorthQueryComputationPartitionPlan::keyed(items, |member| {
            PartitionItemId(member.number)
        }))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &WorthQueryInvariantEntityIdentity<Schema, Body>,
        _: &Self::Item,
    ) -> Result<TotalKey, InputDenial> {
        Ok(TotalKey(0))
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        _: &WorthQueryInvariantEntityIdentity<Schema, Body>,
        partition: WorthQueryComputationPartitionMembers<'_, TotalKey, Self::Item>,
    ) -> Result<Vec<Snapshot>, InputDenial> {
        let mut values = Vec::new();
        for (_, member) in partition.items() {
            let key = reader
                .field(&member.entity, members::DemandKey::reference())?
                .ok_or(WorthQueryComputationInputDenial::Owner(2))?;
            let kind = reader
                .field(&member.entity, members::MemberKind::reference())?
                .ok_or(WorthQueryComputationInputDenial::Owner(4))?;
            let (measure, output) = match kind {
                members::Kind::Geometry => {
                    let observed =
                        reader.output_availability(&member.entity, PlanarOutputDemand::new(key))?;
                    match observed.state() {
                        WorthQueryOutputAvailabilityState::Pending => (Measure::Pending, None),
                        WorthQueryOutputAvailabilityState::Current(identity) => {
                            let value = reader
                                .field(identity, Length::reference())?
                                .ok_or(WorthQueryComputationInputDenial::Owner(3))?;
                            (
                                Measure::Complete(
                                    worth_query_consumer_values::PositiveLength::get(&value),
                                ),
                                Some(members::descriptor(identity)),
                            )
                        }
                    }
                }
                members::Kind::Quantity => {
                    let observed =
                        reader.output_availability(&member.entity, producer::Demand(key))?;
                    match observed.state() {
                        WorthQueryOutputAvailabilityState::Pending => (Measure::Pending, None),
                        WorthQueryOutputAvailabilityState::Current(identity) => {
                            let value = reader
                                .field(identity, members::QuantitySnapshot::reference())?
                                .ok_or(WorthQueryComputationInputDenial::Owner(5))?;
                            (value.measure, Some(members::descriptor(identity)))
                        }
                    }
                }
            };
            values.push(Snapshot {
                count: 1,
                measure: measure.clone(),
                contributors: vec![Contributor {
                    kind,
                    measure,
                    source: members::descriptor(&member.entity),
                    output,
                }],
            });
        }
        Ok(values)
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, TotalKey, Vec<Snapshot>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Snapshot, WorthQueryManagedComputationDenial<u32>> {
        // PreparedCompute dispatches this reader-free kernel only after the
        // projection loan returns. Native enrollment is already gathered;
        // commit preparation/idempotency admission has not begun.
        self.enrollment.before_compute();
        let mut total = Snapshot::empty();
        for value in partition.gathered() {
            checkpoint.advance(1)?;
            total = combine(total, value.clone());
        }
        Ok(total)
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<Snapshot> {
        WorthQueryDeterministicReducer::canonical(Snapshot::empty, |left, right| {
            combine(left.clone(), right.clone())
        })
    }
    fn complete(&self, value: Snapshot) -> Result<Snapshot, u32> {
        Complete::check(&value)?;
        Ok(value)
    }
}
fn combine(mut left: Snapshot, right: Snapshot) -> Snapshot {
    left.count = left
        .count
        .checked_add(right.count)
        .expect("fixture population fits u64");
    left.measure = match (left.measure, right.measure) {
        (Measure::Complete(a), Measure::Complete(b)) => {
            Measure::Complete(a.checked_add(b).expect("fixture total fits u64"))
        }
        _ => Measure::Pending,
    };
    left.contributors.extend(right.contributors);
    left
}
