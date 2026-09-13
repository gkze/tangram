import assert from "node:assert/strict";
import { test } from "node:test";
import { TextDecoder, TextEncoder } from "../src/text.ts";

test("UTF-8 malformed sequences match the native decoder", () => {
	const decoder = new TextDecoder();
	const native = new globalThis.TextDecoder();
	for (let first = 0; first < 256; first++) {
		for (let second = 0; second < 256; second++) {
			const bytes = new Uint8Array([first, second]);
			assert.equal(decoder.decode(bytes), native.decode(bytes));
		}
	}
});

test("streaming preserves every split of multibyte input", () => {
	for (const fatal of [false, true]) {
		for (const ignoreBOM of [false, true]) {
			const bytes = new globalThis.TextEncoder().encode("\ufeffAéあ😀");
			for (let split = 0; split <= bytes.length; split++) {
				const decoder = new TextDecoder("utf-8", { fatal, ignoreBOM });
				const native = new globalThis.TextDecoder("utf-8", {
					fatal,
					ignoreBOM,
				});
				const prefix = bytes.subarray(0, split);
				const suffix = bytes.subarray(split);
				assert.equal(
					decoder.decode(prefix, { stream: true }),
					native.decode(prefix, { stream: true }),
				);
				assert.equal(decoder.decode(suffix), native.decode(suffix));
			}
		}
	}
});

test("encodeInto counts UTF-16 units and never splits a scalar", () => {
	for (const input of ["", "abc", "éあ😀", "\ud800x\udc00", "😀😀"]) {
		for (let capacity = 0; capacity < 20; capacity++) {
			const actual = new Uint8Array(capacity);
			const expected = new Uint8Array(capacity);
			assert.deepEqual(
				new TextEncoder().encodeInto(input, actual),
				new globalThis.TextEncoder().encodeInto(input, expected),
			);
			assert.deepEqual(actual, expected);
		}
	}
});

test("detached buffers and views are rejected", () => {
	const buffer = new ArrayBuffer(4);
	const view = new Uint8Array(buffer);
	structuredClone(buffer, { transfer: [buffer] });
	assert.throws(() => new TextDecoder().decode(buffer), TypeError);
	assert.throws(() => new TextDecoder().decode(view), TypeError);
});
