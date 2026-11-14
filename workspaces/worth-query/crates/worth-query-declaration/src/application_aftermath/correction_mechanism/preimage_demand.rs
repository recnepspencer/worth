//! Field correlation and relation endpoint demands for a recorded inverse.
use super::{DeclaredPreImageLocus, PortablePreImageDemand};
use std::{cmp::Ordering, fmt};
/// Schema-affine retained pre-image demand required by an inverse.
pub struct DeclaredPreImageDemand<Schema> {
    loci: Vec<DeclaredPreImageLocus<Schema>>,
    relations: Vec<super::DeclaredPreImageRelationLocus<Schema>>,
    maximum_encoded_bytes: usize,
    created_entity: bool,
    changed_output: Option<crate::application_operation::ApplicationMutationOutputRoleDescriptor>,
}

impl<Schema> DeclaredPreImageDemand<Schema> {
    pub(super) fn created_entity() -> Self {
        Self {
            loci: Vec::new(),
            relations: Vec::new(),
            maximum_encoded_bytes: 0,
            created_entity: true,
            changed_output: None,
        }
    }
    /// Records a demand expression; operation association checks its bounds.
    pub fn new(
        loci: impl IntoIterator<Item = DeclaredPreImageLocus<Schema>>,
        maximum_encoded_bytes: usize,
    ) -> Self {
        Self {
            loci: loci.into_iter().collect(),
            relations: Vec::new(),
            maximum_encoded_bytes,
            created_entity: false,
            changed_output: None,
        }
    }

    /// Retain only actually changed eligible fields on one exact preserved output.
    /// An unchanged eligible field is not retained; changed fields require an
    /// authorized original value or authenticated absence before publication.
    pub fn changed_fields_for_output(
        output: crate::application_operation::ApplicationMutationOutputRoleDescriptor,
        loci: impl IntoIterator<Item = DeclaredPreImageLocus<Schema>>,
        maximum_encoded_bytes: usize,
    ) -> Self {
        let mut demand = Self::new(loci, maximum_encoded_bytes);
        demand.changed_output = Some(output);
        demand
    }

    /// A relation-only inverse has no field correlation role.
    pub fn relations_only(
        relations: impl IntoIterator<Item = super::DeclaredPreImageRelationLocus<Schema>>,
        maximum_encoded_bytes: usize,
    ) -> Self {
        Self::new([], maximum_encoded_bytes).with_relations(relations)
    }

    /// Records exact prior endpoints for each relation kind changed by this actor.
    pub fn with_relations(
        mut self,
        relations: impl IntoIterator<Item = super::DeclaredPreImageRelationLocus<Schema>>,
    ) -> Self {
        self.created_entity = false;
        self.relations = relations.into_iter().collect();
        self
    }

    pub(crate) fn validate(&self) -> Result<(), DeclaredPreImageDemandDenial> {
        if let Some(output) = self.changed_output {
            use crate::application_operation::{
                ApplicationMutationOutputPosture as Posture,
                ApplicationMutationOutputRoleCardinality as Cardinality,
            };
            if output.posture() != Posture::Preserve
                || output.cardinality() != Cardinality::ExactlyOne
                || self.loci.is_empty()
                || self
                    .loci
                    .iter()
                    .any(|locus| locus.entity() != output.entity())
            {
                return Err(DeclaredPreImageDemandDenial::InvalidChangedFieldOutput);
            }
        }
        if self.created_entity {
            return Ok(());
        }
        if self.loci.is_empty() && self.relations.is_empty() {
            return Err(DeclaredPreImageDemandDenial::EmptyDemand);
        }
        if self.maximum_encoded_bytes == 0 {
            return Err(DeclaredPreImageDemandDenial::ZeroByteBound);
        }
        validate_loci(&self.loci)?;
        for (index, locus) in self.relations.iter().enumerate() {
            if locus.relation().trim().is_empty()
                || locus.from().trim().is_empty()
                || locus.to().trim().is_empty()
            {
                return Err(DeclaredPreImageDemandDenial::EmptyLocusAxis);
            }
            if self.relations[..index].contains(locus) {
                return Err(DeclaredPreImageDemandDenial::DuplicateLocus);
            }
        }
        Ok(())
    }
    pub fn relations(&self) -> &[super::DeclaredPreImageRelationLocus<Schema>] {
        &self.relations
    }

    pub fn loci(&self) -> &[DeclaredPreImageLocus<Schema>] {
        &self.loci
    }

    pub const fn maximum_encoded_bytes(&self) -> usize {
        self.maximum_encoded_bytes
    }

    pub(super) fn into_portable(self) -> PortablePreImageDemand {
        PortablePreImageDemand {
            changed_output: self.changed_output.map(|output| {
                super::PortableChangedFieldOutput::from_untrusted_fields(
                    output.name().to_owned(),
                    output.entity().to_owned(),
                )
            }),
            loci: self
                .loci
                .into_iter()
                .map(DeclaredPreImageLocus::into_portable)
                .collect(),
            relations: self
                .relations
                .into_iter()
                .map(super::DeclaredPreImageRelationLocus::into_portable)
                .collect(),
            maximum_encoded_bytes: self.maximum_encoded_bytes,
        }
    }
}

fn validate_loci<Schema>(
    loci: &[DeclaredPreImageLocus<Schema>],
) -> Result<(), DeclaredPreImageDemandDenial> {
    if loci.iter().any(|locus| {
        locus.entity().trim().is_empty()
            || locus.aspect().trim().is_empty()
            || locus.field().trim().is_empty()
    }) {
        return Err(DeclaredPreImageDemandDenial::EmptyLocusAxis);
    }
    if loci
        .iter()
        .enumerate()
        .any(|(index, locus)| loci[..index].contains(locus))
    {
        return Err(DeclaredPreImageDemandDenial::DuplicateLocus);
    }
    Ok(())
}

impl<Schema> Clone for DeclaredPreImageDemand<Schema> {
    fn clone(&self) -> Self {
        Self {
            loci: self.loci.clone(),
            relations: self.relations.clone(),
            maximum_encoded_bytes: self.maximum_encoded_bytes,
            created_entity: self.created_entity,
            changed_output: self.changed_output,
        }
    }
}

impl<Schema> fmt::Debug for DeclaredPreImageDemand<Schema> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeclaredPreImageDemand")
            .field("loci", &self.loci)
            .field("relations", &self.relations)
            .field("maximum_encoded_bytes", &self.maximum_encoded_bytes)
            .field("created_entity", &self.created_entity)
            .field("changed_output", &self.changed_output)
            .finish()
    }
}

impl<Schema> PartialEq for DeclaredPreImageDemand<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.loci == other.loci
            && self.relations == other.relations
            && self.maximum_encoded_bytes == other.maximum_encoded_bytes
            && self.created_entity == other.created_entity
            && self.changed_output == other.changed_output
    }
}

impl<Schema> Eq for DeclaredPreImageDemand<Schema> {}

impl<Schema> PartialOrd for DeclaredPreImageDemand<Schema> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema> Ord for DeclaredPreImageDemand<Schema> {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            &self.loci,
            &self.relations,
            self.maximum_encoded_bytes,
            self.created_entity,
            self.changed_output,
        )
            .cmp(&(
                &other.loci,
                &other.relations,
                other.maximum_encoded_bytes,
                other.created_entity,
                other.changed_output,
            ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeclaredPreImageDemandDenial {
    EmptyDemand,
    EmptyLocusAxis,
    ZeroByteBound,
    DuplicateLocus,
    InvalidChangedFieldOutput,
}
