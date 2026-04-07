<!-- /directors/accountant/queue — TheAccountant Financial Monitoring (UC-DIR-11) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getFinancialQueue, flagBudgetReport, dirGetFileSignedUrl,
    type FinancialDocument,
  } from '$lib/stores/directors';

  type Tab = 'budget' | 'expenditure' | 'invoices';
  let activeTab = $state<Tab>('budget');

  let queue: FinancialDocument[] = $state([]);
  let loadError = $state('');

  // Per-tab selected document
  let selected: FinancialDocument | null = $state(null);
  let flagReason = $state('');
  let flagError = $state('');
  let flagSuccess = $state('');

  // Invoice viewer
  let invoiceUrl = $state('');
  let invoiceLoading = $state(false);
  let invoiceError = $state('');

  onMount(async () => {
    try {
      queue = await getFinancialQueue();
    } catch (e) {
      loadError = String(e);
    }
  });

  // Derived document lists
  const budgetDocs = $derived(
    queue.filter(d => !isExpenditure(d) && !hasInvoiceOnly(d))
  );
  const expenditureDocs = $derived(
    queue.filter(d => isExpenditure(d))
  );
  const invoiceDocs = $derived(
    queue.filter(d => d.invoice_storage_path != null)
  );

  function isExpenditure(d: FinancialDocument): boolean {
    const p = d.payload as Record<string, unknown>;
    return p?.report_type === 'expenditure' || d.title.toLowerCase().includes('expenditure');
  }

  function hasInvoiceOnly(d: FinancialDocument): boolean {
    return false; // invoices tab shows all docs with storage path — no exclusion needed
  }

  function selectDoc(doc: FinancialDocument) {
    selected = doc;
    flagReason = '';
    flagError = '';
    flagSuccess = '';
    invoiceUrl = '';
    invoiceError = '';
  }

  async function handleFlag() {
    if (!selected || !flagReason.trim()) { flagError = 'Reason required.'; return; }
    flagError = ''; flagSuccess = '';
    try {
      await flagBudgetReport(selected.id, flagReason);
      flagSuccess = `Flagged "${selected.title}" — vote session created.`;
      flagReason = '';
      selected = null;
      queue = await getFinancialQueue();
    } catch (e: unknown) { flagError = e instanceof Error ? e.message : String(e); }
  }

  async function loadInvoice(path: string) {
    if (invoiceUrl) return;
    invoiceLoading = true;
    invoiceError = '';
    try {
      const res = await dirGetFileSignedUrl(path);
      invoiceUrl = res.signed_url;
    } catch (e) {
      invoiceError = String(e);
    } finally {
      invoiceLoading = false;
    }
  }

  function fmtDate(d: string) {
    return new Date(d).toLocaleDateString();
  }

  function fmtCurrency(val: unknown): string {
    const n = Number(val);
    return isNaN(n) ? '—' : `$${n.toFixed(2)}`;
  }

  function payloadFields(d: FinancialDocument): { label: string; value: string }[] {
    const p = d.payload as Record<string, unknown>;
    const fields: { label: string; value: string }[] = [];

    if (p.department) fields.push({ label: 'Department', value: String(p.department) });
    if (p.total_amount != null) fields.push({ label: 'Total Amount', value: fmtCurrency(p.total_amount) });
    if (p.report_type) fields.push({ label: 'Report Type', value: String(p.report_type) });

    const items = p.line_items as unknown[] | undefined;
    if (Array.isArray(items) && items.length > 0) {
      items.forEach((item, i) => {
        if (typeof item === 'object' && item !== null) {
          const it = item as Record<string, unknown>;
          const desc = it.description ?? it.name ?? it.item ?? `Item ${i + 1}`;
          const amt = it.amount ?? it.cost ?? it.price ?? it.quantity;
          fields.push({
            label: `Line item ${i + 1}`,
            value: amt != null ? `${desc} — ${fmtCurrency(amt)}` : String(desc),
          });
        }
      });
    }

    return fields;
  }

  function switchTab(tab: Tab) {
    activeTab = tab;
    selected = null;
    invoiceUrl = '';
    invoiceError = '';
  }
</script>

<h1 class="title">Financial Monitoring Queue</h1>
<p class="subtitle">Review financial submissions. Flag discrepancies to trigger a Director vote.</p>

{#if loadError}
  <div class="banner error">{loadError}</div>
{/if}

<div class="tabs">
  <button class="tab" class:active={activeTab === 'budget'} onclick={() => switchTab('budget')}>
    Budget Reports <span class="count">{budgetDocs.length}</span>
  </button>
  <button class="tab" class:active={activeTab === 'expenditure'} onclick={() => switchTab('expenditure')}>
    Expenditure Reports <span class="count">{expenditureDocs.length}</span>
  </button>
  <button class="tab" class:active={activeTab === 'invoices'} onclick={() => switchTab('invoices')}>
    Invoices <span class="count">{invoiceDocs.length}</span>
  </button>
</div>

{#if activeTab === 'budget' || activeTab === 'expenditure'}
  {@const docs = activeTab === 'budget' ? budgetDocs : expenditureDocs}
  <div class="grid">
    <div class="list-panel">
      {#each docs as doc}
        <button class="card" class:selected={selected?.id === doc.id} onclick={() => selectDoc(doc)}>
          <div class="card-title">{doc.title}</div>
          <div class="card-meta">
            <span class="badge badge-{doc.status}">{doc.status}</span>
            <span>{doc.requester_name}</span>
            <span class="date">{fmtDate(doc.created_at)}</span>
          </div>
        </button>
      {:else}
        <p class="empty">No {activeTab === 'budget' ? 'budget reports' : 'expenditure reports'} found.</p>
      {/each}
    </div>

    <div class="detail-panel">
      {#if selected}
        <h2>{selected.title}</h2>
        <div class="kv-grid">
          <div class="kv"><span class="k">Requester</span><span>{selected.requester_name}</span></div>
          <div class="kv"><span class="k">Status</span><span class="badge badge-{selected.status}">{selected.status}</span></div>
          <div class="kv"><span class="k">Date</span><span>{fmtDate(selected.created_at)}</span></div>
        </div>

        <h3>Details</h3>
        <div class="detail-fields">
          {#each payloadFields(selected) as field}
            <div class="kv">
              <span class="k">{field.label}</span>
              <span>{field.value}</span>
            </div>
          {/each}
        </div>

        {#if selected.invoice_storage_path}
          <p class="invoice-note">📎 Invoice attached — view it in the Invoices tab.</p>
        {/if}

        {#if selected.status === 'pending'}
          <div class="flag-form">
            <h3>Flag Discrepancy</h3>
            <textarea class="textarea" placeholder="Describe the issue..." bind:value={flagReason} rows="3"></textarea>
            {#if flagError}<p class="msg error">{flagError}</p>{/if}
            {#if flagSuccess}<p class="msg success">{flagSuccess}</p>{/if}
            <button class="btn-warn" onclick={handleFlag}>Flag & Create Vote</button>
          </div>
        {/if}
      {:else}
        <div class="empty-state"><p>Select a document to review.</p></div>
      {/if}
    </div>
  </div>

{:else}
  <!-- Invoices tab -->
  <div class="grid">
    <div class="list-panel">
      {#each invoiceDocs as doc}
        <button class="card" class:selected={selected?.id === doc.id}
          onclick={() => { selectDoc(doc); if (doc.invoice_storage_path) loadInvoice(doc.invoice_storage_path); }}>
          <div class="card-title">{doc.title}</div>
          <div class="card-meta">
            <span class="badge badge-{doc.status}">{doc.status}</span>
            <span>{doc.requester_name}</span>
            <span class="date">{fmtDate(doc.created_at)}</span>
          </div>
        </button>
      {:else}
        <p class="empty">No invoices attached to any submission.</p>
      {/each}
    </div>

    <div class="detail-panel">
      {#if selected}
        <h2>{selected.title}</h2>
        <div class="kv-grid">
          <div class="kv"><span class="k">Requester</span><span>{selected.requester_name}</span></div>
          <div class="kv"><span class="k">Status</span><span class="badge badge-{selected.status}">{selected.status}</span></div>
          <div class="kv"><span class="k">Date</span><span>{fmtDate(selected.created_at)}</span></div>
          {#each payloadFields(selected) as f}
            <div class="kv"><span class="k">{f.label}</span><span>{f.value}</span></div>
          {/each}
        </div>

        <h3>Invoice</h3>
        {#if invoiceLoading}
          <p class="loading">Loading invoice…</p>
        {:else if invoiceError}
          <div class="banner error">{invoiceError}</div>
        {:else if invoiceUrl}
          {#if invoiceUrl.match(/\.(png|jpg|jpeg|gif|webp)$/i)}
            <img class="invoice-img" src={invoiceUrl} alt="Invoice" />
          {:else}
            <a class="invoice-link" href={invoiceUrl} target="_blank" rel="noreferrer">Open Invoice (PDF)</a>
          {/if}
        {/if}

        {#if selected.status === 'pending'}
          <div class="flag-form">
            <h3>Flag Discrepancy</h3>
            <textarea class="textarea" placeholder="Describe the issue..." bind:value={flagReason} rows="3"></textarea>
            {#if flagError}<p class="msg error">{flagError}</p>{/if}
            {#if flagSuccess}<p class="msg success">{flagSuccess}</p>{/if}
            <button class="btn-warn" onclick={handleFlag}>Flag & Create Vote</button>
          </div>
        {/if}
      {:else}
        <div class="empty-state"><p>Select a document to view its invoice.</p></div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .tabs { display:flex;gap:0.4rem;margin-bottom:1rem; }
  .tab { padding:0.35rem 0.75rem;border:1px solid #374151;background:#1F2937;color:#94A3B8;border-radius:4px;cursor:pointer;font-size:0.78rem;display:flex;align-items:center;gap:0.35rem; }
  .tab.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .count { background:rgba(58,190,255,0.15);color:#3ABEFF;border-radius:10px;padding:0 0.35rem;font-size:0.7rem; }
  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:300px;overflow-y:auto;flex-shrink:0; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.83rem;font-weight:500;margin-bottom:0.25rem; }
  .card-meta { display:flex;gap:0.5rem;flex-wrap:wrap;font-size:0.7rem;color:#94A3B8;align-items:center; }
  .date { margin-left:auto; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.75rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0.9rem 0 0.4rem; }
  .kv-grid { display:flex;flex-direction:column;gap:0.3rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;align-items:baseline; }
  .k { color:#64748B;min-width:90px;flex-shrink:0; }
  .detail-fields { display:flex;flex-direction:column;gap:0.3rem; }
  .invoice-note { font-size:0.75rem;color:#94A3B8;margin-top:0.5rem; }
  .invoice-img { max-width:100%;border-radius:6px;border:1px solid rgba(58,190,255,0.15);margin-top:0.5rem; }
  .invoice-link { display:inline-block;margin-top:0.5rem;padding:0.4rem 0.8rem;background:rgba(58,190,255,0.1);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;font-size:0.8rem;text-decoration:none; }
  .invoice-link:hover { background:rgba(58,190,255,0.2); }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .flag-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-top:1rem;display:flex;flex-direction:column;gap:0.5rem; }
  .textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif;resize:vertical;box-sizing:border-box; }
  .textarea:focus { outline:none;border-color:#3ABEFF; }
  .btn-warn { padding:0.5rem 1rem;background:rgba(245,158,11,0.15);border:1px solid #F59E0B;color:#F59E0B;border-radius:6px;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem; }
  .badge-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-approved { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-flagged,.badge-in_vote { background:rgba(239,68,68,0.15);color:#EF4444; }
  .msg { font-size:0.78rem;margin:0; }
  .msg.error { color:#EF4444; }
  .msg.success { color:#10B981; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
