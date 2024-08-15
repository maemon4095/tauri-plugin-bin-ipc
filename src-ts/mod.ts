import { type as ostype } from "npm:@tauri-apps/api/os";
import { listen } from "npm:@tauri-apps/api/event";

type BinIpcEventListener = () => void;

const listeners = {} as {
    [id: number]: BinIpcEventListener;
};

listen<number>("bin-ipc:ready", (e) => {
    const id = e.payload;
    listeners[id]?.();
});

function register(id: number, f: () => void) {
    listeners[id] = f;
}

function unregister(id: number) {
    delete listeners[id];
}

export async function invoke_raw(
    name: string,
    command: string,
    payload: Uint8Array,
): Promise<Uint8Array> {
    const origin = await resolveCustomSchemeOrigin(name);
    const id = await spawn(origin, command, payload);
    return await new Promise((resolve, reject) => {
        register(id, () => {
            poll_ready(resolve, reject);
        });
        poll_ready(resolve, reject); // リスナ登録前にコマンドが終了した場合のために、一度pollする
    });

    function poll_ready(
        resolve: (value: Uint8Array) => void,
        reject: (e: unknown) => void,
    ) {
        poll(origin, id)
            .then((r) => {
                if (r === null) return;
                unregister(id);
                resolve(r);
            })
            .catch((e) => {
                unregister(id);
                reject(e);
            });
    }
}

async function spawn(
    origin: string,
    command: string,
    payload: Uint8Array,
): Promise<number> {
    const res = await fetch(`${origin}/ipc/spawn/${command}`, {
        method: "POST",
        body: payload,
    });

    const id = await res.json();
    return id;
}

async function poll(origin: string, id: number): Promise<Uint8Array | null> {
    const res = await fetch(`${origin}/ipc/poll/${id}`, {
        method: "POST",
    });
    if (res.status === 202) {
        return null;
    }
    return await bytes(res.body!);
}

async function resolveCustomSchemeOrigin(scheme: string) {
    const type = await ostype();
    switch (type) {
        case "Darwin":
        case "Linux":
            return `${scheme}://localhost`;
        case "Windows_NT":
            return `https://${scheme}.localhost`;
    }
}

async function bytes(stream: ReadableStream<Uint8Array>) {
    const reader = stream.getReader();
    let first_chunk;
    {
        const { done, value } = await reader.read();
        if (done) {
            return new Uint8Array();
        }
        first_chunk = value;
    }

    const { done, value } = await reader.read();
    if (done) {
        return first_chunk;
    }

    const chunks = [first_chunk, value];
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
