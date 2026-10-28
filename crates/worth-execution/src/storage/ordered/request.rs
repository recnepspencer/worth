//! Original-request edits publish only after their final control handoff.
use super::{
    edit, funding::Funding, node::Entry, ExecutionOrderedMap, ExecutionOrderedMapDenial as Denial,
};
use crate::{ChargedBytes, ExecutionRequest, MapKernelStop, WorkCeilingDenial};

fn funded<'request, 'authority, R>(
    request: ExecutionRequest<'request, 'authority>,
    operation: impl FnOnce(Funding<'request, 'authority>) -> Result<R, Denial>,
) -> Result<R, Denial> {
    let mut held = None;
    let outcome = request.run_native_with_checkpoint(|context| {
        held = Some(
            context
                .poll()
                .map_err(|cause| Denial::Request(WorkCeilingDenial::Stopped(cause)))
                .and_then(|()| {
                    operation(Funding::Request {
                        request,
                        accounted: context.has_accounting(),
                    })
                }),
        );
    });
    match (outcome, held) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(Denial::Request(cause)),
        (Ok(_), None) => unreachable!("completed ordered edit callback"),
    }
}
impl<K: Ord, V> ExecutionOrderedMap<K, V> {
    /// Remove through the original request, funding every copied path cell.
    /// A missing key allocates nothing; refusal retains the original root.
    pub fn remove_for_request(
        &mut self,
        key: &K,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<bool, Denial> {
        let root = funded(request, |funding| {
            if !self.contains_key(key) {
                return Ok(None);
            }
            edit::remove(&self.root, key, funding).map(Some)
        })?;
        match root {
            Some(root) => {
                self.root = root;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
impl<K: Ord + Clone + ChargedBytes, V: Clone + ChargedBytes> ExecutionOrderedMap<K, V> {
    /// Fund the entry payload and copied AVL cells with their original request.
    /// Shared cells keep their original grants; refusal installs no partial root.
    pub fn insert_cloned_for_request(
        &mut self,
        key: &K,
        value: &V,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let root = funded(request, |funding| {
            let entry = Entry::cloned(key, value, funding)?;
            edit::insert(&self.root, entry, funding)
        })?;
        self.root = Some(root);
        Ok(())
    }
}
impl<K: Ord + Clone, V: Clone> ExecutionOrderedMap<K, V> {
    /// Fund framework cells through the original request. Payloads must retain
    /// their own admitted backing rather than being quoted again by this map.
    pub fn insert_cloned_native_for_request(
        &mut self,
        key: &K,
        value: &V,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let root = funded(request, |funding| {
            let entry = Entry::cloned_native(key, value, funding)?;
            edit::insert(&self.root, entry, funding)
        })?;
        self.root = Some(root);
        Ok(())
    }
}
