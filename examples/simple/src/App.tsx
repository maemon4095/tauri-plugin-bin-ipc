import { invoke_raw } from "@bin-ipc";
import { useState } from "preact/hooks";

export default function App() {
  const [response, setResponse] = useState("");

  return (
    <div>
      <button
        onClick={() => {
          invoke_raw("simple-plugin", "hello", new Uint8Array([0, 1, 2]))
            .then(
              (res) => {
                setResponse(new TextDecoder().decode(res));
              },
            );
        }}
      >
        Send hello command
      </button>
      <span>{response}</span>
    </div>
  );
}
