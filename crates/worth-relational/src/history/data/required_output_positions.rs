//! Current complete canonical publication per family/source route at one native root.
//! This derived selection image grants no ownership without native/proof custody.
use super::CommitId;
use crate::identity::data::EntityId;
use crate::mvcc::RequiredOutputPublicationMeaning as Meaning;
use crate::transactions::data::TransactionCommitError;
use std::{cmp::Ordering, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequiredOutputRoutePosition {
    Published(CommitId),
    CodecRetired(CommitId),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RequiredOutputPublicationPositions {
    root: Option<Arc<PositionNode>>,
    // Exact unindexed canonical prefix. Absence in this image is not route absence.
    unresolved_prefix: Option<CommitId>,
}

#[derive(Debug)]
struct PublicationRoute {
    family: String,
    source: EntityId,
    partition: [u8; 32],
    _funding: ExecutionMemoryReservation,
}

#[derive(Debug)]
struct PositionNode {
    route: Arc<PublicationRoute>,
    publication: RequiredOutputRoutePosition,
    left: Option<Arc<Self>>,
    right: Option<Arc<Self>>,
    height: u8,
    _funding: ExecutionMemoryReservation,
}

type Lease<'a, 'scope> = Option<&'a ExecutionResourceLease<'scope>>;

impl RequiredOutputPublicationPositions {
    pub(crate) fn unresolved_at(commit: CommitId) -> Self {
        Self {
            root: None,
            unresolved_prefix: Some(commit),
        }
    }

    pub(crate) fn unresolved_prefix(&self) -> Option<CommitId> {
        self.unresolved_prefix
    }

    pub(crate) fn get(
        &self,
        meaning: &Meaning,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<RequiredOutputRoutePosition>, TransactionCommitError> {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            current = match compare(&node.route, meaning, context)? {
                Ordering::Equal => return Ok(Some(node.publication)),
                Ordering::Greater => node.left.as_deref(),
                Ordering::Less => node.right.as_deref(),
            };
        }
        Ok(None)
    }

    pub(crate) fn insert(
        &mut self,
        meaning: &Meaning,
        publication: RequiredOutputRoutePosition,
        lease: Lease<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        self.root = Some(insert(
            self.root.as_ref(),
            meaning,
            publication,
            lease,
            context,
        )?);
        Ok(())
    }
}

fn insert(
    previous: Option<&Arc<PositionNode>>,
    meaning: &Meaning,
    publication: RequiredOutputRoutePosition,
    lease: Lease<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<PositionNode>, TransactionCommitError> {
    let Some(previous) = previous else {
        let bytes = (std::mem::size_of::<PublicationRoute>()
            + 2 * std::mem::size_of::<usize>()
            + meaning.family.len()) as u64;
        context.checkpoint(bytes).map_err(request)?;
        let funding = reserve(lease, bytes)?;
        let route = Arc::new(PublicationRoute {
            family: meaning.family.clone(),
            source: meaning.source_scope,
            partition: meaning.source_partition,
            _funding: funding,
        });
        return node(route, publication, None, None, lease, context);
    };
    let order = compare(&previous.route, meaning, context)?;
    if order == Ordering::Equal {
        return node(
            previous.route.clone(),
            publication,
            previous.left.clone(),
            previous.right.clone(),
            lease,
            context,
        );
    }
    let (left, right) = if order == Ordering::Greater {
        (
            Some(insert(
                previous.left.as_ref(),
                meaning,
                publication,
                lease,
                context,
            )?),
            previous.right.clone(),
        )
    } else {
        (
            previous.left.clone(),
            Some(insert(
                previous.right.as_ref(),
                meaning,
                publication,
                lease,
                context,
            )?),
        )
    };
    balance(
        node(
            previous.route.clone(),
            previous.publication,
            left,
            right,
            lease,
            context,
        )?,
        lease,
        context,
    )
}

fn compare(
    route: &PublicationRoute,
    meaning: &Meaning,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Ordering, TransactionCommitError> {
    context
        .checkpoint((route.family.len() + meaning.family.len() + 64) as u64)
        .map_err(request)?;
    Ok(route
        .family
        .cmp(&meaning.family)
        .then_with(|| route.source.cmp(&meaning.source_scope))
        .then_with(|| route.partition.cmp(&meaning.source_partition)))
}

fn height(node: &Option<Arc<PositionNode>>) -> u8 {
    node.as_ref().map_or(0, |node| node.height)
}

fn node(
    route: Arc<PublicationRoute>,
    publication: RequiredOutputRoutePosition,
    left: Option<Arc<PositionNode>>,
    right: Option<Arc<PositionNode>>,
    lease: Lease<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<PositionNode>, TransactionCommitError> {
    context.checkpoint(6).map_err(request)?;
    let bytes = (std::mem::size_of::<PositionNode>() + 2 * std::mem::size_of::<usize>()) as u64;
    let funding = reserve(lease, bytes)?;
    let height = height(&left)
        .max(height(&right))
        .checked_add(1)
        .ok_or_else(|| {
            request(crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow)
        })?;
    Ok(Arc::new(PositionNode {
        route,
        publication,
        left,
        right,
        height,
        _funding: funding,
    }))
}

fn balance(
    mut root: Arc<PositionNode>,
    lease: Lease<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<PositionNode>, TransactionCommitError> {
    if i16::from(height(&root.left)) - i16::from(height(&root.right)) > 1 {
        let left = root.left.as_ref().expect("left-heavy native position tree");
        if height(&left.right) > height(&left.left) {
            let left = rotate_left(left, lease, context)?;
            root = node(
                root.route.clone(),
                root.publication,
                Some(left),
                root.right.clone(),
                lease,
                context,
            )?;
        }
        return rotate_right(&root, lease, context);
    }
    if i16::from(height(&root.right)) - i16::from(height(&root.left)) > 1 {
        let right = root
            .right
            .as_ref()
            .expect("right-heavy native position tree");
        if height(&right.left) > height(&right.right) {
            let right = rotate_right(right, lease, context)?;
            root = node(
                root.route.clone(),
                root.publication,
                root.left.clone(),
                Some(right),
                lease,
                context,
            )?;
        }
        return rotate_left(&root, lease, context);
    }
    Ok(root)
}

fn rotate_left(
    root: &Arc<PositionNode>,
    lease: Lease<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<PositionNode>, TransactionCommitError> {
    let pivot = root
        .right
        .as_ref()
        .expect("native left rotation has a pivot");
    let lower = node(
        root.route.clone(),
        root.publication,
        root.left.clone(),
        pivot.left.clone(),
        lease,
        context,
    )?;
    node(
        pivot.route.clone(),
        pivot.publication,
        Some(lower),
        pivot.right.clone(),
        lease,
        context,
    )
}

fn rotate_right(
    root: &Arc<PositionNode>,
    lease: Lease<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<PositionNode>, TransactionCommitError> {
    let pivot = root
        .left
        .as_ref()
        .expect("native right rotation has a pivot");
    let lower = node(
        root.route.clone(),
        root.publication,
        pivot.right.clone(),
        root.right.clone(),
        lease,
        context,
    )?;
    node(
        pivot.route.clone(),
        pivot.publication,
        pivot.left.clone(),
        Some(lower),
        lease,
        context,
    )
}

fn reserve(
    lease: Lease<'_, '_>,
    bytes: u64,
) -> Result<ExecutionMemoryReservation, TransactionCommitError> {
    ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
        .map_err(|stop| request(crate::execution::denial::lease_denial(stop)))
}

fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
