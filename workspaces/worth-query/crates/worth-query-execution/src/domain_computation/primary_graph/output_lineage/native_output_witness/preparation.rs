//! Admit every live and retired role before the native publication.
use super::*;

impl PreparedNativeOutputWitness {
    pub(in crate::domain_computation::primary_graph) fn prepare(
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
        layout: &WorthQueryPrimaryGraphLayout,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        Self::prepare_roles(attempt.native_witness_roles(), layout, owner, admission)
    }

    /// A restoration re-creates an output its producer already performed. The
    /// committed correspondence names the roles that producer's attempt
    /// declared, so the same preparation serves the re-created entities.
    pub(in crate::domain_computation::primary_graph) fn prepare_republication(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let roles = correspondence
            .native_witness_roles()
            .map(|(role, posture, entity_name, _)| (role, posture, entity_name));
        Self::prepare_roles(roles, layout, owner, admission)
    }

    fn prepare_roles<'role>(
        roles: impl ExactSizeIterator<Item = (&'role str, WorthQueryApplicationOutputPosture, &'role str)>
            + Clone,
        layout: &WorthQueryPrimaryGraphLayout,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let role_count = roles.len();
        if role_count == 0 {
            return Ok(None);
        }
        admission.charge_external_work(role_count as u64)?;
        let mut aspect_count = 0usize;
        let mut name_bytes = 0u64;
        let mut catalog_work = 0u64;
        let mut retirement_count = 0u64;
        for (role, posture, entity_name) in roles.clone() {
            let catalog_lookup_work = layout
                .native_output_lookup_work(entity_name)
                .and_then(|work| work.checked_add(3))
                .ok_or_else(overflow)?;
            admission.charge_external_work(catalog_lookup_work)?;
            if layout.entity_kind(entity_name).is_none() {
                return Ok(None);
            }
            name_bytes = name_bytes
                .checked_add(role.len() as u64)
                .and_then(|n| n.checked_add(entity_name.len() as u64))
                .ok_or_else(overflow)?;
            catalog_work = catalog_work
                .checked_add(catalog_lookup_work)
                .ok_or_else(overflow)?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                retirement_count = retirement_count.checked_add(1).ok_or_else(overflow)?;
                continue;
            }
            for aspect in layout.native_output_aspects(entity_name) {
                admission.charge_external_work(1)?;
                aspect_count = aspect_count.checked_add(1).ok_or_else(overflow)?;
                name_bytes = name_bytes
                    .checked_add(aspect.as_str().len() as u64)
                    .ok_or_else(overflow)?;
            }
        }
        let backing = role_count
            .checked_mul(size_of::<RoleProbe>())
            .and_then(|bytes| {
                bytes.checked_add(aspect_count.checked_mul(size_of::<AspectProbe>())?)
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(name_bytes))
            .ok_or_else(overflow)?;
        // A second selected-kind lookup builds the owned probes. Later
        // performed-root reads and canonical role comparisons are also prepaid.
        let visits = role_count
            .checked_mul(2)
            .and_then(|count| count.checked_add(aspect_count))
            .ok_or_else(overflow)? as u64;
        // Vec slot widths and the Arc cell are retained memory. Constructing
        // each initialized row is one visit; only owned names copy UTF-8.
        let copy_work = name_bytes.checked_add(visits).ok_or_else(overflow)?;
        let copy_work = copy_work.checked_add(catalog_work).ok_or_else(overflow)?;
        let copy_work = copy_work
            .checked_add(name_bytes)
            .and_then(|n| n.checked_add(retirement_count.checked_mul(retirement::SEAL_WORK)?))
            .ok_or_else(overflow)?;
        let retained = owner.retain_native_output_witness::<SealedNativeOutputWitness>(
            backing, copy_work, admission,
        )?;
        let cell = Arc::new(OnceLock::new());
        let mut prepared = Self {
            cell,
            roles: Vec::with_capacity(role_count),
            aspects: Vec::with_capacity(aspect_count),
            retained,
        };
        for (role, posture, entity_name) in roles {
            let first_aspect = prepared.aspects.len();
            if posture != WorthQueryApplicationOutputPosture::Retire {
                for aspect in layout.native_output_aspects(entity_name) {
                    prepared.aspects.push(AspectProbe {
                        aspect: aspect.clone(),
                        revision: None,
                    });
                }
            }
            prepared.roles.push(RoleProbe {
                role: role.to_owned(),
                entity_name: entity_name.to_owned(),
                kind: layout
                    .entity_kind(entity_name)
                    .expect("counted installed output kind"),
                posture,
                retirement: None,
                entity: None,
                first_aspect,
                end_aspect: prepared.aspects.len(),
            });
        }
        Ok(Some(prepared))
    }
}
