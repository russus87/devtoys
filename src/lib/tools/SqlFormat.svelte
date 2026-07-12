<script lang="ts">
  import { format } from "sql-formatter";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let dialect = $state("sql");
  let kw = $state("upper");
  let input = $state("");

  const computed = $derived.by(() => {
    const src = input.trim();
    if (!src) return { output: "", error: "" };
    try {
      return {
        output: format(input, {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          language: dialect as any,
          keywordCase: kw === "upper" ? "upper" : "preserve",
        }),
        error: "",
      };
    } catch (e) {
      return { output: "", error: (e as Error).message };
    }
  });
  const output = $derived(computed.output);
  const error = $derived(computed.error);
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={dialect}
      options={[
        { value: "sql", label: "Standard" },
        { value: "postgresql", label: "PostgreSQL" },
        { value: "plsql", label: "Oracle" },
        { value: "mysql", label: "MySQL" },
        { value: "tsql", label: "SQL Server" },
      ]}
    />
    <Segmented
      bind:value={kw}
      options={[
        { value: "upper", label: "MAIUSCOLE parole chiave" },
        { value: "preserve", label: "Mantieni originale" },
      ]}
    />
  </div>
  <div class="split">
    <Pane label="Input" bind:value={input} placeholder="Incolla qui la query SQL…" />
    <div class="stack grow">
      <Pane label="Output" value={output} readonly showPaste={false} />
      {#if error}<div class="notice bad">{error}</div>{/if}
    </div>
  </div>
</div>
