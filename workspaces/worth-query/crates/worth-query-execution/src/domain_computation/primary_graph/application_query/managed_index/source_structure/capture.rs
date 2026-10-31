//! The type-dependent shell exposes actual nodes; the concrete core checks their graph.
use super::super::{
    invocation::provider_result, WorthQueryManagedIndexNodes, WorthQueryManagedIndexProvider,
    WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexWorkDenial as Denial,
};
use super::*;
use worth_execution::{ExecutionArray, ExecutionRequestArrayBuilder};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationManagedIndexMembership as Membership,
        ApplicationManagedIndexSourceStructure as Structure,
    },
    application_schema::ApplicationSchema,
};

pub(in super::super) enum Step<'a> {
    Cold,
    Warm {
        before: &'a SourceTree,
        changes: &'a [Change],
    },
}

pub(in super::super) fn qualify<S, I, P>(
    execution: &ExecutionRequest<'_, '_>,
    provider: &P,
    source: &WorthQueryManagedIndexSourceRows<'_, S, I>,
    nodes: &mut WorthQueryManagedIndexNodes<P::Node>,
    step: Step<'_>,
    control: &Control<'_>,
) -> Result<Option<SourceTree>, Denial>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    if I::STRUCTURE == Structure::Opaque {
        return Ok(None);
    }
    let changed = nodes.take_changes();
    let mut cold = 0..nodes.frontier();
    let mut warm = changed.as_ref().map(|keys| keys.keys().copied());
    let (count, coordinates): (usize, &mut dyn Iterator<Item = usize>) =
        match (&step, warm.as_mut()) {
            (Step::Cold, None) => (cold.len(), &mut cold),
            (Step::Warm { .. }, Some(keys)) => (changed.as_ref().unwrap().len(), keys),
            _ => return Err(control.refuse(Stop::SourceStructureIncomplete).into()),
        };
    let mut describe = |ordinal| {
        let Some(node) = nodes.node_at_ordinal(ordinal, control)? else {
            return Ok(None);
        };
        let region = provider_result(execution, control, provider.decompose(*node, control))?;
        for child in region.children.into_iter().flatten() {
            nodes.get(child, control)?;
        }
        Ok(Some(Region {
            source: region.source,
            children: region.children.map(|child| child.map(|id| id.ordinal)),
        }))
    };
    let previous = match &step {
        Step::Cold => None,
        Step::Warm { before, .. } => Some(*before),
    };
    let edits = capture(
        execution,
        count,
        coordinates,
        previous,
        &mut describe,
        control,
    )?;
    drop(warm); // Exact dirty-key enumeration is complete before custody moves.
    let image = source.image();
    let tree = match &step {
        Step::Cold => SourceTree::empty(image.owner),
        Step::Warm { before, .. } if before.owner == image.owner => (**before).clone(),
        _ => return Err(control.refuse(Stop::ForeignSourceSlot).into()),
    };
    let mut tree = tree.apply_edits(edits.elements(), image, execution, control)?;
    tree.admit(
        nodes.root().map(|id| id.ordinal),
        nodes.len(),
        source.len(),
        edits.elements(),
        control,
    )?;
    if let Step::Warm { before, changes } = &step {
        tree.admit_delta(before, changes, execution, control)?;
    }
    tree.root = nodes.root().map(|id| id.ordinal);
    if I::MEMBERSHIP == Membership::CheckedSummarySourceTree {
        let affected = match &step {
            Step::Cold => None,
            Step::Warm { before, changes } => Some(tree.affected(
                before,
                changes,
                edits.elements(),
                changed.expect("warm key enumeration required actual dirty custody"),
                execution,
                control,
            )?),
        };
        super::summary::qualify(
            execution,
            provider,
            source,
            nodes,
            &tree,
            affected.as_ref(),
            control,
        )?;
    }
    source.check()?;
    Ok(Some(tree))
}

fn capture(
    execution: &ExecutionRequest<'_, '_>,
    count: usize,
    coordinates: &mut dyn Iterator<Item = usize>,
    previous: Option<&SourceTree>,
    describe: &mut dyn FnMut(usize) -> Result<Option<Region>, Denial>,
    control: &Control<'_>,
) -> Result<ExecutionArray<Edit>, Denial> {
    let mut edits = ExecutionRequestArrayBuilder::allocate(*execution, count)
        .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    for ordinal in coordinates {
        control.checkpoint(1)?;
        let before = if let Some(previous) = previous {
            previous.region_at(ordinal, control)?
        } else {
            None
        };
        edits
            .push(Edit {
                ordinal,
                before,
                region: describe(ordinal)?,
            })
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    }
    edits
        .seal()
        .map_err(|cause| control.refuse(Stop::Storage(cause)).into())
}
