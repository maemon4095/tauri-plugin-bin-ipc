import { Fragment, h } from "preact";
import { useState } from "preact/hooks";
import { connect, Connection } from "taurp-plugin-bin-ipc";

export default function App() {
  const [payload, setPayload] = useState("");
  const [conn, setConn] = useState<Connection>();

  return (
    <div>
      <h1>Welcome to Tauri!</h1>
      {conn
        ? (
          <>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (!conn) return;
                const writer = conn.writable.getWriter();
                writer.write(new TextEncoder().encode(payload));
                writer.releaseLock();
              }}
            >
              <input
                onChange={(e) => setPayload(e.currentTarget.value)}
                placeholder="Enter payload"
              />
              <button type="submit">Send</button>
            </form>
            <button
              onClick={() => {
                conn?.close();
                setConn(undefined);
              }}
            >
              disconnect
            </button>
          </>
        )
        : (
          <>
            <button
              onClick={async () => {
                const conn = await connect("bin-ipc");
                setConn(conn);
                console.log("connected");
                console.log(conn);
                const response = conn.readable.pipeThrough(
                  new TextDecoderStream(),
                );
                const reader = response.getReader();
                (async () => {
                  while (true) {
                    const { done, value } = await reader.read();
                    if (done) break;
                    console.log(value);
                  }
                })();
              }}
            >
              connect
            </button>
          </>
        )}
    </div>
  );
}
