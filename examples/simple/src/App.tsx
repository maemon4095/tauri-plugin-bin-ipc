import { invoke_raw } from "@bin-ipc";
import { useReducer, useState } from "preact/hooks";

const commandScheme = "simple-plugin";

type ModifyCommand =
  | {
    type: "new";
    command: string;
    request: string;
  }
  | {
    type: "setResponse";
    requestIndex: number;
    response: string;
  };

export default function App() {
  const [requestCommand, setRequestCommand] = useState("hello");
  const [requestMessage, setRequestMessage] = useState("hello via binary!");
  const [requestResponsePairs, modifyRequestResponsePairs] = useRichReducer(
    (pairs, command: ModifyCommand) => {
      switch (command.type) {
        case "new": {
          const requestIndex = pairs.length;
          return [
            [...pairs, { requestIndex, ...command }],
            requestIndex,
          ];
        }
        case "setResponse": {
          const newPairs = [...pairs];
          const oldPair = newPairs[command.requestIndex];
          newPairs[command.requestIndex] = {
            ...oldPair,
            response: command.response,
          };
          return [newPairs, null];
        }
      }
    },
    [] as ({
      command: string;
      request: string;
      requestIndex: number;
      response?: string;
    })[],
  );

  return (
    <div class="size-full flex flex-col gap-1 items-stretch overflow-hidden p-2">
      <div class="overflow-y-auto flex-1">
        <table class="w-full [&_td]:text-center">
          <thead class="bg-white sticky top-0 shadow">
            <tr>
              <th scope="col">Request No.</th>
              <th scope="col">Command</th>
              <th scope="col">Request Message</th>
              <th scope="col">Response Message</th>
            </tr>
          </thead>
          <tbody>
            {requestResponsePairs.map((e) => (
              <tr key={e.requestIndex}>
                <th scope="row">{e.requestIndex}</th>
                <td>{e.command}</td>
                <td>{e.request}</td>
                <td>{e.response}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div class="flex flex-row items-stretch gap-1 p-1 rounded bg-slate-200">
        <label>
          command:&nbsp;
          <input
            value={requestCommand}
            onInput={(v) => setRequestCommand(v.currentTarget.value)}
          />
        </label>
        <label class="flex-1 flex">
          message:&nbsp;
          <input
            class="flex-1"
            value={requestMessage}
            onInput={(v) => setRequestMessage(v.currentTarget.value)}
          />
        </label>
        <button
          onClick={() => {
            const requestIndex = modifyRequestResponsePairs({
              type: "new",
              command: requestCommand,
              request: requestMessage,
            })!;
            invoke_raw(
              commandScheme,
              requestCommand,
              new TextEncoder().encode(requestMessage),
            )
              .then(
                (res) => {
                  const response = new TextDecoder().decode(res);
                  modifyRequestResponsePairs({
                    type: "setResponse",
                    response,
                    requestIndex,
                  });
                },
              );
          }}
          class="bg-gray-300 px-1 rounded shadow-sm"
        >
          Send
        </button>
      </div>
    </div>
  );
}

function useRichReducer<T, A, U>(
  reducer: (old: T, action: A) => [T, U],
  initialState: T,
) {
  let currentRet: U;
  const [state, dispatch] = useReducer<T, A>((old, action) => {
    const [next, ret] = reducer(old, action);
    currentRet = ret;
    return next;
  }, initialState);

  return [state, (action: A): U => {
    dispatch(action);
    return currentRet;
  }] as const;
}
