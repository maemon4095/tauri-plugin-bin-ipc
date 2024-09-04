import { bytes, resolveCustomSchemeOrigin } from "./util.ts";
import { listen } from "npm:@tauri-apps/api/event";

type BinIpcEventListener = () => void;

const listeners = {} as {
    [id: number]: BinIpcEventListener;
};

const HEADER_BIN_IPC_STATUS = "bin-ipc-status";

await listen<number>("bin-ipc:ready", (e) => {
    const id = e.payload;
    listeners[id]?.();
});

function register(id: number, f: () => void) {
    listeners[id] = f;
}

function unregister(id: number) {
    delete listeners[id];
}

async function spawn(
    origin: string,
    command: string,
    payload: Uint8Array,
): Promise<number> {
    const res = await fetch(`${origin}/ipc/spawn/${command}`, {
        method: "POST",
        body: payload,
        cache: "no-cache",
    });

    const status = res.headers.get(HEADER_BIN_IPC_STATUS);

    switch (status) {
        case "ok": {
            const id = await res.json();
            return id;
        }

        case "error": {
            throw new Error(await res.text());
        }

        default: {
            throw_unknown_status();
        }
    }
}

async function poll(origin: string, id: number): Promise<Uint8Array | null> {
    const res = await fetch(`${origin}/ipc/poll/${id}`, {
        method: "POST",
        cache: "no-cache",
    });

    const status = res.headers.get(HEADER_BIN_IPC_STATUS);

    switch (status) {
        case "ok":
            return await bytes(res.body!);
        case "pending":
            return null;
        case "error":
            throw new Error(await res.text());
        default:
            throw_unknown_status();
    }
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

function throw_unknown_status(): never {
    throw new Error("server responsed with unknown status.");
}
