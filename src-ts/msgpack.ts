import { invoke_raw } from "./core.ts";
import { decode, encode } from "npm:@msgpack/msgpack";

export async function invoke(
    name: string,
    command: string,
    payload: unknown,
): Promise<unknown> {
    const req = encode(payload);
    const res = await invoke_raw(name, command, req);
    return decode(res);
}
