# Converter for Tezbar

Arbitrary input, direct output – Base, Byte and Code converters in Tezbar on Windows or macOS.

## Commands

- **Base Converter** — convert numbers between bases 2–36 (binary, octal, decimal, hex, base32, base36). Simple view shows 2/4/8/10/16/32; advance view lets you pick any two bases. Supports `0x`/`0b`/`0o` prefixes and auto-detection.
- **Byte Converter** — convert between bit/byte units (b, B, KB, MB, GB, TB, PB). Shows precision to 2 decimals and best-unit expression.
- **Code Converter** — bidirectional conversion between native text and Unicode (`\uXXXX`), Base64, UTF-8 (`\x`), ASCII (comma or packed), hex (`0x`), decimal, URL encoding and HTML entity (`&#x`).

All converters respect the `Auto Paste` preference (clipboard auto-fill on open) and per-command preferences: `Advance View`, `Advance View Location`, `Comma separated ASCII`.

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform `@raycast/api` and pure JS/TS helpers and retains the upstream macOS-compatible design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `converter` extension at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author `koinzhang` and contributors `Saafo`, `fmkra` remain credited in `package.json`.

Standalone Rust V2 development port: [v2/README.md](v2/README.md).
