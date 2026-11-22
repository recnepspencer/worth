//! History and continuity extensions for [`LineageWorld`]: in-commit
//! regeneration and cross-commit restore (explicit or birth-classified).

use crate::facade::lineage::{
    LineagePostureClaim, LineageSubject, PersistentNameBirthRequest, PersistentNameHolder,
};

use super::name_world::{record_target, subject_of, LineageWorld, Name, Subject};

impl LineageWorld {
    pub(super) fn retired_names(&self) -> Vec<(Name, LineageSubject)> {
        self.retired_lineages
            .iter()
            .map(|(n, (subj, _))| (*n, *subj))
            .collect()
    }

    pub(super) fn regenerate(&mut self, name: Name) -> PersistentNameHolder {
        let old = self.live_holders[&name].clone();
        self.retire_record(&old);
        let fresh = self.new_record(subject_of(&old));
        let birth = self.replay_birth(name, &fresh);
        self.postures.push(LineagePostureClaim::Retire {
            target: record_target(&old),
            name,
        });
        self.postures.push(LineagePostureClaim::Create {
            target: record_target(&fresh),
            birth,
        });
        fresh
    }

    pub(super) fn restore(&mut self, name: Name, via_birth: bool) -> PersistentNameHolder {
        let subject = match self.retired_lineages[&name].0 {
            LineageSubject::Entity => Subject::Entity,
            LineageSubject::Relation => Subject::Relation,
        };
        let fresh = self.new_record(subject);
        let target = record_target(&fresh);
        if via_birth {
            let birth = self.replay_birth(name, &fresh);
            self.postures
                .push(LineagePostureClaim::Create { target, birth });
        } else {
            self.postures
                .push(LineagePostureClaim::Restore { target, name });
        }
        fresh
    }

    fn replay_birth(
        &self,
        name: Name,
        holder: &PersistentNameHolder,
    ) -> PersistentNameBirthRequest {
        let parts = &self.birth_parts[&name];
        PersistentNameBirthRequest {
            scheme_id: parts.scheme_id().to_owned(),
            occurrence_key: parts.occurrence_key().to_owned(),
            role_path: parts.role_path().to_owned(),
            parent_identities: parts.parent_identities().to_vec(),
            referenced_names: Vec::new(),
            canonical_discriminator: parts.canonical_discriminator().to_vec(),
            holder: holder.clone(),
        }
    }
}
