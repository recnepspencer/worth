//! A successor replaces only its authored definition row, preserving earned claims.
use super::{
    definition, fields::Reader, framing, layout::Layout, rows::Rows, BridgeAuthoredSection,
    BridgeAuthoredSectionDenial as D,
};
use crate::conditional_execution::{BridgeConditionalContract, BridgeConditionalLocation};
use worth_execution::ExecutionRequest;
use worth_signal::facade::durability::{
    SignalBridgeAuthoredSection, SignalDefinitionTransitionArtifact,
};

impl BridgeAuthoredSection {
    /// Encode descriptive successor bytes; this grants no live publication authority.
    pub fn capture_definition_successor(
        execution: &ExecutionRequest<'_, '_>,
        predecessor: &SignalDefinitionTransitionArtifact,
        occurrence: worth_foundational::facade::DurableSignalConditionalDefinitionOccurrence,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
    ) -> Result<BridgeAuthoredSection, D> {
        let bytes = predecessor.bridge_bytes();
        framing::work(execution, bytes.len())?;
        let layout = Layout::read(bytes)?;
        if occurrence.node().branch() != layout.branch {
            return Err(D::ForeignBranch);
        }
        let mut rows = Rows::new(layout.definitions)?;
        let count = rows.len();
        let mut found = false;
        while let Some(bytes) = rows.next()? {
            let mut input = Reader::new(bytes);
            let row = definition::DefinitionView::read(&mut input)?;
            input.finish()?;
            if row.occurrence.node() == occurrence.node() {
                if found || !row.has_identity(contract, location) {
                    return Err(D::SignalDefinitionMismatch);
                }
                found = true;
            }
        }
        rows.finish()?;
        if !found {
            return Err(D::SignalDefinitionMismatch);
        }
        let bytes = framing::encode(execution, &mut |out| {
            out.branch(layout.branch)?;
            out.byte(u8::from(layout.adapter.is_some()))?;
            if let Some(adapter) = layout.adapter {
                out.text(adapter)?;
            }
            out.blob(layout.registrations)?;
            out.blob(layout.allocations)?;
            out.record(&mut |out| {
                out.number(count as u64)?;
                let mut rows = Rows::new(layout.definitions)?;
                while let Some(bytes) = rows.next()? {
                    let mut input = Reader::new(bytes);
                    let row = definition::DefinitionView::read(&mut input)?;
                    input.finish()?;
                    if row.occurrence.node() == occurrence.node() {
                        out.record(&mut |out| {
                            definition::write_parts(out, occurrence, contract, location, row.claims)
                        })?;
                    } else {
                        out.blob(bytes)?;
                    }
                }
                rows.finish()
            })
        })?;
        Ok(BridgeAuthoredSection {
            source: SignalBridgeAuthoredSection::from_bridge_bytes(bytes)
                .map_err(D::SignalCarrier)?,
            branch: layout.branch,
            definition_count: count,
        })
    }
}
