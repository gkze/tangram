use ../../../test.nu *

# Exercise the standard globals in the selected evaluator, including streaming state.
let server = server spawn
let path = artifact {
	tangram.ts: '
		export default () => {
			const equal = (actual, expected) => tg.assert(JSON.stringify(actual) === JSON.stringify(expected));
			const throws = (f, kind) => {
				try { f(); } catch (error) { tg.assert(error instanceof kind); return; }
				throw new Error("expected an exception");
			};
			const encoder = new TextEncoder();
			equal(encoder.encoding, "utf-8");
			equal([...encoder.encode("Aé😀\ud800Z\udc00")], [65,195,169,240,159,152,128,239,191,189,90,239,191,189]);
			equal([...encoder.encode()], []);
			equal([...encoder.encode(null)], [110,117,108,108]);
			throws(() => encoder.encode(Symbol()), TypeError);
			const buffer = new Uint8Array(8).fill(99);
			equal(encoder.encodeInto("😀é", buffer.subarray(1, 4)), {read:0,written:0});
			equal(encoder.encodeInto("😀é", buffer.subarray(1, 6)), {read:2,written:4});
			equal([...buffer], [99,240,159,152,128,99,99,99]);
			equal(encoder.encodeInto("\ud800", buffer.subarray(0, 3)), {read:1,written:3});
			throws(() => encoder.encodeInto("x", new Uint16Array(2)), TypeError);
			const decoder = new TextDecoder(" UTF8\t");
			equal(decoder.encoding, "utf-8");
			equal(decoder.fatal, false);
			equal(decoder.ignoreBOM, false);
			equal(decoder.decode(encoder.encode("é😀")), "é😀");
			equal(decoder.decode(new DataView(new Uint8Array([0,65,66,0]).buffer,1,2)), "AB");
			equal(decoder.decode(new Uint8Array([65]).buffer), "A");
			equal(decoder.decode(new Uint8Array([0xe2,0x28,0xa1])), "�(�");
			equal(decoder.decode(new Uint8Array([0xed,0xa0,0x80])), "���");
			equal(decoder.decode(new Uint8Array([0xef]), {stream:true}), "");
			equal(decoder.decode(new Uint8Array([0xbb,0xbf,65,0xf0,0x9f]), {stream:true}), "A");
			equal(decoder.decode(new Uint8Array([0x98,0x80])), "😀");
			equal(decoder.decode(new Uint8Array([0xef,0xbb,0xbf,66])), "B");
			equal(new TextDecoder("utf-8", {ignoreBOM:true}).decode(new Uint8Array([0xef,0xbb,0xbf])), "\ufeff");
			decoder.decode(new Uint8Array([0xe2]), {stream:true});
			equal(decoder.decode(), "�");
			equal(decoder.decode(), "");
			const fatal = new TextDecoder("utf-8", {fatal:true});
			fatal.decode(new Uint8Array([0xe2]), {stream:true});
			throws(() => fatal.decode(), TypeError);
			equal(fatal.decode(new Uint8Array([65])), "A");
			throws(() => decoder.decode([65]), TypeError);
			throws(() => decoder.decode(null), TypeError);
			throws(() => new TextDecoder("replacement"), RangeError);
			throws(() => new TextDecoder("\u00a0utf8"), RangeError);
			equal(new TextDecoder("latin1").decode(new Uint8Array([0x80])), "€");
			equal(new TextDecoder("utf-16le").decode(new Uint8Array([0xff,0xfe,65,0])), "A");
			equal(new TextDecoder("utf-16be").decode(new Uint8Array([0,65])), "A");
			const japanese = new TextDecoder("shift_jis");
			equal(japanese.decode(new Uint8Array([0x82]), {stream:true}), "");
			equal(japanese.decode(new Uint8Array([0xa0])), "あ");
			return true;
		};
	'
}
assert equal (tg build $path | from json) true

# The compiler exposes the same bounded global API without importing DOM types.
let types = artifact {
	tangram.ts: '
		export default () => {
			const encoder = new TextEncoder();
			const bytes: Uint8Array = encoder.encode("hello");
			const result: TextEncoderEncodeIntoResult = encoder.encodeInto("hi", bytes);
			const decoder = new TextDecoder("utf-8", {fatal: true, ignoreBOM: false});
			const text: string = decoder.decode(new DataView(bytes.buffer), {stream: false});
			return [text, result.read, decoder.encoding, decoder.fatal, decoder.ignoreBOM];
		};
	'
}
success (tg check $types | complete)
