//! Controlled finite operations consumed by the Query-free AVL mechanism.
use super::*;
use std::sync::Mutex;
use worth_query_consumer_values::{compare_prefix_keys, PrefixBuildStorage};

pub(super) struct Build<
    'call,
    'source,
    'node,
    'control,
    S: ApplicationSchema,
    I: PrefixSourceIndex<S>,
> {
    pub sources: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
    pub nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
    pub after_first_node: &'call Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
impl<S: ApplicationSchema, I: PrefixSourceIndex<S>> Build<'_, '_, '_, '_, S, I> {
    fn observed_write(&self) {
        let observe = self.after_first_node.lock().unwrap().take();
        if let Some(observe) = observe {
            observe();
        }
    }
}
fn invalid() -> Stop {
    Stop::Domain(PrefixIndexDenial::InvalidTopology)
}
impl<S: ApplicationSchema, I: PrefixSourceIndex<S>> PrefixBuildStorage
    for Build<'_, '_, '_, '_, S, I>
{
    type Id = WorthQueryManagedIndexNodeId;
    type Source = WorthQueryManagedIndexSourceSlot;
    type Stop = Stop;
    fn checkpoint(&mut self) -> Result<(), Stop> {
        self.control.checkpoint(1).map_err(Into::into)
    }
    fn compare_sources(
        &mut self,
        left: Self::Source,
        right: Self::Source,
    ) -> Result<std::cmp::Ordering, Stop> {
        let a = self.sources.get(left)?;
        let b = self.sources.get(right)?;
        I::with_name(&a, |a| {
            I::with_name(&b, |b| {
                compare_prefix_keys(a, 0, b, 0, |units| {
                    self.control.checkpoint(units).map_err(Into::into)
                })
                .map(|order| order.then_with(|| left.cmp(&right)))
            })
            .map_err(text_denial)?
        })
        .map_err(text_denial)?
    }
    fn append(&mut self, node: Node) -> Result<Self::Id, Stop> {
        let id = self.nodes.push(node)?;
        self.observed_write();
        Ok(id)
    }
    fn read(&mut self, id: Self::Id) -> Result<Node, Stop> {
        self.nodes.get(id).map_err(Into::into)
    }
    fn replace(&mut self, id: Self::Id, node: Node) -> Result<(), Stop> {
        self.nodes.replace(id, node)?;
        self.observed_write();
        Ok(())
    }
    fn remove(&mut self, id: Self::Id) -> Result<(), Stop> {
        self.nodes.remove(id)?;
        self.observed_write();
        Ok(())
    }
    fn invalid(&self) -> Stop {
        invalid()
    }
}
