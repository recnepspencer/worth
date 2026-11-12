//! Named linear progression for an actual Native owner's recovered custody.
use crate::AuthorityMarker;

/// Native has readmitted one exact source publication and its required root
/// custody. The concrete Native payload has private owner-only construction.
/// This phase is distinct from a live `Performed` result and grants no product
/// movement without the concrete product owner's recovered-adoption door.
///
/// A foreign payload cannot replace the concrete owner specialization:
/// ```compile_fail,E0308
/// use worth_proof::{AuthorityMarker, RelationalPublicationReadmitted};
/// struct OwnerCustody;
/// impl AuthorityMarker for OwnerCustody {}
/// struct ForeignCustody;
/// impl AuthorityMarker for ForeignCustody {}
/// fn adopt(_: RelationalPublicationReadmitted<OwnerCustody>) {}
/// adopt(RelationalPublicationReadmitted::from_owner(ForeignCustody));
/// ```
/// The same owned pins cannot be consumed twice:
/// ```compile_fail,E0382
/// use worth_proof::{AuthorityMarker, RelationalPublicationReadmitted};
/// struct OwnerCustody;
/// impl AuthorityMarker for OwnerCustody {}
/// let admitted = RelationalPublicationReadmitted::from_owner(OwnerCustody);
/// admitted.consume(|_| ());
/// admitted.consume(|_| ());
/// ```
#[derive(Debug)]
pub struct RelationalPublicationReadmitted<Custody: AuthorityMarker> {
    custody: Custody,
}

impl<Custody: AuthorityMarker> RelationalPublicationReadmitted<Custody> {
    /// Only possession of the concrete owner's sealed payload may issue the
    /// specialization accepted by that owner's consumer.
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }

    pub fn custody(&self) -> &Custody {
        &self.custody
    }

    pub fn consume<Output>(self, next_owner: impl FnOnce(Custody) -> Output) -> Output {
        next_owner(self.custody)
    }
}
