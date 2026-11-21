<script lang="ts">
  import {
    Home,
    PieChart,
    User,
    Search,
    Wallet,
    ChevronLeft,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    Plus,
    ScanLine,
    ImagePlus,
    Flame,
    X,
    Repeat,
    PenLine,
    ArrowDown,
    Loader2,
  } from "@lucide/svelte";
  import CategoryIcon from "$lib/components/CategoryIcon.svelte";
  import MooMascot, { type MascotMood } from "$lib/components/MooMascot.svelte";
  import RecordSheet from "$lib/components/RecordSheet.svelte";
  import SummaryView from "$lib/components/SummaryView.svelte";
  import SettingsView, { type SettingsSubPage } from "$lib/components/SettingsView.svelte";
  import RulesModal from "$lib/components/RulesModal.svelte";
  import {
    api,
    hasAndroidBridge,
    triggerAndroidDownloadOrShare,
  } from "$lib/api";
  import { formatBaht } from "$lib/format";
  import { t } from "$lib/i18n";
  import { loadSavedTheme, saveTheme, type AppTheme } from "$lib/theme";
  import type {
    Account,
    AppSettings,
    BudgetItem,
    Category,
    ComparePayload,
    HomePayload,
    PayeeRule,
    RecordInput,
    RecordItem,
    RecordKind,
    SlipScanCandidate,
    SlipScanReport,
    SummaryPayload,
    Tag,
  } from "$lib/types";

  let theme = $state<AppTheme>(loadSavedTheme());
  let activeTab = $state<"home" | "summary" | "settings">("home");
  let settingsSubPage = $state<SettingsSubPage>(null);
  let periodOffset = $state<number>(0);
  let summaryKind = $state<RecordKind>("spending");
  let filterAccountIds = $state<string[]>([]);

  let settings = $state<AppSettings>({
    language: "en",
    monthStartDay: 1,
    weekStartDay: 0,
    defaultView: "monthly",
    allowSharedAlbum: false,
    ownerNames: [],
    aksonOcrApiKey: "",
    aksonOcrModel: "AksonOCR-preview",
    englishOcrEngine: "mlkit_latin",
  });
  let accounts = $state<Account[]>([]);
  let categories = $state<Category[]>([]);
  let tags = $state<Tag[]>([]);
  let home = $state<HomePayload | null>(null);
  let summary = $state<SummaryPayload | null>(null);
  let compare = $state<ComparePayload | null>(null);
  let budgets = $state<BudgetItem[]>([]);

  let futureExpanded = $state<boolean>(false);
  let showFabMenu = $state<boolean>(false);
  let showRecordSheet = $state<boolean>(false);
  let recordLightboxOpen = $state<boolean>(false);
  let editingRecord = $state<RecordItem | null>(null);
  let showFilterSheet = $state<boolean>(false);
  let showRulesModal = $state<boolean>(false);
  let payeeRules = $state<PayeeRule[]>([]);
  let showSearch = $state<boolean>(false);
  let searchQuery = $state<string>("");
  let searchResults = $state<RecordItem[]>([]);

  let isScanning = $state<boolean>(false);
  let scanBanner = $state<string | null>(null);

  // Interactive Mascot state
  let mascotMood = $state<MascotMood>("idle");
  let mascotSpeech = $state<string | null>(null);
  let mascotTimer: ReturnType<typeof setTimeout> | null = null;
  let tapCount = $state<number>(0);

  // Pull-to-refresh & hold state on Home
  let pullStartY = $state<number | null>(null);
  let pullDistance = $state<number>(0);
  let pullArmed = $state<boolean>(false);
  let holdTimer: ReturnType<typeof setTimeout> | null = null;
  const PULL_THRESHOLD = 68;

  const lang = $derived(settings.language);
  const anyModalOpen = $derived(
    showRecordSheet || showRulesModal || showSearch || showFilterSheet
  );

  // Apply theme attribute & sync Android status bar icon contrast cleanly per screen!
  $effect(() => {
    saveTheme(theme);
    const hasColoredTopBar =
      activeTab === "summary" ||
      activeTab === "settings" ||
      showRecordSheet ||
      showRulesModal ||
      showSearch ||
      recordLightboxOpen;
    const isLightStatusBar = theme === "pink-white" && !hasColoredTopBar;
    window.MooJotAndroid?.setStatusBarLight?.(isLightStatusBar);
  });

  function celebrateMascot(msgEn: string, msgTh: string, durationMs = 3200) {
    mascotMood = "happy";
    mascotSpeech = lang === "th" ? msgTh : msgEn;
    if (mascotTimer) clearTimeout(mascotTimer);
    mascotTimer = setTimeout(() => {
      mascotMood = "idle";
      mascotSpeech = null;
    }, durationMs);
  }

  function handleMascotTap() {
    tapCount += 1;
    const linesEn = [
      "Oink! Pull down at the top to scan new slips!",
      "100% offline & local — your money stays yours!",
      "Saving every satang like a pro piggy!",
      "Tap any slip row to inspect the attached photo!",
    ];
    const linesTh = [
      "อู๊ดๆ! ดึงหน้าจอลงค้างไว้เพื่อสแกนสลิปใหม่ได้เลย!",
      "ข้อมูลอยู่ในเครื่อง 100% ปลอดภัยหายห่วง!",
      "เก็บออมทุกบาททุกสตางค์ เก่งมากมนุษย์หมู!",
      "แตะที่รายการเพื่อดูรูปสลิปเต็มจอได้นะ!",
    ];
    const idx = tapCount % linesEn.length;
    celebrateMascot(linesEn[idx], linesTh[idx], 2800);
  }

  function clearHoldTimer() {
    if (holdTimer) {
      clearTimeout(holdTimer);
      holdTimer = null;
    }
  }

  function onHomeTouchStart(e: TouchEvent) {
    if (activeTab !== "home" || anyModalOpen || isScanning) return;
    if (window.scrollY <= 2 && e.touches.length === 1) {
      pullStartY = e.touches[0].clientY;
      pullDistance = 0;
      pullArmed = false;
    }
  }

  function onHomeTouchMove(e: TouchEvent) {
    if (pullStartY === null || isScanning || anyModalOpen) return;
    if (window.scrollY > 2) {
      pullStartY = null;
      pullDistance = 0;
      pullArmed = false;
      clearHoldTimer();
      return;
    }
    const delta = e.touches[0].clientY - pullStartY;
    if (delta <= 0) {
      pullDistance = 0;
      pullArmed = false;
      clearHoldTimer();
      return;
    }
    const damped = Math.min(104, delta * 0.52);
    pullDistance = damped;
    if (damped >= PULL_THRESHOLD && !pullArmed) {
      pullArmed = true;
      clearHoldTimer();
      // Hold at top past threshold for 550ms -> auto-trigger scan!
      holdTimer = setTimeout(() => {
        if (pullArmed && !isScanning) {
          pullStartY = null;
          pullDistance = 0;
          pullArmed = false;
          void triggerSlipScan();
        }
      }, 550);
    } else if (damped < PULL_THRESHOLD && pullArmed) {
      pullArmed = false;
      clearHoldTimer();
    }
  }

  function onHomeTouchEnd() {
    clearHoldTimer();
    if (pullArmed && !isScanning) {
      pullStartY = null;
      pullDistance = 0;
      pullArmed = false;
      void triggerSlipScan();
      return;
    }
    pullStartY = null;
    pullDistance = 0;
    pullArmed = false;
  }

  async function refreshAll() {
    const [s, accs, cats, tgs] = await Promise.all([
      api.getSettings(),
      api.getAccounts(),
      api.getCategories(),
      api.getTags(),
    ]);
    settings = s;
    accounts = accs;
    categories = cats;
    tags = tgs;
    await refreshPeriodData();
  }

  async function refreshPeriodData() {
    const [h, sum, cmp, bdgs] = await Promise.all([
      api.getHomeData(periodOffset, filterAccountIds),
      api.getSummaryData(periodOffset, summaryKind, filterAccountIds),
      api.getCompareData(periodOffset, summaryKind, 6),
      api.getBudgets(periodOffset),
    ]);
    home = h;
    summary = sum;
    compare = cmp;
    budgets = bdgs;
  }

  async function handleCandidates(candidates: SlipScanCandidate[]) {
    try {
      const report: SlipScanReport = await api.importSlipCandidates(candidates);
      scanBanner =
        lang === "th"
          ? `สแกน ${report.scanned} · นำเข้า ${report.imported} · ซ้ำ ${report.duplicates}`
          : `Scanned ${report.scanned} · Imported ${report.imported} · Dup ${report.duplicates}`;
      await refreshPeriodData();
      celebrateMascot(
        report.imported > 0 ? `Imported ${report.imported} new slip(s)!` : "All slips are up to date!",
        report.imported > 0 ? `นำเข้าสลิปใหม่ ${report.imported} รายการแล้ว!` : "สลิปทั้งหมดอัปเดตล่าสุดแล้ว!"
      );
    } catch (e) {
      scanBanner = `Scan error: ${String(e)}`;
    } finally {
      isScanning = false;
      setTimeout(() => (scanBanner = null), 4500);
    }
  }

  async function triggerSlipScan() {
    showFabMenu = false;
    if (!hasAndroidBridge() || !window.MooJotAndroid) {
      scanBanner =
        lang === "th"
          ? "การสแกนอัลบั้มสลิปใช้งานได้บนแอป Android"
          : "Gallery slip scanning runs on the Android app.";
      setTimeout(() => (scanBanner = null), 3500);
      return;
    }
    if (!window.MooJotAndroid.hasGalleryPermission()) {
      window.MooJotAndroid.requestGalleryPermission();
      return;
    }
    isScanning = true;
    const keywords = await api.getAlbumScanKeywords();
    window.MooJotAndroid.scanBankSlips(
      JSON.stringify(keywords),
      settings.allowSharedAlbum,
      40,
      false
    );
  }

  function triggerPickPhoto() {
    showFabMenu = false;
    if (hasAndroidBridge() && window.MooJotAndroid) {
      isScanning = true;
      window.MooJotAndroid.pickSlipPhoto(false);
    }
  }

  // Reset scroll position to top when switching bottom tabs
  $effect(() => {
    void activeTab;
    if (typeof window !== "undefined") {
      window.scrollTo({ top: 0, behavior: "instant" });
    }
  });

  // Lock background body scroll whenever any fullscreen modal/sheet is open
  $effect(() => {
    if (typeof document !== "undefined") {
      document.body.style.overflow = anyModalOpen ? "hidden" : "";
    }
    return () => {
      if (typeof document !== "undefined") {
        document.body.style.overflow = "";
      }
    };
  });

  // Wire Android hardware/gesture Back button handler
  $effect(() => {
    window.__moojotHandleBack = () => {
      if (recordLightboxOpen) {
        recordLightboxOpen = false;
        return "handled";
      }
      if (showRecordSheet) {
        showRecordSheet = false;
        editingRecord = null;
        return "handled";
      }
      if (showRulesModal) {
        showRulesModal = false;
        return "handled";
      }
      if (showSearch) {
        showSearch = false;
        return "handled";
      }
      if (showFilterSheet) {
        showFilterSheet = false;
        return "handled";
      }
      if (showFabMenu) {
        showFabMenu = false;
        return "handled";
      }
      if (activeTab === "settings" && settingsSubPage !== null) {
        settingsSubPage = null;
        return "handled";
      }
      if (activeTab !== "home") {
        activeTab = "home";
        return "handled";
      }
      return `root:${lang}`;
    };
    return () => {
      delete window.__moojotHandleBack;
    };
  });

  $effect(() => {
    refreshAll();

    const onCandidates = (ev: Event) => {
      const detail = (ev as CustomEvent<SlipScanCandidate[]>).detail;
      if (Array.isArray(detail)) handleCandidates(detail);
    };
    const onSharedSlips = (ev: Event) => {
      const detail = (ev as CustomEvent<SlipScanCandidate[]>).detail;
      if (Array.isArray(detail)) handleCandidates(detail);
    };
    const onPerm = (ev: Event) => {
      const detail = (ev as CustomEvent<{ granted: boolean }>).detail;
      if (detail?.granted) triggerSlipScan();
    };
    const onErr = (ev: Event) => {
      const detail = (ev as CustomEvent<{ error: string }>).detail;
      isScanning = false;
      scanBanner = detail?.error ?? "Scan error";
      setTimeout(() => (scanBanner = null), 3500);
    };
    const onBackfill = async (ev: Event) => {
      const detail = (
        ev as CustomEvent<
          { contentSha256: string; imagePath: string | null; ocrText: string | null }[]
        >
      ).detail;
      if (Array.isArray(detail) && detail.length > 0) {
        await api.applySlipBackfill(detail);
      }
      const res = await api.reparseAllSlips();
      scanBanner =
        lang === "th"
          ? `อัปเดต ${res.updated} · โอนเงิน ${res.transfersDetected} · แนบสลิปแล้ว`
          : `Updated ${res.updated} · Transfers ${res.transfersDetected} · Slips attached`;
      await refreshPeriodData();
      isScanning = false;
      activeTab = "home";
      celebrateMascot("Slips refreshed & attached!", "อัปเดตและแนบสลิปเรียบร้อย!");
      setTimeout(() => (scanBanner = null), 5000);
    };

    window.addEventListener("moojot:scan-candidates", onCandidates);
    window.addEventListener("moojot:shared-slips", onSharedSlips);
    window.addEventListener("moojot:permission", onPerm);
    window.addEventListener("moojot:scan-error", onErr);
    window.addEventListener("moojot:backfill-done", onBackfill);
    return () => {
      window.removeEventListener("moojot:scan-candidates", onCandidates);
      window.removeEventListener("moojot:shared-slips", onSharedSlips);
      window.removeEventListener("moojot:permission", onPerm);
      window.removeEventListener("moojot:scan-error", onErr);
      window.removeEventListener("moojot:backfill-done", onBackfill);
    };
  });

  function categoryMeta(id: string | null) {
    return categories.find((c) => c.id === id) ?? null;
  }

  function accountMeta(id: string | null) {
    return accounts.find((a) => a.id === id) ?? null;
  }

  async function runSearch(q: string) {
    searchQuery = q;
    if (!q.trim()) {
      searchResults = [];
      return;
    }
    searchResults = await api.searchRecords(q);
  }

  async function handleSaveRecord(input: RecordInput) {
    await api.saveRecord(input);
    showRecordSheet = false;
    recordLightboxOpen = false;
    editingRecord = null;
    await refreshPeriodData();
    celebrateMascot("Jotted! Nice job!", "จดเรียบร้อยแล้ว เยี่ยมมาก!");
  }

  async function handleDeleteRecord(id: string) {
    await api.deleteRecord(id);
    showRecordSheet = false;
    recordLightboxOpen = false;
    editingRecord = null;
    await refreshPeriodData();
  }
</script>

<div class="min-h-screen bg-[var(--bg-canvas)] text-[var(--text-primary)] flex flex-col">
  {#if activeTab === "home"}
    <main
      class="flex-1 pb-28"
      ontouchstart={onHomeTouchStart}
      ontouchmove={onHomeTouchMove}
      ontouchend={onHomeTouchEnd}
      ontouchcancel={onHomeTouchEnd}
    >
      <!-- Pull-to-refresh hint ONLY while dragging down (never duplicates scanning banner!) -->
      {#if pullDistance > 6 && !isScanning}
        <div
          class="overflow-hidden flex items-end justify-center bg-[var(--bg-canvas)] transition-[height] duration-150"
          style="height: {Math.round(pullDistance + 24)}px; padding-top: max(env(safe-area-inset-top), 36px);"
        >
          <div class="flex items-center gap-2 pb-2 px-4 rounded-full bg-[var(--bg-card)] border border-[var(--brand-border)] shadow-sm">
            {#if pullArmed}
              <ScanLine class="size-4 text-[var(--brand)] animate-pulse" />
              <span class="text-xs font-bold text-[var(--brand-text)]">
                {lang === "th" ? "ค้างไว้หรือปล่อยเพื่อสแกน!" : "Hold or release to scan!"}
              </span>
            {:else}
              <ArrowDown class="size-4 text-[var(--text-muted)]" />
              <span class="text-xs font-medium text-[var(--text-secondary)]">
                {lang === "th" ? "ดึงลงค้างไว้เพื่อสแกนสลิป" : "Pull down & hold to scan slips"}
              </span>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Top Bar seamlessly matching canvas status bar -->
      <div
        class="px-4 pb-2 flex items-center justify-between bg-[var(--bg-canvas)]"
        style="padding-top: {pullDistance > 6 && !isScanning ? '10px' : 'max(env(safe-area-inset-top), 38px)'};"
      >
        <div
          class="inline-flex items-center gap-2 rounded-full bg-[var(--bg-card)] border border-[var(--border-subtle)] px-3 py-1.5 text-xs font-bold text-[var(--brand-text)] shadow-xs"
        >
          <Flame class="size-4 text-[var(--brand)]" />
          <span>
            {Math.max(home?.syncStreakDays ?? 0, home?.categorizeStreakDays ?? 0)}
            {t(lang, "days")}
          </span>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => {
              showSearch = true;
              searchQuery = "";
              searchResults = [];
            }}
            class="size-9 rounded-full bg-[var(--bg-card)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--text-primary)] shadow-xs active:scale-95 transition-transform"
            aria-label={t(lang, "search")}
          >
            <Search class="size-4" />
          </button>
          <button
            type="button"
            onclick={() => (showFilterSheet = true)}
            class="relative size-9 rounded-full bg-[var(--bg-card)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--text-primary)] shadow-xs active:scale-95 transition-transform"
            aria-label={t(lang, "filterRecords")}
          >
            <Wallet class="size-4" />
            {#if filterAccountIds.length > 0}
              <span class="absolute -top-0.5 -right-0.5 size-2.5 rounded-full bg-[var(--brand)]"></span>
            {/if}
          </button>
        </div>
      </div>

      <!-- Compact Mascot Speech Bubble (single source of status, points toward big piggy!) -->
      <div class="px-4 mt-1 flex justify-end">
        <div class="relative w-full rounded-2xl bg-[var(--bg-card)] border border-[var(--border-subtle)] px-4 py-2.5 shadow-xs">
          {#if isScanning}
            <div class="text-xs font-bold text-[var(--brand-text)] flex items-center gap-2">
              <Loader2 class="size-3.5 animate-spin shrink-0 text-[var(--brand)]" />
              <span>{lang === "th" ? "กำลังสแกนสลิปในเครื่อง..." : "Scanning slips locally..."}</span>
            </div>
          {:else if scanBanner}
            <div class="text-xs font-semibold text-emerald-500">{scanBanner}</div>
          {:else if mascotSpeech}
            <div class="text-xs font-semibold text-[var(--brand-text)]">{mascotSpeech}</div>
          {:else}
            <div class="flex items-center justify-between gap-2">
              <div class="text-xs font-bold text-[var(--text-primary)] truncate">
                {(home?.todayJottedCount ?? 0) > 0
                  ? `${t(lang, "jottedToday")}: ${home?.todayJottedCount}`
                  : lang === "th"
                    ? "พร้อมจดสลิปอัตโนมัติแล้ว!"
                    : "Ready to jot your Thai e-slips!"}
              </div>
              {#if (home?.needsReviewCount ?? 0) > 0}
                <span class="text-[11px] text-[var(--text-muted)] shrink-0">
                  {lang === "th"
                    ? `รอตรวจ ${home?.needsReviewCount} รายการ`
                    : `${home?.needsReviewCount} to review`}
                </span>
              {/if}
            </div>
          {/if}
          <!-- Little speech tail pointing down-right toward the mascot -->
          <span
            class="absolute -bottom-1.5 right-16 size-3 rotate-45 bg-[var(--bg-card)] border-r border-b border-[var(--border-subtle)]"
          ></span>
        </div>
      </div>

      <!-- Signature Monthly Hero Card with BIG animated MooJot piggy overflowing the bottom right! -->
      <div
        class="mx-4 mt-3 rounded-3xl text-white px-4 pt-4 pb-4 shadow-xl overflow-visible relative"
        style="background: linear-gradient(135deg, var(--hero-from), var(--hero-to)); box-shadow: 0 14px 32px -12px var(--hero-shadow);"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1 font-bold text-white">
            <button
              type="button"
              onclick={async () => {
                periodOffset -= 1;
                await refreshPeriodData();
              }}
              class="size-7 rounded-full flex items-center justify-center active:bg-black/15"
              aria-label="Prev month"
            >
              <ChevronLeft class="size-5" />
            </button>
            <span class="text-sm font-extrabold">{home?.period.label ?? "..."}</span>
            <button
              type="button"
              onclick={async () => {
                periodOffset += 1;
                await refreshPeriodData();
              }}
              class="size-7 rounded-full flex items-center justify-center active:bg-black/15"
              aria-label="Next month"
            >
              <ChevronRight class="size-5" />
            </button>
          </div>

          <button
            type="button"
            onclick={() => (activeTab = "summary")}
            class="inline-flex items-center gap-1.5 rounded-full px-4 py-1.5 text-xs font-extrabold shadow-md active:scale-95 transition-transform"
            style="background-color: var(--hero-btn-bg); color: var(--hero-btn-text);"
          >
            <PieChart class="size-3.5" />
            <span>{t(lang, "summary")}</span>
          </button>
        </div>

        <!-- Total Spending (left) + BIG Interactive Animated Piggy overflowing past the bottom! -->
        <div class="mt-2 flex items-end justify-between gap-2">
          <div class="pb-1 min-w-0">
            <div class="text-xs font-semibold text-white/85">
              {t(lang, "totalSpending")}
            </div>
            <div class="text-3xl font-extrabold tracking-tight text-white leading-tight mt-0.5">
              {formatBaht(home?.totalSpendingSatang ?? 0, true)}
            </div>
          </div>

          <div class="relative z-10 -mb-10 -mr-2 shrink-0">
            <MooMascot
              mood={isScanning || pullArmed ? "scan" : mascotMood}
              size="hero"
              onTap={handleMascotTap}
              alt="MooJot mascot"
            />
          </div>
        </div>
      </div>

      <!-- Collapsible Future / Recurring Records Drawer -->
      {#if home && home.futureRecords.length > 0}
        <div class="mx-4 mt-8 rounded-2xl bg-[var(--bg-card)] border border-[var(--border-subtle)] overflow-hidden shadow-xs">
          <button
            type="button"
            onclick={() => (futureExpanded = !futureExpanded)}
            class="w-full px-4 py-3 flex items-center justify-between text-left"
          >
            <div class="flex items-center gap-2.5">
              <span class="size-6 rounded-full bg-[var(--brand-soft)] text-[var(--brand-text)] text-xs font-bold flex items-center justify-center">
                {home.futureRecords.length}
              </span>
              <div>
                <div class="text-xs font-bold text-[var(--text-primary)]">{t(lang, "futureRecords")}</div>
                <div class="text-[11px] text-[var(--text-muted)]">
                  {formatBaht(home.futureTotalSatang, true)}
                </div>
              </div>
            </div>
            {#if futureExpanded}
              <ChevronUp class="size-4 text-[var(--text-muted)]" />
            {:else}
              <ChevronDown class="size-4 text-[var(--text-muted)]" />
            {/if}
          </button>

          {#if futureExpanded}
            <div class="border-t border-[var(--border-subtle)] divide-y divide-[var(--border-subtle)]">
              {#each home.futureRecords as f}
                {@const cat = categoryMeta(f.categoryId)}
                <div class="px-4 py-3 flex items-center justify-between bg-[var(--bg-card-sub)]">
                  <div class="flex items-center gap-3">
                    <CategoryIcon
                      icon={cat?.icon ?? "leaf"}
                      color={cat?.color ?? "#71717a"}
                      size="sm"
                    />
                    <div>
                      <div class="text-xs font-bold text-[var(--text-primary)] flex items-center gap-1.5">
                        <span>
                          {cat ? (lang === "th" ? cat.nameTh : cat.nameEn) : t(lang, "spending")}
                        </span>
                        <Repeat class="size-3 text-[var(--brand)]" />
                      </div>
                      <div class="text-[11px] text-[var(--text-muted)]">
                        {f.dueDate}{f.note ? ` · ${f.note}` : ""}
                      </div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold text-[var(--text-primary)]">
                      {formatBaht(f.amountSatang)}
                    </span>
                    <button
                      type="button"
                      onclick={async () => {
                        await api.deleteRecurringRule(f.ruleId);
                        await refreshPeriodData();
                      }}
                      class="text-[var(--text-muted)] hover:text-rose-500 text-xs"
                      aria-label="Remove recurring"
                    >
                      <X class="size-4" />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Daily Day-Gutter Timeline -->
      <div class="{home && home.futureRecords.length > 0 ? 'mt-4' : 'mt-8'} space-y-3">
        {#if !home || home.dayGroups.length === 0}
          <div class="mx-4 rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-8 text-center space-y-3 flex flex-col items-center shadow-xs">
            <MooMascot mood={isScanning ? "scan" : "idle"} size="lg" onTap={handleMascotTap} />
            <div class="text-sm font-bold text-[var(--text-primary)]">{t(lang, "noRecords")}</div>
            <p class="text-xs text-[var(--text-muted)]">
              {lang === "th"
                ? "กด + เพื่อบันทึกรายการ หรือดึงหน้าจอลงค้างไว้เพื่อสแกนสลิป"
                : "Tap + Add Record or pull down & hold at the top to scan bank slips."}
            </p>
          </div>
        {:else}
          {#each home.dayGroups as group}
            <div class="flex items-stretch">
              <!-- Left Day Gutter -->
              <div class="w-16 shrink-0 flex flex-col items-center pt-2 relative">
                <span
                  class="absolute left-0 top-2 bottom-2 w-1 rounded-r-full {group.dayOfWeek === 'Today'
                    ? 'bg-[var(--brand)]'
                    : 'bg-[var(--border-strong)]'}"
                ></span>
                <span
                  class="text-[11px] font-semibold {group.dayOfWeek === 'Today'
                    ? 'text-[var(--brand-text)]'
                    : 'text-[var(--text-muted)]'}"
                >
                  {group.dayOfWeek}
                </span>
                <span class="text-lg font-extrabold text-[var(--text-primary)] leading-tight">
                  {group.dayNumber}
                </span>
              </div>

              <!-- Day Card Stack -->
              <div class="flex-1 mr-4 rounded-2xl bg-[var(--bg-card)] border border-[var(--border-subtle)] overflow-hidden shadow-xs">
                <div class="px-4 py-2.5 bg-[var(--bg-card-sub)] flex items-baseline justify-between">
                  <span class="text-[11px] text-[var(--text-muted)] font-medium">
                    {#if group.transferCount > 0}
                      {group.transferCount} {t(lang, "transfers")}
                    {/if}
                  </span>
                  <div class="text-right">
                    <span class="text-[11px] text-[var(--text-muted)] mr-1.5">↑ {t(lang, "spending")}</span>
                    <span class="text-base font-extrabold text-[var(--text-primary)]">
                      {formatBaht(group.spendingSatang)}
                    </span>
                  </div>
                </div>

                <div class="divide-y divide-[var(--border-subtle)]">
                  {#each group.records as rec}
                    {@const cat = categoryMeta(rec.categoryId)}
                    {@const acc = accountMeta(rec.accountId)}
                    <button
                      type="button"
                      onclick={() => {
                        editingRecord = rec;
                        recordLightboxOpen = false;
                        showRecordSheet = true;
                      }}
                      class="w-full relative px-4 py-3 flex items-center justify-between gap-3 text-left active:opacity-80"
                    >
                      <!-- Brand corner triangle on unreviewed auto-jotted slip -->
                      {#if !rec.isReviewed}
                        <span
                          class="absolute top-0 left-0 size-0 border-t-[12px] border-t-[var(--brand)] border-r-[12px] border-r-transparent"
                        ></span>
                      {/if}

                      <div class="flex items-center gap-3 min-w-0">
                        <CategoryIcon
                          icon={rec.kind === "transfer" ? "transfer" : cat?.icon ?? "leaf"}
                          color={rec.kind === "transfer" ? (theme === "pink-white" ? "#FF4D94" : "#FF6B1A") : cat?.color ?? "#71717a"}
                          size="sm"
                        />
                        <div class="min-w-0">
                          <div class="text-sm font-semibold text-[var(--text-primary)] truncate">
                            {#if rec.kind === "transfer"}
                              {t(lang, "transfer")}
                            {:else if cat}
                              {lang === "th" ? cat.nameTh : cat.nameEn}
                            {:else}
                              {t(lang, rec.kind === "income" ? "income" : "spending")}
                            {/if}
                          </div>
                          <div
                            class="text-xs truncate {rec.parseStatus !== 'ok'
                              ? 'text-[var(--brand-text)] font-medium'
                              : 'text-[var(--text-muted)]'}"
                          >
                            {rec.payeeRaw || rec.note || (acc ? (lang === "th" ? acc.nameTh : acc.nameEn) : "")}
                          </div>
                        </div>
                      </div>

                      <div class="flex items-center gap-2 shrink-0">
                        {#if rec.parseStatus !== "ok"}
                          <PenLine class="size-4 text-[var(--brand)]" />
                        {/if}
                        <span
                          class="text-sm font-bold {rec.kind === 'income'
                            ? 'text-emerald-500'
                            : rec.kind === 'transfer'
                              ? 'text-[var(--brand-text)]'
                              : 'text-[var(--text-primary)]'}"
                        >
                          {rec.kind === "income" ? "+" : ""}{formatBaht(rec.amountSatang)}
                        </span>
                      </div>
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </main>

    <!-- Speed-Dial Backdrop -->
    {#if showFabMenu}
      <button
        type="button"
        onclick={() => (showFabMenu = false)}
        class="fixed inset-0 z-30 bg-black/60 backdrop-blur-xs"
        aria-label="Close menu"
      ></button>
      <div class="fixed right-4 bottom-36 z-40 flex flex-col items-end gap-3">
        <button
          type="button"
          onclick={triggerSlipScan}
          class="flex items-center gap-3 text-sm font-bold text-white"
        >
          <span>{t(lang, "scanESlips")}</span>
          <span class="size-11 rounded-full bg-white text-[var(--brand)] flex items-center justify-center shadow-lg">
            <ScanLine class="size-5" />
          </span>
        </button>
        <button
          type="button"
          onclick={triggerPickPhoto}
          class="flex items-center gap-3 text-sm font-bold text-white"
        >
          <span>{lang === "th" ? "นำเข้าจากรูปสลิป" : "Import Slip Photo"}</span>
          <span class="size-11 rounded-full bg-white text-[var(--brand)] flex items-center justify-center shadow-lg">
            <ImagePlus class="size-5" />
          </span>
        </button>
      </div>
    {/if}

    <!-- Split Floating Action Button (+ Add record | ^) -->
    <div
      class="fixed right-4 z-40 inline-flex items-center rounded-full bg-[var(--brand)] text-white shadow-xl overflow-hidden"
      style="bottom: calc(max(env(safe-area-inset-bottom), 20px) + 64px); box-shadow: 0 10px 24px -6px var(--hero-shadow);"
    >
      <button
        type="button"
        onclick={() => {
          editingRecord = null;
          recordLightboxOpen = false;
          showFabMenu = false;
          showRecordSheet = true;
        }}
        class="flex items-center gap-1.5 pl-4 pr-3 py-3 text-sm font-bold active:bg-[var(--brand-deep)]"
      >
        <Plus class="size-4" />
        <span>{t(lang, "addRecord")}</span>
      </button>
      <span class="w-px h-5 bg-white/30"></span>
      <button
        type="button"
        onclick={() => (showFabMenu = !showFabMenu)}
        class="px-3 py-3 active:bg-[var(--brand-deep)]"
        aria-label="More actions"
      >
        <ChevronUp class="size-4 transition-transform {showFabMenu ? 'rotate-180' : ''}" />
      </button>
    </div>
  {:else if activeTab === "summary"}
    <SummaryView
      {lang}
      {summary}
      {compare}
      {budgets}
      {categories}
      activeKind={summaryKind}
      onChangeKind={async (k) => {
        summaryKind = k;
        await refreshPeriodData();
      }}
      onPrevPeriod={async () => {
        periodOffset -= 1;
        await refreshPeriodData();
      }}
      onNextPeriod={async () => {
        periodOffset += 1;
        await refreshPeriodData();
      }}
      onAddBudget={async (input) => {
        await api.upsertBudget({
          scopeType: input.scopeType,
          scopeId: input.scopeId,
          periodType: "monthly",
          limitSatang: input.limitSatang,
          alertThresholdPct: 80,
        });
        await refreshPeriodData();
      }}
      onDeleteBudget={async (id) => {
        await api.deleteBudget(id);
        await refreshPeriodData();
      }}
    />
  {:else}
    <SettingsView
      {settings}
      {accounts}
      {categories}
      {tags}
      {isScanning}
      {mascotMood}
      {theme}
      bind:subPage={settingsSubPage}
      onChangeTheme={(nextTheme) => {
        theme = nextTheme;
      }}
      onSaveSettings={async (next) => {
        await api.updateSettings(next);
        settings = next;
        await refreshPeriodData();
      }}
      onToggleAccount={async (id, enabled) => {
        await api.toggleAccountEnabled(id, enabled);
        accounts = await api.getAccounts();
      }}
      onScanNow={triggerSlipScan}
      onPickPhoto={triggerPickPhoto}
      onReparseSlips={async () => {
        if (hasAndroidBridge() && window.MooJotAndroid?.hasGalleryPermission()) {
          const targets = await api.getBackfillTargets();
          if (targets.length > 0) {
            isScanning = true;
            window.MooJotAndroid.backfillExistingSlips(JSON.stringify(targets));
            return;
          }
        }
        const res = await api.reparseAllSlips();
        scanBanner =
          lang === "th"
            ? `อัปเดต ${res.updated} รายการ · โอนเงิน ${res.transfersDetected} · ลบที่ไม่ใช่สลิป ${res.removedNonSlips}`
            : `Updated ${res.updated} · Transfers ${res.transfersDetected} · Removed ${res.removedNonSlips} non-slips`;
        await refreshPeriodData();
        activeTab = "home";
        celebrateMascot("Slips refreshed!", "ประมวลผลสลิปเรียบร้อย!");
        setTimeout(() => (scanBanner = null), 5000);
      }}
      onOpenRules={async () => {
        payeeRules = await api.listPayeeRules();
        showRulesModal = true;
      }}
      onCreateCategory={async (input) => {
        await api.createCategory(input);
        categories = await api.getCategories();
        celebrateMascot("New category added!", "เพิ่มหมวดหมู่ใหม่แล้ว!");
      }}
      onDeleteCategory={async (id) => {
        await api.deleteCategory(id);
        categories = await api.getCategories();
        await refreshPeriodData();
      }}
      onCreateTag={async (name) => {
        await api.createTag(name);
        tags = await api.getTags();
      }}
      onToggleTag={async (id, disabled) => {
        await api.toggleTagDisabled(id, disabled);
        tags = await api.getTags();
      }}
      onExportCsv={async () => {
        const csv = await api.exportRecordsCsv();
        triggerAndroidDownloadOrShare("moojot-records.csv", "text/csv", csv);
      }}
      onExportJson={async () => {
        const json = await api.exportBackupJson();
        triggerAndroidDownloadOrShare("moojot-backup.json", "application/json", json);
      }}
    />
  {/if}

  <!-- Bottom Navigation Bar -->
  <nav
    class="fixed bottom-0 left-0 right-0 z-30 backdrop-blur-md border-t border-[var(--border-subtle)] grid grid-cols-3 pt-2"
    style="background-color: var(--nav-bg); padding-bottom: max(env(safe-area-inset-bottom), 20px);"
  >
    <button
      type="button"
      onclick={() => (activeTab = "home")}
      class="flex flex-col items-center gap-0.5 text-[11px] font-bold {activeTab === 'home'
        ? 'text-[var(--brand)]'
        : 'text-[var(--text-muted)]'}"
    >
      <Home class="size-5" />
      <span>{t(lang, "home")}</span>
    </button>
    <button
      type="button"
      onclick={() => (activeTab = "summary")}
      class="flex flex-col items-center gap-0.5 text-[11px] font-bold {activeTab === 'summary'
        ? 'text-[var(--brand)]'
        : 'text-[var(--text-muted)]'}"
    >
      <PieChart class="size-5" />
      <span>{t(lang, "summary")}</span>
    </button>
    <button
      type="button"
      onclick={() => {
        settingsSubPage = null;
        activeTab = "settings";
      }}
      class="flex flex-col items-center gap-0.5 text-[11px] font-bold {activeTab === 'settings'
        ? 'text-[var(--brand)]'
        : 'text-[var(--text-muted)]'}"
    >
      <User class="size-5" />
      <span>{t(lang, "myMoo")}</span>
    </button>
  </nav>

  <!-- Add / Edit Record Modal -->
  {#if showRecordSheet}
    <RecordSheet
      {lang}
      {accounts}
      {categories}
      {tags}
      editing={editingRecord}
      bind:showLightbox={recordLightboxOpen}
      onSave={handleSaveRecord}
      onDelete={handleDeleteRecord}
      onClose={() => {
        showRecordSheet = false;
        recordLightboxOpen = false;
        editingRecord = null;
      }}
    />
  {/if}

  <!-- Auto-Categorize Rules Modal -->
  {#if showRulesModal}
    <RulesModal
      {lang}
      {categories}
      rules={payeeRules}
      onAddRule={async (pattern, categoryId) => {
        await api.upsertPayeeRule(pattern, categoryId);
        payeeRules = await api.listPayeeRules();
      }}
      onDeleteRule={async (id) => {
        await api.deletePayeeRule(id);
        payeeRules = await api.listPayeeRules();
      }}
      onApplyAll={async () => {
        const count = await api.autoCategorizeUncategorized();
        await refreshPeriodData();
        celebrateMascot(`Auto-categorized ${count} record(s)!`, `จัดหมวดอัตโนมัติไป ${count} รายการ!`);
        return count;
      }}
      onClose={() => (showRulesModal = false)}
    />
  {/if}

  <!-- Search Full-Screen Overlay -->
  {#if showSearch}
    <div class="fixed inset-0 z-50 bg-[var(--bg-canvas)] flex flex-col overscroll-contain">
      <div
        class="bg-[var(--brand)] px-3 pb-3 flex items-center gap-2 shrink-0"
        style="padding-top: max(env(safe-area-inset-top), 36px);"
      >
        <button
          type="button"
          onclick={() => (showSearch = false)}
          class="size-9 rounded-full flex items-center justify-center text-white active:bg-black/15"
          aria-label={t(lang, "back")}
        >
          <ChevronLeft class="size-6" />
        </button>
        <input
          type="text"
          value={searchQuery}
          oninput={(e) => runSearch(e.currentTarget.value)}
          placeholder={t(lang, "searchPlaceholder")}
          class="flex-1 rounded-full bg-white text-[#18181b] placeholder:text-zinc-500 px-4 py-2 text-sm outline-none"
        />
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-4 space-y-2">
        {#if searchQuery.trim() && searchResults.length === 0}
          <p class="text-center text-sm text-[var(--text-muted)] py-10">{t(lang, "noResults")}</p>
        {:else}
          {#each searchResults as rec}
            {@const cat = categoryMeta(rec.categoryId)}
            <button
              type="button"
              onclick={() => {
                showSearch = false;
                editingRecord = rec;
                recordLightboxOpen = false;
                showRecordSheet = true;
              }}
              class="w-full flex items-center justify-between rounded-2xl bg-[var(--bg-card)] border border-[var(--border-subtle)] px-4 py-3 text-left"
            >
              <div class="flex items-center gap-3 min-w-0">
                <CategoryIcon icon={cat?.icon ?? "leaf"} color={cat?.color ?? "#71717a"} size="sm" />
                <div class="min-w-0">
                  <div class="text-sm font-semibold text-[var(--text-primary)] truncate">
                    {rec.payeeRaw || rec.note || (cat ? (lang === "th" ? cat.nameTh : cat.nameEn) : rec.kind)}
                  </div>
                  <div class="text-xs text-[var(--text-muted)]">{rec.localDate}</div>
                </div>
              </div>
              <div class="text-sm font-bold text-[var(--text-primary)]">{formatBaht(rec.amountSatang, true)}</div>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}

  <!-- Filter Records Bottom Sheet (All 20 Banks + Cash) -->
  {#if showFilterSheet}
    <div class="fixed inset-0 z-50 bg-black/60 flex flex-col justify-end overscroll-contain">
      <div class="bg-[var(--bg-card)] text-[var(--text-primary)] border-t border-[var(--border-subtle)] rounded-t-3xl max-h-[80vh] flex flex-col">
        <div class="px-5 py-4 border-b border-[var(--border-subtle)] flex items-center justify-between shrink-0">
          <h3 class="font-bold text-base">{t(lang, "filterRecords")}</h3>
          <button
            type="button"
            onclick={async () => {
              filterAccountIds = [];
              await refreshPeriodData();
              showFilterSheet = false;
            }}
            class="text-sm font-bold text-[var(--brand)]"
          >
            {t(lang, "reset")}
          </button>
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-5 py-3 space-y-2">
          {#each accounts as acc}
            {@const checked = filterAccountIds.includes(acc.id)}
            <label class="flex items-center justify-between py-2">
              <div class="flex items-center gap-3">
                <span
                  class="size-4 rounded-full"
                  style="background-color: {acc.color};"
                ></span>
                <span class="text-sm font-medium text-[var(--text-primary)]">
                  {lang === "th" ? acc.nameTh : acc.nameEn}
                </span>
              </div>
              <input
                type="checkbox"
                {checked}
                onchange={() => {
                  if (checked) {
                    filterAccountIds = filterAccountIds.filter((x) => x !== acc.id);
                  } else {
                    filterAccountIds = [...filterAccountIds, acc.id];
                  }
                }}
                class="size-5 accent-[var(--brand)]"
              />
            </label>
          {/each}
        </div>
        <div
          class="p-4 border-t border-[var(--border-subtle)] shrink-0"
          style="padding-bottom: max(env(safe-area-inset-bottom), 20px);"
        >
          <button
            type="button"
            onclick={async () => {
              await refreshPeriodData();
              showFilterSheet = false;
            }}
            class="w-full py-3 rounded-full bg-[var(--brand)] text-white font-bold text-sm active:bg-[var(--brand-deep)]"
          >
            {t(lang, "apply")}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
