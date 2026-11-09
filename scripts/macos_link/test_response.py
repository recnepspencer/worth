"""Response files remain a transport, including the OS E2BIG fallback."""
import errno
import io
from pathlib import Path
import tempfile
import unittest
from contextlib import redirect_stderr
from unittest.mock import patch
from .driver import link, main
from .macho import Image, Segment
from .placement import HIGH_BASE
from .loader import LoaderBounds
from .response import tokens, expanded, ResponseFileError


class ResponseTests(unittest.TestCase):
    def test_gnu_escaping_applies_inside_both_quote_kinds(self):
        self.assertEqual(tokens(r"'a\'b' c\ d \"literal\" "+'""'),
                         ["a'b", 'c d', '"literal"', ''])
        self.assertEqual(tokens('"a\\\\b"'), ['a\\b'])
        self.assertEqual(tokens("'unterminated"), ['unterminated'])

    def test_large_response_is_inspected_but_never_expanded_into_cc_argv(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output, response = root/'out', root/'args'
            response.write_text(('object.o ' * 50000)+'-o '+str(output))
            arguments = ['@'+str(response)]
            high = Image(2, 0x200000, (Segment('__LINKEDIT', HIGH_BASE, 0x4000, 1),), False)
            def compiler(command):
                self.assertIn(arguments[0], command)
                self.assertNotIn('object.o', command)
                stage = command[max(i for i, arg in enumerate(command) if arg == '-o')+1]
                Path(stage).write_bytes(b'linked')
                return 0
            with patch('scripts.macos_link.driver.subprocess.call', side_effect=compiler), \
                 patch('scripts.macos_link.driver.read', return_value=high), \
                 patch('scripts.macos_link.placement.host_loader', return_value=LoaderBounds(2**22, 0x16c000)):
                self.assertEqual(link(arguments), 0)
            self.assertEqual(output.read_bytes(), b'linked')

    def test_command_limit_is_not_misclassified_as_geometry(self):
        with patch('scripts.macos_link.driver.link', side_effect=OSError(errno.E2BIG, 'argv too long')), \
             redirect_stderr(io.StringIO()) as output:
            self.assertNotEqual(main(), 0)
        self.assertIn('MACOS_LINK_COMMAND', output.getvalue())
        self.assertNotIn('MACOS_EXEC_LAYOUT', output.getvalue())

    def test_response_cycles_are_command_refusals(self):
        with tempfile.TemporaryDirectory() as directory:
            response = Path(directory)/'recursive'
            response.write_text('@'+str(response))
            with self.assertRaises(ResponseFileError):
                expanded(['@'+str(response)])
