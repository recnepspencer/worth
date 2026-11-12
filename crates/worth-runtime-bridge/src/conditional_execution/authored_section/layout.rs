//! The authored section contains declarations, allocations and exact lowerings.
use super::{
    definition::DefinitionView,
    fields::Reader,
    framing,
    registration::RegistrationView,
    rows::Rows,
    targets::{AllocationView, DeclarationView, PerformedTargetView},
    BridgeAuthoredSectionDenial as D,
};
use worth_foundational::facade::DurableSignalBranchIdentity;

pub(super) struct Layout<'a> {
    pub(super) branch: DurableSignalBranchIdentity,
    pub(super) adapter: Option<&'a str>,
    pub(super) registrations: &'a [u8],
    pub(super) allocations: &'a [u8],
    pub(super) definitions: &'a [u8],
}
impl<'a> Layout<'a> {
    pub(super) fn read(bytes: &'a [u8]) -> Result<Self, D> {
        let mut input = framing::payload(bytes)?;
        let branch = input.branch()?;
        let adapter = match input.byte()? {
            0 => None,
            1 => Some(input.text()?),
            _ => return Err(D::Malformed),
        };
        if adapter.is_some_and(|text| text.trim().is_empty()) {
            return Err(D::Malformed);
        }
        let registrations = input.blob()?;
        let allocations = input.blob()?;
        let definitions = input.blob()?;
        input.finish()?;
        Ok(Self {
            branch,
            adapter,
            registrations,
            allocations,
            definitions,
        })
    }
    pub(super) fn inspect(&self) -> Result<(), D> {
        let mut registrations = Rows::new(self.registrations)?;
        if registrations.len() != 0 && self.adapter.is_none() {
            return Err(D::SourceProfileRequired);
        }
        let mut previous = None;
        while let Some(bytes) = registrations.next()? {
            let mut input = Reader::new(bytes);
            let row = RegistrationView::read(&mut input)?;
            input.finish()?;
            if row.reference.definition().owner() != self.branch.owner()
                || previous.is_some_and(|old| old >= row.reference)
            {
                return Err(D::Malformed);
            }
            previous = Some(row.reference);
            serde_json::from_slice::<serde::de::IgnoredAny>(row.meaning)
                .map_err(|_| D::Malformed)?;
            let mut targets = Rows::new(row.targets)?;
            while let Some(bytes) = targets.next()? {
                let mut input = Reader::new(bytes);
                let target = DeclarationView::read(&mut input)?;
                if target.node.branch() != self.branch {
                    return Err(D::ForeignBranch);
                }
                input.finish()?;
            }
            targets.finish()?;
        }
        registrations.finish()?;
        self.inspect_allocations()?;
        self.inspect_definitions()
    }
    fn inspect_allocations(&self) -> Result<(), D> {
        let mut rows = Rows::new(self.allocations)?;
        while let Some(bytes) = rows.next()? {
            let mut input = Reader::new(bytes);
            let row = AllocationView::read(&mut input)?;
            if row.owner.definition().owner() != self.branch.owner()
                || row.declaration.node.branch() != self.branch
            {
                return Err(D::ForeignBranch);
            }
            input.finish()?;
        }
        rows.finish()
    }
    fn inspect_definitions(&self) -> Result<(), D> {
        let mut rows = Rows::new(self.definitions)?;
        let mut previous = None;
        while let Some(bytes) = rows.next()? {
            let mut input = Reader::new(bytes);
            let row = DefinitionView::read(&mut input)?;
            if row.occurrence.node().branch() != self.branch {
                return Err(D::ForeignBranch);
            }
            if previous.is_some_and(|node| node >= row.occurrence.node()) {
                return Err(D::Malformed);
            }
            previous = Some(row.occurrence.node());
            input.finish()?;
            let mut claims = Rows::new(row.claims)?;
            while let Some(bytes) = claims.next()? {
                let mut input = Reader::new(bytes);
                let reference = input.claim()?;
                if reference.definition().owner() != self.branch.owner() {
                    return Err(D::ForeignBranch);
                }
                let mut targets = Rows::new(input.blob()?)?;
                while let Some(bytes) = targets.next()? {
                    let mut target_input = Reader::new(bytes);
                    let target = PerformedTargetView::read(&mut target_input)?;
                    if target.node.branch() != self.branch {
                        return Err(D::ForeignBranch);
                    }
                    target_input.finish()?;
                    let mut sources = Reader::new(target.sources);
                    for _ in 0..target.sources.len() / super::fields::CLAIM_BYTES {
                        if sources.claim()?.definition().owner() != self.branch.owner() {
                            return Err(D::ForeignBranch);
                        }
                    }
                    sources.finish()?;
                }
                targets.finish()?;
                input.finish()?;
            }
            claims.finish()?;
        }
        rows.finish()
    }
}
