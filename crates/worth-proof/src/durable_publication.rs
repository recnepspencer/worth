use crate::AuthorityMarker;

/// Move-only permission to cut over the exact publication committed by a medium.
///
/// The owning runtime seals `Publication`'s constructor and creates it only
/// after its medium answers Committed. It carries the actual subject and its
/// readmission custody. A concrete cutover accepts that exact specialization;
/// an unrelated marker cannot substitute for the owner's committed result.
///
/// A caller cannot fabricate the wrapper from subject descriptions:
///
/// ```compile_fail,E0451
/// use worth_proof::{AuthorityMarker, DurablyCommitted};
/// struct Description;
/// impl AuthorityMarker for Description {}
/// let forged = DurablyCommitted { publication: Description };
/// ```
///
/// A different authority does not satisfy a concrete owner's cutover:
///
/// ```compile_fail,E0308
/// use worth_proof::{AuthorityMarker, DurablyCommitted};
/// struct OwnerPublication;
/// impl AuthorityMarker for OwnerPublication {}
/// struct ForeignPublication;
/// impl AuthorityMarker for ForeignPublication {}
/// fn cutover(_: DurablyCommitted<OwnerPublication>) {}
/// cutover(DurablyCommitted::from_owner(ForeignPublication));
/// ```
///
/// The valid counterpart carries the owner's actual subject into one cutover:
///
/// ```
/// mod owner {
///     use worth_proof::{AuthorityMarker, DurablyCommitted};
///     pub struct CommittedPublication { subject: String }
///     impl AuthorityMarker for CommittedPublication {}
///     pub fn completed() -> DurablyCommitted<CommittedPublication> {
///         DurablyCommitted::from_owner(CommittedPublication { subject: "original".into() })
///     }
///     pub fn cutover(proof: DurablyCommitted<CommittedPublication>) -> String {
///         proof.consume(|publication| publication.subject)
///     }
/// }
/// assert_eq!(owner::cutover(owner::completed()), "original");
/// ```
///
/// Consuming a proof cannot authorize a second cutover:
///
/// ```compile_fail,E0382
/// use worth_proof::{AuthorityMarker, DurablyCommitted};
/// struct OwnerPublication;
/// impl AuthorityMarker for OwnerPublication {}
/// let proof = DurablyCommitted::from_owner(OwnerPublication);
/// proof.consume(|_| ());
/// proof.consume(|_| ());
/// ```
#[derive(Debug)]
pub struct DurablyCommitted<Publication: AuthorityMarker> {
    publication: Publication,
}

impl<Publication: AuthorityMarker> DurablyCommitted<Publication> {
    /// The same sealing contract as `AuthorityWitness::from_authority_marker`:
    /// possession of the concrete owner publication is the authority. Its
    /// constructor must remain private to the medium binding that grants it.
    pub fn from_owner(publication: Publication) -> Self {
        Self { publication }
    }

    /// Spend the proof and its subject in the owning runtime's live cutover.
    pub fn consume<Output>(self, cutover: impl FnOnce(Publication) -> Output) -> Output {
        cutover(self.publication)
    }
}
