use worth_proof::CanonicalVec;
use worth_store_physical_format::{
    encode_data_frame_page_lsn, DurableFrameKind, PhysicalPageLsn, PhysicalRecordFormatDeclaration,
    PhysicalRewriteRedo,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::{
    CertifiedPriorPageBasis, PageWalBasis, PhysicalDataFrameIdentity, PhysicalDataFrameKind,
    PhysicalRedoLsn, PhysicalRedoTargetClaim,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDataPlanBindingDenial {
    EmptyFrameSet,
    EmptyRedoDelta,
    RedoOrdinalOutOfRange,
    LsnOverflow,
    InvalidFrame,
    InvalidWalBasis,
}

pub(in crate::physical_runtime) struct PreparedPhysicalDataFrame {
    target: PhysicalDataFrameIdentity,
    prior: CertifiedPriorPageBasis,
    redo_ordinals: CanonicalVec<u32>,
    bytes: Vec<u8>,
}

pub(in crate::physical_runtime) struct PreparedPhysicalDataPlan {
    frames: Vec<PreparedPhysicalDataFrame>,
    record_count: u32,
    rewrite: Option<PhysicalRewriteRedo>,
}

pub(in crate::physical_runtime) struct WalBoundPhysicalDataFrame {
    basis: PageWalBasis,
    bytes: Vec<u8>,
}

pub(in crate::physical_runtime) struct WalBoundPhysicalDataPlan {
    frames: Vec<WalBoundPhysicalDataFrame>,
    record_count: u32,
    redo_targets: Vec<CanonicalVec<PhysicalRedoTargetClaim>>,
    rewrite: Option<PhysicalRewriteRedo>,
}

impl PreparedPhysicalDataFrame {
    pub(in crate::physical_runtime) fn new(
        target: PhysicalDataFrameIdentity,
        prior: CertifiedPriorPageBasis,
        redo_ordinals: Vec<u32>,
        bytes: Vec<u8>,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, PhysicalDataPlanBindingDenial> {
        let kind = durable_kind(target.kind());
        if !prior.admits_target(target)
            || !target.admits_bytes(format, &bytes)
            || worth_store_physical_format::decode_data_frame_page_lsn(&bytes, kind)
                != Ok(prior.page_lsn())
        {
            return Err(PhysicalDataPlanBindingDenial::InvalidFrame);
        }
        let redo_ordinals = CanonicalVec::try_from_sorted(redo_ordinals)
            .map_err(|_| PhysicalDataPlanBindingDenial::EmptyRedoDelta)?;
        if redo_ordinals.as_slice().is_empty() {
            return Err(PhysicalDataPlanBindingDenial::EmptyRedoDelta);
        }
        Ok(Self {
            target,
            prior,
            redo_ordinals,
            bytes,
        })
    }
}

impl PreparedPhysicalDataPlan {
    pub(in crate::physical_runtime) fn new(
        frames: Vec<PreparedPhysicalDataFrame>,
        record_count: u32,
    ) -> Result<Self, PhysicalDataPlanBindingDenial> {
        if frames.is_empty() {
            return Err(PhysicalDataPlanBindingDenial::EmptyFrameSet);
        }
        if record_count == 0
            || frames.iter().any(|frame| {
                frame
                    .redo_ordinals
                    .as_slice()
                    .iter()
                    .any(|ordinal| *ordinal >= record_count)
            })
        {
            return Err(PhysicalDataPlanBindingDenial::RedoOrdinalOutOfRange);
        }
        Ok(Self {
            frames,
            record_count,
            rewrite: None,
        })
    }

    pub(in crate::physical_runtime) fn with_rewrite(mut self, rewrite: PhysicalRewriteRedo) -> Self {
        self.rewrite = Some(rewrite);
        self
    }

    /// New artifact bytes this plan will retain, one charge per artifact generation.
    pub(in crate::physical_runtime) fn retained_growth(&self) -> Vec<(u64, u64)> {
        let mut charges = std::collections::BTreeMap::<u64, u64>::new();
        for frame in &self.frames {
            let generation = match frame.target.coordinate().artifact() {
                worth_store_physical_format::RecordArtifactFile::Segment { generation, .. }
                | worth_store_physical_format::RecordArtifactFile::Extent { generation, .. } => {
                    generation
                }
                _ => continue,
            };
            let entry = charges.entry(generation).or_insert(0);
            *entry = entry.saturating_add(u64::from(frame.target.coordinate().length()));
        }
        charges.into_iter().collect()
    }

    pub(in crate::physical_runtime) fn bind(
        self,
        range: WalLsnRange,
    ) -> Result<WalBoundPhysicalDataPlan, (Self, PhysicalDataPlanBindingDenial)> {
        let rewrite = self.rewrite;
        match bind_frames(self.frames, self.record_count, range) {
            Ok(mut plan) => {
                plan.rewrite = rewrite;
                Ok(plan)
            }
            Err((frames, denial)) => Err((
                Self {
                    frames,
                    record_count: self.record_count,
                    rewrite,
                },
                denial,
            )),
        }
    }
}

impl WalBoundPhysicalDataPlan {
    pub(in crate::physical_runtime) fn redo_targets(
        &self,
    ) -> &[CanonicalVec<PhysicalRedoTargetClaim>] {
        &self.redo_targets
    }

    pub(in crate::physical_runtime) fn frames(&self) -> &[WalBoundPhysicalDataFrame] {
        &self.frames
    }

    pub(in crate::physical_runtime) const fn rewrite(&self) -> Option<PhysicalRewriteRedo> {
        self.rewrite
    }

    pub(in crate::physical_runtime) fn into_prepared(mut self) -> PreparedPhysicalDataPlan {
        let frames = self
            .frames
            .drain(..)
            .map(|mut frame| {
                encode_data_frame_page_lsn(
                    &mut frame.bytes,
                    durable_kind(frame.basis.target().kind()),
                    frame.basis.prior().page_lsn(),
                )
                .expect("a WAL-bound frame was admitted from this exact durable frame");
                PreparedPhysicalDataFrame {
                    target: frame.basis.target(),
                    prior: frame.basis.prior(),
                    redo_ordinals: CanonicalVec::try_from_sorted(
                        frame
                            .basis
                            .delta()
                            .iter()
                            .map(|redo| redo.ordinal())
                            .collect(),
                    )
                    .expect("the bound delta preserves canonical nonempty ordinals"),
                    bytes: frame.bytes,
                }
            })
            .collect();
        PreparedPhysicalDataPlan {
            frames,
            record_count: self.record_count,
            rewrite: self.rewrite,
        }
    }
}

impl WalBoundPhysicalDataFrame {
    pub(in crate::physical_runtime) const fn basis(&self) -> &PageWalBasis {
        &self.basis
    }

    pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn bind_frames(
    frames: Vec<PreparedPhysicalDataFrame>,
    record_count: u32,
    range: WalLsnRange,
) -> Result<
    WalBoundPhysicalDataPlan,
    (
        Vec<PreparedPhysicalDataFrame>,
        PhysicalDataPlanBindingDenial,
    ),
> {
    let mut bound = Vec::with_capacity(frames.len());
    let mut targets = vec![Vec::new(); record_count as usize];
    let mut remaining = frames.into_iter();
    while let Some(mut frame) = remaining.next() {
        let delta = match bind_delta(&frame, range) {
            Ok(delta) => delta,
            Err(denial) => return Err((restore(bound, frame, remaining), denial)),
        };
        let resulting_lsn = delta
            .as_slice()
            .last()
            .expect("prepared frames have a nonempty redo delta")
            .lsn();
        let kind = durable_kind(frame.target.kind());
        if encode_data_frame_page_lsn(
            &mut frame.bytes,
            kind,
            PhysicalPageLsn::new(resulting_lsn.get()),
        )
        .is_err()
        {
            return Err((
                restore(bound, frame, remaining),
                PhysicalDataPlanBindingDenial::InvalidFrame,
            ));
        }
        let basis = match PageWalBasis::from_encoded_frame(
            frame.target,
            frame.prior,
            delta,
            &frame.bytes,
        ) {
            Some(basis) => basis,
            None => {
                return Err((
                    restore(bound, frame, remaining),
                    PhysicalDataPlanBindingDenial::InvalidWalBasis,
                ))
            }
        };
        let claim = PhysicalRedoTargetClaim::new(frame.target, basis.resulting_payload_digest());
        for ordinal in frame.redo_ordinals.as_slice() {
            targets[*ordinal as usize].push(claim);
        }
        bound.push(WalBoundPhysicalDataFrame {
            basis,
            bytes: frame.bytes,
        });
    }
    let mut redo_targets = Vec::with_capacity(targets.len());
    for claims in targets {
        let claims = match CanonicalVec::try_from_sorted(claims) {
            Ok(claims) => claims,
            Err(_) => {
                let frames = bound.into_iter().map(unbind_frame).collect::<Vec<_>>();
                return Err((frames, PhysicalDataPlanBindingDenial::EmptyRedoDelta));
            }
        };
        redo_targets.push(claims);
    }
    Ok(WalBoundPhysicalDataPlan {
        frames: bound,
        record_count,
        redo_targets,
        rewrite: None,
    })
}

fn bind_delta(
    frame: &PreparedPhysicalDataFrame,
    range: WalLsnRange,
) -> Result<CanonicalVec<PhysicalRedoLsn>, PhysicalDataPlanBindingDenial> {
    let delta = frame
        .redo_ordinals
        .as_slice()
        .iter()
        .map(|ordinal| {
            let value = range
                .start()
                .get()
                .checked_add(u64::from(*ordinal))
                .ok_or(PhysicalDataPlanBindingDenial::LsnOverflow)?;
            let lsn = LogSequenceNumber::new(value);
            if !range.contains(lsn) {
                return Err(PhysicalDataPlanBindingDenial::RedoOrdinalOutOfRange);
            }
            Ok(PhysicalRedoLsn::new(*ordinal, lsn))
        })
        .collect::<Result<Vec<_>, _>>()?;
    CanonicalVec::try_from_sorted(delta).map_err(|_| PhysicalDataPlanBindingDenial::EmptyRedoDelta)
}

fn restore(
    bound: Vec<WalBoundPhysicalDataFrame>,
    current: PreparedPhysicalDataFrame,
    remaining: impl Iterator<Item = PreparedPhysicalDataFrame>,
) -> Vec<PreparedPhysicalDataFrame> {
    bound
        .into_iter()
        .map(unbind_frame)
        .chain(std::iter::once(current))
        .chain(remaining)
        .collect()
}

fn unbind_frame(mut frame: WalBoundPhysicalDataFrame) -> PreparedPhysicalDataFrame {
    encode_data_frame_page_lsn(
        &mut frame.bytes,
        durable_kind(frame.basis.target().kind()),
        frame.basis.prior().page_lsn(),
    )
    .expect("a WAL-bound frame was admitted from this exact durable frame");
    PreparedPhysicalDataFrame {
        target: frame.basis.target(),
        prior: frame.basis.prior(),
        redo_ordinals: CanonicalVec::try_from_sorted(
            frame
                .basis
                .delta()
                .iter()
                .map(|redo| redo.ordinal())
                .collect(),
        )
        .expect("the bound delta preserves canonical nonempty ordinals"),
        bytes: frame.bytes,
    }
}

const fn durable_kind(kind: PhysicalDataFrameKind) -> DurableFrameKind {
    match kind {
        PhysicalDataFrameKind::InlinePage => DurableFrameKind::InlinePage,
        PhysicalDataFrameKind::ExtentChunk => DurableFrameKind::Extent,
    }
}
