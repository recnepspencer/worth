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
            }),
        }
        self.assertEqual(
            ordinary_storage_totals('production_explicit', references), {'storage -> runtime': 1})
        self.assertEqual(
            ordinary_storage_totals('inherent_impl', references),
            {'storage -> runtime': 1, 'runtime -> storage': 1})

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


if __name__ == '__main__':
    unittest.main()
