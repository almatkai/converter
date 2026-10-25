import assert from "node:assert/strict";
import test, { describe, it } from "node:test";
import { parseBigInt } from "../src/utils/common-utils";
import { buildBases, setConverterAuto } from "../src/utils/base-converter-utils";
import {
  convertToBytes,
  convert,
  capitalize,
  insertDot,
  parseBigFloat,
} from "../src/utils/byte-converter-utils";
import {
  nativeToUnicode,
  unicodeToNative,
  nativeToBase64,
  base64ToNative,
  nativeToUtf8,
  utf8ToNative,
  nativeToHex,
  hexToNative,
  nativeToDecimal,
  decimalToNative,
  nativeToUrl,
  urlToNative,
  nativeToEntity,
  entityToNative,
} from "../src/utils/code-converter-utils";

describe("parseBigInt", () => {
  it("parses valid numbers in different radices", () => {
    assert.equal(parseBigInt("10", 10), BigInt(10));
    assert.equal(parseBigInt("ff", 16), BigInt(255));
    assert.equal(parseBigInt("1010", 2), BigInt(10));
    assert.equal(parseBigInt("777", 8), BigInt(511));
  });
  it("returns null on empty or invalid", () => {
    assert.equal(parseBigInt("", 10), null);
    assert.equal(parseBigInt("zz", 10), null);
    assert.equal(parseBigInt("12a", 10), null);
  });
  it("handles base 36", () => {
    assert.equal(parseBigInt("z", 36), BigInt(35));
    assert.equal(parseBigInt("10", 36), BigInt(36));
  });
});

describe("buildBases", () => {
  it("builds 35 bases from 2 to 36", () => {
    const bases = buildBases();
    assert.equal(bases.length, 35);
    assert.equal(bases[0], "2");
    assert.equal(bases[bases.length - 1], "36");
  });
});

describe("setConverterAuto", () => {
  it("detects hex, binary, octal prefixes", () => {
    const calls: unknown[] = [];
    const setter = (base: unknown, value: string, prefix?: string) => calls.push([base, value, prefix]);
    setConverterAuto(setter as never, "0xFF");
    assert.deepEqual(calls[0], [16, "0xFF", "0x"]);
    calls.length = 0;
    setConverterAuto(setter as never, "0b1010");
    assert.deepEqual(calls[0], [2, "0b1010", "0b"]);
    calls.length = 0;
    setConverterAuto(setter as never, "0o777");
    assert.deepEqual(calls[0], [8, "0o777", "0o"]);
  });
  it("detects base32 vs base36 vs decimal", () => {
    const calls: unknown[] = [];
    const setter = (base: unknown, value: string) => calls.push([base, value]);
    setConverterAuto(setter as never, "hello"); // only a-v letters => base32?
    // hello contains 'h','e','l','l','o' - h=17, e=14, l=21, o=24 -> within 0-31? v is 31, so hello letters are < 32? Check regex a-v => hello qualifies (h,e,l,l,o all <= v). So base 32
    assert.deepEqual(calls[0], [32, "hello"]);
    calls.length = 0;
    setConverterAuto(setter as never, "xyz"); // x,y,z > v => base36
    assert.deepEqual(calls[0], [36, "xyz"]);
    calls.length = 0;
    setConverterAuto(setter as never, "12345");
    assert.deepEqual(calls[0], [10, "12345"]);
  });
  it("trims whitespace", () => {
    const calls: unknown[] = [];
    const setter = (base: unknown, value: string, prefix?: string) => calls.push([base, value, prefix]);
    setConverterAuto(setter as never, "  0x10  ");
    assert.deepEqual(calls[0], [16, "0x10", "0x"]);
  });
});

describe("byte converters", () => {
  it("convertToBytes and convert round-trip", () => {
    // unitIndex: 0=b, 1=B, 2=KB, 3=MB etc. Based on utils: 0 is bits
    assert.equal(convertToBytes(8, 0), 1); // 8 bits ->1 byte
    assert.equal(convertToBytes(1, 1), 1); // 1 B ->1 byte
    assert.equal(convertToBytes(1, 2), 1024); // 1 KB ->1024 bytes
    assert.equal(convert(1024, 2), 1); // 1024 bytes ->1 KB
    assert.equal(convert(8, 0), 8 * 8); // wait convert bytes->bits? Actually convert(1,0) =8
    assert.equal(convert(1, 0), 8);
    // 1KB -> bytes -> back
    const bytes = convertToBytes(2, 3); // 2 MB -> 2*1024*1024
    assert.equal(bytes, 2 * 1024 * 1024);
    assert.equal(convert(bytes, 3), 2);
  });
  it("capitalize", () => {
    assert.equal(capitalize("hello"), "Hello");
    assert.equal(capitalize(""), "");
    assert.equal(capitalize("a"), "A");
  });
  it("insertDot formats with 2 decimal places", () => {
    // insertDot(100) -> "1.00" trimmed to "1" ?
    // implementation: n bigint, pad to 3, insert dot 2 from end, trim trailing zeros
    assert.equal(insertDot(BigInt(100)), "1");
    assert.equal(insertDot(BigInt(150)), "1.5");
    assert.equal(insertDot(BigInt(1234)), "12.34");
    assert.equal(insertDot(BigInt(1)), "0.01");
    assert.equal(insertDot(BigInt(10)), "0.1");
  });
  it("parseBigFloat handles exponent and dot", () => {
    const [r, q] = parseBigFloat("1.5");
    // m = "15", exp = 1 ? mantissa len 3, dot idx 1 => exp = 3 -1 -1 =1 => quotient 10, r=15
    // Actually compute: "1.5" -> dot=1, m="15", exp = 3 -1 -1? Wait mantissa len 3, dot 1 => 3 -1 -1 =1 => quotient 10, r=15 => 15/10=1.5
    assert.equal(r, BigInt(15));
    assert.equal(q, BigInt(10));
    const [r2, q2] = parseBigFloat("1e3");
    // mantissa "1", exponent 3 => r=1 *1000=1000, q=1
    assert.equal(r2, BigInt(1000));
    assert.equal(q2, BigInt(1));
  });
});

describe("code converters", () => {
  it("unicode round-trip", () => {
    assert.equal(nativeToUnicode("A"), "\\u0041");
    assert.equal(nativeToUnicode("AB"), "\\u0041\\u0042");
    assert.equal(unicodeToNative("\\u0041\\u0042"), "AB");
    assert.equal(unicodeToNative("  \\u0041 "), "A");
  });
  it("base64 round-trip", () => {
    assert.equal(nativeToBase64("hello"), "aGVsbG8=");
    assert.equal(base64ToNative("aGVsbG8="), "hello");
    assert.equal(base64ToNative("  aGVsbG8=  "), "hello");
  });
  it("utf8 round-trip", () => {
    const s = "hello world";
    assert.equal(utf8ToNative(nativeToUtf8(s)), s);
    // utf8 uses encodeURIComponent -> % -> \x
    assert.equal(nativeToUtf8("a b"), "a\\x20b");
    assert.equal(utf8ToNative("a\\x20b"), "a b");
    assert.equal(utf8ToNative("invalid %"), "");
  });
  it("hex round-trip", () => {
    assert.equal(nativeToHex("A"), "0x41");
    assert.equal(hexToNative("0x41"), "A");
    assert.equal(hexToNative("41"), "A");
    assert.equal(nativeToHex(""), "");
    assert.equal(hexToNative("0x"), "");
  });
  it("decimal round-trip", () => {
    assert.equal(nativeToDecimal("A"), "65");
    assert.equal(decimalToNative("65"), "A");
    assert.equal(nativeToDecimal(""), "");
    assert.equal(decimalToNative(""), "");
    assert.equal(decimalToNative("abc"), "");
    // multi-char
    const s = "Hi";
    const dec = nativeToDecimal(s);
    assert.equal(decimalToNative(dec), s);
  });
  it("url round-trip", () => {
    assert.equal(nativeToUrl("a b"), "a%20b");
    assert.equal(urlToNative("a%20b"), "a b");
    assert.equal(urlToNative("invalid%2"), "");
  });
  it("entity round-trip", () => {
    assert.equal(nativeToEntity("A"), "&#x0041;");
    assert.equal(entityToNative("&#x0041;"), "A");
    assert.equal(entityToNative("&#x0041;&#x0042;"), "AB");
    assert.equal(entityToNative(""), "");
  });
});

test("package.json declares Windows+macOS and Tezbar metadata", async () => {
  const pkg = await import("../package.json", { with: { type: "json" } });
  const data = (pkg as unknown as { default: Record<string, unknown> }).default ?? (pkg as unknown as Record<string, unknown>);
  assert.deepEqual((data as { platforms: string[] }).platforms, ["macOS", "Windows"]);
  const tezbar = (data as { tezbar: { platforms: string[]; repository: string; upstream: { commit: string; path: string } } }).tezbar;
  assert.deepEqual(tezbar.platforms, ["macos", "windows"]);
  assert.equal(tezbar.repository, "https://github.com/almatkai/converter");
  assert.equal(tezbar.upstream.path, "extensions/converter");
  assert.equal(tezbar.upstream.commit, "6734d85992290066fd31a7f5bee3cca3d3545a1a");
  const commands = (data as { commands: { platforms: string[] }[] }).commands;
  assert.equal(commands.length, 3);
  for (const cmd of commands) {
    assert.deepEqual(cmd.platforms, ["macOS", "Windows"]);
  }
});
