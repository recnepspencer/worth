//! Passive admitted Query meaning; no retained authorization or READ grant.
use super::*;
use crate::domain_computation::authorization::{
    retained_capability_request::material::Material, WorthQueryCapabilityUseIdentity,
};
use crate::domain_computation::primary_graph::SourceFactsAllocation;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters as Parameters;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosurePosture as Posture;
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation) struct WorthQueryRetainedQueryInfluence {
    meaning: Meaning,
    descriptor: [u8; 32],
}
enum Meaning {
    Public,
    Governed {
        use_identity: WorthQueryCapabilityUseIdentity,
        disclosure: AspectValue,
        disclosure_work: u64,
        _allocation: Arc<SourceFactsAllocation>,
    },
}
#[derive(Debug)]
pub(in crate::domain_computation) enum WorthQueryRetainedQueryInfluenceDenial {
    ForeignAdmission,
    Execution(Cause),
}
impl From<Cause> for WorthQueryRetainedQueryInfluenceDenial {
    fn from(cause: Cause) -> Self {
        Self::Execution(cause)
    }
}
impl WorthQueryApplicationQueryGovernance {
    pub(in crate::domain_computation) fn retain_managed_influence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        parent: &WorthQueryManagedGraphWorkSession,
        association: &WorthQueryInstalledManagedIndexParent,
        parameters: &Parameters,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<WorthQueryRetainedQueryInfluence, WorthQueryRetainedQueryInfluenceDenial> {
        use WorthQueryRetainedQueryInfluenceDenial::ForeignAdmission;
        let scope = parent.entity_access_context().ok_or(ForeignAdmission)?;
        let installed = association.access_contract();
        if parent.binding() != association.binding_identity()
            || parent.obligation() != installed.parent_obligation()
            || !self.authorization_belongs_to_session(parent.identity())
            || !self.computation_matches(
                parent,
                parent.runtime_authority(),
                installed.meaning().parent_query(),
                parameters.identity(),
                parent.principal(),
                scope,
            )
        {
            return Err(ForeignAdmission);
        }
        let mut admit = |work| SourceFactsAllocation::work(execution, work);
        let overflow = || Cause::WorkCounterOverflow;
        let (meaning, descriptor) = match (self, installed.meaning().parent_disclosure_posture()) {
            (Self::Public, Posture::Public) if parent.capability_access_context().is_none() => {
                let mut writer = Material::new(&mut admit, &overflow);
                writer.bytes(b"worth-query:admitted-query-influence:v1")?;
                writer.bytes(b"public")?;
                (Meaning::Public, writer.finish())
            }
            (
                Self::Governed {
                    computation,
                    authorization,
                    ..
                },
                Posture::Governed,
            ) => {
                if parent.capability_access_context()
                    != Some(authorization.installed_capability_identity())
                {
                    return Err(ForeignAdmission);
                }
                let use_identity =
                    WorthQueryCapabilityUseIdentity::retain(authorization, execution, allocation)?;
                let disclosure = &computation.disclosure_value;
                let bytes = u64::try_from(disclosure.owned_allocation_capacity_bytes())
                    .map_err(|_| Cause::ChargedBytesOverflow)?;
                allocation.claim(execution, bytes)?;
                admit(bytes)?;
                let use_descriptor = use_identity.descriptor(&mut admit, &overflow)?;
                let mut writer = Material::new(&mut admit, &overflow);
                writer.bytes(b"worth-query:admitted-query-influence:v1")?;
                writer.bytes(b"governed")?;
                writer.bytes(&use_descriptor)?;
                writer.value(disclosure)?;
                let descriptor = writer.finish();
                (
                    Meaning::Governed {
                        use_identity,
                        disclosure: disclosure.clone(),
                        disclosure_work: bytes
                            .checked_add(size_of::<AspectValue>() as u64)
                            .ok_or(Cause::WorkCounterOverflow)?,
                        _allocation: Arc::clone(allocation),
                    },
                    descriptor,
                )
            }
            _ => return Err(ForeignAdmission),
        };
        admit(0)?;
        Ok(WorthQueryRetainedQueryInfluence {
            meaning,
            descriptor,
        })
    }
}
impl WorthQueryRetainedQueryInfluence {
    pub(in crate::domain_computation) fn descriptor(&self) -> [u8; 32] {
        self.descriptor
    }
    /// The enclosing captured comparison preserves any owned error behind this
    /// bool-erased checkpoint; this method never turns a stop into equality.
    pub(in crate::domain_computation) fn same_meaning(
        &self,
        other: &Self,
        checkpoint: &mut dyn FnMut(u64) -> bool,
    ) -> bool {
        if self.descriptor != other.descriptor {
            return false;
        }
        match (&self.meaning, &other.meaning) {
            (Meaning::Public, Meaning::Public) => checkpoint(0),
            (
                Meaning::Governed {
                    use_identity: left,
                    disclosure: a,
                    disclosure_work: aw,
                    ..
                },
                Meaning::Governed {
                    use_identity: right,
                    disclosure: b,
                    disclosure_work: bw,
                    ..
                },
            ) => {
                checkpoint(*aw)
                    && checkpoint(*bw)
                    && a == b
                    && left
                        .same_use(right, &mut |work| {
                            if checkpoint(work) {
                                Ok(())
                            } else {
                                Err(())
                            }
                        })
                        .unwrap_or(false)
            }
            _ => false,
        }
    }
}
