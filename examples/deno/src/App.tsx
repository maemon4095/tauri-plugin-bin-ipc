import { h } from "preact";
import { useEffect, useRef, useState } from "preact/hooks";
import { connect, Connection } from "taurp-plugin-bin-ipc";

export default function App() {
  const [payload, setPayload] = useState("");
  const [conn, setConn] = useState<Connection>();
  useEffect(() => {
    (async () => {
      const conn = await connect("bin-ipc");
      setConn(conn);
      console.log("connected");
      console.log(conn);
      const response = conn.readable.pipeThrough(new TextDecoderStream());
      const reader = response.getReader();
      (async () => {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          console.log(value);
        }
      })();
    })();
  }, []);

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          const writer = conn!.writable.getWriter();
          writer.write(new TextEncoder().encode(payload));
          writer.releaseLock();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setPayload(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
    </div>
  );
}
