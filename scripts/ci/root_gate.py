#!/usr/bin/env python3
"""Run every nonignored root test partition through the shared prebuilt gate."""
import sys

from query_gate import ROOT, main as gate_main


ROOT_SUITES = ('relational', 'signal', 'execution', 'root-rest', 'root-process',
               'root-exhaustive', 'root-determinism', 'root-compile-fail')


def main(argv=None):
    return gate_main(argv, default_suites=ROOT_SUITES,
                     default_out=ROOT / 'target/root-gate',
                     allowed_suites=(*ROOT_SUITES, 'root-scale'))


if __name__ == '__main__':
    sys.exit(main())
