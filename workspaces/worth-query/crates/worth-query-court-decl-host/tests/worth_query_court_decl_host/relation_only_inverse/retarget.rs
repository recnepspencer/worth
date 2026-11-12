//! Retarget one existing edge; a static inverse demand also names an unchanged kind.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Input {
    pub id: u64,
    pub old: u64,
    pub target: u64,
}
worth_query_structured_value_binding!(pub InputValue for Input {identity: "relation-only-inverse.retarget.input.v1"});
worth_query_operation!(pub Retarget for Schema, input InputValue; access {
    read_entities:[Node]; read_fields:[Id::at("Id")];
    read_relations:[Holds,Unrelated]; links:[Holds]; unlinks:[Holds];
});
impl OperationReads<Retarget> for Id {}
pub struct Targets {
    from: Target,
    old: Target,
    to: Target,
}
fn scope(_: &Input) -> u64 {
    SCOPE
}
worth_query_mutation_binding!(pub Binding for Input, schema Schema,
    identity "relation-only-inverse.retarget.v1", input InputValue,
    operation Retarget, result DoneBinding,
    idempotency String, identity "relation-only-inverse.retarget.key.v1",
    decision Targets, denial RefusedBinding, handler identity "relation-only-inverse.retarget.handler.v1",
    program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id, field Id::at("Id"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Targets, Refused> {
        let result = (|| {
            let from = reader.resolve_entity(Id::at("Id"), input.id)?;
            let to = reader.resolve_entity(Id::at("Id"), input.target)?;
            reader.entity(&from, Node::reference())?;
            reader.entity(&to, Node::reference())?;
            let edges = reader
                .relations_from(Holds::reference(), &from)
                .map_err(HandlerExecutionDenial::new)?;
            assert_eq!(edges.len(), 1, "one existing held edge");
            let old = edges[0].to();
            reader.entity(old, Node::reference())?;
            assert_eq!(
                reader.field(old, Id::at("Id"))?,
                Some(input.old),
                "the decision observes its expected old edge"
            );
            Ok::<_, HandlerExecutionDenial>(Targets {
                from: reader.mutation_target(&from)?,
                old: reader.mutation_target(old)?,
                to: reader.mutation_target(&to)?,
            })
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        targets: Targets,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Done, Refused> {
        let result = (|| {
            let from = writer.projected_entity(&targets.from)?;
            let old = writer.projected_entity(&targets.old)?;
            let to = writer.projected_entity(&targets.to)?;
            writer.unlink(Holds::reference(), &from, &old)?;
            writer.link(
                Holds::reference(),
                format!("retarget-{}-{}", input.id, input.target),
                &from,
                &to,
            )?;
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Done),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub struct Undo;
impl ApplicationCorrectionBinding<Schema> for Undo {
    const IDENTITY: &'static str = "relation-only-inverse.retarget.recorded.v1";
    type Source = Binding;
    type InverseIntent = Input;
    type Disclosure = query::Binding;
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, Retarget, Input, true, false> {
        Retarget::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Input,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::relations_only(
                [
                    DeclaredPreImageRelationLocus::from_relation(Unrelated::reference()),
                    DeclaredPreImageRelationLocus::from_relation(Holds::reference()),
                ],
                4096,
            ),
        )
    }
    fn derive_correction(
        original: &Input,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Input, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(image) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        assert_eq!(
            image.relation_count(&DeclaredPreImageRelationLocus::from_relation(
                Unrelated::reference()
            )),
            Some(0),
            "an unchanged demanded kind has no changed rows, even with a live edge"
        );
        let locus = DeclaredPreImageRelationLocus::from_relation(Holds::reference());
        assert_eq!(
            image.relation_count(&locus),
            Some(2),
            "only the removed and born pair are retained"
        );
        let rows = [
            image.relation_at(&locus, 0).ok_or(MissingPreImage)?,
            image.relation_at(&locus, 1).ok_or(MissingPreImage)?,
        ];
        assert_eq!(rows.iter().filter(|row| row.before().is_some()).count(), 1);
        assert_eq!(rows.iter().filter(|row| row.after().is_some()).count(), 1);
        // The original guarded decision certified the old logical key. The
        // retained pair is the inverse's native completeness witness.
        Ok(Input {
            id: original.id,
            old: original.target,
            target: original.old,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("retarget:inverse:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
