import { invoke } from "@tauri-apps/api/core";
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

export const api = {
  hashText: (input: string) => invoke<Hashes>("hash_text", { input }),
  hmacText: (algo: string, key: string, message: string) =>
    invoke<string>("hmac_text", { algo, key, message }),
  genUuids: (version: string, count: number) =>
    invoke<string[]>("gen_uuids", { version, count }),
  base64EncodeBytes: (bytes: number[]) =>
    invoke<string>("base64_encode_bytes", { bytes }),
};

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
