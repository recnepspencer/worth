//! Decode only the installed provenance profile and validate its certified correspondence.
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout as Layout;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationFieldPresence as Presence,
    ApplicationOperationBodyCaptureRole as Role, ApplicationOperationBodyCreatedField as Field,
    ApplicationReadableScalarValueBinding, ApplicationSigningEntityCoordinates as Coordinates,
    ApplicationSigningGraphSubjectProvenance as Graph,
    ApplicationSigningGraphSubjectProvenanceBinding as GraphBinding,
    ApplicationSigningProvenanceEncoding as Encoding,
    ApplicationSigningProvenanceParts as Authentication,
    ApplicationSigningSubjectObservedFieldVersion as Observed,
    ApplicationSigningSubjectProvenance as Scope,
    ApplicationSigningSubjectProvenanceBinding as ScopeBinding,
    ApplicationSigningSubjectSelectionDependency as Selection,
};
enum PublishedSubjectData {
    Scope(Scope),
    Graph(Graph),
}
pub(in crate::domain_computation::primary_graph) struct PublishedCertifiedSubject(
    PublishedSubjectData,
);
impl PublishedCertifiedSubject {
    pub(in crate::domain_computation::primary_graph) fn decode(
        layout: &Layout,
        field: &Locus,
        raw: &AspectValue,
    ) -> Result<Self, ()> {
        match layout.signing_provenance_encoding(field).ok_or(())? {
            Encoding::CertifiedSubjectV2 => ScopeBinding::decode(raw)
                .map(|value| Self(PublishedSubjectData::Scope(value)))
                .map_err(|_| ()),
            Encoding::CertifiedGraphSubjectV3 => {
                let value = GraphBinding::decode(raw).map_err(|_| ())?;
                validate_graph(layout, field, &value)?;
                Ok(Self(PublishedSubjectData::Graph(value)))
            }
            Encoding::AuthenticationV1 => Err(()),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn authentication(&self) -> &Authentication {
        match &self.0 {
            PublishedSubjectData::Scope(value) => value.authentication(),
            PublishedSubjectData::Graph(value) => value.authentication(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn dependencies(
        &self,
    ) -> impl Iterator<Item = (Coordinates, &Locus)> {
        let (scope, graph) = match &self.0 {
            PublishedSubjectData::Scope(value) => (Some(value.dependencies()), None),
            PublishedSubjectData::Graph(value) => (None, Some(value.dependencies())),
        };
        scope
            .into_iter()
            .flatten()
            .map(|d| (d.source, &d.source_field))
            .chain(
                graph
                    .into_iter()
                    .flatten()
                    .map(|d| (d.source, &d.source_field)),
            )
    }
    pub(in crate::domain_computation::primary_graph) fn selections(&self) -> &[Selection] {
        match &self.0 {
            PublishedSubjectData::Scope(_) => &[],
            PublishedSubjectData::Graph(value) => value.selections(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn source_for_capture(
        &self,
        capture: &Locus,
    ) -> Option<&Locus> {
        match &self.0 {
            PublishedSubjectData::Scope(value) => value
                .dependencies()
                .iter()
                .find(|d| &d.captured_field == capture)
                .map(|d| &d.source_field),
            PublishedSubjectData::Graph(value) => value
                .dependencies()
                .iter()
                .find(|d| &d.captured_field == capture)
                .map(|d| &d.source_field),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn has_valid_subject(
        &self,
        subject: Coordinates,
    ) -> bool {
        self.authentication().subject == subject
            && match &self.0 {
                PublishedSubjectData::Scope(value) => {
                    value.dependencies().iter().all(|d| d.source == subject)
                }
                // V3's codec and compiled creation contract prove graph correspondence.
                PublishedSubjectData::Graph(_) => true,
            }
    }
}

fn validate_graph(layout: &Layout, locus: &Locus, value: &Graph) -> Result<(), ()> {
    let metadata = &value.authentication().metadata;
    let (scope, creation) = layout
        .signed_creation_contract(&metadata.operation)
        .ok_or(())?;
    let signing = creation.signing().ok_or(())?;
    if signing.provenance() != locus
        || signing.purpose() != metadata.purpose
        || !creation.fields().iter().any(|step| {
            matches!(step, Field::PrincipalIdentity {field, mutation_binding, ..}
            if field == signing.actor() && mutation_binding == &metadata.binding)
        })
    {
        return Err(());
    }
    let mut count = 0;
    for step in creation.fields() {
        let (source, binding, source_presence, target_presence, path) = match step {
            Field::CapturedScopeField {
                source,
                binding,
                source_presence,
                role: Role::Subject,
                ..
            } => (source, binding, *source_presence, Presence::Required, None),
            Field::CapturedPathField {
                source,
                binding,
                role: Role::Subject,
                target_presence,
                ..
            } => {
                if !source.has_supported_shape() || source.path.start_entity() != scope {
                    return Err(());
                }
                (
                    &source.field,
                    binding,
                    source.presence,
                    *target_presence,
                    Some(&source.path),
                )
            }
            _ => continue,
        };
        count += 1;
        let dependency = value
            .dependencies()
            .iter()
            .find(|d| d.captured_field == *step.field())
            .ok_or(())?;
        if dependency.source_field != *source || dependency.value_binding != binding.as_str() {
            return Err(());
        }
        if matches!(dependency.observed_version, Observed::Absent { .. })
            && (source_presence != Presence::Optional || target_presence != Presence::Optional)
        {
            return Err(());
        }
        let mut selections = value
            .selections()
            .iter()
            .filter(|s| s.captured_field == *step.field());
        for hop in path.into_iter().flat_map(|path| path.steps()) {
            let observed = selections.next().ok_or(())?;
            if observed.relation != hop.relation() || observed.direction != hop.direction() {
                return Err(());
            }
        }
        if selections.next().is_some() {
            return Err(());
        }
    }
    if count != value.dependencies().len() {
        return Err(());
    }
    Ok(())
}
