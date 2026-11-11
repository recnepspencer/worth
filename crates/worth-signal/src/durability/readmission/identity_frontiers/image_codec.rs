//! Persist issuance bounds, including bodies never selected by a branch advance.
use super::*;
use crate::durability::frame::{Reader, Writer};
use crate::durability::SignalDurableFrameDenial as F;
use worth_execution::ExecutionRequest;

impl SignalRecordedIdentityContribution {
    pub(crate) fn merge(
        &mut self,
        other: &Self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), D> {
        self.check_owner(other.owner)?;
        self.branch = self.branch.max(other.branch);
        self.cell = self.cell.max(other.cell);
        self.snapshot = self.snapshot.max(other.snapshot);
        self.admission = self.admission.max(other.admission);
        self.conditional = self.conditional.max(other.conditional);
        self.definition = self.definition.max(other.definition);
        self.advance = self.advance.max(other.advance);
        for (branch, nodes) in other.nodes.iter() {
            for (index, generation) in nodes.iter() {
                self.observe_node(
                    worth_foundational::facade::DurableSignalNodeIdentity::from_recorded(
                        *branch,
                        *index,
                        *generation,
                    ),
                    execution,
                )?;
            }
        }
        Ok(())
    }
    pub(in crate::durability) fn write_image_bounds(&self, out: &mut Writer<'_>) -> Result<(), F> {
        out.owner(self.owner)?;
        for value in [
            self.branch,
            self.cell,
            self.admission,
            self.conditional,
            self.definition,
            self.advance,
        ] {
            out.number(value)?;
        }
        out.byte(u8::from(self.snapshot.is_some()))?;
        if let Some(snapshot) = self.snapshot {
            out.number(snapshot)?;
        }
        out.number(self.nodes.len() as u64)?;
        for (branch, nodes) in self.nodes.iter() {
            out.branch(*branch)?;
            out.number(nodes.len() as u64)?;
            for (index, generation) in nodes.iter() {
                out.number(*index as u64)?;
                out.number(*generation as u64)?;
            }
        }
        Ok(())
    }
    pub(in crate::durability) fn read_image_bounds(
        input: &mut Reader<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, D> {
        let mut bounds = Self::empty(input.owner().map_err(D::Frame)?);
        bounds.branch = input.number().map_err(D::Frame)?;
        bounds.cell = input.number().map_err(D::Frame)?;
        bounds.admission = input.number().map_err(D::Frame)?;
        bounds.conditional = input.number().map_err(D::Frame)?;
        bounds.definition = input.number().map_err(D::Frame)?;
        bounds.advance = input.number().map_err(D::Frame)?;
        bounds.snapshot = match input.byte().map_err(D::Frame)? {
            0 => None,
            1 => Some(input.number().map_err(D::Frame)?),
            _ => return Err(D::Frame(F::Malformed)),
        };
        let count = input.number().map_err(D::Frame)?;
        if count > input.bytes.len() as u64 / 40 {
            return Err(D::Frame(F::Malformed));
        }
        let mut previous_branch = None;
        for _ in 0..count {
            let branch = input.branch().map_err(D::Frame)?;
            bounds.check_owner(branch.owner())?;
            if previous_branch.is_some_and(|prior| prior >= branch) {
                return Err(D::Frame(F::Malformed));
            }
            previous_branch = Some(branch);
            let count = input.number().map_err(D::Frame)?;
            if count > input.bytes.len() as u64 / 16 {
                return Err(D::Frame(F::Malformed));
            }
            let mut previous_node = None;
            for _ in 0..count {
                let index: u32 = input
                    .number()
                    .map_err(D::Frame)?
                    .try_into()
                    .map_err(|_| D::Frame(F::Malformed))?;
                let generation: u32 = input
                    .number()
                    .map_err(D::Frame)?
                    .try_into()
                    .map_err(|_| D::Frame(F::Malformed))?;
                if previous_node.is_some_and(|prior| prior >= index) {
                    return Err(D::Frame(F::Malformed));
                }
                previous_node = Some(index);
                bounds.observe_node(
                    worth_foundational::facade::DurableSignalNodeIdentity::from_recorded(
                        branch, index, generation,
                    ),
                    execution,
                )?;
            }
        }
        bounds.check_issuable()?;
        Ok(bounds)
    }
}
