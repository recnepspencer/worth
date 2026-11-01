//! Encode normalized tables, with foreign-key occurrences and dispositions.
use super::*;
use crate::durability::codec::{self, fields::*, owner_value};
use crate::durability::{
    RecoveredCustodyDisposition, WorldDurableFrameDenial as Frame, WorldRecord,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

impl WorldCheckpointImage {
    pub(crate) fn encode(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, crate::durability::WorldDurableMediumDenial> {
        super::framing::encode(request, &mut |out| self.write_image(out, request))
    }
    fn write_image(
        &self,
        out: &mut Emit<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::durability::WorldDurableMediumDenial> {
        let mut stop = None;
        self.write_fields(out, request, &mut stop).map_err(|cause| {
            stop.unwrap_or(crate::durability::WorldDurableMediumDenial::Frame(cause))
        })
    }
    fn write_fields(
        &self,
        out: &mut Emit<'_>,
        request: &ExecutionRequest<'_, '_>,
        stop: &mut Option<crate::durability::WorldDurableMediumDenial>,
    ) -> Result<(), Frame> {
        owner(out, self.binding.owner())?;
        number(out, self.binding.incarnation().get())?;
        out(&self.binding.program_revision())?;
        for coverage in [
            &self.coverage.branches,
            &self.coverage.custody,
            &self.coverage.occurrences,
        ] {
            location(out, coverage.position, &coverage.digest)?;
        }
        for frontier in [
            self.frontiers.branch,
            self.frontiers.commit,
            self.frontiers.bootstrap,
            self.frontiers.attempt,
            self.frontiers.custody,
        ] {
            number(out, frontier)?;
        }
        match self.owner {
            None => out(&[0])?,
            Some(value) => {
                out(&[1])?;
                location(out, value.position, &value.digest)?;
            }
        }
        number(out, self.branches.len() as u64)?;
        for branch in self.branches.keys() {
            location(out, branch.position, &branch.digest)?;
            semantic_row(out, &branch.record().record, request, stop)?;
        }
        number(out, self.opens.len() as u64)?;
        for open in self.opens.values() {
            location(out, open.position(), open.digest())?;
            semantic_row(out, &open.record().record, request, stop)?;
        }
        number(out, self.dispositions.len() as u64)?;
        for disposition in self.dispositions.values() {
            location(out, disposition.position(), disposition.digest())?;
            number(out, disposition.open().association.attempt().ordinal())?;
            match disposition.disposition() {
                RecoveredCustodyDisposition::Continued => out(&[2])?,
                RecoveredCustodyDisposition::Released(None) => out(&[0])?,
                RecoveredCustodyDisposition::Released(Some(evidence)) => {
                    out(&[1])?;
                    sized(
                        out,
                        &mut |emit| owner_value::emit(emit, evidence),
                        request,
                        stop,
                    )?;
                }
            }
        }
        number(out, self.occurrences.len() as u64)?;
        for occurrence in self.occurrences.values() {
            location(
                out,
                occurrence.location().position(),
                occurrence.location().digest(),
            )?;
            number(out, occurrence.open().association.attempt().ordinal())?;
            evidence(out, occurrence.relational())?;
            number(out, occurrence.publishing_incarnation().get())?;
        }
        self.write_graph(out)
    }
    fn write_graph(&self, out: &mut Emit<'_>) -> Result<(), Frame> {
        number(out, self.history.len() as u64)?;
        for (commit, head) in self.history.iter() {
            number(out, commit.ordinal())?;
            image_head(out, head)?;
        }
        number(out, self.heads.len() as u64)?;
        for (key, head) in self.heads.iter() {
            branch(out, key.branch())?;
            image_head(out, head)?;
        }
        Ok(())
    }
}
fn location(out: &mut Emit<'_>, position: u64, digest: &[u8; 32]) -> Result<(), Frame> {
    number(out, position)?;
    out(digest)
}
fn semantic_row(
    out: &mut Emit<'_>,
    record: &WorldRecord,
    request: &ExecutionRequest<'_, '_>,
    stop: &mut Option<crate::durability::WorldDurableMediumDenial>,
) -> Result<(), Frame> {
    sized(
        out,
        &mut |emit| codec::write_record(record, emit),
        request,
        stop,
    )
}
fn sized(
    out: &mut Emit<'_>,
    emit: &mut dyn FnMut(&mut Emit<'_>) -> Result<(), Frame>,
    request: &ExecutionRequest<'_, '_>,
    stop: &mut Option<crate::durability::WorldDurableMediumDenial>,
) -> Result<(), Frame> {
    let mut size = 0usize;
    emit(&mut |bytes| {
        if let Err(cause) = super::framing::charge(request, bytes.len()) {
            *stop = Some(cause);
            return Err(Frame::Malformed);
        }
        size = size
            .checked_add(bytes.len())
            .filter(|size| *size <= super::framing::MAXIMUM)
            .ok_or(Frame::SizeLimit)?;
        Ok(())
    })?;
    number(out, size as u64)?;
    emit(out)
}
fn image_head(out: &mut Emit<'_>, value: &WorldImageHead) -> Result<(), Frame> {
    let (identity, incarnation, commit, generation) = value.fields();
    branch(out, identity)?;
    number(out, incarnation.ordinal())?;
    number(out, generation.get())?;
    number(out, commit.ordinal())
}
