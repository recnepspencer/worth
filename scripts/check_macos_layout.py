#!/usr/bin/python3 -I
"""Inspect an arm64 Darwin executable without running it."""
import sys
from pathlib import Path
if sys.version_info < (3, 9):
    raise SystemExit("MACOS_LINK_RUNTIME: Apple Command Line Tools Python 3.9+ is required")
sys.path.insert(0, str(Path(__file__).resolve().parent))
from macos_link.macho import read
from macos_link.placement import validate_historical
try:
    validate_historical(read(sys.argv[1]))
except (OSError, ValueError) as error:
    raise SystemExit(f'MACOS_EXEC_LAYOUT: {error}')
