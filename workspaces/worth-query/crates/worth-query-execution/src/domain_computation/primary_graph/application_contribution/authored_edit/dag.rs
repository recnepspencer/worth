//! Installed declarations decide order; source IDs never prove independence.
use super::*;
mod cycle;
mod independent_prefix;
use worth_query_installation::facade::{
    WorthQueryOperationGraphReadScope, WorthQueryOperationReadTouchOverlapIndex,
    WorthQueryOperationTouchScope,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredEditStage {
    pub identity: ApplicationEditStageIdentity,
    pub binding: &'static str,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclaredEditParticipant {
    Stage(DeclaredEditStage),
    Producer {
        family: String,
        implementations: Vec<String>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclaredEditReadScope {
    Graph(WorthQueryOperationGraphReadScope),
    Query(DeclaredQueryRead),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredEditDependency {
    pub writer: DeclaredEditParticipant,
    pub reader: DeclaredEditParticipant,
    pub read_scope: DeclaredEditReadScope,
    pub write_scope: WorthQueryOperationTouchScope,
}
/// A declaration cycle is distinct from Native's actual record conflict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredEditScopeCycle {
    pub stages: Vec<DeclaredEditStage>,
    pub overlaps: Vec<DeclaredEditDependency>,
}
#[derive(Debug)]
pub enum DeclaredEditOrderRefusal {
    Cycle(DeclaredEditScopeCycle),
    Planning(WorthQueryOutputDemandDenial),
    Execution(WorthQueryApplicationAttemptDenial),
    /// This entry requires a genuine authored prefix before any producer wave.
    StageAfterProducer(DeclaredEditDependency),
}
struct Node<'a> {
    subject: DeclaredEditParticipant,
    contracts: Vec<&'a WorthQueryOperationReadTouchOverlapIndex>,
    queries: Vec<&'a DeclaredQueryRead>,
}
fn overlap(
    reader: &Node<'_>,
    writer: &Node<'_>,
    schema: &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
) -> Option<DeclaredEditDependency> {
    for write in writer
        .contracts
        .iter()
        .flat_map(|contract| contract.touches())
    {
        for contract in &reader.contracts {
            for read in contract.reads().iter() {
                if contract.may_intersect(read, write) {
                    return Some(DeclaredEditDependency {
                        writer: writer.subject.clone(),
                        reader: reader.subject.clone(),
                        read_scope: DeclaredEditReadScope::Graph(read.clone()),
                        write_scope: write.clone(),
                    });
                }
            }
        }
        for query in &reader.queries {
            if reader.contracts[0].query_intersects(schema, &query.contract, write) {
                return Some(DeclaredEditDependency {
                    writer: writer.subject.clone(),
                    reader: reader.subject.clone(),
                    read_scope: DeclaredEditReadScope::Query((*query).clone()),
                    write_scope: write.clone(),
                });
            }
        }
    }
    None
}
pub fn order_authored_stages<S>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stages: &mut [CapturedAuthoredEditStage<S>],
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    unseen: &UnseenAuthoredEnvelope<'_>,
) -> Result<Vec<usize>, DeclaredEditOrderRefusal>
where
    S: worth_query_installation::facade::ApplicationSchema,
{
    let Some(first) = stages.first() else {
        return Ok(Vec::new());
    };
    let mut nodes = stages
        .iter()
        .map(|stage| Node {
            subject: DeclaredEditParticipant::Stage(DeclaredEditStage {
                identity: stage.identity,
                binding: stage.binding,
            }),
            contracts: vec![&stage.contracts],
            queries: stage.queries.iter().collect(),
        })
        .collect::<Vec<_>>();
    for producer in stages.iter().flat_map(|stage| &stage.producers) {
        checkpoint(execution)?;
        if let Some(node) = nodes.iter_mut().find(|node| matches!(&node.subject, DeclaredEditParticipant::Producer { family, .. } if family == producer.family)) {
            if let DeclaredEditParticipant::Producer { implementations, .. } = &mut node.subject
                && !implementations.contains(&producer.identity) {
                implementations.push(producer.identity.clone()); node.contracts.push(&producer.contract); node.queries.extend(producer.queries.iter());
            }
        } else {
            nodes.push(Node { subject: DeclaredEditParticipant::Producer { family: producer.family.into(), implementations: vec![producer.identity.clone()] }, contracts: vec![&producer.contract], queries: producer.queries.iter().collect() });
        }
    }
    let mut edges = Vec::new();
    let mut incoming = vec![0usize; nodes.len()];
    for (writer, writes) in nodes.iter().enumerate() {
        for (reader, reads) in nodes.iter().enumerate() {
            checkpoint(execution)?;
            if writer == reader {
                continue;
            }
            if let Some(dependency) = overlap(reads, writes, &first.schema) {
                edges.push((writer, reader, Some(dependency)));
                incoming[reader] += 1;
            }
        }
    }
    for (parent, child) in stages.iter().flat_map(|stage| &stage.producer_connections) {
        let family_node = |kind: &std::any::TypeId| {
            let family = stages
                .iter()
                .flat_map(|stage| &stage.producers)
                .find(|producer| producer.family_type == *kind)?
                .family;
            nodes.iter().position(|node| {
                matches!(&node.subject,
                DeclaredEditParticipant::Producer { family: actual, .. } if actual == family)
            })
        };
        if let (Some(parent), Some(child)) = (family_node(parent), family_node(child))
            && parent != child
            && !edges
                .iter()
                .any(|(from, to, _)| *from == parent && *to == child)
        {
            checkpoint(execution)?;
            edges.push((parent, child, None));
            incoming[child] += 1;
        }
    }
    independent_prefix::refine(
        execution,
        stages,
        &nodes,
        &mut edges,
        &AuthoredEditStageContext { phase, application },
        unseen,
    )?;
    incoming.fill(0);
    for (_, reader, _) in &edges {
        incoming[*reader] += 1;
    }
    let mut order = Vec::new();
    while let Some(next) = incoming.iter().enumerate().find_map(|(ordinal, incoming)| {
        (*incoming == 0 && !order.contains(&ordinal)).then_some(ordinal)
    }) {
        checkpoint(execution)?;
        order.push(next);
        for (_, reader, _) in edges.iter().filter(|(writer, _, _)| *writer == next) {
            incoming[*reader] -= 1;
        }
    }
    if order.len() != nodes.len() {
        let cycle = cycle::find(nodes.len(), &edges);
        let overlaps = cycle
            .iter()
            .zip(cycle.iter().cycle().skip(1))
            .filter_map(|(writer, reader)| {
                edges
                    .iter()
                    .find(|(from, to, _)| from == writer && to == reader)
                    .expect("cycle follows actual declared edges")
                    .2
                    .clone()
            })
            .collect();
        return Err(DeclaredEditOrderRefusal::Cycle(DeclaredEditScopeCycle {
            stages: cycle
                .iter()
                .filter_map(|ordinal| match &nodes[*ordinal].subject {
                    DeclaredEditParticipant::Stage(stage) => Some(stage.clone()),
                    _ => None,
                })
                .collect(),
            overlaps,
        }));
    }
    if order.first().is_some_and(|first| *first >= stages.len()) {
        let dependency = edges
            .iter()
            .find_map(|(writer, reader, dependency)| {
                (*writer >= stages.len() && *reader < stages.len())
                    .then(|| dependency.clone())
                    .flatten()
            })
            .expect(
                "a producer precedes every authored stage only through a declared read dependency",
            );
        return Err(DeclaredEditOrderRefusal::StageAfterProducer(dependency));
    }
    let mut families = Vec::new();
    let mut authored = Vec::new();
    for ordinal in order {
        match &nodes[ordinal].subject {
            DeclaredEditParticipant::Stage(_) => authored.push((ordinal, families.clone())),
            DeclaredEditParticipant::Producer { family, .. } => families.push(family.clone()),
        }
    }
    drop(nodes);
    for (ordinal, barrier) in &authored {
        stages[*ordinal].producer_barrier = barrier.clone();
    }
    Ok(authored.into_iter().map(|(ordinal, _)| ordinal).collect())
}

fn checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), DeclaredEditOrderRefusal> {
    let refused = |cause: worth_execution::WorkCeilingDenial| {
        DeclaredEditOrderRefusal::Execution(
            WorthQueryApplicationAttemptDenial::relational_read_stopped(
                cause.into(),
                "edit envelope dependency admission",
            ),
        )
    };
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
        })
        .map_err(refused)?
        .0
        .map_err(|cause| refused(worth_execution::WorkCeilingDenial::Stopped(cause)))
}
