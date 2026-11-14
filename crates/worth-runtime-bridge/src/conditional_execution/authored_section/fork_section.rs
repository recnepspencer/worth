//! Rebase only branch-scoped authored names; semantic claims keep their origins.
use super::{
    definition::DefinitionView,
    fields::{Reader, Writer},
    framing,
    layout::Layout,
    registration::RegistrationView,
    rows::Rows,
    targets::{self, AllocationView, DeclarationView, PerformedTargetView},
    BridgeAuthoredSection, BridgeAuthoredSectionDenial as D,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurableSignalBranchIdentity, DurableSignalNodeIdentity};
use worth_signal::facade::durability::{SignalBridgeAuthoredSection, SignalForkPublicationContext};

impl BridgeAuthoredSection {
    pub(super) fn capture_fork(
        execution: &ExecutionRequest<'_, '_>,
        context: &SignalForkPublicationContext<'_>,
    ) -> Result<Self, D> {
        let bytes = context.predecessor().bridge_bytes();
        framing::work(execution, bytes.len())?;
        let layout = Layout::read(bytes)?;
        layout.inspect()?;
        let target = context.branch();
        if target.owner() != layout.branch.owner() || target == layout.branch {
            return Err(D::ForeignBranch);
        }
        let definitions = Rows::new(layout.definitions)?.len();
        if definitions != context.occurrences().len() {
            return Err(D::SignalDefinitionMismatch);
        }
        let source = framing::encode(execution, &mut |out| {
            out.branch(target)?;
            out.byte(u8::from(layout.adapter.is_some()))?;
            if let Some(adapter) = layout.adapter {
                out.text(adapter)?;
            }
            rewrite_rows(out, layout.registrations, &mut |out, bytes| {
                let mut input = Reader::new(bytes);
                let row = RegistrationView::read(&mut input)?;
                input.finish()?;
                out.claim(row.reference)?;
                out.blob(row.meaning)?;
                rewrite_rows(out, row.targets, &mut |out, bytes| {
                    let mut input = Reader::new(bytes);
                    let row = DeclarationView::read(&mut input)?;
                    input.finish()?;
                    declaration(out, &row, target)
                })
            })?;
            rewrite_rows(out, layout.allocations, &mut |out, bytes| {
                let mut input = Reader::new(bytes);
                let row = AllocationView::read(&mut input)?;
                input.finish()?;
                out.claim(row.owner)?;
                declaration(out, &row.declaration, target)?;
                out.byte(row.selected.index() as u8)
            })?;
            rewrite_rows(out, layout.definitions, &mut |out, bytes| {
                let mut input = Reader::new(bytes);
                let row = DefinitionView::read(&mut input)?;
                input.finish()?;
                let node = rebased(row.occurrence.node(), target);
                let ordinal = context
                    .occurrences()
                    .binary_search_by_key(&node, |row| row.node())
                    .map_err(|_| D::SignalDefinitionMismatch)?;
                let occurrence = context.occurrences()[ordinal];
                row.write_rebased(out, occurrence, &mut |out| {
                    // Claims are stable identities; only their performed targets move.
                    write_rows_body(out, row.claims, &mut |out, bytes| {
                        let mut input = Reader::new(bytes);
                        out.claim(input.claim()?)?;
                        let targets = input.blob()?;
                        input.finish()?;
                        rewrite_rows(out, targets, &mut |out, bytes| {
                            let mut input = Reader::new(bytes);
                            let row = PerformedTargetView::read(&mut input)?;
                            input.finish()?;
                            out.node(rebased(row.node, target))?;
                            out.text(row.partition)?;
                            out.text(row.mapping)?;
                            out.text(row.mapping_basis)?;
                            out.byte(row.selected.index() as u8)?;
                            targets::write_precision(out, row.precision)?;
                            targets::write_widening(out, row.widening)?;
                            out.blob(row.sources)
                        })
                    })
                })
            })?;
            let definitions = super::owned_async::read(layout.owned_async, layout.branch)?;
            let rows: Vec<_> = definitions.values().collect();
            out.record(&mut |out| super::owned_async::write(out, target, &rows))
        })?;
        let source =
            SignalBridgeAuthoredSection::from_bridge_bytes(source).map_err(D::SignalCarrier)?;
        Layout::read(source.as_bytes())?.inspect()?;
        Ok(Self {
            source,
            branch: target,
            definition_count: definitions,
        })
    }
}
fn rebased(
    node: DurableSignalNodeIdentity,
    branch: DurableSignalBranchIdentity,
) -> DurableSignalNodeIdentity {
    DurableSignalNodeIdentity::from_recorded(branch, node.index(), node.generation())
}
fn declaration(
    out: &mut Writer<'_>,
    row: &DeclarationView<'_>,
    branch: DurableSignalBranchIdentity,
) -> Result<(), D> {
    out.node(rebased(row.node, branch))?;
    out.text(row.partition)?;
    out.text(row.mapping)?;
    targets::write_slot(out, row.slot)
}
fn rewrite_rows(
    out: &mut Writer<'_>,
    bytes: &[u8],
    rewrite: &mut dyn FnMut(&mut Writer<'_>, &[u8]) -> Result<(), D>,
) -> Result<(), D> {
    out.record(&mut |out| write_rows_body(out, bytes, rewrite))
}
fn write_rows_body(
    out: &mut Writer<'_>,
    bytes: &[u8],
    rewrite: &mut dyn FnMut(&mut Writer<'_>, &[u8]) -> Result<(), D>,
) -> Result<(), D> {
    let mut rows = Rows::new(bytes)?;
    out.number(rows.len() as u64)?;
    while let Some(bytes) = rows.next()? {
        out.record(&mut |out| rewrite(out, bytes))?;
    }
    rows.finish()
}
