import {
	TextDecoder as Decoder,
	TextEncoder as Encoder,
} from "@kayahr/text-encoding";

// Adapt the library's extended API to the standard global API.
export class TextEncoder implements globalThis.TextEncoder {
	#encoder = new Encoder();

	get encoding(): string {
		return this.#encoder.encoding;
	}

	encode(input = ""): Uint8Array<ArrayBuffer> {
		return this.#encoder.encode(`${input}`);
	}

	encodeInto(
		source: string,
		destination: Uint8Array,
	): TextEncoderEncodeIntoResult {
		if (arguments.length < 2 || !(destination instanceof Uint8Array)) {
			throw new TypeError("encodeInto requires a string and a Uint8Array");
		}
		return this.#encoder.encodeInto(`${source}`, destination);
	}

	get [Symbol.toStringTag](): string {
		return "TextEncoder";
	}
}

export class TextDecoder implements globalThis.TextDecoder {
	#decoder: Decoder;

	constructor(label = "utf-8", options: TextDecoderOptions = {}) {
		// Encoding labels trim ASCII whitespace, not all Unicode whitespace.
		label = `${label}`.replace(/^[\t\n\f\r ]+|[\t\n\f\r ]+$/g, "");
		if (/[^\x20-\x7e]/.test(label)) {
			throw new RangeError("unsupported encoding label");
		}
		options = dictionary(options);
		this.#decoder = new Decoder(label, {
			fatal: Boolean(options?.fatal),
			ignoreBOM: Boolean(options?.ignoreBOM),
		});
	}

	get encoding(): string {
		return this.#decoder.encoding;
	}

	get fatal(): boolean {
		return this.#decoder.fatal;
	}

	get ignoreBOM(): boolean {
		return this.#decoder.ignoreBOM;
	}

	decode(
		input?: ArrayBufferLike | ArrayBufferView,
		options: TextDecodeOptions = {},
	): string {
		if (
			input !== undefined &&
			!(input instanceof ArrayBuffer) &&
			!(
				typeof SharedArrayBuffer === "function" &&
				input instanceof SharedArrayBuffer
			) &&
			!ArrayBuffer.isView(input)
		) {
			throw new TypeError("decode requires a buffer source");
		}
		// Constructing a view also rejects a detached buffer, even when its length is zero.
		if (ArrayBuffer.isView(input)) {
			input = new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
		}
		options = dictionary(options);
		return this.#decoder.decode(input, { stream: Boolean(options?.stream) });
	}

	get [Symbol.toStringTag](): string {
		return "TextDecoder";
	}
}

function dictionary<T extends object>(value: T): T {
	if (
		value !== null &&
		value !== undefined &&
		typeof value !== "object" &&
		typeof value !== "function"
	) {
		throw new TypeError("options must be a dictionary");
	}
	return value;
}
