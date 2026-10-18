"""Decode libtest failures and report bounded command diagnostics, retaining full logs."""


def failed_tests(lines):
    return sorted({line.split()[1] for line in lines
                   if line.startswith("test ") and line.rstrip().endswith("FAILED")})


def report_failed_command(label, code, log, lines):
    tail = "\n".join(lines[-80:]).encode("utf-8")[-16384:].decode(
        "utf-8", errors="ignore"
    )
    print(f"GATE failed command {label}: exit {code}; complete log: {log.resolve()}",
          flush=True)
    print(tail, flush=True)
