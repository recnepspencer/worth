//! The generated producer's partitioned computation: its membership is the
//! declared Parent probed through the lent computation reader, at its own or
//! a peer Directory — the `ContactOwner::partitions` shape. It is installed on
//! the root Catalog feature beside the operation computation, leaving the
//! producer's own output feature undecorated.
use super::super::super::host::Feature;
use super::super::super::peer_scope::{self, ProbeRefusal, OBSERVED, PRODUCER_PROBE};
use super::super::super::{query, schema::*};
use super::{GenerateItem, ResultBinding};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_contribution::*;

#[derive(serde::Serialize)]
pub struct Request {
    pub directory: String,
}
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = Request;
    const IDENTITY: &'static str = "maintained-generated.peer-probe.input.v1";
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Row {
    pub identity: String,
}
impl ChargedBytes for Row {
    fn additional_charged_bytes(&self) -> u64 {
        self.identity.capacity() as u64
    }
}
impl ApplicationComputationPartition for Row {
    const IDENTITY: &'static str = "maintained-generated.peer-probe.row.v1";
}
pub struct Port;
impl ApplicationOutputPort<Catalog, Feature> for Port {
    type Value = ResultBinding;
    const IDENTITY: &'static str = "maintained-generated.peer-probe.port.v1";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "maintained-generated.peer-probe.root.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Catalog, Feature> for Artifact {
    type Output = Port;
    type Locality = Locality;
    const IDENTITY: &'static str = "maintained-generated.peer-probe.artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str = "maintained-generated.peer-probe";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "exact-parent";
    const STOPPED_OUTCOME: &'static str = "refused";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "maintained-generated.peer-probe.reuse.v1";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "maintained-generated.peer-probe.stop.v1";
}
pub struct Computation;
impl ApplicationManagedComputation<Catalog, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Row;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "maintained-generated.peer-probe.computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
/// The owner never refuses on its own; every stop is a read denial.
#[derive(Debug)]
pub enum Refused {}
pub struct Owner;
pub type Installed =
    WorthQueryInstalledPartitionedComputation<Catalog, Feature, Computation, Owner>;
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, Catalog, GenerateItem>;
type InputDenial = WorthQueryComputationInputDenial<Refused>;
fn identity(row: &Row) -> PartitionItemId {
    PartitionItemId(
        row.identity
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
            }),
    )
}
impl WorthQueryPartitionedComputationOwner<Catalog, Feature, Computation> for Owner {
    type Operation = GenerateItem;
    type Item = Row;
    type Gathered = Row;
    type PartitionResult = Vec<u8>;
    type Output = Vec<String>;
    type Stopped = Refused;
    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &Request,
    ) -> Result<WorthQueryComputationPartitionPlan<Row>, InputDenial> {
        use peer_scope::ProducerProbe;
        let probe = PRODUCER_PROBE.get().unwrap_or(ProducerProbe::Own);
        let directory = peer_scope::probed(probe, &input.directory);
        let rows = match probe {
            ProducerProbe::OrderedOwn => reader.index_query::<
                super::super::super::ordered_decision::ParentBinding,
            >(super::super::super::ordered_decision::Find { directory, prefix: String::new() }),
            ProducerProbe::Undeclared => reader.index_query::<query::PrefixAliasBinding>(
                crate::application_invariant_acceptance::managed_index::prefix_alias_input::FindPrefixAlias { directory, prefix: String::new() },
            ),
            _ => reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                directory, prefix: String::new(),
            }),
        };
        let observed = match &rows {
            Ok(rows) => Ok(rows.iter().map(|row| row.identity.clone()).collect()),
            Err(WorthQueryComputationReadDenial::ManagedIndex(denial)) if denial.decision_plan_denial().is_some_and(|cause| cause.kind() ==
                worth_query_host::facade::primary_graph::WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget) => Err(ProbeRefusal::Undeclared),
            Err(WorthQueryComputationReadDenial::ManagedIndex(denial)) => Err(denial
                .managed_index_denial()
                .map_or(ProbeRefusal::Other, |denial| peer_scope::refusal(denial.cause()))),
            Err(_) => Err(ProbeRefusal::Other),
        };
        OBSERVED.with(|all| all.borrow_mut().push(observed));
        let rows = rows?.into_iter().map(|row| Row {
            identity: row.identity,
        });
        Ok(WorthQueryComputationPartitionPlan::keyed(rows, identity))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Request,
        item: &Row,
    ) -> Result<Row, InputDenial> {
        Ok(item.clone())
    }
    fn gather(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Request,
        partition: WorthQueryComputationPartitionMembers<'_, Row, Row>,
    ) -> Result<Row, InputDenial> {
        Ok(partition.key().clone())
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Row, Row>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Vec<u8>, WorthQueryManagedComputationDenial<Refused>> {
        checkpoint.poll()?;
        Ok(format!("{};", partition.gathered().identity).into_bytes())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<Vec<u8>> {
        WorthQueryDeterministicReducer::canonical(Vec::new, |a, b| {
            a.iter().chain(b).copied().collect()
        })
    }
    fn complete(&self, reduced: Vec<u8>) -> Result<Self::Output, Refused> {
        let reduced = String::from_utf8(reduced).expect("authored identities are text");
        let mut rows: Vec<_> = reduced.split_terminator(';').map(str::to_owned).collect();
        rows.sort();
        Ok(rows)
    }
}
