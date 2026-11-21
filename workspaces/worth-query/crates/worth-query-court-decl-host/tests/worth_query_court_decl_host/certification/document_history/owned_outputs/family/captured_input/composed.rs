//! Two stored captured readers in one stage, reached across ranks (#254).
//!
//! A model edit roots `first:carriage:late`. It owes the stored reader
//! `captured:capture:p` and the ordinary route `first:capture:s`. The reader
//! `p` owes the writer `first:carriage:d`; the route `s` reaches the stored
//! reader `captured:capture:r`, whose decision reads `first:carriage:d`
//! without declaring it. The two branches are independent, and no discovery
//! reads a value the edit changes. Reader `r` also owes `first:capture:t`.
//! With waits, the writer waits on `p` and `first:capture:t` waits on `r`.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::*;

macro_rules! connection {
    ($binding:ident, $connection:ident, $source:ty, $output:ty, $input:ty, $root:path, $owed:literal, $identity:literal) => {
        pub struct $binding;
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, $source, hierarchy::Parent>
            for $binding
        {
        }
        type $connection =
            ApplicationConnectionRef<Schema, $source, $output, hierarchy::Parent, $input, $binding>;
        /// Every connection into a route reads its prerequisites. A member
        /// `wait:<key>` of model `waits:<target>` makes an owed target wait
        /// on the stored reader `<key>`; other targets read the empty model.
        impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for $binding {
            type Prerequisites = query::members::Read;
            type PrerequisiteRow = query::RowBinding;
            type Selection = ApplicationQueryResultRelationRef<
                query::members::Query,
                query::members::MembersSlot,
                Schema,
                ModelMember,
                Model,
                Member,
                ForwardResultTraversal,
                ManyResults,
            >;
            fn prerequisite_rows() -> Self::Selection {
                query::members::members()
            }
            fn prerequisites_from_target(
                target: &first::Demand,
            ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(query::members::Read(match target.0.as_str() {
                    "carriage:d" | "capture:t" if $owed => format!("waits:{}", target.0),
                    _ => "composed:none".into(),
                }))
            }
            fn parent_from_prerequisite(
                row: &query::Row,
            ) -> Result<$root, WorthQueryRequiredOutputConnectionDenial> {
                Ok($root(row.key.trim_start_matches("wait:").to_owned()))
            }
        }
    };
}
connection!(
    Sibling,
    SiblingConnection,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::ContinuationInput,
    first::Demand,
    false,
    "captured-input.composed.sibling"
);
/// The root's ordinary route, the authored parent of reader `r`.
impl WorthQueryApplicationDependentOutputConnection<Schema> for Sibling {
    type RootDemand = first::Demand;
    type Demand = first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(match row.key.as_str() {
            "first:carriage:late" => vec![first::Demand("capture:s".into())],
            _ => Vec::new(),
        })
    }
}
connection!(
    Owed,
    OwedConnection,
    program::Feature,
    program::Output,
    hierarchy::PredecessorInput,
    Demand,
    true,
    "captured-input.composed.owed"
);
/// Reader `p` owes the writer of the value reader `r` reads; reader `r`
/// owes the route `first:capture:t`.
impl WorthQueryApplicationDependentOutputConnection<Schema> for Owed {
    type RootDemand = Demand;
    type Demand = first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("captured:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(match row.key.as_str() {
            "captured:capture:p" => vec![first::Demand("carriage:d".into())],
            "captured:capture:r" => vec![first::Demand("capture:t".into())],
            _ => Vec::new(),
        })
    }
}
pub struct Composed;
impl ApplicationOccurrenceComponent<Schema> for Composed {
    type Connections = (
        ApplicationCapturedInputConnection<program::SeedConnection, Capture>,
        ApplicationCapturedInputConnection<program::AnchorConnection, Capture>,
        SiblingConnection,
        OwedConnection,
    );
}
pub type Tree =
    ApplicationOutputGraph<supplier::SupplierConnection, ApplicationOutputOccurrences<Composed>>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.composed");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::feature_specs::<fixture::Delivery<Tree>>()
    }
}

/// Seed the composed readers, then `width` same-rank siblings that each read
/// `first:carriage:late` and declare `first:carriage:d`. With `waits`, the
/// writer `first:carriage:d` waits on reader `p` and the route
/// `first:capture:t` waits on reader `r`.
pub(in super::super::super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    width: usize,
    waits: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let siblings = (0..width).map(|k| format!("capture:w{k}"));
    let models = [
        ("carriage:late".into(), 5),
        ("carriage:d".into(), 4),
        ("capture:s".into(), 6),
        ("capture:p".into(), 2),
        ("capture:r".into(), 3),
        ("capture:t".into(), 7),
        ("composed:none".into(), 0),
        ("waits:carriage:d".into(), 0),
        ("waits:capture:t".into(), 0),
    ];
    for (key, value) in models
        .into_iter()
        .chain(siblings.clone().map(|key| (key, 3)))
    {
        let key = key.as_str();
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Model::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(ModelKey::at("ModelKey"), key.to_owned())
            .field(ModelValue::at("ModelValue"), value),
        )?;
    }
    let members = [
        (
            "capture:p".into(),
            "capture:p-late".into(),
            "carriage:late",
            1,
        ),
        ("capture:r".into(), "capture:r-s".into(), "capture:s", 1),
        (
            "capture:r".into(),
            "capture:r-d".into(),
            "",
            handler::UNDECLARED,
        ),
    ];
    let waiting = [
        ("waits:carriage:d".into(), "wait:capture:p".into(), "", 1),
        ("waits:capture:t".into(), "wait:capture:r".into(), "", 1),
    ];
    let members = members
        .into_iter()
        .chain(waiting.into_iter().filter(|_| waits));
    let wide = siblings.flat_map(|owner: String| {
        [("late", "carriage:late", 1), ("d", "carriage:d", 2)].map(|(role, parent, weight)| {
            (owner.clone(), format!("{owner}-{role}"), parent, weight)
        })
    });
    for (owner, key, parent, weight) in members.chain(wide) {
        let (owner, key): (String, String) = (owner, key);
        let (owner, key) = (owner.as_str(), key.as_str());
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Member::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(MemberKey::at("MemberKey"), key.to_owned())
            .field(MemberValue::at("MemberValue"), weight),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ModelMember::reference(),
            format!("owns:{key}"),
            WorthQueryApplicationEntityKey::new(owner).unwrap(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        ))?;
        // Undeclared reads and waits link no parent.
        if !parent.is_empty() {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                UpstreamOf::reference(),
                format!("uses:{key}"),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
                WorthQueryApplicationEntityKey::new(parent).unwrap(),
            ))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod courts {
    use super::*;
    use worth_query_host::facade::{
        application_entry::WorthQueryEditEnvelopeOutcome as Outcome,
        runtime::ExecutionAllocationPolicy,
    };
    type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

    fn edit(host: &Host, key: &str, value: u64) -> Outcome {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let mut edit = (request.edit_envelope())
            .idempotency(key.to_owned())
            .runtime_stages(host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<Tree, _>(fixture::model::Intent {
            key: "carriage:d".into(),
            value,
            key_derived_target: false,
        })
        .unwrap();
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    }
    fn row(host: &Host, key: &str) -> query::Row {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let rows = request
            .query(query::first_output::Read(key.into()))
            .execute();
        rows.unwrap_or_else(|_| panic!("{key} exists")).rows()[0].clone()
    }

    /// A known-answer court: reader `r` sees the value written by the writer
    /// that the lower-rank reader `p` owes, so the edit commits and derives it.
    fn follows(waits: bool) {
        let host: Host = world::install_configured(
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
            world::Configuration { envelope_records: true, captured_composed: Some((0, waits)), ..Default::default() });
        let born = edit(&host, "composed-born", 4);
        assert!(matches!(born, Outcome::Committed(_)), "{born:?}");
        let (p, r) = (
            row(&host, "captured:capture:p"),
            row(&host, "captured:capture:r"),
        );
        // p: source 2 + late50. r: source 3 + s60 + the undeclared 9 * d40.
        assert_eq!((p.value, r.value), (2 + 50, 3 + 60 + 9 * 40));
        let outcome = edit(&host, "composed-changed", 5);
        assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
        assert_eq!(row(&host, "first:carriage:d").value, 50);
        let after = row(&host, "captured:capture:r");
        assert_eq!(
            after.value,
            3 + 60 + 9 * 50,
            "derived from the settled writer"
        );
        assert_eq!(after.native_id, r.native_id);
        assert_eq!(row(&host, "captured:capture:p"), p);
        assert_eq!(row(&host, "first:capture:t").value, 70);
    }
    #[test]
    fn a_reader_follows_a_writer_owed_by_an_independent_lower_rank_reader() {
        follows(false);
    }
    /// Both readers are awaited: the writer waits on `p` and the route
    /// `first:capture:t` waits on `r`, and `r` reads what the writer writes.
    /// `p` runs first, the writer runs, then `r` sees the settled writer.
    #[test]
    fn awaited_readers_seal_in_the_order_they_ran() {
        follows(true);
    }
}
