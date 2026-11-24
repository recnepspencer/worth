//! Malformed queries are refused and valid queries lower with defaults and hygiene.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the query is refused")
        .to_string()
}

const HEADER: &str = "#[worth_query(schema = S, context = C, namespace = NS, \
     query = Q, binding = B, result = R, limits = 32)]";

#[test]
fn malformed_queries_are_refused() {
    let cases = [
        (
            format!("{HEADER} struct Read {{ q: String }}"),
            "missing query scope",
        ),
        (
            format!("#[worth_query(scope = ProjectKey)] {HEADER} struct Read;"),
            "requires `#[worth_query(scope_value = ..)]`",
        ),
        (
            format!("#[worth_query(scope_value = rules::inbox)] {HEADER} struct Read;"),
            "requires `#[worth_query(scope = ..)]`",
        ),
        (
            format!(
                "#[worth_query(principal_scope = Owner, scope = ProjectKey, scope_value = rules::inbox)] \
                 {HEADER} struct Read;"
            ),
            "exactly one scope",
        ),
        (
            format!(
                "#[worth_query(principal_scope = Owner)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64 }}"
            ),
            "exactly one scope",
        ),
        (
            format!(
                "#[worth_query(scope_column = \"Key\")] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64 }}"
            ),
            "`scope_column` requires struct-level",
        ),
        (
            format!("{HEADER} struct Read {{ #[worth_query(column = \"Key\")] project: u64 }}"),
            "`column` names the column of a `scope` field",
        ),
        (
            format!(
                "#[worth_query(cardinality = ExactlyOne)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64 }}"
            ),
            "`cardinality` does not apply to a query without `root = Entity`",
        ),
        (
            format!(
                "#[worth_query(lanes = OneShot)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64 }}"
            ),
            "`lanes` does not apply to a query without `root = Entity`",
        ),
        (
            format!(
                "{HEADER} struct Read {{ #[worth_query(scope = ProjectKey)] project: u64, \
                 #[worth_query(name = \"q\")] a: String, #[worth_query(name = \"q\")] b: String }}"
            ),
            "duplicate parameter `q`",
        ),
        (
            format!(
                "{HEADER} struct Read {{ #[worth_query(scope = ProjectKey)] project: u64, \
                 definition: String }}"
            ),
            "parameter selector `definition` conflicts",
        ),
        (
            format!(
                "{HEADER} struct Read {{ #[worth_query(scope = ProjectKey)] project: u64, \
                 #[worth_query(skip, param)] helper: u32 }}"
            ),
            "`param` does not apply to a `skip` field",
        ),
        (
            format!(
                "#[worth_query(parameters_fn = custom_params)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64, \
                 #[worth_query(name = \"q\")] q: String }}"
            ),
            "`name` does not apply to a query with `parameters_fn`",
        ),
        (
            format!(
                "#[worth_query(unknown_attr = 1)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64 }}"
            ),
            "unknown key for #[derive(WorthQuery)]",
        ),
        (
            format!("{HEADER} struct Read(u64);"),
            "struct with named fields or a unit struct",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}

#[test]
fn query_derives_defaults_omits_no_parameters_and_is_copy_clone_clean() {
    let input = syn::parse_str(
        "#[worth_query(schema = Schema, context = BoardContext, namespace = SEARCH, \
         query = SearchQuery, binding = SearchBinding, result = RowBinding, \
         root = Task, limits = 32, name = \"task_search\")] \
         pub struct Search { #[worth_query(scope = ProjectKey)] pub project: u64, pub q: String }",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(
        expanded.contains("worth_query_application_query !"),
        "{expanded}"
    );
    // Omits explicit `parameters ...` in application_query macro (retires NoParameters).
    let app_query = expanded
        .split_once("worth_query_application_query !")
        .and_then(|(_, rest)| rest.split_once(';'))
        .map(|(head, _)| head)
        .unwrap();
    assert!(!app_query.contains("parameters "), "{app_query}");
    assert!(expanded.contains("pub mod search_params"), "{expanded}");
    assert!(expanded.contains("pub const fn q ()"), "{expanded}");
    assert!(
        expanded.contains("# [allow (clippy :: clone_on_copy)]"),
        "{expanded}"
    );
    assert!(
        expanded.contains("ApplicationQueryCardinality :: Many"),
        "{expanded}"
    );
    assert!(
        expanded.contains("ApplicationQueryDisclosureContract :: public ()"),
        "{expanded}"
    );
    assert!(
        expanded.contains("ApplicationQueryBasisSupport :: current_and_pinned ()"),
        "{expanded}"
    );
    assert!(
        expanded.contains("ApplicationQueryLaneEligibility :: one_shot ()"),
        "{expanded}"
    );
    assert!(
        expanded.contains("fn worth_query_try_parameters"),
        "{expanded}"
    );
    assert!(
        expanded.contains("try_parameters => Search :: worth_query_try_parameters"),
        "{expanded}"
    );
    assert!(!expanded.contains(". expect ("), "{expanded}");
    assert!(expanded.contains("fn definition ()"), "{expanded}");
    assert!(
        expanded.contains(". parameter (Self :: q ())"),
        "{expanded}"
    );
}

#[test]
fn explicit_shape_query_omits_default_definition_and_requires_root() {
    let err = refusal(&format!(
        "#[worth_query(explicit_shape)] {HEADER} \
         struct Read {{ #[worth_query(scope = ProjectKey)] project: u64 }}"
    ));
    assert!(
        err.contains("`explicit_shape` does not apply to a query without `root = Entity`"),
        "{err}"
    );
    let input = syn::parse_str(
        "#[worth_query(schema = Schema, context = BoardContext, namespace = READ, \
         query = DomainQuery, binding = DomainBinding, result = DomainCarrierBinding, \
         root = Task, explicit_shape, limits = 8)] \
         pub struct ReadDomain { #[worth_query(scope = ProjectKey)] pub project: u64 }",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(!expanded.contains("fn definition ()"), "{expanded}");
    assert!(expanded.contains("fn definition_with ("), "{expanded}");
}

#[test]
fn unit_struct_query_with_scope_value_and_overridden_defaults_expands_cleanly() {
    let input = syn::parse_str(
        "#[worth_query(schema = Schema, context = BoardContext, namespace = INBOX, \
         query = InboxQuery, binding = InboxBinding, result = RowBinding, \
         scope = ProjectKey, scope_value = inbox_key, root = Task, \
         cardinality = ExactlyOne, basis_support = CurrentOnly, limits = 1)] \
         pub struct ReadInbox;",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(expanded.contains("value inbox_key"), "{expanded}");
    assert!(
        expanded.contains("ApplicationQueryCardinality :: ExactlyOne"),
        "{expanded}"
    );
    assert!(
        expanded.contains("ApplicationQueryBasisSupport :: current_only ()"),
        "{expanded}"
    );
    assert!(!expanded.contains("worth_query_parameters"), "{expanded}");
}

#[test]
fn managed_index_query_wires_probe_binding_defaults_and_refuses_malformed_shapes() {
    let cases = [
        (
            format!(
                "{HEADER} struct Read {{ #[worth_query(scope = ProjectKey)] project: u64, \
                 #[worth_query(probe)] p: String }}"
            ),
            "`probe` requires struct-level `#[worth_query(managed_index = Index)]`",
        ),
        (
            format!(
                "#[worth_query(managed_index = MyIdx)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64, p: String }}"
            ),
            "`managed_index` requires one field marked `#[worth_query(probe)]`",
        ),
        (
            format!(
                "#[worth_query(managed_index = MyIdx)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64, \
                 #[worth_query(probe)] a: String, #[worth_query(probe)] b: String }}"
            ),
            "at most one `probe` field",
        ),
        (
            format!(
                "#[worth_query(managed_index_refined)] {HEADER} \
                 struct Read {{ #[worth_query(scope = ProjectKey)] project: u64 }}"
            ),
            "`managed_index_refined` does not apply to a query without `managed_index = Index`",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }

    let input = syn::parse_str(
        "#[worth_query(schema = Schema, context = ShopContext, namespace = FIND, \
         scope = FloorId, managed_index = EquipmentBoxIndex, lanes = OneShotAndLive, limits = 64)] \
         pub struct FindEquipmentInBox { \
             #[worth_query(scope)] pub floor: String, \
             #[worth_query(probe)] pub region: QueryBox2D, \
         }",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(expanded.contains("FindEquipmentInBoxQuery"), "{expanded}");
    assert!(expanded.contains("FindEquipmentInBoxBinding"), "{expanded}");
    assert!(expanded.contains("ProbeBinding"), "{expanded}");
    assert!(expanded.contains(". managed_index ("), "{expanded}");
    assert!(
        !expanded.contains(". parameter (Self :: region ())"),
        "{expanded}"
    );
    assert!(expanded.contains(". with_live ()"), "{expanded}");
    assert!(
        expanded.contains("worth_query_try_parameters"),
        "{expanded}"
    );
}
