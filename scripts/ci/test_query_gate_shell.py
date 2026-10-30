"""Select Git Bash before Windows' WSL launcher for the line-cap guard."""
from pathlib import Path
import unittest
from unittest.mock import patch

from query_gate import bash


class GateShellSelectionTests(unittest.TestCase):
    def test_git_usr_binary_precedes_an_unusable_wsl_path(self):
        installed = 'C:/Program Files/Git/usr/bin/bash.exe'
        with patch.object(Path, 'exists', lambda path: path.as_posix() == installed), \
                patch('query_gate.shutil.which', return_value='C:/Windows/system32/bash.exe'):
            self.assertEqual(bash(), installed)

    def test_git_bin_layout_and_non_windows_path_remain_supported(self):
        installed = 'C:/Program Files/Git/bin/bash.exe'
        with patch.object(Path, 'exists', lambda path: path.as_posix() == installed):
            self.assertEqual(bash(), installed)
        with patch.object(Path, 'exists', return_value=False), \
                patch('query_gate.shutil.which', return_value='/usr/bin/bash'):
            self.assertEqual(bash(), '/usr/bin/bash')


if __name__ == '__main__':
    unittest.main()
