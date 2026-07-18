import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import QuickLaunch from "./lib/QuickLaunch.svelte";

// The quick-launch popup runs in its own Tauri window ("quicklaunch") but from
// the same bundle. Pick which UI to mount based on the current window label.
function windowLabel(): string {
  try {
    const internals = (window as unknown as {
      __TAURI_INTERNALS__?: { metadata?: { currentWindow?: { label?: string } } };
    }).__TAURI_INTERNALS__;
    return internals?.metadata?.currentWindow?.label ?? "main";
  } catch {
    return "main";
  }
}

const isQuick = windowLabel() === "quicklaunch";
if (isQuick) {
  document.documentElement.classList.add("quicklaunch");
}

const app = mount(isQuick ? QuickLaunch : App, {
  target: document.getElementById("app")!,
});

export default app;
