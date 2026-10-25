//! Ordinary edits of feeding edges and their endpoints, without a law checker.
use super::schema::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    worth_query_operation_creates, worth_query_operation_deletes, worth_query_operation_links,
    worth_query_operation_reads, worth_query_operation_unlinks, worth_query_operation_writes,
};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, serde::Serialize)]
pub enum FeedsChange {
    Replace { first_leaf: u64, count: u32 },
    DeleteSource,
    DeleteLeaf { key: u64 },
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct FeedsEditInput {
    pub serial: u64,
    pub context: u64,
    pub change: FeedsChange,
}
worth_query_structured_value_binding!(pub FeedsInputBinding for FeedsEditInput { identity: "worth.query.relation-laws.feeds-input.v1" });
use super::operation::{Edited, EditedBinding, GraphDenial, GraphDenialBinding};

pub struct EditFeeds;
impl<S: GraphSchemaBinding> ApplicationOperationMarkerIdentity<S> for EditFeeds {
    type InputBinding = FeedsInputBinding;
    const IDENTIFIER: &'static str = "EditFeeds";
}
impl EditFeeds {
    pub fn reference<S: GraphSchemaBinding>() -> ApplicationOperationRef<S, Self, FeedsEditInput> {
        ApplicationOperationRef::from_declaration()
    }
}
worth_query_operation_reads!(EditFeeds => [NodeKey, LeafKey, Feeds, Node, Leaf]);
worth_query_operation_links!(EditFeeds => [Feeds]);
worth_query_operation_unlinks!(EditFeeds => [Feeds]);
worth_query_operation_deletes!(EditFeeds => [Node, Leaf]);
worth_query_operation_creates!(EditFeeds => [Leaf]);
worth_query_operation_writes!(EditFeeds => [LeafKey]);

pub struct FeedsBinding<L: LawPolicy>(PhantomData<fn() -> L>);
type NodeScope<L> = ApplicationMutationFieldScope<
    GraphSchema<L>,
    Node,
    NodeFacts,
    NodeKey,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;
impl<L: LawPolicy> ApplicationMutationBinding<GraphSchema<L>> for FeedsBinding<L> {
    type Input = FeedsEditInput;
    type InputBinding = FeedsInputBinding;
    type Result = Edited;
    type ResultBinding = EditedBinding;
    type IdempotencyKey = u64;
    type Operation = EditFeeds;
    type Decision = FeedDecision<L>;
    type Denial = GraphDenial;
    type DenialBinding = GraphDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = NodeScope<L>;
    type PrincipalBinding = GraphPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.relation-laws.feeds-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.relation-laws.feeds-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.relation-laws.feeds-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        GraphSchema<L>,
        Node,
        NodeFacts,
        NodeKey,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        NodeKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        GraphSchema<L>,
        GraphPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        GraphPrincipal::reference()
    }
}
#[derive(Clone)]
pub struct FeedsEdit<L: LawPolicy> {
    pub input: FeedsEditInput,
    pub marker: PhantomData<fn() -> L>,
}
impl<L: LawPolicy> ApplicationMutationIntent<GraphSchema<L>> for FeedsEdit<L> {
    type Binding = FeedsBinding<L>;
    fn input(&self) -> &FeedsEditInput {
        &self.input
    }
    fn scope_binding(&self) -> NodeScope<L> {
        NodeScope::new(NodeKey::reference(), self.input.context)
    }
}

type NodeTarget<L> = WorthQueryInvariantMutationTarget<GraphSchema<L>, Node>;
type LeafTarget<L> = WorthQueryInvariantMutationTarget<GraphSchema<L>, Leaf>;
pub enum FeedDecision<L: LawPolicy> {
    Replace {
        source: NodeTarget<L>,
        removed: Vec<LeafTarget<L>>,
        first: u64,
        count: u32,
    },
    DeleteSource(NodeTarget<L>),
    DeleteLeaf(LeafTarget<L>),
}
pub struct FeedsHandler;
impl<L: LawPolicy> OperationHandler<GraphSchema<L>, FeedsBinding<L>> for FeedsHandler {
    fn decide(
        &self,
        input: &FeedsEditInput,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, FeedsBinding<L>>,
    ) -> HandlerResult<FeedDecision<L>, GraphDenial> {
        match decide_feeds(input, reader) {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _: &FeedsEditInput,
        decision: FeedDecision<L>,
        writer: &mut CandidateWriter<'_, GraphSchema<L>, FeedsBinding<L>>,
    ) -> HandlerResult<Edited, GraphDenial> {
        match write_feeds(decision, writer) {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
fn decide_feeds<L: LawPolicy>(
    input: &FeedsEditInput,
    reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, FeedsBinding<L>>,
) -> Result<FeedDecision<L>, HandlerExecutionDenial> {
    let source = reader.resolve_entity(NodeKey::reference(), input.context)?;
    Ok(match input.change {
        FeedsChange::DeleteSource => {
            reader.entity(&source, Node::reference())?;
            FeedDecision::DeleteSource(reader.mutation_target(&source)?)
        }
        FeedsChange::DeleteLeaf { key } => {
            let leaf = reader.resolve_entity(LeafKey::reference(), key)?;
            reader.entity(&leaf, Leaf::reference())?;
            FeedDecision::DeleteLeaf(reader.mutation_target(&leaf)?)
        }
        FeedsChange::Replace { first_leaf, count } => {
            let observed = reader
                .relations_from(Feeds::reference(), &source)
                .map_err(HandlerExecutionDenial::new)?;
            let mut removed = Vec::new();
            let mut commands = std::collections::BTreeSet::new();
            for edge in observed {
                if commands.insert(edge.to().entity_id()) {
                    removed.push(reader.mutation_target(edge.to())?);
                }
            }
            FeedDecision::Replace {
                source: reader.mutation_target(&source)?,
                removed,
                first: first_leaf,
                count,
            }
        }
    })
}
fn write_feeds<L: LawPolicy>(
    decision: FeedDecision<L>,
    writer: &mut CandidateWriter<'_, GraphSchema<L>, FeedsBinding<L>>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    match decision {
        FeedDecision::DeleteSource(source) => {
            let source = writer.projected_entity(&source)?;
            writer.delete_entity(Node::reference(), &source)?;
        }
        FeedDecision::DeleteLeaf(leaf) => {
            let leaf = writer.projected_entity(&leaf)?;
            writer.delete_entity(Leaf::reference(), &leaf)?;
        }
        FeedDecision::Replace {
            source,
            removed,
            first,
            count,
        } => {
            let source = writer.projected_entity(&source)?;
            for leaf in removed {
                let leaf = writer.projected_entity(&leaf)?;
                writer.unlink(Feeds::reference(), &source, &leaf)?;
            }
            for offset in 0..count {
                let key = first + u64::from(offset);
                let leaf = writer.create_entity_in_context(
                    &source,
                    Leaf::reference(),
                    WorthQueryApplicationEntityKey::new(format!("leaf-{key}")).unwrap(),
                )?;
                writer.initialize_field(&leaf, LeafKey::reference(), key)?;
                writer.link(Feeds::reference(), format!("feed-{key}"), &source, &leaf)?;
            }
        }
    }
    Ok(())
}
