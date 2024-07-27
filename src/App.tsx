import "./styles.css";
import { Metar } from "./Metar.tsx";
import { batch, createSignal, For } from "solid-js";
import { createStore } from "solid-js/store";
// @ts-ignore
import { autofocus } from "@solid-primitives/autofocus";
import { getCurrentWindow, PhysicalSize } from "@tauri-apps/api/window";
import { logIfDev } from "./logging.ts";

const [inputId, setInputId] = createSignal("");
const [ids, setIds] = createStore<string[]>([]);

function removeIndex<T>(array: readonly T[], index: number): T[] {
  return [...array.slice(0, index), ...array.slice(index + 1)];
}

function App() {
  let containerRef: HTMLDivElement | undefined;
  let window = getCurrentWindow();

  // Setup titlebar
  document.getElementById("titlebar-minimize")?.addEventListener("click", () => window.minimize());
  document.getElementById("titlebar-close")?.addEventListener("click", () => window.close());

  async function resetWindowHeight() {
    if (containerRef !== undefined) {
      let currentSize = await window.innerSize();
      logIfDev("Current window size", currentSize);
      logIfDev("containerRef height", containerRef.offsetHeight);
      let scaleFactor = await window.scaleFactor();
      logIfDev("Scale factor", scaleFactor);
      await window.setSize(
        new PhysicalSize(currentSize.width, (containerRef.offsetHeight + 24) * scaleFactor)
      );
    }
  }

  async function addStation(e: SubmitEvent) {
    e.preventDefault();
    batch(() => {
      if (inputId().length >= 3 && inputId().length <= 4) {
        setIds(ids.length, inputId());
        setInputId("");
      }
    });
    await resetWindowHeight();
  }

  async function removeStation(index: number) {
    setIds((ids) => removeIndex(ids, index));
    await resetWindowHeight();
  }

  return (
    <div class="flex flex-col bg-black text-white" ref={containerRef}>
      <div class="flex flex-col grow">
        <For each={ids}>
          {(id, i) => (
            <div class="flex">
              <div
                class="flex w-4 h-5 items-center cursor-pointer"
                onClick={async () => removeStation(i())}
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke-width="1.5"
                  class="size-4 stroke-red-700 hover:stroke-red-500 transition-colors"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 12h14" />
                </svg>
              </div>
              <Metar requestedId={id} resizeFn={resetWindowHeight} />
            </div>
          )}
        </For>
        <form onSubmit={async (e) => addStation(e)}>
          <input
            id="stationId"
            name="stationId"
            type="text"
            class="w-16 text-white font-mono bg-gray-900 mx-1 my-1 border-gray-700 border focus:outline-none focus:border-gray-500 px-1"
            value={inputId()}
            onInput={(e) => setInputId(e.currentTarget.value)}
            use:autofocus
            autofocus
            formNoValidate
            autocomplete="off"
          />
        </form>
      </div>
    </div>
  );
}

export default App;
