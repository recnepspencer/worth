//! Mutable page custody exists only within one actual predecessor writer.
use super::*;
use crate::domain_computation::primary_graph::access_strategy::ordered_storage as maps;
use std::{cell::RefCell, rc::Rc};
use worth_execution::ExecutionOrderedMap;
mod seal;

pub(in super::super::super) struct PageEdit<'a, Node> {
    base: NodePages<Node>,
    execution: ExecutionRequest<'a, 'a>,
    touched: ExecutionOrderedMap<usize, Rc<WorkingPage<'a, Node>>>,
    sealed_branches: Rc<Cell<usize>>,
}
struct WorkingPage<'a, Node> {
    open: RefCell<OpenPage<'a, Node>>,
    _memory: Option<ExecutionMemoryReservation>,
}
struct OpenPage<'a, Node> {
    builder: Option<ExecutionRequestArrayBuilder<'a, 'a, Option<Node>>>,
    occupied: usize,
}
impl<'a, Node: Copy> PageEdit<'a, Node> {
    pub(in super::super::super) fn new(
        base: NodePages<Node>,
        execution: ExecutionRequest<'a, 'a>,
    ) -> Self {
        Self {
            base,
            execution,
            touched: ExecutionOrderedMap::default(),
            sealed_branches: Rc::new(Cell::new(0)),
        }
    }
    pub(in super::super) fn execution(&self) -> ExecutionRequest<'a, 'a> {
        self.execution
    }
    pub(in super::super::super) fn get(
        &self,
        ordinal: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<Node>, Stop> {
        maps::lookup(control, self.touched.len(), size_of::<usize>())?;
        if let Some(page) = self.touched.get(&(ordinal >> PAGE_BITS)) {
            let open = page.open.borrow();
            return Ok(open
                .builder
                .as_ref()
                .expect("only consuming seal takes a page")
                .elements()[ordinal & (PAGE_SIZE - 1)]);
        }
        self.base.get(ordinal, control).map(|node| node.copied())
    }
    pub(in super::super::super) fn replace(
        &mut self,
        ordinal: usize,
        value: Option<Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        let key = ordinal >> PAGE_BITS;
        maps::lookup(control, self.touched.len(), size_of::<usize>())?;
        let page = match self.touched.get(&key) {
            Some(page) => Rc::clone(page),
            None => {
                let page = self.copy_page(key, control)?;
                maps::insert(&mut self.touched, &key, &page, &self.execution, control)?;
                page
            }
        };
        let mut open = page.open.borrow_mut();
        let builder = open.builder.as_mut().expect("writer has not sealed");
        let index = ordinal & (PAGE_SIZE - 1);
        let before = builder.elements()[index].is_some();
        builder
            .replace_initialized(index, value)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        match (before, value.is_some()) {
            (false, true) => open.occupied += 1,
            (true, false) => open.occupied -= 1,
            _ => {}
        }
        drop(open);
        control.poll()
    }
    fn copy_page(
        &self,
        key: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Rc<WorkingPage<'a, Node>>, Stop> {
        let original = self.base.page_values(key, control)?;
        let mut builder = ExecutionRequestArrayBuilder::allocate(self.execution, PAGE_SIZE)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        let mut occupied = 0;
        for index in 0..PAGE_SIZE {
            control.checkpoint(1)?;
            let value = original.and_then(|page| page[index]);
            occupied += usize::from(value.is_some());
            builder
                .push(value)
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        let memory = header_memory::<WorkingPage<'a, Node>>(&self.execution, control)?;
        Ok(Rc::new(WorkingPage {
            open: RefCell::new(OpenPage {
                builder: Some(builder),
                occupied,
            }),
            _memory: memory,
        }))
    }
    pub(in super::super::super) fn seal(
        self,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<NodePages<Node>, Stop> {
        control.poll()?;
        if self.touched.is_empty() {
            return Ok(self.base);
        }
        let mut ordered =
            ExecutionRequestArrayBuilder::allocate(self.execution, self.touched.len())
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for (key, page) in self.touched.iter() {
            control.checkpoint(1)?;
            ordered
                .push((*key, page.as_ref()))
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        let ordered = ordered
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        let root = seal::Batch {
            pages: ordered.elements(),
            execution: self.execution,
            control,
            allocations: &self.sealed_branches,
        }
        .subtree(self.base.root, 0..self.touched.len())?;
        control.poll()?;
        Ok(NodePages { root })
    }
}

#[cfg(test)]
mod tests;
