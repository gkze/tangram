import { strict as assert } from "node:assert";
import { test } from "node:test";
import { TextDecoder, TextEncoder } from "../src/text.ts";

// The host runtime's native implementation is the reference for the standard's UTF-8 algorithms.
let NativeDecoder = globalThis.TextDecoder;
let NativeEncoder = globalThis.TextEncoder;

// Represent a decoding failure as a value so that results compare without the error messages.
let failure = Symbol("TypeError");

let outcome = (decode: () => string): string | typeof failure => {
	try {
		return decode();
	} catch (error) {
		assert(error instanceof TypeError);
		return failure;
	}
};

let compare = (bytes: Uint8Array) => {
	for (let fatal of [false, true]) {
		let actual = outcome(() =>
			new TextDecoder("utf-8", { fatal }).decode(bytes),
		);
		let expected = outcome(() =>
			new NativeDecoder("utf-8", { fatal }).decode(bytes),
		);
		assert.equal(actual, expected, `bytes ${[...bytes]} fatal ${fatal}`);
	}
};

test("every one and two byte sequence decodes like the native decoder", () => {
	for (let first = 0; first < 256; first++) {
		compare(new Uint8Array([first]));
		for (let second = 0; second < 256; second++) {
			compare(new Uint8Array([first, second]));
		}
	}
});

test("every three byte sequence with a multibyte lead decodes like the native decoder", () => {
	for (let first = 0xc0; first < 0x100; first++) {
		for (let second = 0; second < 256; second++) {
			for (let third = 0; third < 256; third++) {
				let bytes = new Uint8Array([first, second, third]);
				let actual = new TextDecoder().decode(bytes);
				let expected = new NativeDecoder().decode(bytes);
				if (actual !== expected) {
					assert.equal(actual, expected, `bytes ${[...bytes]}`);
				}
			}
		}
	}
});

test("four byte sequences decode like the native decoder", () => {
	let edges = [
		0x00, 0x41, 0x7f, 0x80, 0x8f, 0x90, 0x9f, 0xa0, 0xbf, 0xc0, 0xf4, 0xff,
	];
	for (let first = 0xf0; first < 0x100; first++) {
		for (let second = 0; second < 256; second++) {
			for (let third of edges) {
				for (let fourth of edges) {
					compare(new Uint8Array([first, second, third, fourth]));
				}
			}
		}
	}
});

test("streaming preserves every split of valid and malformed input", () => {
	let inputs = [
		new NativeEncoder().encode("﻿Aéࠀあ￿😀\u{10ffff}"),
		new Uint8Array([0xef, 0xbb, 0xbf, 0xef, 0xbb, 0xbf, 0x41]),
		new Uint8Array([
			0xe2, 0x28, 0xa1, 0xf0, 0x9f, 0x98, 0xed, 0xa0, 0x80, 0xf4, 0x90, 0x80,
			0x80, 0xc0, 0xaf, 0xe0, 0x80, 0xe2, 0x82,
		]),
	];
	for (let bytes of inputs) {
		for (let ignoreBOM of [false, true]) {
			let expected = new NativeDecoder("utf-8", { ignoreBOM }).decode(bytes);
			for (let first = 0; first <= bytes.length; first++) {
				for (let second = first; second <= bytes.length; second++) {
					let decoder = new TextDecoder("utf-8", { ignoreBOM });
					let actual =
						decoder.decode(bytes.subarray(0, first), { stream: true }) +
						decoder.decode(bytes.subarray(first, second), { stream: true }) +
						decoder.decode(bytes.subarray(second));
					assert.equal(
						actual,
						expected,
						`bytes ${[...bytes]} splits ${first} ${second}`,
					);
				}
			}
			let decoder = new TextDecoder("utf-8", { ignoreBOM });
			let actual =
				[...bytes]
					.map((byte) =>
						decoder.decode(new Uint8Array([byte]), { stream: true }),
					)
					.join("") + decoder.decode();
			assert.equal(actual, expected);
		}
	}
});

test("a decoder resets after a call that does not stream", () => {
	let decoder = new TextDecoder();
	assert.equal(decoder.decode(new Uint8Array([0xef, 0xbb, 0xbf, 0x41])), "A");
	assert.equal(decoder.decode(new Uint8Array([0xef, 0xbb, 0xbf, 0x42])), "B");
	assert.equal(decoder.decode(new Uint8Array([0xe2]), { stream: true }), "");
	assert.equal(decoder.decode(), "�");
	assert.equal(decoder.decode(), "");
	assert.equal(
		new TextDecoder("utf-8", { ignoreBOM: true }).decode(
			new Uint8Array([0xef, 0xbb, 0xbf]),
		),
		"﻿",
	);
});

test("a fatal decoder throws and keeps the unprocessed bytes of a streaming call", () => {
	let decoder = new TextDecoder("utf-8", { fatal: true });
	decoder.decode(new Uint8Array([0xe2]), { stream: true });
	assert.throws(() => decoder.decode(), TypeError);
	assert.equal(decoder.decode(new Uint8Array([0x41])), "A");
	assert.throws(
		() => decoder.decode(new Uint8Array([0xe2, 0x41]), { stream: true }),
		TypeError,
	);
	assert.equal(decoder.decode(new Uint8Array([0x42])), "AB");
	assert.throws(() => decoder.decode(new Uint8Array([0xff, 0x41])), TypeError);
	assert.equal(decoder.decode(new Uint8Array([0x42])), "B");
});

test("encode and encodeInto match the native encoder", () => {
	let inputs = [
		"",
		"abc",
		"é",
		"あ",
		"😀",
		"\ud800",
		"\udc00",
		"\ud800x\udc00",
		"😀😀",
		"a􏿿b",
		"\udc00\ud800",
	];
	for (let input of inputs) {
		assert.deepEqual(
			new TextEncoder().encode(input),
			new NativeEncoder().encode(input),
		);
		for (let capacity = 0; capacity < 12; capacity++) {
			let actual = new Uint8Array(capacity).fill(0x63);
			let expected = new Uint8Array(capacity).fill(0x63);
			assert.deepEqual(
				new TextEncoder().encodeInto(input, actual),
				new NativeEncoder().encodeInto(input, expected),
			);
			assert.deepEqual(actual, expected);
		}
	}
});

test("arguments are converted like the standard's Web IDL", () => {
	let encoder = new TextEncoder();
	assert.equal(encoder.encoding, "utf-8");
	assert.deepEqual([...encoder.encode()], []);
	assert.deepEqual(
		[...encoder.encode(null as unknown as string)],
		[0x6e, 0x75, 0x6c, 0x6c],
	);
	assert.throws(() => encoder.encode(Symbol() as unknown as string), TypeError);
	assert.throws(
		() => encoder.encodeInto("x", new Uint16Array(2) as unknown as Uint8Array),
		TypeError,
	);
	assert.throws(
		() => (encoder.encodeInto as (source: string) => unknown)("x"),
		TypeError,
	);
	assert.throws(() => TextEncoder.prototype.encode.call({}, "x"), TypeError);

	let decoder = new TextDecoder();
	assert.equal(
		decoder.decode(
			new DataView(new Uint8Array([0, 0x41, 0x42, 0]).buffer, 1, 2),
		),
		"AB",
	);
	assert.equal(decoder.decode(new Uint8Array([0x41]).buffer), "A");
	assert.equal(decoder.decode(new Uint16Array([0x4241])), "AB");
	assert.equal(
		decoder.decode(new Uint8Array([0x41]), null as unknown as undefined),
		"A",
	);
	assert.throws(
		() => decoder.decode([0x41] as unknown as Uint8Array),
		TypeError,
	);
	assert.throws(() => decoder.decode(null as unknown as undefined), TypeError);
	assert.throws(
		() => decoder.decode(new Uint8Array(), 1 as unknown as undefined),
		TypeError,
	);
	assert.throws(
		() => new TextDecoder("utf-8", true as unknown as undefined),
		TypeError,
	);
	assert.throws(
		() =>
			Object.getOwnPropertyDescriptor(
				TextDecoder.prototype,
				"fatal",
			)!.get!.call({}),
		TypeError,
	);
	assert.equal(Object.prototype.toString.call(decoder), "[object TextDecoder]");
	assert.equal(Object.prototype.toString.call(encoder), "[object TextEncoder]");
});

test("a detached buffer decodes as empty input", () => {
	let buffer = new ArrayBuffer(4);
	let view = new Uint8Array(buffer);
	buffer.transfer();
	assert.equal(new TextDecoder().decode(buffer), "");
	assert.equal(new TextDecoder().decode(view), "");
});

test("only UTF-8 labels are supported", () => {
	for (let label of [
		"utf-8",
		"UTF-8",
		" utf8\t",
		"\nUnicode-1-1-UTF-8\f",
		"unicode11utf8",
		"unicode20utf8",
		"x-unicode20utf8",
		undefined,
	]) {
		let decoder = new TextDecoder(label);
		assert.equal(decoder.encoding, "utf-8");
		assert.equal(decoder.fatal, false);
		assert.equal(decoder.ignoreBOM, false);
	}
	for (let label of [
		"utf-16le",
		"latin1",
		"shift_jis",
		"replacement",
		" utf8",
		"utf-8\u0000",
		"",
		"utf 8",
	]) {
		assert.throws(() => new TextDecoder(label), RangeError);
	}
	assert.throws(
		() => new TextDecoder(Symbol() as unknown as string),
		TypeError,
	);
});
