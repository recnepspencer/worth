//! Field correlation and relation endpoint demands for a recorded inverse.
use super::{DeclaredPreImageLocus, PortablePreImageDemand};
use std::{cmp::Ordering, fmt};
/// Schema-affine retained pre-image demand required by an inverse.
pub struct DeclaredPreImageDemand<Schema> {
    loci: Vec<DeclaredPreImageLocus<Schema>>,
    relations: Vec<super::DeclaredPreImageRelationLocus<Schema>>,
    maximum_encoded_bytes: usize,
}

impl<Schema> DeclaredPreImageDemand<Schema> {
    pub(super) fn created_entity() -> Self {
        Self {
            loci: Vec::new(),
            relations: Vec::new(),
            maximum_encoded_bytes: 0,
        }
    }
    pub fn new(
        loci: impl IntoIterator<Item = DeclaredPreImageLocus<Schema>>,
        maximum_encoded_bytes: usize,
    ) -> Result<Self, DeclaredPreImageDemandDenial> {
        let loci = loci.into_iter().collect::<Vec<_>>();
        validate_preimage_demand(&loci, maximum_encoded_bytes)?;
        Ok(Self {
            loci,
            relations: Vec::new(),
            maximum_encoded_bytes,
        })
    }

    /// A relation-only inverse has no field correlation role.
    pub fn relations_only(
        relations: impl IntoIterator<Item = super::DeclaredPreImageRelationLocus<Schema>>,
        maximum_encoded_bytes: usize,
    ) -> Result<Self, DeclaredPreImageDemandDenial> {
        Self {
            loci: Vec::new(),
            relations: Vec::new(),
            maximum_encoded_bytes,
        }
        .with_relations(relations)
    }

    /// Capture exact prior endpoints for each declared relation kind changed by this actor.
    pub fn with_relations(
        mut self,
        relations: impl IntoIterator<Item = super::DeclaredPreImageRelationLocus<Schema>>,
    ) -> Result<Self, DeclaredPreImageDemandDenial> {
        let relations = relations.into_iter().collect::<Vec<_>>();
        if self.loci.is_empty() && relations.is_empty() {
            return Err(DeclaredPreImageDemandDenial::EmptyDemand);
        }
        if self.maximum_encoded_bytes == 0 {
            return Err(DeclaredPreImageDemandDenial::ZeroByteBound);
        }
        for (index, locus) in relations.iter().enumerate() {
            if locus.relation().trim().is_empty()
                || locus.from().trim().is_empty()
                || locus.to().trim().is_empty()
            {
                return Err(DeclaredPreImageDemandDenial::EmptyLocusAxis);
            }
            if relations[..index].contains(locus) {
                return Err(DeclaredPreImageDemandDenial::DuplicateLocus);
            }
        }
        self.relations = relations;
        Ok(self)
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

fn validate_preimage_demand<Schema>(
    loci: &[DeclaredPreImageLocus<Schema>],
    maximum_encoded_bytes: usize,
) -> Result<(), DeclaredPreImageDemandDenial> {
    if loci.is_empty() {
        return Err(DeclaredPreImageDemandDenial::EmptyDemand);
    }
    if maximum_encoded_bytes == 0 {
        return Err(DeclaredPreImageDemandDenial::ZeroByteBound);
    }
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
    if loci
        .iter()
        .skip(1)
        .any(|locus| locus.entity() != loci[0].entity())
    {
        return Err(DeclaredPreImageDemandDenial::MultipleEntityRoles);
    }
    Ok(())
}

impl<Schema> Clone for DeclaredPreImageDemand<Schema> {
    fn clone(&self) -> Self {
        Self {
            loci: self.loci.clone(),
            relations: self.relations.clone(),
            maximum_encoded_bytes: self.maximum_encoded_bytes,
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
            .finish()
    }
}

impl<Schema> PartialEq for DeclaredPreImageDemand<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.loci == other.loci
            && self.relations == other.relations
            && self.maximum_encoded_bytes == other.maximum_encoded_bytes
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
        (&self.loci, &self.relations, self.maximum_encoded_bytes).cmp(&(
            &other.loci,
            &other.relations,
            other.maximum_encoded_bytes,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeclaredPreImageDemandDenial {
    EmptyDemand,
    EmptyLocusAxis,
    ZeroByteBound,
    DuplicateLocus,
    MultipleEntityRoles,
}
