export interface BinIpcStream {
    readonly id: number,
    readonly writable: WritableStream<Uint8Array>;
    readonly readable: ReadableStream<Uint8Array>;
    close(): void;
}