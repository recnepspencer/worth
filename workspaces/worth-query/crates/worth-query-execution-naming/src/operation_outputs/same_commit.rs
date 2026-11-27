//! Discriminator references to names born by the same commit.
//!
//! A later-role output of one occurrence may tell itself apart by an output
//! the same candidate named earlier. Its discriminator carries a placeholder
//! where that name goes; nothing knows the name until every held parent is
//! resolved at the commit basis. Lowering then walks the named outputs in
//! declaration order, writes each referenced name (the identity Relational
//! will mint for it) into the placeholders that wait on it, and declares the
//! reference as a same-occurrence one. Relational still validates every such
//! reference against the births it actually mints.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_query_declaration::facade::application_naming::SAME_COMMIT_NAME_PLACEHOLDER;
use worth_relational::facade::lineage::PersistentNameIdentity;

use super::{
    OperationNamedOutput, OperationNamingProgram, WorthQueryOutputBirth, WorthQueryOutputNameDenial,
};
use crate::denial::ExecutionNamingDenial;
use crate::lowering::LoweredExecutionNameSchemeCatalog;

/// Admits the same-commit references of `birth`, about to be named under
/// `scheme_id` in `program`, before any effect.
///
/// Refuses a reference `program` did not issue, one whose output the
/// program does not name, one into another occurrence, and one to a role
/// that is not strictly earlier than, and a declared dependency of, the
/// birth's own role.
///
/// Complexity: O(R * (E + D)) for R references, E named outputs and D
/// declared dependencies of the role.
pub fn admit_same_commit_names<T>(
    catalog: &LoweredExecutionNameSchemeCatalog,
    program: &OperationNamingProgram<T>,
    scheme_id: &str,
    birth: &WorthQueryOutputBirth,
) -> Result<(), WorthQueryOutputNameDenial> {
    for slot in &birth.same_commit_names {
        let reference = slot.reference();
        let referenced = reference
            .issued_by(program.candidate())
            .then(|| program.named_outputs().nth(reference.output()))
            .flatten()
            .ok_or(WorthQueryOutputNameDenial::ForeignOutputReference {
                role_path: birth.role_path.clone(),
            })?;
        admit_reference(catalog, scheme_id, birth, &referenced.birth)?;
    }
    Ok(())
}

/// The occurrence and role laws of one same-commit reference.
fn admit_reference(
    catalog: &LoweredExecutionNameSchemeCatalog,
    scheme_id: &str,
    birth: &WorthQueryOutputBirth,
    referenced: &WorthQueryOutputBirth,
) -> Result<(), WorthQueryOutputNameDenial> {
    if referenced.occurrence_key != birth.occurrence_key {
        return Err(WorthQueryOutputNameDenial::CrossOccurrenceReference {
            role_path: birth.role_path.clone(),
            occurrence_key: birth.occurrence_key.clone(),
            referenced_occurrence_key: referenced.occurrence_key.clone(),
        });
    }
    let assembly = |denial| Err(WorthQueryOutputNameDenial::Assembly(denial));
    let Some(layout) = catalog.scheme_by_id(scheme_id) else {
        return assembly(ExecutionNamingDenial::UnknownScheme {
            scheme_id: scheme_id.to_owned(),
        });
    };
    let rank = layout.role_rank(&birth.role_path);
    let referenced_rank = layout.role_rank(&referenced.role_path);
    if !matches!((referenced_rank, rank), (Some(earlier), Some(later)) if earlier < later) {
        return assembly(ExecutionNamingDenial::NonPrecedingSameOccurrenceRole {
            scheme_id: scheme_id.to_owned(),
            role_path: birth.role_path.clone(),
            referenced_role: referenced.role_path.clone(),
        });
    }
    let declared = layout
        .roles()
        .iter()
        .find(|role| role.role() == birth.role_path)
        .is_some_and(|role| role.depends_on().contains(&referenced.role_path));
    if !declared {
        return assembly(ExecutionNamingDenial::UndeclaredSameOccurrenceDependency {
            scheme_id: scheme_id.to_owned(),
            role_path: birth.role_path.clone(),
            referenced_role: referenced.role_path.clone(),
        });
    }
    Ok(())
}

/// One named output's discriminator with every same-commit reference
/// written, and the identity Relational will mint for the output.
pub(super) struct ResolvedOutputName {
    pub(super) identity: PersistentNameIdentity,
    pub(super) canonical_discriminator: Vec<u8>,
    pub(super) embedded_name_digests: Vec<[u8; 32]>,
    /// Role each same-commit digest was born under.
    pub(super) same_occurrence_roles: BTreeMap<[u8; 32], String>,
}

/// Resolves the outputs of the program `candidate` identifies, in
/// declaration order. Each output comes with its resolved parent identities
/// (none for a plain birth).
///
/// A reference must be one this program issued, and always designates an
/// earlier output, so one forward pass writes every placeholder before the
/// name that carries it is computed.
///
/// Complexity: O(E * (R + B)) for E outputs of R references and B
/// discriminator bytes.
pub(super) fn resolve_same_commit_names<'program, T: 'program>(
    catalog: &LoweredExecutionNameSchemeCatalog,
    candidate: &Arc<()>,
    outputs: impl Iterator<
        Item = (
            &'program OperationNamedOutput<T>,
            Vec<PersistentNameIdentity>,
        ),
    >,
) -> Result<Vec<ResolvedOutputName>, WorthQueryOutputNameDenial> {
    let mut resolved: Vec<ResolvedOutputName> = Vec::new();
    let mut births: Vec<&WorthQueryOutputBirth> = Vec::new();
    for (output, parents) in outputs {
        let birth = &output.birth;
        let mut canonical_discriminator = birth.canonical_discriminator.clone();
        let mut embedded_name_digests = birth.embedded_name_digests.clone();
        let mut same_occurrence_roles = BTreeMap::new();
        for slot in &birth.same_commit_names {
            let foreign = || WorthQueryOutputNameDenial::ForeignOutputReference {
                role_path: birth.role_path.clone(),
            };
            // An ordinal only means something in the program that issued it.
            if !slot.reference().issued_by(candidate) {
                return Err(foreign());
            }
            let ordinal = slot.reference().output();
            let (Some(referenced), Some(referenced_birth)) =
                (resolved.get(ordinal), births.get(ordinal))
            else {
                return Err(foreign());
            };
            admit_reference(catalog, &output.scheme_id, birth, referenced_birth)?;
            let digest = *referenced.identity.as_bytes();
            write_name(&mut canonical_discriminator, slot.offset(), &digest).ok_or_else(foreign)?;
            embedded_name_digests.push(digest);
            same_occurrence_roles.insert(digest, referenced_birth.role_path.clone());
        }
        let identity = PersistentNameIdentity::of_pending_birth(
            &output.scheme_id,
            &birth.occurrence_key,
            &birth.role_path,
            &parents,
            &canonical_discriminator,
            catalog
                .scheme_by_id(&output.scheme_id)
                .is_some_and(|layout| layout.structural_identity()),
        );
        resolved.push(ResolvedOutputName {
            identity,
            canonical_discriminator,
            embedded_name_digests,
            same_occurrence_roles,
        });
        births.push(birth);
    }
    Ok(resolved)
}

/// Writes `digest` over the placeholder at `offset`, which must still be an
/// unwritten name reference.
fn write_name(discriminator: &mut [u8], offset: usize, digest: &[u8; 32]) -> Option<()> {
    let end = offset.checked_add(digest.len())?;
    if discriminator.get(offset..end)? != SAME_COMMIT_NAME_PLACEHOLDER {
        return None;
    }
    discriminator[offset..end].copy_from_slice(digest);
    Some(())
}
