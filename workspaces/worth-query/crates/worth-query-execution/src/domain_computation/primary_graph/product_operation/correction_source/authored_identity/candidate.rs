//! Compare the actual inverse output reference before any native commit.
use super::{Posture, PreparedAuthoredIdentity};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_relational::facade::transactions::{
    CreateIntent, EntityMutationIntent, EntityReference, MutationIntent, WorkerIntentBatch,
};

pub(in crate::domain_computation::primary_graph) enum AuthoredCorrectionCandidateDenial {
    Identity(&'static str),
    Interrupted(WorthQueryRequestInterruption),
}
use AuthoredCorrectionCandidateDenial::{Identity, Interrupted};

fn poll(request: &WorthQueryRequestScope) -> Result<(), AuthoredCorrectionCandidateDenial> {
    match request.interruption() {
        Some(stop) => Err(Interrupted(stop)),
        None => Ok(()),
    }
}

impl PreparedAuthoredIdentity {
    pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn validate_candidate<
        'a,
    >(
        &self,
        roles: impl Iterator<Item = (&'a str, Posture, &'a str, &'a EntityReference)>,
        batch: &WorkerIntentBatch,
        request: &WorthQueryRequestScope,
    ) -> Result<(), AuthoredCorrectionCandidateDenial> {
        poll(request)?;
        let parts = self.declaration.parts();
        let worth_query_declaration::facade::application_schema::ApplicationSchemaMember::Field {
            entity,
            ..
        } = parts.logical_key.as_ref()
        else {
            return Err(Identity(
                "authored correction lost its installed logical key",
            ));
        };
        let mut matched = None;
        for (name, posture, kind, target) in roles {
            poll(request)?;
            if name == parts.inverse_role && matched.replace((posture, kind, target)).is_some() {
                return Err(Identity(
                    "authored inverse repeated its declared output role",
                ));
            }
        }
        let (posture, kind, target) = matched.ok_or(Identity(
            "authored inverse omitted its declared output role",
        ))?;
        if kind != entity {
            return Err(Identity(
                "authored inverse output has a different entity family",
            ));
        }
        match (parts.source_posture, posture, target) {
            (Posture::Create, Posture::Retire, EntityReference::Existing(actual))
                if actual == &self.original =>
            {
                let mut deleted = false;
                for intent in &batch.intents {
                    poll(request)?;
                    if let MutationIntent::Entity(EntityMutationIntent::Delete(delete)) = intent
                        && delete.entity_id == self.original
                    {
                        if deleted {
                            return Err(Identity("authored inverse repeated its deletion intent"));
                        }
                        deleted = true;
                    }
                }
                if deleted {
                    Ok(())
                } else {
                    Err(Identity(
                        "authored inverse output has no actual deletion intent",
                    ))
                }
            }
            (Posture::Retire, Posture::Create, EntityReference::Created(created)) => {
                let mut matched = None;
                for intent in &batch.intents {
                    poll(request)?;
                    if let MutationIntent::Create(CreateIntent::Entity(spec)) = intent
                        && spec.partition_id == created.partition_id
                        && spec.kind_id == created.kind_id
                        && spec.client_key == created.client_key
                        && matched.replace(spec).is_some()
                    {
                        return Err(Identity("authored inverse repeated its creation intent"));
                    }
                }
                let spec = matched.ok_or(Identity(
                    "authored inverse output has no actual creation intent",
                ))?;
                if spec.fields.get(&self.locator) != Some(&self.key) {
                    return Err(Identity(
                        "authored inverse changed the declared logical key",
                    ));
                }
                Ok(())
            }
            _ => Err(Identity(
                "authored inverse does not correspond to the exact original entity",
            )),
        }
    }
}
