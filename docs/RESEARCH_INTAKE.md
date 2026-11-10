# Local Z2352N research intake (no raw data committed)

The operator's Arch workstation currently has local research under:

- ~/Downloads/Z2352N_FDL_RESEARCH

This path is a local reference only. It is **not** an input automatically ingested by the CLI or GitHub Actions.

## Review process

1. Record the exact research phase identifier and source SHA-256.
2. Distinguish tested on real Z2352N hardware, only analyzed offline, and unverified hypothesis.
3. Extract sanitized methodology, test outcomes, failure modes and reproducer metadata.
4. Separately verify firmware / build identifiers and reproducibility before marking a capability working.
5. Preserve the original raw evidence **outside this public repository**.

## Prohibited content in public commits

- Raw userdata, persist / FRP or NVRAM partitions.
- IMEI, serial, private keys, accounts, access tokens, Wi-Fi secrets.
- Vendor firmware images, proprietary FDL loaders or PAC packages without redistribution rights.
- Unreviewed binary patches or instructions that assume another Unisoc device is compatible.

## Initial claims ledger

| Topic | Current claim allowed in public docs |
|---|---|
| Z2352N identity | Reported nubia Neo 2 5G, P820F03, UNISOC T820 |
| FDL | Local experiments exist; compatibility and release process not qualified by P001 |
| U-Boot / TrustOS | Offline research exists; no generic Z2352N patch released |
| Bootloader unlock | Status requires independent verification; property reports alone are insufficient |
| Root | Persistent Magisk root reported on research device; not yet a published, reproducible workflow |
| FRP | Personal research exists; public release offers authorized recovery guidance only |
