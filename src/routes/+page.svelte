<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Wallet, Sparkles, CheckCircle2 } from "@lucide/svelte";

  let rustStatus = $state("Testing Rust IPC...");
  let connected = $state(false);

  $effect(() => {
    invoke<string>("greet", { name: "MooJot" })
      .then((res) => {
        rustStatus = res;
        connected = true;
      })
      .catch((err) => {
        rustStatus = `IPC error: ${String(err)}`;
      });
  });
</script>

<main class="min-h-screen flex flex-col justify-between bg-[#071b2e] text-slate-100 px-5 py-8">
  <section class="space-y-5">
    <div class="inline-flex items-center gap-2 rounded-full bg-amber-400/15 border border-amber-400/30 px-3.5 py-1 text-xs font-medium text-amber-300">
      <Sparkles class="size-3.5" />
      <span>Phase 0 · Tauri v2 + Svelte 5</span>
    </div>

    <div class="space-y-2">
      <div class="flex items-center gap-3">
        <div class="size-12 rounded-2xl bg-amber-400 text-[#071b2e] flex items-center justify-center shadow-lg">
          <Wallet class="size-6" />
        </div>
        <div>
          <h1 class="text-2xl font-bold tracking-tight">MooJot</h1>
          <p class="text-sm text-slate-400">จดรายรับรายจ่ายโอเพนซอร์ส · Local-First</p>
        </div>
      </div>
    </div>

    <div class="rounded-2xl bg-[#0e2742] border border-slate-700/60 p-4 space-y-2">
      <div class="flex items-center gap-2 text-sm font-medium">
        <CheckCircle2 class="size-4 {connected ? 'text-emerald-400' : 'text-amber-400'}" />
        <span>Rust Core Bridge</span>
      </div>
      <p class="text-sm text-slate-300">{rustStatus}</p>
    </div>
  </section>

  <footer class="text-center text-xs text-slate-500">
    MooJot v0.1.0 · Android-First Open Source
  </footer>
</main>
