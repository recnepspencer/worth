"""Behaviour proofs for the facade audit over a fixture runtime type."""
from pathlib import Path
import unittest

from facade_audit import audit, markdown

FIXTURE = Path(__file__).parent / 'fixtures/facade'


class FacadeAuditTests(unittest.TestCase):
    def setUp(self):
        self.result = audit(FIXTURE / 'src/lib.rs', 'Runtime', 'runtime', FIXTURE)
        self.methods = {m['method']: m for m in self.result['methods']}

    def test_only_public_inherent_production_methods_of_the_type_are_listed(self):
        self.assertEqual(sorted(self.methods), ['arithmetic', 'branchy', 'negated', 'open', 'read',
                                                'thin_field', 'with_macro', 'wrap'])
        self.assertEqual(self.result['public_methods'], 8)

    def test_extension_trait_calls_name_the_trait(self):
        call, = self.methods['read']['delegates']
        self.assertEqual((call['kind'], call['trait'], call['target']), ('trait', 'QueryOps', 'crate::query::QueryOps'))
        self.assertEqual(self.methods['read']['verdict'], 'thin')

    def test_plumbing_constructors_and_field_reads_stay_thin(self):
        self.assertEqual([c['name'] for c in self.methods['wrap']['delegates']], ['lookup'])
        self.assertEqual(self.methods['thin_field']['delegates'], [])
        self.assertEqual(self.methods['thin_field']['verdict'], 'thin')

    def test_logic_is_reported_with_its_reasons(self):
        self.assertEqual(self.methods['branchy']['logic'], ['if'])
        self.assertEqual(self.methods['arithmetic']['logic'], ['operator +'])
        self.assertEqual(self.methods['negated']['logic'], ['operator !', 'operator &&'])
        self.assertEqual(self.methods['open']['logic'], ['closure'])
        self.assertEqual(self.methods['with_macro']['logic'], ['macro format!'])
        self.assertEqual(self.result['thin'], 3)

    def test_method_chains_record_their_receivers(self):
        targets = {(c['target'], c['name']) for c in self.methods['open']['delegates']}
        self.assertIn(('self.core().session()', 'open'), targets)

    def test_output_is_sorted_and_free_of_absolute_paths(self):
        self.assertEqual(self.result['methods'], sorted(self.result['methods'], key=lambda m: (m['method'], m['file'])))
        self.assertEqual({m['file'] for m in self.result['methods']},
                         {'src/runtime/facade.rs', 'src/runtime/inherent.rs'})
        self.assertNotIn(str(FIXTURE.resolve()), markdown(self.result))
        self.assertEqual(markdown(self.result), markdown(audit(FIXTURE / 'src/lib.rs', 'Runtime', 'runtime', FIXTURE)))


if __name__ == '__main__':
    unittest.main()
