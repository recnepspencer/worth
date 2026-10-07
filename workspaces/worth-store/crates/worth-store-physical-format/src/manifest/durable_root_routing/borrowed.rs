//! Allocation-free routing grammar shared by owned decode and borrowed reads.
use super::*;

#[derive(Debug, Clone, Copy)]
enum BodyKind {
    Leaf,
    Branch,
}

/// Framed and count-checked input, not a validated route or custody claim.
/// Its count permits an owner to reserve exact uniqueness scratch first.
#[derive(Debug, Clone, Copy)]
pub struct RootRoutingBlockPreflight<'a> {
    body: &'a [u8],
    schema: u8,
    tree_identity: u64,
    generation: u64,
    block: u64,
    level: u16,
    count: usize,
    kind: BodyKind,
}

/// Every entry/reference, order, and coordinate has been checked by the
/// shared Format grammar; the bytes remain borrowed from the input frame.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalRootRoutingBlockView<'a> {
    preflight: RootRoutingBlockPreflight<'a>,
    first: PersistedRecordIdentity,
    last: PersistedRecordIdentity,
}

impl<'a> RootRoutingBlockPreflight<'a> {
    pub fn inspect_frame(
        bytes: &'a [u8],
        capacity: u16,
        limits: RootRoutingBlockDecodeLimits,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), BoundedRootRoutingBlockDecodeDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::RootRoutingBlock)
            .map_err(RootRoutingBlockDenial::Frame)?;
        let preflight = Self::inspect_payload(
            frame.payload,
            frame.identity,
            capacity,
            limits,
            frame.schema,
        )?;
        Ok((preflight, format))
    }

    pub fn inspect_payload(
        payload: &'a [u8],
        block_identity: u64,
        capacity: u16,
        limits: RootRoutingBlockDecodeLimits,
        schema: u8,
    ) -> Result<Self, BoundedRootRoutingBlockDecodeDenial> {
        if payload.len() < ROUTING_BLOCK_PREFIX_BYTES
            || payload[21..24] != [0; 3]
            || payload[32..40] != [0; 8]
        {
            return Err(RootRoutingBlockDenial::MalformedPrefix.into());
        }
        let tree_identity = u64::from_le_bytes(payload[..8].try_into().unwrap());
        let block = u64::from_le_bytes(payload[8..16].try_into().unwrap());
        let level = u16::from_le_bytes(payload[16..18].try_into().unwrap());
        let count = usize::from(u16::from_le_bytes(payload[18..20].try_into().unwrap()));
        let generation = u64::from_le_bytes(payload[24..32].try_into().unwrap());
        if tree_identity == 0
            || generation == 0
            || block == 0
            || block != block_identity
            || count == 0
            || count > usize::from(capacity)
        {
            return Err(RootRoutingBlockDenial::IdentityOrCapacity.into());
        }
        let (kind, width) = match payload[20] {
            1 if level == 0 => (BodyKind::Leaf, ROUTING_LEAF_ENTRY_BYTES),
            2 if level != 0 => (BodyKind::Branch, ROUTING_REFERENCE_BYTES),
            _ => return Err(RootRoutingBlockDenial::LevelOrKind.into()),
        };
        if payload.len() != ROUTING_BLOCK_PREFIX_BYTES + count * width {
            return Err(RootRoutingBlockDenial::MalformedLength.into());
        }
        let observed = count as u64;
        match kind {
            BodyKind::Leaf if observed > limits.leaf_entries => {
                return Err(BoundedRootRoutingBlockDecodeDenial::LeafEntries {
                    observed,
                    admitted: limits.leaf_entries,
                });
            }
            BodyKind::Branch if observed > limits.branch_children => {
                return Err(BoundedRootRoutingBlockDecodeDenial::BranchChildren {
                    observed,
                    admitted: limits.branch_children,
                });
            }
            _ => {}
        }
        Ok(Self {
            body: &payload[ROUTING_BLOCK_PREFIX_BYTES..],
            schema,
            tree_identity,
            generation,
            block,
            level,
            count,
            kind,
        })
    }

    pub const fn coordinate_scratch_slots(self) -> usize {
        match self.kind {
            BodyKind::Leaf => self.count,
            BodyKind::Branch => 0,
        }
    }

    pub fn validate(
        self,
        scratch: &mut Vec<RootRoutingCoordinateKey>,
    ) -> Result<PhysicalRootRoutingBlockView<'a>, BoundedRootRoutingBlockDecodeDenial> {
        let required = self.coordinate_scratch_slots();
        if scratch.capacity() < required {
            return Err(
                BoundedRootRoutingBlockDecodeDenial::CoordinateScratchInsufficient {
                    required,
                    provided: scratch.capacity(),
                },
            );
        }
        let (first, last) = match self.kind {
            BodyKind::Leaf => self.validate_leaf(scratch)?,
            BodyKind::Branch => self.validate_branch()?,
        };
        Ok(PhysicalRootRoutingBlockView {
            preflight: self,
            first,
            last,
        })
    }

    fn validate_leaf(
        self,
        scratch: &mut Vec<RootRoutingCoordinateKey>,
    ) -> Result<
        (PersistedRecordIdentity, PersistedRecordIdentity),
        BoundedRootRoutingBlockDecodeDenial,
    > {
        scratch.clear();
        let mut first = None;
        let mut previous = None;
        for bytes in self.body.chunks_exact(ROUTING_LEAF_ENTRY_BYTES) {
            let placement =
                decode_entry(bytes, self.schema).map_err(RootRoutingBlockDenial::Placement)?;
            let record = placement.record();
            if previous.is_some_and(|prior| prior >= record) {
                return Err(RootRoutingBlockDenial::CanonicalOrder.into());
            }
            first.get_or_insert(record);
            previous = Some(record);
            scratch.push(RootRoutingCoordinateKey::from_placement(placement));
        }
        scratch.sort_unstable();
        let unique = scratch.windows(2).all(|pair| pair[0] != pair[1]);
        scratch.clear();
        if !unique {
            return Err(RootRoutingBlockDenial::CanonicalOrder.into());
        }
        Ok((first.unwrap(), previous.unwrap()))
    }

    fn validate_branch(
        self,
    ) -> Result<
        (PersistedRecordIdentity, PersistedRecordIdentity),
        BoundedRootRoutingBlockDecodeDenial,
    > {
        let mut first = None;
        let mut previous = None;
        for bytes in self.body.chunks_exact(ROUTING_REFERENCE_BYTES) {
            let child = decode_reference(bytes).ok_or(RootRoutingBlockDenial::InvalidReference)?;
            if child.level().checked_add(1) != Some(self.level)
                || child.generation() > self.generation
                || previous.is_some_and(|last| last >= child.first())
            {
                return Err(RootRoutingBlockDenial::CanonicalOrder.into());
            }
            first.get_or_insert(child.first());
            previous = Some(child.last());
        }
        Ok((first.unwrap(), previous.unwrap()))
    }
}

impl<'a> PhysicalRootRoutingBlockView<'a> {
    pub const fn tree_identity(self) -> u64 {
        self.preflight.tree_identity
    }
    pub const fn generation(self) -> u64 {
        self.preflight.generation
    }
    pub const fn block(self) -> u64 {
        self.preflight.block
    }
    pub const fn level(self) -> u16 {
        self.preflight.level
    }
    pub const fn count(self) -> usize {
        self.preflight.count
    }
    pub const fn first(self) -> PersistedRecordIdentity {
        self.first
    }
    pub const fn last(self) -> PersistedRecordIdentity {
        self.last
    }
    pub fn reference(self, checksum: u32) -> ManifestBlockReference {
        ManifestBlockReference::new(
            self.generation(),
            self.block(),
            self.level(),
            checksum,
            self.first,
            self.last,
        )
        .expect("validated route view has a valid reference")
    }
    pub fn entries(
        &self,
    ) -> Option<
        impl ExactSizeIterator<Item = CurrentPhysicalRecordPlacement> + DoubleEndedIterator + '_,
    > {
        match self.preflight.kind {
            BodyKind::Leaf => Some(
                self.preflight
                    .body
                    .chunks_exact(ROUTING_LEAF_ENTRY_BYTES)
                    .map(|bytes| {
                        decode_entry(bytes, self.preflight.schema)
                            .expect("validated route placement")
                    }),
            ),
            BodyKind::Branch => None,
        }
    }
    pub fn children(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = ManifestBlockReference> + DoubleEndedIterator + '_>
    {
        match self.preflight.kind {
            BodyKind::Branch => Some(
                self.preflight
                    .body
                    .chunks_exact(ROUTING_REFERENCE_BYTES)
                    .map(|bytes| decode_reference(bytes).expect("validated route reference")),
            ),
            BodyKind::Leaf => None,
        }
    }
    pub(super) fn into_owned(self) -> PhysicalRootRoutingBlock {
        match self.preflight.kind {
            BodyKind::Leaf => PhysicalRootRoutingBlock::Leaf {
                tree_identity: self.tree_identity(),
                generation: self.generation(),
                block: self.block(),
                entries: self.entries().unwrap().collect(),
            },
            BodyKind::Branch => PhysicalRootRoutingBlock::Branch {
                tree_identity: self.tree_identity(),
                generation: self.generation(),
                block: self.block(),
                level: self.level(),
                children: self.children().unwrap().collect(),
            },
        }
    }
}
