"""Final publication must reject bad compiler output independently of planning."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from .driver import link
from .macho import Image, Segment
from .placement import HIGH_BASE, UnsafeLayout
from .loader import LoaderBounds


class FinalAdmissionTests(unittest.TestCase):
    def test_a_low_final_top_is_refused_after_sound_initial_planning(self):
        high = Image(2, 0x200000, (Segment('__LINKEDIT', HIGH_BASE, 0x4000, 1),), False)
        low = Image(2, 0x200000, (Segment('__TEXT', 2**32, 0x4000, 5),), False)
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'output'
            def compiler(arguments):
                Path(arguments[max(i for i, arg in enumerate(arguments) if arg == '-o') + 1]).write_bytes(b'pending compiler output')
                return 0
            with patch('scripts.macos_link.driver.subprocess.call', side_effect=compiler), \
                 patch('scripts.macos_link.driver.read', side_effect=[high, low]), \
                 patch('scripts.macos_link.placement.host_loader', return_value=LoaderBounds(2**22, 0x16c000)):
                with self.assertRaisesRegex(UnsafeLayout, 'top mapped segment'):
                    link(['object.o', '-o', str(output)])
            self.assertFalse(output.exists(), 'a refused artifact must not be published')
