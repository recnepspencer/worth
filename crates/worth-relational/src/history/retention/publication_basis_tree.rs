//! Persistent, paid route nodes share holds without retaining an old image.
use super::publication_basis::{PublicationBasisRoute, RetainedPublicationBasis};
use crate::transactions::data::TransactionCommitError;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

type Link = Option<Arc<Node>>;
#[derive(Debug)]
struct Node {
    key: PublicationBasisRoute,
    value: Arc<RetainedPublicationBasis>,
    left: Link,
    right: Link,
    height: u8,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct PublicationBasisRoutes {
    root: Link,
    len: usize,
}
impl PublicationBasisRoutes {
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn get(
        &self,
        key: &PublicationBasisRoute,
    ) -> Option<&Arc<RetainedPublicationBasis>> {
        self.find_by(|node| key.cmp(node))
    }
    pub(crate) fn get_by_meaning(
        &self,
        meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
    ) -> Option<&Arc<RetainedPublicationBasis>> {
        self.find_by(|node| node.compare_meaning(meaning).reverse())
    }
    fn find_by(
        &self,
        compare: impl Fn(&PublicationBasisRoute) -> std::cmp::Ordering,
    ) -> Option<&Arc<RetainedPublicationBasis>> {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            use std::cmp::Ordering::*;
            match compare(&node.key) {
                Equal => return Some(&node.value),
                Less => current = node.left.as_deref(),
                Greater => current = node.right.as_deref(),
            }
        }
        None
    }
    pub(super) fn insert(
        &mut self,
        key: PublicationBasisRoute,
        value: Arc<RetainedPublicationBasis>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let added = self.get(&key).is_none();
        self.root = Some(insert(self.root.as_ref(), key, value, execution, context)?);
        self.len += usize::from(added);
        Ok(())
    }
    pub(super) fn remove(
        &mut self,
        key: &PublicationBasisRoute,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        if self.get(key).is_some() {
            self.root = remove(self.root.as_ref(), key, execution, context)?;
            self.len -= 1;
        }
        Ok(())
    }
}
fn height(link: &Link) -> u8 {
    link.as_ref().map_or(0, |n| n.height)
}
fn node(
    key: PublicationBasisRoute,
    value: Arc<RetainedPublicationBasis>,
    left: Link,
    right: Link,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<Node>, TransactionCommitError> {
    crate::execution::request_stage::checkpoint(context, 1)
        .map_err(super::publication_basis::request)?;
    let funding = execution
        .reserve_retained_memory(
            (std::mem::size_of::<Node>() + 2 * std::mem::size_of::<usize>()) as u64,
        )
        .map_err(super::publication_basis::request)?;
    Ok(Arc::new(Node {
        height: height(&left).max(height(&right)) + 1,
        key,
        value,
        left,
        right,
        _funding: funding,
    }))
}
fn balance(
    key: PublicationBasisRoute,
    value: Arc<RetainedPublicationBasis>,
    left: Link,
    right: Link,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<Node>, TransactionCommitError> {
    if height(&left) > height(&right) + 1 {
        let l = left.as_ref().expect("left-heavy tree has a left child");
        if height(&l.left) >= height(&l.right) {
            let r = node(key, value, l.right.clone(), right, execution, context)?;
            return node(
                l.key.clone(),
                l.value.clone(),
                l.left.clone(),
                Some(r),
                execution,
                context,
            );
        }
        let middle = l.right.as_ref().expect("inner-heavy child exists");
        let a = node(
            l.key.clone(),
            l.value.clone(),
            l.left.clone(),
            middle.left.clone(),
            execution,
            context,
        )?;
        let b = node(key, value, middle.right.clone(), right, execution, context)?;
        return node(
            middle.key.clone(),
            middle.value.clone(),
            Some(a),
            Some(b),
            execution,
            context,
        );
    }
    if height(&right) > height(&left) + 1 {
        let r = right.as_ref().expect("right-heavy tree has a right child");
        if height(&r.right) >= height(&r.left) {
            let l = node(key, value, left, r.left.clone(), execution, context)?;
            return node(
                r.key.clone(),
                r.value.clone(),
                Some(l),
                r.right.clone(),
                execution,
                context,
            );
        }
        let middle = r.left.as_ref().expect("inner-heavy child exists");
        let a = node(key, value, left, middle.left.clone(), execution, context)?;
        let b = node(
            r.key.clone(),
            r.value.clone(),
            middle.right.clone(),
            r.right.clone(),
            execution,
            context,
        )?;
        return node(
            middle.key.clone(),
            middle.value.clone(),
            Some(a),
            Some(b),
            execution,
            context,
        );
    }
    node(key, value, left, right, execution, context)
}
fn insert(
    previous: Option<&Arc<Node>>,
    key: PublicationBasisRoute,
    value: Arc<RetainedPublicationBasis>,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<Node>, TransactionCommitError> {
    let Some(old) = previous else {
        return node(key, value, None, None, execution, context);
    };
    use std::cmp::Ordering::*;
    match key.cmp(&old.key) {
        Equal => node(
            key,
            value,
            old.left.clone(),
            old.right.clone(),
            execution,
            context,
        ),
        Less => balance(
            old.key.clone(),
            old.value.clone(),
            Some(insert(old.left.as_ref(), key, value, execution, context)?),
            old.right.clone(),
            execution,
            context,
        ),
        Greater => balance(
            old.key.clone(),
            old.value.clone(),
            old.left.clone(),
            Some(insert(old.right.as_ref(), key, value, execution, context)?),
            execution,
            context,
        ),
    }
}
fn remove(
    previous: Option<&Arc<Node>>,
    key: &PublicationBasisRoute,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Link, TransactionCommitError> {
    let Some(old) = previous else {
        return Ok(None);
    };
    use std::cmp::Ordering::*;
    Ok(Some(match key.cmp(&old.key) {
        Less => balance(
            old.key.clone(),
            old.value.clone(),
            remove(old.left.as_ref(), key, execution, context)?,
            old.right.clone(),
            execution,
            context,
        )?,
        Greater => balance(
            old.key.clone(),
            old.value.clone(),
            old.left.clone(),
            remove(old.right.as_ref(), key, execution, context)?,
            execution,
            context,
        )?,
        Equal => match (&old.left, &old.right) {
            (None, _) => return Ok(old.right.clone()),
            (_, None) => return Ok(old.left.clone()),
            (Some(_), Some(right)) => {
                let mut successor = right;
                while let Some(left) = &successor.left {
                    successor = left;
                }
                balance(
                    successor.key.clone(),
                    successor.value.clone(),
                    old.left.clone(),
                    remove(old.right.as_ref(), &successor.key, execution, context)?,
                    execution,
                    context,
                )?
            }
        },
    }))
}
