<script lang="ts">
  import { copy, paste } from "../api";

  interface Props {
    label: string;
    value?: string;
    placeholder?: string;
    readonly?: boolean;
    rows?: number;
    showPaste?: boolean;
    onInput?: (v: string) => void;
    children?: import("svelte").Snippet;
  }

  let {
    label,
    value = $bindable(""),
    placeholder = "",
    readonly = false,
    rows = 10,
    showPaste = true,
    onInput,
    children,
  }: Props = $props();

  let copied = $state(false);

  async function doCopy() {
    await copy(value);
    copied = true;
    setTimeout(() => (copied = false), 1100);
  }
  async function doPaste() {
    const t = await paste();
    if (t) {
      value = t;
      onInput?.(t);
    }
  }
  function clear() {
    value = "";
    onInput?.("");
  }
</script>

<div class="pane">
  <div class="pane-head">
    <span class="field-label">{label}</span>
    <div class="pane-actions">
      {@render children?.()}
      {#if showPaste && !readonly}
        <button class="tool-btn" title="Incolla" onclick={doPaste} aria-label="Incolla">⎘</button>
      {/if}
      {#if !readonly}
        <button class="tool-btn" title="Svuota" onclick={clear} aria-label="Svuota">✕</button>
      {/if}
      <button class="tool-btn" class:ok={copied} title="Copia" onclick={doCopy} aria-label="Copia">
        {copied ? "✓" : "⧉"}
      </button>
    </div>
  </div>
  <textarea
    class="code"
    class:readonly
    bind:value
    {placeholder}
    {readonly}
    {rows}
    spellcheck="false"
    autocapitalize="off"
    autocomplete="off"
    oninput={() => onInput?.(value)}
  ></textarea>
</div>

<style>
  .pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }
  .pane-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    min-height: 26px;
  }
  .pane-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .tool-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    font-size: 14px;
    color: var(--ink-dim);
    transition: background 0.14s ease, color 0.14s ease;
  }
  .tool-btn:hover {
    background: var(--surface-3);
    color: var(--ink);
  }
  .tool-btn.ok {
    color: var(--green);
  }
  textarea {
    flex: 1;
    width: 100%;
    resize: none;
    padding: 13px 15px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border-2);
    background: var(--surface);
    color: var(--ink);
    outline: none;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }
  textarea:focus {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px var(--primary-soft);
  }
  textarea.readonly {
    background: var(--surface-2);
  }
  textarea::placeholder {
    color: var(--ink-faint);
  }
</style>
