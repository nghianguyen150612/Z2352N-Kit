# Z2352N-Kit

Open-source research and diagnostics for the **nubia Neo 2 5G (Z2352N / P820F03, UNISOC T820 / UMS9620)**.

**P001 status: read-only foundation.** No unlock, FRP bypass, firmware flash, or partition write is implemented in this release. Commands and test results must not be interpreted as a verified hardware exploit or an OEM unlock.

## Install on Linux

Requirements: Rust stable. Optional: Android platform-tools for connected-device diagnostics.

~~~sh
git clone https://github.com/nghianguyen150612/Z2352N-Kit.git
cd Z2352N-Kit
cargo build --release --locked
./target/release/z2352n --help
~~~

## Commands

~~~sh
z2352n doctor               # host prerequisites
z2352n doctor --json
z2352n probe                # ADB / Fastboot discovery and read-only Android properties
z2352n unlock status        # reported bootloader/AVB state; does not unlock
z2352n report --json        # redacted, machine-readable report
z2352n wizard               # first-connection check
z2352n research hash boot.img
z2352n frp recovery         # legitimate ownership/account recovery guidance
~~~

The tool never invokes root or writes flash. ADB device serials are used neither as report fields nor as identifiers in logs. Multiple attached devices cause a fail-closed result. The currently supported device identity check requires both the expected Neo 2 model string and the P820F03 board/device identifier, as reported by Android.

### What the status means

An Android property report of unlocked is **not** independent proof that the bootloader has been permanently OEM-unlocked. Modified boot components can report different values. Unavailable or contradictory properties remain unverified. Firmware variants must not be assumed compatible based on sharing a SoC.

## Development

~~~sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
~~~

See [ROADMAP](docs/ROADMAP.md) and [RESEARCH_INTAKE](docs/RESEARCH_INTAKE.md).

## Safety and responsible disclosure

- Do not flash unknown images or substitute offsets from another T820 device.
- Before any future write-enabled release: verify firmware identity, image integrity, A/B layouts, rollback behavior, backup completeness, and a realistic recovery path.
- Do not publish original partition dumps, FRP/credential material, device identifiers, secret keys, or proprietary vendor loaders.
- The FRP command provides authorized ownership recovery guidance, not an anti-theft bypass.
- PRs that add device writes must include offline tests, explicit confirmation and a documented recovery process.

MIT license. See [LICENSE](LICENSE).
