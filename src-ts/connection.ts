export interface Connection {
    readonly writable: WritableStream<Uint8Array>;
    readonly readable: ReadableStream<Uint8Array>;
    close(): void;
}