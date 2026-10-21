//! Admit every live and retired role before the native publication.
use super::*;

impl PreparedNativeOutputWitness {
    pub(in crate::domain_computation::primary_graph) fn prepare(
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        Self::prepare_roles(attempt.native_witness_roles(), layout)
    }

    /// A restoration re-creates an output its producer already performed. The
    /// committed correspondence names the roles that producer's attempt
    /// declared, so the same preparation serves the re-created entities.
    pub(in crate::domain_computation::primary_graph) fn prepare_republication(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let roles = correspondence
            .native_witness_roles()
            .map(|(role, posture, entity_name, _)| (role, posture, entity_name));
        Self::prepare_roles(roles, layout)
    }

    fn prepare_roles<'role>(
        roles: impl ExactSizeIterator<Item = (&'role str, WorthQueryApplicationOutputPosture, &'role str)>
            + Clone,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let role_count = roles.len();
        if role_count == 0 {
            return Ok(None);
        }

        let mut aspect_count = 0usize;
        for (_, posture, entity_name) in roles.clone() {
            if layout.entity_kind(entity_name).is_none() {
                return Ok(None);
            }
            if posture != WorthQueryApplicationOutputPosture::Retire {
                aspect_count = aspect_count
                    .checked_add(layout.native_output_aspects(entity_name).count())
                    .ok_or_else(overflow)?;
            }
        }
        let cell = Arc::new(OnceLock::new());
        let mut prepared = Self {
            cell,
            roles: Vec::with_capacity(role_count),
            aspects: Vec::with_capacity(aspect_count),
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
