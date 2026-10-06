//! The share of a caller's budget one read may spend. Only the budget that
//! owns dimension `D` can mint a grant of it: minting consumes that budget's
//! recorded `Performed<ReadGranted, D::Authority, u64>`. A read past its grant
//! hands the grant's dimension and the read's real length back, so the owner
//! states its own limit with its own counts.
//!
//! A read no caller budgets is `ReadGrant<Uncharged>`: its ceiling and the
//! observation's own bytes bound it, and it can never pass a grant, because
//! `Uncharged` has no values.

use worth_foundational::LimitDimension;
use worth_proof::{ActionMarker, Performed};

worth_proof::authority_marker!(pub UnchargedReadAuthority);

/// The dimension of a read no caller budgets. It has no values, so no grant of
/// it can be passed and no limit of it can be minted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uncharged {}

impl LimitDimension for Uncharged {
    type Authority = UnchargedReadAuthority;
}

/// The action a budget records when it grants bytes to one read.
#[derive(Debug)]
pub enum ReadGranted {}

impl ActionMarker for ReadGranted {}

/// What one read may spend of a caller's budget.
#[derive(Debug, PartialEq, Eq)]
pub struct ReadGrant<D: LimitDimension> {
    bound: Option<(D, u64)>,
}

impl ReadGrant<Uncharged> {
    /// A read no caller budgets.
    pub const fn ceiling_only() -> Self {
        Self { bound: None }
    }
}

impl<D: LimitDimension> ReadGrant<D> {
    /// `granted` bytes of `dimension`, from its owner's recorded grant.
    pub fn granted(dimension: D, grant: Performed<ReadGranted, D::Authority, u64>) -> Self {
        Self {
            bound: Some((dimension, grant.into_outcome())),
        }
    }

    /// The bytes granted; `None` where no caller budgets the read.
    pub const fn bytes(&self) -> Option<u64> {
        match self.bound {
            Some((_, bytes)) => Some(bytes),
            None => None,
        }
    }

    /// The read's real `length` passed this grant: the overrun its owner
    /// turns into a limit. `None` within the grant.
    pub(crate) fn overrun(&self, length: u64) -> Option<GrantOverrun<D>> {
        match self.bound {
            Some((dimension, granted)) if length > granted => Some(GrantOverrun {
                dimension,
                granted,
                length,
            }),
            Some(_) | None => None,
        }
    }
}

/// One grant as the reads that share it spend it, such as the files of one
/// WAL inventory: each is asked for no more than is left, and one past it is
/// refused with what the reads before it took plus its own real length.
#[derive(Debug)]
pub(crate) struct GrantShare<'grant, D: LimitDimension> {
    grant: &'grant ReadGrant<D>,
    spent: u64,
}

/// What the reads sharing one grant took, with the next read's length, went
/// past every count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharePastEveryCount;

impl<'grant, D: LimitDimension> GrantShare<'grant, D> {
    /// The whole grant, for the reads that share it.
    pub(crate) const fn of(grant: &'grant ReadGrant<D>) -> Self {
        Self { grant, spent: 0 }
    }

    /// What is left of the grant; `None` where no caller budgets the reads.
    pub(crate) fn left(&self) -> Option<u64> {
        self.grant.bytes().map(|granted| granted - self.spent)
    }

    /// The next read's real `length` passed what is left: the overrun counts
    /// from the grant's first byte.
    pub(crate) fn overrun(
        &self,
        length: u64,
    ) -> Result<Option<GrantOverrun<D>>, SharePastEveryCount> {
        if self.grant.bound.is_none() {
            return Ok(None);
        }
        let through = self.spent.checked_add(length).ok_or(SharePastEveryCount)?;
        Ok(self.grant.overrun(through))
    }

    /// Spends what a read returned, which was no more than was left. A read
    /// no caller budgets spends nothing.
    pub(crate) fn spend(&mut self, bytes: u64) {
        if let Some(left) = self.left() {
            assert!(
                bytes <= left,
                "a read returns no more than is left of its grant"
            );
            self.spent += bytes;
        }
    }
}

/// A read whose real length passed what its grant allowed. Only a grant can
/// produce one, so an `Uncharged` read never does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrantOverrun<D: LimitDimension> {
    dimension: D,
    granted: u64,
    length: u64,
}

impl GrantOverrun<Uncharged> {
    /// No read passes a grant no caller budgets.
    pub fn impossible(&self) -> ! {
        match self.dimension {}
    }
}

impl<D: LimitDimension> GrantOverrun<D> {
    pub const fn dimension(&self) -> D {
        self.dimension
    }

    pub const fn granted(&self) -> u64 {
        self.granted
    }

    /// The read's real length, from the artifact's metadata or its exact
    /// range.
    pub const fn length(&self) -> u64 {
        self.length
    }
}

/// A budget a test owns, to grant its reads.
#[cfg(test)]
pub(crate) mod for_test {
    use worth_foundational::LimitDimension;
    use worth_proof::Performed;

    use super::ReadGrant;

    worth_proof::authority_marker!(pub(crate) TestBudgetAuthority);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct TestBytes;

    impl LimitDimension for TestBytes {
        type Authority = TestBudgetAuthority;
    }

    pub(crate) fn grant(bytes: u64) -> ReadGrant<TestBytes> {
        ReadGrant::granted(
            TestBytes,
            Performed::record(&TestBudgetAuthority::witness(), bytes),
        )
    }
}
