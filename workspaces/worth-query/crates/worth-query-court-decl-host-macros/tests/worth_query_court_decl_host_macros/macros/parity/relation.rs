//! A derived projection that traverses relations forward and in reverse
//! declares exactly the nested shape written with relation selectors.
use super::super::relations::*;
use super::super::schema::*;
use super::board;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::portable_identity::WorthQueryPortableType;
use worth_query_decl::facade::worth_query_portable_type;

/// The same slots written as markers, identities, selectors and builders.
/// `Owner` selects the reverse relation's slot, so a twin can differ there.
mod written {
    use super::*;

    pub struct KeySlot;
    pub struct TasksSlot;
    pub struct TaskKeySlot;
    pub struct TitleSlot;
    pub struct OwnerSlot;
    pub struct OtherOwnerSlot;
    pub struct OwnerKeySlot;
    worth_query_portable_type!(KeySlot => "macros.projects.outline.v1.slot.key");
    worth_query_portable_type!(TasksSlot => "macros.projects.outline.v1.slot.tasks");
    worth_query_portable_type!(TaskKeySlot => "macros.projects.outline.v1.slot.task_key");
    worth_query_portable_type!(TitleSlot => "macros.projects.outline.v1.slot.title");
    worth_query_portable_type!(OwnerSlot => "macros.projects.owner.v1");
    worth_query_portable_type!(OtherOwnerSlot => "macros.projects.owner.v2");
    worth_query_portable_type!(OwnerKeySlot => "macros.projects.outline.v1.slot.owner_key");

    pub fn tasks() -> ApplicationQueryResultRelationRef<
        OutlineQuery,
        TasksSlot,
        Schema,
        Contains,
        Project,
        Task,
        ForwardResultTraversal,
        ManyResults,
    > {
        ApplicationQueryResultRelationRef::forward_many("tasks", Contains::reference())
    }

    pub fn shape<Owner: WorthQueryPortableType>() -> TypedApplicationQueryResultShape<
        Schema,
        OutlineQuery,
        Project,
        ProjectOutline,
        OutlineBinding,
    > {
        let owner = ApplicationQueryResultShapeBuilder::<_, _, _, (), NestedBinding>::new(
            Project::reference(),
        )
        .field(ApplicationQueryResultFieldRef::<
            OutlineQuery,
            OwnerKeySlot,
            Schema,
            ProjectKey,
        >::new("owner_key", ProjectKey::at("ProjectKey")));
        let task = ApplicationQueryResultShapeBuilder::<_, _, _, (), NestedBinding>::new(
            Task::reference(),
        )
        .field(ApplicationQueryResultFieldRef::<
            OutlineQuery,
            TaskKeySlot,
            Schema,
            TaskKey,
        >::new("task_key", TaskKey::at("TaskKey")))
        .field(ApplicationQueryResultFieldRef::<
            OutlineQuery,
            TitleSlot,
            Schema,
            Title,
        >::new("title", Title::at("Title")))
        .relation(
            ApplicationQueryResultRelationRef::<
                OutlineQuery,
                Owner,
                Schema,
                Contains,
                Project,
                Task,
                ReverseResultTraversal,
                ExactlyOneResult,
            >::reverse_one("owner", Contains::reference()),
            owner,
        );
        ApplicationQueryResultShapeBuilder::new(Project::reference())
            .field(ApplicationQueryResultFieldRef::<
                OutlineQuery,
                KeySlot,
                Schema,
                ProjectKey,
            >::new("key", ProjectKey::at("ProjectKey")))
            .relation(tasks(), task)
            .build()
    }
}

fn declared(
    shape: TypedApplicationQueryResultShape<
        Schema,
        OutlineQuery,
        Project,
        ProjectOutline,
        OutlineBinding,
    >,
) -> ApplicationSchemaDeclaration<Schema> {
    board()
        .application_query(outline_with(shape))
        .build()
        .expect("the outline query declares")
}

#[test]
fn a_derived_relation_projection_declares_the_written_nested_shape() {
    let derived = declared(ProjectOutline::result_shape());
    let written = declared(written::shape::<written::OwnerSlot>());
    assert_eq!(derived.identity(), written.identity());
    assert_eq!(derived.erased(), written.erased());
    let twin = declared(written::shape::<written::OtherOwnerSlot>());
    assert_ne!(derived.erased(), twin.erased());
    assert_eq!(
        ProjectOutline::tasks::<OutlineQuery>().slot_key(),
        written::tasks().slot_key()
    );
}

#[test]
fn derived_relation_matches_manual_relation_integrity_and_reference() {
    mod derived_rel {
        use worth_query_decl::facade::application_schema::fragment::NativeScalars;
        use worth_query_decl::facade::application_schema::*;
        use worth_query_decl::facade::{WorthQueryRecord, WorthQueryRelation, WorthQuerySchema};

        #[derive(Clone, Copy, WorthQuerySchema)]
        #[worth_query(
            schema,
            owner = "macros.parity.rel",
            name = "rel_parity",
            members(Graph)
        )]
        pub struct Root;
        #[derive(Clone, Copy, WorthQuerySchema)]
        #[worth_query(fragment, name = "graph", bindings = NativeScalars, members(Parent, Child))]
        pub struct Graph;
        #[derive(Clone, Copy, WorthQuerySchema)]
        #[worth_query(entity, aspects(ParentFacts), relations(Owns))]
        pub struct Parent;
        #[derive(Clone, Copy, WorthQuerySchema)]
        #[worth_query(entity, aspects(ChildFacts))]
        pub struct Child;
        #[derive(WorthQueryRelation)]
        #[worth_query(
            to = Child,
            from = Parent,
            reference(Root, Graph),
            acyclic,
            self_edges = false,
            unique = ApplicationRelationUniquenessScope::DirectedSemanticEdge,
        )]
        pub struct Owns;
        #[derive(WorthQueryRecord)]
        #[worth_query(bindings = NativeScalars, aspect = ParentFacts, fragment = Graph, entity = Parent)]
        pub struct ParentRow {
            #[worth_query(column = "ParentKey", equality, read_only)]
            pub key: u64,
        }
        #[derive(WorthQueryRecord)]
        #[worth_query(bindings = NativeScalars, aspect = ChildFacts, fragment = Graph, entity = Child)]
        pub struct ChildRow {
            #[worth_query(column = "ChildKey", equality, read_only)]
            pub key: u64,
        }
    }

    mod written_rel {
        use worth_query_decl::facade::application_schema::fragment::{
            Entity, NativeScalars, Relation,
        };
        use worth_query_decl::facade::application_schema::*;
        use worth_query_decl::facade::{WorthQueryRecord, WorthQuerySchema};

        #[derive(Clone, Copy, WorthQuerySchema)]
        #[worth_query(
            schema,
            owner = "macros.parity.rel",
            name = "rel_parity",
            members(Graph)
        )]
        pub struct Root;
        #[derive(Clone, Copy, WorthQuerySchema)]
        #[worth_query(fragment, name = "graph", bindings = NativeScalars, members(Parent, Child))]
        pub struct Graph;
        #[derive(Clone, Copy)]
        pub struct Parent;
        impl Entity for Parent {
            const NAME: &'static str = "Parent";
            type Aspects = (ParentFacts,);
            type Relations = (Owns,);
        }
        #[derive(Clone, Copy, WorthQuerySchema)]
        #[worth_query(entity, aspects(ChildFacts))]
        pub struct Child;
        pub struct Owns;
        impl Relation for Owns {
            const NAME: &'static str = "Owns";
            type To = Child;
            const ACYCLIC: bool = true;
            const SELF_EDGES: bool = false;
            const UNIQUE: Option<ApplicationRelationUniquenessScope> =
                Some(ApplicationRelationUniquenessScope::DirectedSemanticEdge);
        }
        impl Owns {
            pub fn reference() -> ApplicationRelationRef<Root, Self, Parent, Child> {
                worth_query_decl::facade::application_schema::fragment::relation(
                    Graph, Parent, Self,
                )
            }
        }
        #[derive(WorthQueryRecord)]
        #[worth_query(bindings = NativeScalars, aspect = ParentFacts, fragment = Graph, entity = Parent)]
        pub struct ParentRow {
            #[worth_query(column = "ParentKey", equality, read_only)]
            pub key: u64,
        }
        #[derive(WorthQueryRecord)]
        #[worth_query(bindings = NativeScalars, aspect = ChildFacts, fragment = Graph, entity = Child)]
        pub struct ChildRow {
            #[worth_query(column = "ChildKey", equality, read_only)]
            pub key: u64,
        }
    }

    let derived = derived_rel::Root::declaration().expect("derived relation schema declares");
    let written = written_rel::Root::declaration().expect("written relation schema declares");
    assert_eq!(derived.identity(), written.identity());
    assert_eq!(derived.erased(), written.erased());
    assert_eq!(
        derived_rel::Owns::reference().integrity(),
        written_rel::Owns::reference().integrity()
    );
    assert_eq!(
        derived_rel::Owns::reference().name(),
        written_rel::Owns::reference().name()
    );

    let declaration = board().build().unwrap();
    let law = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Relation {
                relation,
                integrity,
                ..
            } if relation == "Contains" => Some(integrity),
            _ => None,
        })
        .expect("the derived Contains relation is declared");
    assert!(law.acyclic);
    assert!(!law.endpoints.self_edges_allowed);
    assert_eq!(
        law.unique,
        Some(ApplicationRelationUniquenessScope::DirectedSemanticEdge)
    );
}
