"""Source fixtures for the Relational DAG ratchet."""
from collections import Counter
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_relational_dag import (
    Audit, SOURCE, compare, facade_failures, ordinary_storage_totals, totals,
)


class RelationalDagTests(unittest.TestCase):
    def setUp(self):
        fixture = tempfile.TemporaryDirectory()
        self.addCleanup(fixture.cleanup)
        self.root = Path(fixture.name)
        self.write('lib.rs', 'mod low; mod high; mod runtime;')
        self.write('low.rs', '')
        self.write('high.rs', 'pub struct High<T>(T); pub struct Other;')
        self.write('runtime.rs', 'pub use crate::high::High as Exported;')
        self.config = {
            'court_seed': 'courts',
            'owners': {
                'low': {'crate': 'worth-relational-contracts', 'layer': 0, 'depends': []},
                'high': {'crate': 'worth-relational-branch', 'layer': 2, 'depends': ['low']},
                'runtime': {'crate': 'worth-relational', 'layer': 6, 'depends': ['low', 'high']},
            },
            'paths': {'lib.rs': 'runtime', 'low': 'low', 'high': 'high', 'runtime': 'runtime'},
            'boundaries': [],
        }

    def write(self, path, text):
        path = self.root / SOURCE / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def audit(self):
        audit = Audit(self.root, self.config)
        audit.scan()
        return audit

    def test_explicit_import_leaves_and_macro_tokens_fail_a_zero_direction(self):
        self.write('low.rs', '''
            use crate::high::{High as Alias, Other};
            macro_rules! m { () => { super::high::Other }; }
            // crate::high::Other
            const TEXT: &str = "crate::high::Other";
        ''')
        directions = self.audit().ledger['production_explicit']
        self.assertEqual(totals(directions), {'low -> high': 3})
        self.assertIn('low -> high: 3 > 0', compare(directions, {}, {}))

    def test_owner_resolution_charges_the_defining_owner_through_an_alias(self):
        self.write('low.rs', 'use crate::runtime::Exported;')
        audit = self.audit()
        self.assertEqual(totals(audit.ledger['production_explicit']), {'low -> runtime': 1})
        self.assertEqual(totals(audit.ledger['production_resolved']), {'low -> high': 1})

    def test_upward_public_reexport_uses_explicit_and_defining_owners(self):
        self.write('low.rs', 'pub use crate::runtime::Exported as Public;')
        audit = self.audit()
        self.assertEqual(totals(audit.reexports), {'low -> high': 1, 'low -> runtime': 1})

    def test_upward_public_reexport_also_fails_under_a_test_gate(self):
        self.write('low.rs', '#[cfg(test)] pub use crate::high::Other;')
        audit = self.audit()
        self.assertEqual(totals(audit.reexports), {'low -> high': 1})
        self.assertEqual(totals(audit.ledger['production_explicit']), {})
        self.assertEqual(totals(audit.ledger['cfg_test']), {'low -> high': 1})

    def test_inherent_impl_follows_alias_generic_and_qualified_paths(self):
        self.write('low.rs', '''
            use crate::runtime::Exported as Alias;
            struct Local;
            type OtherAlias = crate::high::Other;
            trait A {}
            impl<T: Clone> Alias<T> where T: Default {}
            impl super::high::Other {}
            impl OtherAlias {}
            impl A for crate::high::High<u8> {}
            impl Local {}
        ''')
        audit = self.audit()
        self.assertEqual(totals(audit.ledger['inherent_impl']), {'low -> high': 3})
        self.assertTrue(compare(audit.ledger['inherent_impl'], {}, {}))

    def test_inherent_impl_inside_a_test_family_uses_its_local_imports(self):
        self.write('low.rs', '/* owner file */ #[cfg(test)] mod test_impl;')
        self.write('low/test_impl.rs', 'use crate::high::Other as Alias; impl Alias {}')
        self.assertEqual(totals(self.audit().ledger['inherent_impl']), {'low -> high': 1})

    def test_ordinary_storage_ratchet_counts_both_sides_of_inherent_coherence(self):
        references = {
            'storage -> runtime': Counter({
                ((SOURCE / 'storage/residency/cell.rs').as_posix(), 'runtime::Payer'): 3,
                ((SOURCE / 'storage/partition/write.rs').as_posix(), 'runtime::Payer'): 1,
            }),
            'runtime -> storage': Counter({
                ((SOURCE / 'runtime/edit.rs').as_posix(), 'storage::Partition'): 1,
            }),
            'storage -> durability': Counter({
                ((SOURCE / 'storage/partition/write.rs').as_posix(), 'durability::Frame'): 1,
                ((SOURCE / 'storage/residency/cell.rs').as_posix(), 'durability::Frame'): 2,
            }),
            'durability -> storage': Counter({
                ((SOURCE / 'durability/data/capture.rs').as_posix(), 'storage::Partition'): 1,
            }),
        }
        outgoing = {'storage -> runtime': 1, 'storage -> durability': 1}
        for projection in ('production_explicit', 'production_resolved'):
            with self.subTest(projection=projection):
                self.assertEqual(ordinary_storage_totals(projection, references), outgoing)
        self.assertEqual(
            ordinary_storage_totals('inherent_impl', references),
            {**outgoing, 'runtime -> storage': 1, 'durability -> storage': 1})

    def test_ordinary_storage_test_ratchet_includes_durability(self):
        reference = 'durability::Frame'
        counts = {'storage -> durability': Counter({
            ((SOURCE / 'storage/data/frame_tests.rs').as_posix(), reference): 2,
            ((SOURCE / 'storage/residency/frame_tests.rs').as_posix(), reference): 3,
        })}
        self.assertEqual(ordinary_storage_totals('cfg_test', counts), {'storage -> durability': 2})

    def test_test_only_scope_including_associated_items_and_nested_use(self):
        self.write('lib.rs', '''
            mod low; mod high; mod runtime;
            #[cfg(test)] mod tests;
            #[cfg(test)] mod test_execution_policy;
            mod presentation;
        ''')
        self.write('tests.rs', '')
        self.write('test_execution_policy.rs', '')
        self.write('presentation.rs', '#[cfg(test)] pub mod harness {}')
        self.config['paths'].update(
            tests='courts', presentation='runtime', test_execution_policy='runtime')
        self.write('low.rs', '''
            #[cfg(all(test, unix))]
            mod test_module { use crate::{tests, presentation::harness, test_execution_policy}; }
            struct Local;
            impl Local {
                #[cfg(test)]
                fn test_hook() {
                    use crate::runtime::Exported;
                    use worth_testkit_relational as kit;
                    kit::some_fixture();
                }
                fn production() {
                    #[cfg(test)]
                    let x = crate::high::Other;
                }
            }
        ''')
        audit = self.audit()
        self.assertEqual(totals(audit.ledger['production_explicit']), {})
        self.assertEqual(totals(audit.ledger['production_resolved']), {})
        self.assertEqual(totals(audit.ledger['cfg_test']), {
            'low -> courts': 3, 'low -> high': 2, 'low -> runtime': 1, 'low -> testkit': 2,
        })
        self.assertTrue(compare(audit.ledger['cfg_test'], {}, {}))

    def test_unassigned_file_fails_even_if_not_mounted(self):
        self.write('unowned.rs', 'pub struct Unowned;')
        with self.assertRaisesRegex(ValueError, 'unassigned source:.*unowned.rs'):
            self.audit()

    def test_owner_rule_must_win_for_at_least_one_file(self):
        for dead in ('absent', 'low'):
            with self.subTest(rule=dead):
                self.config['paths']['low.rs'] = self.config['paths'].pop('low')
                self.config['paths'][dead] = 'high'
                with self.assertRaisesRegex(ValueError, f'unused owner rules: {dead}'):
                    self.audit()
                self.config['paths'].pop(dead)
                self.config['paths']['low'] = self.config['paths'].pop('low.rs')

    def test_facade_fails_on_added_and_removed_paths(self):
        self.write('lib.rs', 'mod low; mod high; mod runtime; pub mod facade;')
        self.config['paths']['facade'] = 'runtime'
        self.write('facade.rs', 'pub use crate::high::Other as New;')
        baseline = {'facade_exports': ['facade::Old'], 'subordinate_test_export_tokens': []}
        self.assertEqual(facade_failures(self.audit(), baseline), [
            'facade export added: facade::New',
            'facade export removed: facade::Old',
        ])

    def test_subordinate_test_export_budget_follows_the_definition(self):
        self.write('lib.rs', 'mod low; mod high; mod runtime; pub mod facade;')
        self.config['paths']['facade'] = 'runtime'
        self.write('facade.rs', 'pub use crate::runtime::{ControlForTest, LocalForTest};')
        self.write('runtime.rs', 'pub struct ControlForTest; pub struct LocalForTest;')
        baseline = {
            'facade_exports': self.audit().exports(),
            'subordinate_test_export_tokens': [],
        }
        self.assertEqual(facade_failures(self.audit(), baseline), [])

        self.write('high.rs', 'pub struct Hook;')
        self.write('runtime.rs', """
            pub use crate::high::Hook as ControlForTest;
            pub struct LocalForTest;
        """)
        audit = self.audit()
        self.assertEqual(audit.subordinate_test_export_tokens(audit.exports()), ['ControlForTest'])
        self.assertEqual(facade_failures(audit, baseline), [
            'unbudgeted subordinate test export: ControlForTest',
        ])
        baseline['subordinate_test_export_tokens'] = ['ControlForTest', 'RetiredForTest']
        self.assertEqual(facade_failures(audit, baseline), [])

    def test_unmounted_file_cannot_hide_under_an_assigned_prefix(self):
        self.write('low/hidden.rs', 'use crate::high::Other;')
        with self.assertRaisesRegex(ValueError, 'unmounted source:.*low/hidden.rs'):
            self.audit()

    def test_a_removed_reference_does_not_pay_for_another_at_the_same_direction(self):
        self.write('low.rs', 'use crate::high::Other;')
        directions = self.audit().ledger['production_explicit']
        path = (SOURCE / 'low.rs').as_posix()
        ceilings = {'low -> high': 1}
        rows = {'low -> high': [[path, 'high::High', 1]]}
        self.assertEqual(compare(directions, ceilings, rows), [
            f'low -> high: {path} :: high::Other: 1 > 0',
        ])
        self.write('low.rs', 'use crate::high::Other; fn f() { let _ = crate::high::Other; }')
        directions = self.audit().ledger['production_explicit']
        self.assertIn('low -> high: 2 > 1', compare(directions, ceilings, rows))

    def test_intra_crate_rules_gate_direction_even_with_same_package(self):
        self.write('low.rs', 'mod first; mod second;')
        self.write('low/first.rs', 'use super::second::Record; pub struct Input;')
        self.write('low/second.rs', 'use super::first::Input; pub struct Record;')
        self.config['boundaries'] = [{
            'owner': 'low',
            'paths': {'first': ['low/first'], 'second': ['low/second']},
            'allowed': [['first', 'second']],
        }]
        audit = self.audit()
        expected = {'low/second -> low/first': 1}
        self.assertEqual(totals(audit.ledger['production_explicit']), expected)
        self.assertEqual(totals(audit.ledger['production_resolved']), expected)

    def extract_low_owner(self):
        self.config['owners']['low']['physical'] = True
        self.config['paths'].pop('low')
        (self.root / SOURCE / 'low.rs').unlink()
        self.write('lib.rs', '''
            use worth_relational_contracts::internal_port::low;
            mod high;
            mod runtime;
            pub mod facade { pub use crate::low::Low; }
        ''')
        source = self.root / 'crates/worth-relational-contracts/src'
        source.mkdir(parents=True)
        (source / 'lib.rs').write_text('pub mod internal_port { pub mod low; }')
        (source / 'internal_port').mkdir()
        (source / 'internal_port/low.rs').write_text('pub struct Low;')
        (source.parent / 'Cargo.toml').write_text('[dependencies]\n')
        (self.root / SOURCE.parent / 'Cargo.toml').write_text(
            '[dependencies]\nworth-relational-contracts.workspace = true\n')
        return source

    def test_facade_resolves_export_to_the_physical_owner(self):
        self.extract_low_owner()
        audit = self.audit()
        self.assertEqual(audit.exports(), ['facade::Low'])
        location = ('worth_relational_contracts', 'internal_port', 'low')
        self.assertEqual(audit.locations[location], ('low', None))

    def test_physical_glob_preserves_the_item_export_kind(self):
        self.extract_low_owner()
        self.write('lib.rs', """
            mod bridge { pub use worth_relational_contracts::internal_port::*; }
            mod high;
            mod runtime;
            pub mod facade { pub use crate::bridge::low::Low; }
        """)
        self.assertEqual(self.audit().exports(), ['facade::Low'])

    def test_physical_owner_keeps_its_internal_direction(self):
        source = self.extract_low_owner()
        (source / 'internal_port/low.rs').write_text(
            'pub mod custody; pub mod projection; pub struct Low;')
        directory = source / 'internal_port/low'
        directory.mkdir()
        (directory / 'custody.rs').write_text('pub struct Custody;')
        (directory / 'projection.rs').write_text('use super::custody::Custody;')
        self.config['boundaries'] = [{
            'owner': 'low',
            'paths': {
                'custody': ['internal_port/low/custody'],
                'projection': ['internal_port/low/projection'],
            },
            'allowed': [['custody', 'projection']],
        }]
        audit = self.audit()
        direction = {'low/projection -> low/custody': 1}
        self.assertEqual(totals(audit.ledger['production_explicit']), direction)
        self.assertEqual(totals(audit.ledger['production_resolved']), direction)

    def test_physical_budget_cannot_include_remaining_runtime_paths(self):
        self.extract_low_owner()
        self.config['paths']['runtime.rs'] = 'low'
        with self.assertRaisesRegex(ValueError, 'physical owner low cannot own Runtime path'):
            self.audit()

    def test_module_mount_into_a_sibling_package_is_rejected(self):
        source = self.extract_low_owner()
        (source / 'lib.rs').write_text(
            '#[path = "../../worth-relational/src/high.rs"] mod escaped;')
        with self.assertRaisesRegex(ValueError, 'module escapes its package'):
            self.audit()

    def test_physical_owner_cannot_add_a_relational_dev_dependency(self):
        source = self.extract_low_owner()
        (source.parent / 'Cargo.toml').write_text(
            '[dev-dependencies]\nfixture = { package = "worth-relational", path = "../worth-relational" }\n')
        with self.assertRaisesRegex(ValueError, 'worth-relational must be an ordinary dependency'):
            self.audit()


if __name__ == '__main__':
    unittest.main()
