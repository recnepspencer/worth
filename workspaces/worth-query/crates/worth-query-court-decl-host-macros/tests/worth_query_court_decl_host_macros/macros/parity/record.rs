//! A derived schema root, fragment, entity and record, with persisted-name
//! and column overrides, declare exactly the schema their hand-written trait
//! impls declare.
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{WorthQueryRecord, WorthQuerySchema};

mod derived {
    use super::*;

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        schema,
        owner = "worth.query.macros.parity",
        name = "parity_notes",
        members(Notes)
    )]
    pub struct Root;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(fragment, name = "notes", bindings = NativeScalars, members(Note))]
    pub struct Notes;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(entity, aspects(NoteFacts))]
    pub struct Note;
    #[derive(WorthQueryRecord)]
    #[worth_query(bindings = NativeScalars, aspect = NoteFacts, fragment = Notes, entity = Note)]
    pub struct NoteFactsValue {
        #[worth_query(equality, read_only)]
        pub note_key: u64,
        #[worth_query(column = "LegacyBody")]
        pub body: String,
        pub due: Option<CanonicalDate>,
    }
}

mod written {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Root;
    impl Fragment for Root {
        const NAME: &'static str = "parity_notes";
        type Bindings = NoBindings;
        type Members = (Notes,);
    }
    impl ApplicationSchema for Root {
        const OWNER: &'static str = "worth.query.macros.parity";
        const NAME: &'static str = <Self as Fragment>::NAME;
        fn declaration(
        ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>
        {
            declare(Self)
        }
    }
    impl ApplicationSchemaComposition for Root {
        type Contributions = <Self as Fragment>::Members;
    }
    #[derive(Clone, Copy)]
    pub struct Notes;
    impl Fragment for Notes {
        const NAME: &'static str = "notes";
        type Bindings = NativeScalars;
        type Members = (Note,);
    }
    #[derive(Clone, Copy)]
    pub struct Note;
    impl Entity for Note {
        const NAME: &'static str = "Note";
        type Aspects = (NoteFacts,);
        type Relations = ();
    }
    pub struct NoteFacts;
    impl Aspect for NoteFacts {
        const NAME: &'static str = "NoteFacts";
        type Row = NoteFactsValue;
    }
    pub struct NoteFactsValue {
        note_key: u64,
        body: String,
        due: Option<CanonicalDate>,
    }
    impl Record<NoteFactsValue> for NativeScalars {
        fn record() -> impl Shape<NoteFactsValue, Self> {
            field("NoteKey", |row: &NoteFactsValue| row.note_key)
                .equality_queryable(true)
                .read_only()
                .and("LegacyBody", |row: &NoteFactsValue| row.body.clone())
                .and("Due", |row: &NoteFactsValue| row.due)
                .build(|note_key, body, due, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(NoteFactsValue {
                        note_key,
                        body,
                        due,
                    })
                })
        }
    }
}

#[test]
fn derived_schema_markers_and_records_declare_the_written_schema() {
    let derived = derived::Root::declaration().expect("the derived root declares");
    let written = written::Root::declaration().expect("the written root declares");
    assert_eq!(derived.identity(), written.identity());
    assert_eq!(derived.erased(), written.erased());
    // Each selection alias is named after its column, overrides included,
    // and selects the optional column's value type.
    type Written<T> = FieldSelection<written::Notes, written::Note, written::NoteFacts, T>;
    assert_eq!(
        derived::LegacyBody::at("LegacyBody").binding_recipe(),
        Written::<String>::at("LegacyBody").binding_recipe()
    );
    assert_eq!(
        derived::Due::at("Due").binding_recipe(),
        Written::<CanonicalDate>::at("Due").binding_recipe()
    );
}
