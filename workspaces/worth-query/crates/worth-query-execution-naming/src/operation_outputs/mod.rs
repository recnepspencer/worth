//! Persistent-name births and lifecycle postures an installed operation
//! declares over its own outputs, kept as an erased program until commit.
//!
//! An operation never handles a minted name for a record it creates; it
//! declares what each output means (authored occurrence, role path, typed
//! discriminator) and which held names its effects split, merge or retire.
//! Relational alone mints and validates them during commit finalization.
//!
//! The program is authored against stable output keys (`kind`, client key),
//! bound to the exact created-record references the commit lowered, and
//! finally lowered against the commit basis's name index into one
//! [`worth_relational::facade::lineage::CommitNamingPlan`].

mod denial;
mod lowering;
mod same_commit;

pub use denial::WorthQueryOutputNameDenial;
pub use lowering::{admit_output_birth, OutputBirthPosture};
pub use same_commit::admit_same_commit_names;

use std::sync::Arc;

use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder, ApplicationNameSubjectKind,
    SameCommitNameSlot, WorthQueryName,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::lineage::PendingBirthTarget;

/// What one created output means: its authored occurrence, its declared role
/// and, for siblings sharing both, a typed discriminator.
///
/// The occurrence key comes from the application's authored identity family
/// (a feature's own identity, never the operation's name), so two
/// independently authored occurrences never mint the same name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputBirth {
    occurrence_key: String,
    role_path: String,
    canonical_discriminator: Vec<u8>,
    embedded_name_digests: Vec<[u8; 32]>,
    /// Where the discriminator references names born by the same commit;
    /// lowering writes each one once its referenced birth is known.
    same_commit_names: Vec<SameCommitNameSlot>,
}

impl WorthQueryOutputBirth {
    pub fn new(occurrence_key: impl Into<String>, role_path: impl Into<String>) -> Self {
        Self {
            occurrence_key: occurrence_key.into(),
            role_path: role_path.into(),
            canonical_discriminator: Vec::new(),
            embedded_name_digests: Vec::new(),
            same_commit_names: Vec::new(),
        }
    }

    /// Distinguishes siblings of one occurrence and role. Embedded
    /// `WorthQueryName` references must name records live at the basis and
    /// not retired in the committing transaction. An embedded
    /// `WorthQuerySameCommitName` must designate an output the same
    /// candidate named earlier, in the same occurrence, under a role this
    /// birth's role declares it depends on.
    pub fn with_discriminator<D: ApplicationNameDiscriminator>(
        mut self,
        discriminator: &D,
    ) -> Self {
        let mut encoder = ApplicationNameDiscriminatorEncoder::new(D::DISCRIMINATOR_IDENTITY);
        discriminator.encode_canonical(&mut encoder);
        let (bytes, references, same_commit_names) = encoder.finish_with_same_commit_names();
        self.canonical_discriminator = bytes;
        self.embedded_name_digests = references;
        self.same_commit_names = same_commit_names;
        self
    }

    pub fn occurrence_key(&self) -> &str {
        &self.occurrence_key
    }

    pub fn role_path(&self) -> &str {
        &self.role_path
    }

    pub fn has_discriminator(&self) -> bool {
        !self.canonical_discriminator.is_empty()
    }
}

/// Keeper of a merge an operation declares over held names of kind `K`;
/// `Target` is the created-output handle of the same kind.
///
/// A keeper of another kind does not compile:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
/// use worth_query_execution_naming::facade::WorthQueryOutputMergeKeeper;
///
/// struct Face;
/// struct Edge;
///
/// fn misuse(edge: WorthQueryName<Edge>) -> WorthQueryOutputMergeKeeper<Face, ()> {
///     WorthQueryOutputMergeKeeper::KeepParentInPlace(edge)
/// }
/// ```
///
/// The same-kind keeper compiles:
///
/// ```
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
/// use worth_query_execution_naming::facade::WorthQueryOutputMergeKeeper;
///
/// struct Face;
///
/// fn keep(face: WorthQueryName<Face>) -> WorthQueryOutputMergeKeeper<Face, ()> {
///     WorthQueryOutputMergeKeeper::KeepParentInPlace(face)
/// }
/// ```
#[derive(Clone, Debug)]
pub enum WorthQueryOutputMergeKeeper<K, Target> {
    /// Keep this listed parent's record and name in place.
    KeepParentInPlace(WorthQueryName<K>),
    /// Retire this listed parent's record and carry its name onto `target`,
    /// a record this operation created.
    KeepParentOnto {
        keeper: WorthQueryName<K>,
        target: Target,
    },
    /// Mint a new name on `target`, carrying every parent identity.
    MintNew {
        target: Target,
        birth: WorthQueryOutputBirth,
    },
}

/// One created output named under its kind's installed scheme.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationNamedOutput<T> {
    pub target: T,
    pub scheme_id: String,
    pub birth: WorthQueryOutputBirth,
}

/// Stable authored key of a created output: unique per kind in one program.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperationOutputKey {
    pub subject: ApplicationNameSubjectKind,
    pub kind: KindId,
    pub key: String,
}

/// Keeper of a declared merge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationMergeKeeper<T> {
    /// The keeper parent's record survives in place under its own name.
    KeepParentInPlace([u8; 32]),
    /// The keeper parent's record is retired and its name moves onto a record
    /// created in this commit.
    KeepParentOnto { keeper: [u8; 32], target: T },
    /// A created record mints a new name carrying every parent identity.
    MintNew(OperationNamedOutput<T>),
}

/// One declared naming effect of an installed operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationNamingEntry<T> {
    /// A created output is born named (`Create`, or `Preserve` continuity
    /// when the same commit retires the name's current holder).
    Birth(OperationNamedOutput<T>),
    /// A held name ends with the record this commit deletes.
    Retire {
        subject: ApplicationNameSubjectKind,
        name: [u8; 32],
    },
    /// A held name's record is retired and split into created children.
    Split {
        subject: ApplicationNameSubjectKind,
        parent: [u8; 32],
        children: Vec<OperationNamedOutput<T>>,
    },
    /// Two or more held names merge under `keeper`.
    Merge {
        subject: ApplicationNameSubjectKind,
        parents: Vec<[u8; 32]>,
        keeper: OperationMergeKeeper<T>,
    },
}

/// Ordered naming effects of one operation. Empty for every unnamed or
/// schemaless program, in which case no naming plan reaches the commit.
///
/// The program is the candidate its same-commit name references are issued
/// by: a reference designates one of this program's own named outputs, and
/// both authoring and lowering refuse one another program issued. Two
/// programs are equal when they declare the same effects.
#[derive(Clone, Debug)]
pub struct OperationNamingProgram<T> {
    candidate: Arc<()>,
    entries: Vec<OperationNamingEntry<T>>,
}

impl<T> Default for OperationNamingProgram<T> {
    fn default() -> Self {
        Self {
            candidate: Arc::new(()),
            entries: Vec::new(),
        }
    }
}

impl<T: PartialEq> PartialEq for OperationNamingProgram<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<T: Eq> Eq for OperationNamingProgram<T> {}

/// The program as authored against stable output keys.
pub type AuthoredOperationNaming = OperationNamingProgram<OperationOutputKey>;
/// The program bound to the exact created-record references Relational
/// resolves against its commit allocations.
pub type BoundOperationNaming = OperationNamingProgram<PendingBirthTarget>;

impl<T> OperationNamingEntry<T> {
    /// The outputs this entry names, in declaration order.
    pub fn named_outputs(&self) -> &[OperationNamedOutput<T>] {
        match self {
            Self::Birth(output)
            | Self::Merge {
                keeper: OperationMergeKeeper::MintNew(output),
                ..
            } => std::slice::from_ref(output),
            Self::Split { children, .. } => children,
            Self::Retire { .. } | Self::Merge { .. } => &[],
        }
    }
}

impl<T> OperationNamingProgram<T> {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The identity this program issues same-commit name references under.
    pub(crate) fn candidate(&self) -> &Arc<()> {
        &self.candidate
    }

    /// Every output the program names, in declaration order. An output's
    /// position here is the ordinal a same-commit reference designates.
    pub fn named_outputs(&self) -> impl Iterator<Item = &OperationNamedOutput<T>> {
        self.entries
            .iter()
            .flat_map(|entry| entry.named_outputs().iter())
    }

    pub fn push(&mut self, entry: OperationNamingEntry<T>) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[OperationNamingEntry<T>] {
        &self.entries
    }

    /// Binds every output target through `bind`, refusing the first target
    /// it cannot bind.
    ///
    /// Complexity: O(E) calls of `bind` for E named outputs.
    pub fn try_bind<U, E>(
        self,
        mut bind: impl FnMut(T) -> Result<U, E>,
    ) -> Result<OperationNamingProgram<U>, E> {
        let mut entries = Vec::with_capacity(self.entries.len());
        for entry in self.entries {
            entries.push(match entry {
                OperationNamingEntry::Birth(output) => {
                    OperationNamingEntry::Birth(bind_output(&mut bind, output)?)
                }
                OperationNamingEntry::Retire { subject, name } => {
                    OperationNamingEntry::Retire { subject, name }
                }
                OperationNamingEntry::Split {
                    subject,
                    parent,
                    children,
                } => OperationNamingEntry::Split {
                    subject,
                    parent,
                    children: children
                        .into_iter()
                        .map(|child| bind_output(&mut bind, child))
                        .collect::<Result<_, E>>()?,
                },
                OperationNamingEntry::Merge {
                    subject,
                    parents,
                    keeper,
                } => OperationNamingEntry::Merge {
                    subject,
                    parents,
                    keeper: match keeper {
                        OperationMergeKeeper::KeepParentInPlace(name) => {
                            OperationMergeKeeper::KeepParentInPlace(name)
                        }
                        OperationMergeKeeper::KeepParentOnto { keeper, target } => {
                            OperationMergeKeeper::KeepParentOnto {
                                keeper,
                                target: bind(target)?,
                            }
                        }
                        OperationMergeKeeper::MintNew(output) => {
                            OperationMergeKeeper::MintNew(bind_output(&mut bind, output)?)
                        }
                    },
                },
            });
        }
        Ok(OperationNamingProgram {
            candidate: self.candidate,
            entries,
        })
    }
}

fn bind_output<T, U, E>(
    bind: &mut impl FnMut(T) -> Result<U, E>,
    output: OperationNamedOutput<T>,
) -> Result<OperationNamedOutput<U>, E> {
    Ok(OperationNamedOutput {
        target: bind(output.target)?,
        scheme_id: output.scheme_id,
        birth: output.birth,
    })
}
