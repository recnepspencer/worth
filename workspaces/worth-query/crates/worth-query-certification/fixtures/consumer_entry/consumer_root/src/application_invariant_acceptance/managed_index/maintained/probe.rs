//! A partitioned computation whose membership is the declared managed Parent,
//! read through the computation reader's issued decision index read.
use super::{host::Feature, mutation, query, schema::*};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_contribution::*;

std::thread_local! {
    /// The completed rows of each run on this request thread, for the court
    /// to compare with its independent authored universe.
    pub static COMPLETED: std::cell::RefCell<Vec<Vec<(String, String)>>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// Each computation managed index refusal the handler forwarded.
    pub static REFUSED: std::cell::RefCell<Vec<WorthQueryComputationManagedIndexDenial>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// Runs once at the first gather, after the membership read completed.
    #[allow(clippy::type_complexity)]
    pub static AFTER_MEMBERSHIP: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        const { std::cell::RefCell::new(None) };
}
/// Which Parent read the owner's membership call makes.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub enum Probe {
    /// The declared Parent in the operation's own scope.
    Declared,
    /// The declared Parent in the operation's own scope, with another prefix.
    Prefix(&'static str),
    /// The declared Parent resolved to another Directory than the operation's.
    OtherScope,
    /// A Parent the operation never declared with `operation_managed_index_read`.
    Undeclared,
}
#[derive(serde::Serialize)]
pub struct ProbeRequest {
    pub directory: String,
    pub prefix: String,
    pub probe: Probe,
}
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = ProbeRequest;
    const IDENTITY: &'static str = "managed-catalog.probe.input.v1";
}
/// One selected Parent row: an item, and the partition it alone belongs to.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Row {
    pub identity: String,
    pub name: String,
}
impl ChargedBytes for Row {
    fn additional_charged_bytes(&self) -> u64 {
        (self.identity.capacity() + self.name.capacity()) as u64
    }
}
impl ApplicationComputationPartition for Row {
    const IDENTITY: &'static str = "managed-catalog.probe.row.v1";
}
pub struct Port;
impl ApplicationOutputPort<Catalog, Feature> for Port {
    type Value = mutation::ResultBinding;
    const IDENTITY: &'static str = "managed-catalog.probe.port.v1";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "managed-catalog.probe.root.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Catalog, Feature> for Artifact {
    type Output = Port;
    type Locality = Locality;
    const IDENTITY: &'static str = "managed-catalog.probe.artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str = "managed-catalog.probe";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "exact-parent";
    const STOPPED_OUTCOME: &'static str = "refused";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "managed-catalog.probe.reuse.v1";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "managed-catalog.probe.stop.v1";
}
pub struct Computation;
impl ApplicationManagedComputation<Catalog, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Row;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "managed-catalog.probe.computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
/// The owner never refuses on its own; every stop is a read denial.
#[derive(Debug)]
pub enum Refused {}
pub struct Owner;
pub type Installed =
    WorthQueryInstalledPartitionedComputation<Catalog, Feature, Computation, Owner>;
pub type Denial = WorthQueryPartitionedComputationDenial<Refused>;
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, Catalog, mutation::RenameItem>;
type InputDenial = WorthQueryComputationInputDenial<Refused>;
/// A stable identity of the row itself, never its position in the Parent.
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
    type Operation = mutation::RenameItem;
    type Item = Row;
    type Gathered = Row;
    type PartitionResult = Vec<u8>;
    type Output = Vec<(String, String)>;
    type Stopped = Refused;
    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &ProbeRequest,
    ) -> Result<WorthQueryComputationPartitionPlan<Row>, InputDenial> {
        let rows = match input.probe {
            Probe::Declared => reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                directory: input.directory.clone(),
                prefix: input.prefix.clone(),
            }),
            Probe::Prefix(prefix) => {
                reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                    directory: input.directory.clone(),
                    prefix: prefix.into(),
                })
            }
            Probe::OtherScope => reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                directory: "B".into(),
                prefix: input.prefix.clone(),
            }),
            Probe::Undeclared => reader.index_query::<query::SourceBinding>(query::ReadSource {
                directory: input.directory.clone(),
            }),
        }?;
        let rows = rows.into_iter().map(|row| Row {
            identity: row.identity,
            name: row.name,
        });
        Ok(WorthQueryComputationPartitionPlan::keyed(rows, identity))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &ProbeRequest,
        item: &Row,
    ) -> Result<Row, InputDenial> {
        Ok(item.clone())
    }
    fn gather(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &ProbeRequest,
        partition: WorthQueryComputationPartitionMembers<'_, Row, Row>,
    ) -> Result<Row, InputDenial> {
        if let Some(hold) = AFTER_MEMBERSHIP.with(|hold| hold.borrow_mut().take()) {
            hold();
        }
        Ok(partition.key().clone())
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Row, Row>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Vec<u8>, WorthQueryManagedComputationDenial<Refused>> {
        checkpoint.poll()?;
        let row = partition.gathered();
        Ok(format!("{}={};", row.identity, row.name).into_bytes())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<Vec<u8>> {
        WorthQueryDeterministicReducer::canonical(Vec::new, |a, b| {
            a.iter().chain(b).copied().collect()
        })
    }
    fn complete(&self, reduced: Vec<u8>) -> Result<Self::Output, Refused> {
        let reduced = String::from_utf8(reduced).expect("authored rows are text");
        let mut rows: Vec<_> = reduced
            .split_terminator(';')
            .map(|row| {
                let (identity, name) = row.split_once('=').expect("identity=name");
                (identity.to_owned(), name.to_owned())
            })
            .collect();
        rows.sort();
        COMPLETED.with(|completed| completed.borrow_mut().push(rows.clone()));
        Ok(rows)
    }
}
