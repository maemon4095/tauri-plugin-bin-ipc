import { type as ostype } from "npm:@tauri-apps/api/os";

const SCHEME_PATTERN = /[A-Za-z]([A-Za-z0-9+-.]*)/;

export async function resolveCustomSchemeOrigin(scheme: string) {
    if (!SCHEME_PATTERN.test(scheme)) {
        throw new Error("invalid scheme");
    }
    const type = await ostype();
    switch (type) {
        case "Darwin":
        case "Linux":
            return `${scheme}://localhost`;
        case "Windows_NT":
            return `https://${scheme}.localhost`;
    }
}

export async function bytes(stream: ReadableStream<Uint8Array>) {
    const reader = stream.getReader();
    const { done: done0, value: first_chunk } = await reader.read();
    if (done0) {
        return new Uint8Array();
    }
    const { done: done1, value: second_chunk } = await reader.read();
    if (done1) {
        return first_chunk;
    }

    return slow_pass([first_chunk, second_chunk], reader);

    async function slow_pass(
        chunks: Uint8Array[],
        reader: ReadableStreamDefaultReader<Uint8Array>,
    ) {
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            chunks.push(value);
        }

        const len = chunks.reduce((acc, now) => acc + now.length, 0);
        const buf = new Uint8Array(len);

        let offset = 0;
        for (const chunk of chunks) {
            buf.set(chunk, offset);
            offset += chunk.length;
        }
        return buf;
    }
}
