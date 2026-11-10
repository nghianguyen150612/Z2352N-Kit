# Development plan

## P001 — Read-only foundation (current)

- Rust CLI with doctor, probe, unlock status, report, wizard, research hash and FRP recovery guidance.
- Strict selection of one attached device; redacted output and no root/flash/write calls.
- Device identity check and consistency assessment of reported bootloader / verified boot properties.
- Offline simulated tests and CI.

## P002 — Evidence import and reproducibility

- Offline inventory tool for approved, sanitized P-series findings.
- Per-firmware profile schema: source, SHA-256, slot map, verification status.
- Reject unreviewed binary files, sensitive identifiers and secrets from the public repository.
- Synthetic fixtures for AVB / DHTB / SIMGHDR parsers; independent format validation.

## P003 — Backup and recovery qualification

- Explicit owner-mediated read-only backup flow with provenance and checksums.
- Partition discovery, transfer integrity verification and recovery planning.
- No device writes until device-specific recovery qualification exists.

## P004 — Bootloader research workflow

- Separate hardware lock state, reported AVB state and persistent root observations.
- Verify compatibility and demonstrate repeatability for each firmware/board revision.
- Standard, manufacturer-allowed unlock workflows may be integrated only with explicit consent and complete preflight.

## P005 — Release quality

- Clear CLI/TUI experience, automated offline tests, hardware qualification records.
- Versioned release artifacts and reproducible build instructions.
- No speculative one-click feature until test and recovery criteria are met.

## Ownership and FRP

- Publish non-sensitive research, legitimate recovery guidance and diagnostic findings.
- Do not distribute automation that neutralizes anti-theft protections or handles other people's device credentials.

## Acceptance gates for any future write operation

1. Strong match on device identity and firmware revision.
2. Source and target hashes, storage layout, A/B slot and size checks.
3. Verified stock backup with independent read-back check.
4. Defined rollback and recovery procedure demonstrated on target hardware.
5. Human confirmation naming the exact partitions involved.
6. Hardware regression tests and explicit version compatibility matrix.
