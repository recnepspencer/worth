#!/usr/bin/env python3
"""Inspect an arm64 Darwin executable without running it."""
import sys
from macos_link.macho import read
from macos_link.placement import validate
try:
    validate(read(sys.argv[1]))
except (OSError, ValueError) as error:
    raise SystemExit(f'MACOS_EXEC_LAYOUT: {error}')
