import { useState } from "preact/hooks";
import { invoke } from "@bin-ipc";

// @deno-types="@types/file.d.ts"
import denoLogo from "../public/deno.svg";
// @deno-types="@types/file.d.ts"
import tauriLogo from "../public/tauri.svg";
// @deno-types="@types/file.d.ts"
import preactLogo from "../public/preact.svg";

export default function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("bin-ipc", "greet", { name }) as string);
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
        <a href="https://deno.com" target="_blank">
          <img
            src={denoLogo}
            className="logo deno"
            alt="Deno logo bit modified from https://github.com/denolib/high-res-deno-logo"
          />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src={tauriLogo} className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://preactjs.com" target="_blank">
          <img src={preactLogo} className="logo preact" alt="Preact logo" />
        </a>
      </div>

      <p>Click on the Tauri, Deno, and Preact logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          invoke("bin-ipc", "greet", {});
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <p>{greetMsg}</p>
    </div>
  );
}
