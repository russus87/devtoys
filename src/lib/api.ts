import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  writeText,
  readText,
} from "@tauri-apps/plugin-clipboard-manager";

export interface Hashes {
  md5: string;
  sha1: string;
  sha256: string;
  sha384: string;
  sha512: string;
  sha3_256: string;
  sha3_512: string;
  crc32: string;
}

/** Result of a self-update check (mirrors the Rust `UpdateInfo`). */
export interface UpdateInfo {
  current: string;
  latest: string | null;
  available: boolean;
  notes: string | null;
  published_at: string | null;
  asset_name: string | null;
  asset_url: string | null;
  installable: boolean;
}

/** Progress of a running update download/install. */
export interface UpdateProgress {
  phase: "download" | "install" | "done" | "error";
  message: string;
  percent: number;
}

/** A category as the Linux tray menu expects it. */
export interface TrayCategory {
  label: string;
  emoji: string;
  tools: { id: string; name: string }[];
}

export const api = {
  hashText: (input: string) => invoke<Hashes>("hash_text", { input }),
  hmacText: (algo: string, key: string, message: string) =>
    invoke<string>("hmac_text", { algo, key, message }),
  genUuids: (version: string, count: number) =>
    invoke<string[]>("gen_uuids", { version, count }),
  base64EncodeBytes: (bytes: number[]) =>
    invoke<string>("base64_encode_bytes", { bytes }),

  // --- tray + self-update -------------------------------------------------
  updateCheck: () => invoke<UpdateInfo>("update_check"),
  updateInstall: () => invoke<void>("update_install"),

  // --- quick-launch popup window -----------------------------------------
  /** Open a tool in the main window and hide the quick-launch popup. */
  quickOpen: (id: string) => invoke<void>("quick_open", { id }),
  /** Hide the quick-launch popup. */
  quickHide: () => invoke<void>("quick_hide"),

  /** Feed the (Linux) tray menu the full tool list, grouped by category. */
  setTrayMenu: (categories: TrayCategory[]) =>
    invoke<void>("set_tray_menu", { categories }),
};

/** True when running inside the Tauri shell (vs. a plain browser dev server). */
export function inTauri(): boolean {
  return typeof (window as unknown as { __TAURI_INTERNALS__?: unknown })
    .__TAURI_INTERNALS__ !== "undefined";
}

/** Subscribe to a Tauri event, tolerating the non-Tauri case. */
export async function on<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  if (!inTauri()) return () => {};
  try {
    return await listen<T>(event, (e) => handler(e.payload));
  } catch {
    return () => {};
  }
}

/** Copy text to the OS clipboard, tolerating the non-Tauri (browser) case. */
export async function copy(text: string): Promise<void> {
  try {
    await writeText(text);
  } catch {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      /* ignore */
    }
  }
}

/** Read text from the OS clipboard, tolerating the non-Tauri case. */
export async function paste(): Promise<string> {
  try {
    return await readText();
  } catch {
    try {
      return await navigator.clipboard.readText();
    } catch {
      return "";
    }
  }
}
