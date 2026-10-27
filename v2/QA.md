# Finite validation record — 2026-10-26

The checked source HEAD remains `e20b6a750728dc54f6e6f16c87fa21361e8fe151` on
`feat/native-v2-rust`; all port changes are unstaged/uncommitted. No publication,
install, profile mutation, clipboard/selection/browser write, clone deletion,
ledger change or sibling edit was performed.

## Executed checks

| Check | Actual result |
| --- | --- |
| Rust stable 1.98.0, ARM64 | Tests pass; strict fmt and all-target Clippy pass |
| Rust 1.85.0 | Same five Rust tests pass; all-target Clippy passes |
| Production domain oracle | 1,859 actual TS-hook cases, 9 original throws explicitly adapted |
| Source UI oracle | Nine actual TSX layout/action snapshots; all match native Forms/actions |
| Actual JSONL OS-pipes | 3,013 counted checks plus assertions; debug and extracted release binary pass |
| Lifecycle | 140 cancelled opens without dispose, 128-session cap, disposal/reopen, EOF during pending reverse RPC pass |
| ACK/retry | Null, boolean, false, absent/extra keys, null message, error contradictions rejected; explicit exact-byte retry and consumed stale action checks pass |
| Draft/concurrency | A/B/A, delayed field revisions, whole-form unflushed copy, latest search, malformed field failure, surrogate draft recovery pass |
| Pinned schema | Real draft-2020-12 schema with Ajv; positive manifest and six negative cases pass |
| Host runtime/registry | Three Rust fixture tests compile exact validator/registry/asset functions; negatives and exact aliases pass |
| Host renderer | Actual Form/List parsers and patched Form/surface callbacks execute in VM; delayed A/B/A draft and all-field actions pass |
| Original V1 | Disposable root-lock npm ci; 19 original tests and typecheck pass; exact original package-lock untouched |
| Isolated production build | Fresh ignored Cargo home fetched HTTPS SDK revision; ARM64 release build passes without sibling SDK dependency |
| Local development ZIP | Exact inventory/hash verification and extracted-executable process suite pass |
| Windows GNU | `cargo check --locked --target x86_64-pc-windows-gnu --all-targets` passes |
| Windows GNU linking | Failed: `x86_64-w64-mingw32-dlltool` missing; no compiler/tool installation attempted |
| Windows MSVC/live Windows | Not run |
| macOS live GUI/real clipboard | Not run; all producers mocked |
| Full host build | Not run; integration patch remains unapplied |

The SDK `--self-test`/`--protocol-info` switch is metadata only and is not counted
as domain acceptance. No artifact URLs/target pins/catalog entry were invented.
Manifest verification stays `not-run` on both native desktop platforms.

## What the oracle covers

The original own source, root lock, TS/TSX and command assets are retained under
`qa/fixtures/original`. `scripts/oracle.cjs` compiles that exact code with locked
TypeScript 5.9.3 and executes actual hooks/utilities with mocked React state and
Raycast API. The VM only permits those fixture-relative modules and explicit
React/Raycast mocks. Unknown modules and filesystem/OS/process/network/clipboard/
browser producers are denied. Clipboard input is injected, never read from OS.
Node 26.7.0 Buffer is the actual builtin implementation, not a Base64 expectation
mirror. Neither this runtime nor the QA VM ships with the executable.

The corpus spans every radix 2–36; large integers beyond Number precision;
empty, whitespace, BOM, signed, fractional, NaN/Infinity, leading junk and prefix
cases; all eight byte units; unsupported decimal/binary unit labels; scientific
notation, uppercase E, partial/hex exponent parsing, tiny values, integer bit
truncation, and best-unit ordering/rounding. Source byte arithmetic is BigInt,
not the unused floating-point legacy helpers. The original V1 tests separately
exercise those helpers.

Code cases include emoji, CJK, Kazakh, combining marks, NUL/control characters,
malformed UTF-8, padded/unpadded/whitespace/junk/URL-alphabet Base64, percent URI
errors, entity quirks, bracketed hex, comma/packed ASCII, signs/leading-junk and
large parseInt rounding. Deterministic seed `0x12345678` adds finite encoded and
native samples. Source has no random producer. The full finite expectations and
throw observations are inspectable in `qa/oracle.json`; they are not an exhaustive
proof over all strings. Transport rejects unpaired-surrogate JSON according to
the SDK's JSON parser; encoded surrogate drafts get the explicit V2 adaptation.

The oracle initially exposed two real mismatches: Base64 containing an emoji must
use UTF-16 low bytes like Node Buffer. Both were fixed by the Rust decoder; they
are retained as corpus cases. All 1,859 now agree, with the nine original reducer
throws checked as visible bounded rejection. Rust 1.85 Clippy also caught two
format/collect patterns; the final implementation passes both toolchains.

## Effect and draft ownership

The adapter's bounded fence is separate from SDK staged state. It consumes an
action before awaiting an external effect, keeps exact prepared parameters after
cancellation, and requires strict acknowledgement before success. Old action IDs
remain rejected after a failed/cancelled operation. Refresh issues new authority
and exposes an uncertain-operation retry without producing another effect. Editing
or clearing intentionally abandons an uncertain prepared retry. The scoped SDK
cleanup future removes a cancelled open's fence even if dispose never arrives.

Within the documented input bounds, failed conversion drafts remain visible and
old converted values have no copy actions. The host patch's monotonic edit counter
and whole-form values are required for reliable rapid-field GUI behavior. The
source uses no persistent domain store, so corruption/migration/store-symlink QA
would test invented behavior and is not claimed.

## Integration and provenance limits

`tests/host.rs` stubs the list returned by verified package discovery; it does not
reimplement or claim testing full installation. Runtime validator and alias/asset
function bodies come from exact original/patched fixture text; their extraction
hashes are in `qa/host-function-hashes.json`. The patched functions build in the
fixture harness. The patch dry-run applies cleanly to the pinned fixture copy.
It has not been applied to or compiled with the full host.

Initial XERJ generation 1 indexed 22 files/131 records, including all 16 code
files; it exited 3 for four skipped/junk non-code files. Non-code coverage was
incomplete, so exact Git objects, full source reads and rg were authoritative.
No namespace bypass, global mutation or server restart was used. Final index
result is recorded separately in `qa/xerj-final.txt`: it aborted on a frozen
dataset/schema evolution error for v2/Cargo.toml. No new namespace was created.
Final coverage is incomplete; Git byte checks remain authoritative.

Cargo and npm authentic license copies are recorded in `qa/license-evidence.json`.
Six installed-package entries have no recovered standalone license file: r-efi
6.0.0; @esbuild/darwin-arm64 0.27.7 and 0.28.2; @raycast/api 1.104.25; filelist
1.0.6; jake 10.9.4. Their metadata declarations are recorded as declarations only.
r-efi is not linked on the ARM64 or Windows targets; the npm packages are QA only.
No grant was fabricated. Disposable npm audit reported three V1 vulnerabilities
(two low, one high) and one moderate QA Ajv-tree vulnerability; the source lock was
preserved rather than silently updated. These tools are not product dependencies.

`qa/dev-package.json` carries hashes for the local ZIP and every member. Binary
rebuilding may change hashes with paths/toolchain; regenerate the inventory instead
of treating this development artifact as a published release. A clean live host
install and native Windows release/GUI checks remain parent release gates.
