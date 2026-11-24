//! Untrusted portable records cannot smuggle filter operands past schema
//! closure: identifiers, fields, scalar families and presence are rechecked.
use super::schema::Schema;
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
use worth_query_decl::facade::application_query::{
    ApplicationQueryPredicate, ApplicationQueryWhereFilterOperand as Operand,
    ApplicationQueryWhereFilterProgram as Program, ApplicationQueryWhereFilterSource as Source,
    ErasedApplicationQueryDefinition,
};
use worth_query_decl::facade::application_schema::*;

fn task_field(aspect: &str, field: &str, scalar: ScalarAspectType, optional: bool) -> Source {
    Source::Field {
        entity: "Task".into(),
        aspect: aspect.into(),
        field: field.into(),
        scalar_family: scalar,
        presence: if optional {
            ApplicationFieldPresence::Optional
        } else {
            ApplicationFieldPresence::Required
        },
    }
}

/// The board schema with `task_search`'s filter replaced by `x == x` over
/// one untrusted operand source.
fn readmit(source: Option<Source>) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let declaration = Schema::declaration().unwrap().into_erased();
    let mut parts = WorthQueryPortableApplicationSchemaRecord::project(&declaration).into_parts();
    for member in &mut parts.members {
        let ApplicationSchemaMember::ApplicationQuery { definition } = member else {
            continue;
        };
        if definition.name() != "task_search" {
            continue;
        }
        let Some(source) = source.clone() else {
            continue;
        };
        let mut query = definition.clone().into_parts();
        let draft = worth_foundational::expression_api::expressions()
            .parse("x == x")
            .unwrap()
            .encode();
        query.where_filter = Some(
            Program::from_untrusted(vec![Operand::from_untrusted("x".into(), source)], &draft)
                .expect("the untrusted program is well-typed on its own"),
        );
        *definition = ErasedApplicationQueryDefinition::from_untrusted_parts(query);
    }
    validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
    )
    .map(|_| ())
}

#[test]
fn readmission_refuses_forged_filter_operands_and_accepts_the_genuine_record() {
    let aspect = || {
        let declaration = Schema::declaration().unwrap().into_erased();
        declaration
            .members()
            .iter()
            .find_map(|member| match member {
                ApplicationSchemaMember::Field {
                    entity,
                    aspect,
                    field,
                    ..
                } if entity == "Task" && field == "Rank" => Some(aspect.clone()),
                _ => None,
            })
            .expect("the board declares Task.Rank")
    };
    assert_eq!(readmit(None), Ok(()));
    let genuine = task_field(&aspect(), "Rank", ScalarAspectType::Int64, false);
    assert_eq!(
        readmit(Some(genuine)),
        Ok(()),
        "a genuine root field readmits"
    );
    for forged in [
        task_field("", "Rank", ScalarAspectType::Int64, false),
        task_field(&aspect(), " Rank", ScalarAspectType::Int64, false),
        task_field(&aspect(), "Ghost", ScalarAspectType::Int64, false),
        task_field(&aspect(), "Rank", ScalarAspectType::Int32, false),
        task_field(&aspect(), "Rank", ScalarAspectType::Int64, true),
        task_field(&aspect(), "Due", ScalarAspectType::Date, false),
    ] {
        assert!(
            readmit(Some(forged.clone())).is_err(),
            "{forged:?} must refuse"
        );
    }

    // Forged optional parameter operand against required `q`, and forged
    // required parameter operand against optional `max_due`, both refuse.
    assert!(readmit(Some(Source::OptionalParameter {
        parameter: "q".into(),
        scalar_family: ScalarAspectType::String,
    }))
    .is_err());

    let readmit_query = |query_name: &str,
                         mutate: &dyn Fn(&mut worth_query_decl::facade::application_query::WorthQueryPortableApplicationQueryParts)| {
        let declaration = Schema::declaration().unwrap().into_erased();
        let mut parts =
            WorthQueryPortableApplicationSchemaRecord::project(&declaration).into_parts();
        for member in &mut parts.members {
            let ApplicationSchemaMember::ApplicationQuery { definition } = member else {
                continue;
            };
            if definition.name() != query_name {
                continue;
            }
            let mut query = definition.clone().into_parts();
            mutate(&mut query);
            *definition = ErasedApplicationQueryDefinition::from_untrusted_parts(query);
        }
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .map(|_| ())
    };

    assert!(readmit_query("task_due", &|query| {
        let draft = worth_foundational::expression_api::expressions()
            .parse("max_due == max_due")
            .unwrap()
            .encode();
        query.where_filter = Some(
            Program::from_untrusted(
                vec![Operand::from_untrusted(
                    "max_due".into(),
                    Source::Parameter {
                        parameter: "max_due".into(),
                        scalar_family: ScalarAspectType::Date,
                    },
                )],
                &draft,
            )
            .unwrap(),
        );
    })
    .is_err());

    // Forged literal value predicate with wrong scalar family or non-equality field refuses.
    let task_aspect = aspect();
    assert!(readmit_query("open_inbox", &|query| {
        query.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
            "Task".into(),
            task_aspect.clone(),
            "Status".into(),
            AspectValue::Int64(1),
            ScalarAspectType::String,
        )];
    })
    .is_err());
    assert!(readmit_query("open_inbox", &|query| {
        query.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
            "Task".into(),
            task_aspect.clone(),
            "Title".into(),
            AspectValue::String(InternedString::Raw("Mill housing".into())),
            ScalarAspectType::String,
        )];
    })
    .is_err());
}
