# Converter — standalone Rust V2 development beta

All three Recommended Converter commands are implemented as an offline Rust
process: Base Converter, Code Converter, and Byte Converter. This is work on
`feat/native-v2-rust`, prepared for parent review. It is not installed, published,
or a verified desktop release. Source `artifacts` remains `{}`.

The original V1 tree is byte-for-byte preserved except the README discovery link.
The executable uses the independent HTTPS Git SDK dependency pinned in Cargo.lock;
it does not link Tezbar/Tauri, import a sibling SDK path, run JavaScript, or require
Node on the user's machine. MSRV is Rust 1.85.

## Behavior

- Base: arbitrary radices 2–36, six simple inputs, two advanced dropdowns,
  separate advanced input/output, source prefix and clipboard detection, exact
  BigInt digits and source overrides. Negative/fractional/junk input follows
  the original rejection behavior. Original focused-copy, fixed-base copies,
  Copy Output, Clear All, and configuration actions retain their order.
- Code: native, UTF-16 Unicode escapes, Node-compatible lenient Base64, URI-based
  UTF-8 escapes, comma/packed ASCII, bracketed hex, arbitrary decimal, encoded
  URL, and hexadecimal HTML entities. These preserve the source's `charCodeAt`
  versus code-point iteration quirks, prefix parsing, NUL, and URI exceptions.
  Text is rendered as native form values, never as HTML or user Markdown.
- Byte: the actual source has `bits`, `Bytes`, KB, MB, GB, TB, PB, **EB**; binary
  powers and integer bit truncation precede two-decimal truncation. Clipboard
  suffix matching is case-sensitive and accepts those exact labels. It does
  not interpret SI decimal units, `b`, `B`, or `KiB` as extra aliases. Those
  rejected/ignored inputs are tested rather than silently added as features.

The four original preferences keep their defaults: Auto Paste on, advanced view
on and below simple inputs, comma-separated ASCII on. Each field converts live.
Opening reads clipboard text automatically only when enabled. The source never
reads selection, pastes output, opens browsers, writes domain storage, or retains
conversion history. V2 likewise keeps drafts only for the current session; host
preferences remain host-owned. There is no migration, background lease, poll,
timer, or detached work. `refreshMs` is explicitly null.

## Explicit adaptations

Inputs are bounded to 4,096 UTF-8 bytes and decimal scientific exponents to a
4,096-digit shift. Oversized manual input retains a bounded prefix with an error;
this is not unlimited source compatibility. Valid in-bound drafts and invalid
conversion drafts are retained. Invalid code conversions disable all copy actions
until repaired or cleared, so a previous output cannot be copied accidentally.
Unpaired UTF-16 surrogates cannot be represented by the UTF-8 wire; the original
reducer's URI encoder throws on them. V2 shows this error and retains the encoded
draft instead of substituting a replacement character or crashing the view.

Host Form does not currently provide every needed live-field contract. The
separate **unapplied** [host patch](integration/host.patch) adds whole-form action
values, edit revisions, delayed-result draft protection, and focus callbacks.
It also fixes verified alias routing and active-version resource paths. Parent
integration is required for complete GUI behavior; see [INTEGRATION.md](INTEGRATION.md).
No Detail or List visual change is bundled because these commands use Form.

Both source configuration actions open the host's Extensions settings surface;
the protocol has no separate command-specific settings destination. Failed
clipboard/settings effects have explicit Retry, exact prepared bytes, and a
warning that an uncertain effect may repeat. Nothing automatically replays them.
A cancelled action remains consumed even when the SDK discards staged state.
A deliberate view refresh recovers current actions and any prepared uncertain
retry. Clipboard read failure offers a deliberate retry or normal field editing.

## Build and QA

From `v2/`: `cargo build --locked`, `cargo test --locked`,
`cargo clippy --locked --all-targets -- -D warnings`, and `cargo fmt --check`.
The same tests/Clippy pass on installed `+1.85.0`. Full reproduction (requires
preinstalled Rust stable/1.85, Node/npm, Python, and network to fetch locked QA
packages): set `RUSTUP_HOME` and `CARGO_HOME` to the configured Rust directories,
then run `scripts/verify.sh`. It confines disposable JS tools and process HOME,
TMPDIR and outputs to ignored `.qa/`; it installs no system tool.

`qa/oracle.json` contains 1,859 results from executing pinned original TS hooks
and actual Node Buffer in a producer-denying VM. `qa/ui-oracle.json` contains nine
executed TSX snapshots. The same cases run through real executable JSONL pipes.
The scripts regenerate their expectations from source; no expected arithmetic is
handwritten. Source conversion helpers have no third-party conversion library;
TypeScript is the exact root-lock 5.9.3. QA uses Node 26.7.0. See [QA.md](QA.md)
for finite coverage, failures repaired, bounds, and platform gaps.

The local ARM64 ZIP and extracted binary live only in ignored `.qa/package/`.
[qa/dev-package.json](qa/dev-package.json) inventories the actual development
archive and hashes. This is not release metadata. Windows GNU `cargo check`
passes; GNU linking is blocked by missing `x86_64-w64-mingw32-dlltool`. Windows
MSVC, native Windows execution, live GUI and a full host build remain not run.

## Evidence and licenses

[qa/provenance.json](qa/provenance.json) pins source, SDK, host, original icon and
fixture hashes. [INDEX.sha256](INDEX.sha256) inventories every Git-visible file,
except itself. Authentic license copies and missing evidence are recorded in
[NOTICE](NOTICE), `notices/`, and `qa/license-evidence.json`.
