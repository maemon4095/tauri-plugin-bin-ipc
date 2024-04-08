import { type as ostype } from "@tauri-apps/api/os";
import { InterruptibleLock as Lock } from "https://raw.githubusercontent.com/maemon4095/ts_components/release/v0.3.0/lock/mod.ts";
import { listen } from "@tauri-apps/api/event";

const BIN_IPC_EVENT_NAME = "bin-ipc-signal";
type BinPicEventType = "ready-to-pop" | "disconnect";
const readyToPopListeners = {} as { [id: number]: undefined | (() => void) | (() => Promise<unknown>); };
type BinIpcEventHandler = (() => unknown) | (() => Promise<unknown>);
type BinIpcEventListener = {
    readyToPop: BinIpcEventHandler;
    disconnect: BinIpcEventHandler;
};

const listeners = {} as {
    [scheme: string]: {
        [id: number]: BinIpcEventListener;
    };
};

await listen<{ type: BinPicEventType; scheme: string; id: number; }>(BIN_IPC_EVENT_NAME, e => {
    const { type, scheme, id } = e.payload;
    switch (type) {
        case "ready-to-pop": {
            listeners[scheme][id].readyToPop();
            break;
        }
        case "disconnect": {
            listeners[scheme][id].disconnect();
            break;
        }
    }
});

async function resolveBinaryChannel(scheme: string) {
    const type = await ostype();
    switch (type) {
        case "Darwin":
        case "Linux":
            return `${scheme}://localhost`;
        case "Windows_NT":
            return `$https://${scheme}.localhost`;
    }
}

async function handshake(host: string) {
    const res = await fetch(`${host}/connect`, { method: "POST" });
    const js = await res.json() as { id: unknown, key: unknown; };
    if (!Number.isSafeInteger(js.id)) {
        throw new Error(); // too many channels
    }
    if (!Number.isSafeInteger(js.key)) {
        throw new Error(); // key generation error
    }
    return js as { id: number, key: number; };
}

export async function connect(scheme: string) {
    const host = await resolveBinaryChannel(scheme);
    const { id, key } = await handshake(host);
    const channel = `${host}/${id}/${key}`;
    const popURL = `${channel}/pop`;
    const pushURL = `${channel}/push`;
    const closeUpstreamURL = `${channel}/close/up`;
    const closeDownstreamURL = `${channel}/close/down`;
    const disconnectURL = `${channel}/disconnect`;
    const listener = {} as BinIpcEventListener;
    const channels = listeners[scheme];
    channels[id] = listener as BinIpcEventListener;

    const upstreamAbortController = new AbortController();
    let disconnected = false;
    listener.disconnect = async () => {
        disconnected = true;
        upstreamAbortController.abort();
        delete channels[id];
        await fetch(disconnectURL, { method: "POST" });
    };

    const upstreamLock = new Lock();
    const upstream = new WritableStream({
        async write(chunk, controller) {
            if (disconnected) {
                controller.error();
                return;
            }
            try {
                await upstreamLock.acquire();
                await fetch(pushURL, {
                    signal: AbortSignal.any([controller.signal, upstreamAbortController.signal]),
                    method: "POST",
                    body: chunk,
                });
            } catch (e) {
                controller.error(e);
            } finally {
                upstreamLock.release();
            }
        },
        async close() {
            try {
                await upstreamLock.acquire();
                await fetch(closeUpstreamURL, {
                    method: "POST",
                });
            } finally {
                upstreamLock.release();
            }
        },
        async abort() {
            try {
                await upstreamLock.interrupt();
                await fetch(closeUpstreamURL, {
                    method: "POST",
                });
            } finally {
                upstreamLock.release();
            }
        }
    });

    const downstreamLock = new Lock();
    const downstream = new ReadableStream({
        type: "bytes",
        start(controller) {
            const discon = listener.disconnect;
            listener.disconnect = async () => {
                controller.close();
                await discon();
            };

            listener.readyToPop = async () => {
                try {
                    await downstreamLock.acquire();
                    const res = await fetch(popURL, {
                        method: "POST",
                    });

                    switch (res.status) {
                        case 100: {
                            break;
                        }
                        case 200: {
                            // Depending on enviroment, ReadableStream may not be AsyncIterable.
                            const reader = res.body!.getReader();
                            try {
                                while (true) {
                                    const { done, value } = await reader.read();
                                    if (done) {
                                        break;
                                    }
                                    controller.enqueue(value);
                                }
                            } finally {
                                reader.releaseLock();
                            }
                            break;
                        }
                        case 204: {
                            controller.close();
                            break;
                        }
                    }
                } catch (e) {
                    controller.error(e);
                } finally {
                    downstreamLock.release();
                }
            };
        },
        async cancel() {
            try {
                await downstreamLock.acquire();
                delete readyToPopListeners[id];
                await fetch(closeDownstreamURL, {
                    method: "POST",
                });
            } finally {
                downstreamLock.release();
            }
        },
    });


    return { id, upstream, downstream };
}