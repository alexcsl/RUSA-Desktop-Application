<!--
  LatexPreview.svelte — Renders LaTeX strings using KaTeX.
  Props:
    source: string — the LaTeX source to render
    displayMode?: boolean — block (true) vs inline (false, default)
-->
<script lang="ts">
  import katex from 'katex';
  import 'katex/dist/katex.min.css';
  
  let { source = '', displayMode = false }: { source: string; displayMode?: boolean } = $props();

  let rendered = $derived(renderLatex(source));

  function renderLatex(src: string): string {
    if (!src.trim()) return '<span style="color:#475569;font-style:italic;font-size:0.8rem">No LaTeX content yet</span>';
    try {
      return katex.renderToString(src, {
        displayMode,
        throwOnError: false,
        trust: true,
        strict: false,
      });
    } catch {
      return `<span style="color:#F59E0B;font-size:0.8rem">Parse error</span>`;
    }
  }
</script>

<div class="latex-box" class:display-mode={displayMode}>
  {@html rendered}
</div>

<style>
  .latex-box {
    background: #0B0F1A;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 0.75rem;
    color: #E6EDF3;
    font-size: 1rem;
    line-height: 1.8;
    overflow-x: auto;
    min-height: 2.5rem;
  }

  .latex-box.display-mode {
    text-align: center;
    font-size: 1.15rem;
    padding: 1rem;
  }
</style>
