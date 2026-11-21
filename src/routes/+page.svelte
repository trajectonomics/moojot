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
    X,
    Repeat,
    PenLine,
  } from "@lucide/svelte";
  import CategoryIcon from "$lib/components/CategoryIcon.svelte";
  import EmptyMonthArt from "$lib/components/EmptyMonthArt.svelte";
  import MooMascot, { type MascotMood } from "$lib/components/MooMascot.svelte";
  import SwipeRecordRow from "$lib/components/SwipeRecordRow.svelte";
  import RecordSheet from "$lib/components/RecordSheet.svelte";
  import SummaryView from "$lib/components/SummaryView.svelte";
  import SettingsView, { type SettingsSubPage } from "$lib/components/SettingsView.svelte";
  import RulesModal from "$lib/components/RulesModal.svelte";
  import ThumbSheet from "$lib/components/ThumbSheet.svelte";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import {
    deviceLanguage,
    markOnboardingDone,
    markOnboardingPending,
    onboardingState,
  } from "$lib/onboarding";
  import {
    api,
    hasAndroidBridge,
    triggerAndroidDownloadOrShare,
  } from "$lib/api";
  import { formatBaht } from "$lib/format";
  import { t } from "$lib/i18n";
  import { omitFromCompare, omitFromSummary, omitRecord } from "$lib/ledgerPatch";
  import { loadSavedHand, loadSavedTheme, saveHand, saveTheme, type AppTheme, type HandMode } from "$lib/theme";
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

  // Read before the theme effect saves its keys: see onboardingState().
  const firstRunState = onboardingState();
  let showOnboarding = $state(false);
  let onboardingLanguage = $state<"en" | "th">("en");
  let onboardingBack = $state<(() => void) | null>(null);
  let theme = $state<AppTheme>(loadSavedTheme());
  let hand = $state<HandMode>(loadSavedHand());
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
  let stickyVisible = $state(false);
  let heroEl = $state<HTMLDivElement | null>(null);
  let openSwipeId = $state<string | null>(null);
  let pdfPrompt = $state<{ pdfBase64: string; wrong: boolean } | null>(null);
  let pdfPassword = $state("");
  let pdfBusy = $state(false);
  let scanStartPrompt = $state(false);
  let monthPicker = $state(false);
  let scannedMonths = $state<string[]>([]);
  let scanSinceSec = 0;
  const monthSheet = { current: null as (() => void) | null };
  const filterSheet = { current: null as (() => void) | null };
  const scanSheet = { current: null as (() => void) | null };
  let undoDelete = $state<{
    recordId: string;
    home: HomePayload;
    summary: SummaryPayload | null;
    compare: ComparePayload | null;
    timer: ReturnType<typeof setTimeout>;
  } | null>(null);
  let deleteChain: Promise<void> = Promise.resolve();
  let scanChain: Promise<void> = Promise.resolve();

  // Interactive Mascot state
  let mascotMood = $state<MascotMood>("idle");
  let mascotSpeech = $state<string | null>(null);
  let mascotTimer: ReturnType<typeof setTimeout> | null = null;
  let tapCount = $state<number>(0);

  // Pull-to-refresh & hold state on Home
  let pullStartY = $state<number | null>(null);
  let pullDistance = $state<number>(0);
  let pullArmed = $state<boolean>(false);
  const PULL_THRESHOLD = 68;

  const lang = $derived(settings.language);
  const anyModalOpen = $derived(
    showRecordSheet ||
    showRulesModal ||
    showSearch ||
    showFilterSheet ||
    pdfPrompt !== null ||
    scanStartPrompt ||
    monthPicker
  );

  // Apply theme attribute & sync Android status bar icon contrast cleanly per screen!
  $effect(() => {
    saveTheme(theme);
    saveHand(hand);
    const hasColoredTopBar =
      activeTab === "summary" ||
      activeTab === "settings" ||
      showRecordSheet ||
      showRulesModal ||
      showSearch ||
      recordLightboxOpen;
    const isLightStatusBar =
      theme === "pink-white" &&
      (showOnboarding || (!hasColoredTopBar && activeTab !== "home"));
    window.MooJotAndroid?.setStatusBarLight?.(isLightStatusBar);
  });

  // Pin the compact header once the monthly hero has left the viewport.
  // At scrollY === 0 the hero is on screen, so pull-to-refresh keeps the top.
  $effect(() => {
    if (activeTab !== "home") {
      stickyVisible = false;
      return;
    }
    const el = heroEl;
    if (!el || typeof IntersectionObserver === "undefined") return;
    const observer = new IntersectionObserver(
      (entries) => {
        const entry = entries[0];
        stickyVisible = !!entry && !entry.isIntersecting && window.scrollY > 8;
      },
      { threshold: 0 }
    );
    observer.observe(el);
    const onScroll = () => {
      if (window.scrollY <= 2) stickyVisible = false;
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      observer.disconnect();
      window.removeEventListener("scroll", onScroll);
    };
  });

  $effect(() => {
    const el = homeMain;
    if (!el) return;
    const move = (event: TouchEvent) => onHomeTouchMove(event);
    el.addEventListener("touchmove", move, { passive: false });
    return () => el.removeEventListener("touchmove", move);
  });

  async function goToPeriod(offset: number) {
    monthPicker = false;
    if (offset === periodOffset) return;
    periodOffset = offset;
    stickyVisible = false;
    monthDrag = 0;
    if (typeof window !== "undefined") {
      window.scrollTo({ top: 0, behavior: "instant" });
    }
    const request = ++periodRequest;
    const [h, sum, cmp, bdgs] = await Promise.all([
      api.getHomeData(offset, filterAccountIds),
      api.getSummaryData(offset, summaryKind, filterAccountIds),
      api.getCompareData(offset, summaryKind, 6),
      api.getBudgets(offset),
    ]);
    if (request !== periodRequest) return;
    home = h;
    summary = sum;
    compare = cmp;
    budgets = bdgs;
  }

  function changePeriod(delta: number) {
    void goToPeriod(periodOffset + delta);
  }

  function openMonthPicker() {
    monthPicker = true;
  }

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

  let homeMain = $state<HTMLElement | null>(null);
  let monthDrag = $state(0);
  let monthDragging = $state(false);
  let gesture: { x: number; y: number; axis: "undecided" | "h" | "v" } | null = null;
  let periodRequest = 0;

  function onHomeTouchStart(e: TouchEvent) {
    if (activeTab !== "home" || anyModalOpen || isScanning) return;
    const target = e.target;
    if (target instanceof Element && target.closest("[data-swipe-row], button, a, input")) return;
    if (e.touches.length !== 1) return;
    gesture = {
      x: e.touches[0].clientX,
      y: e.touches[0].clientY,
      axis: "undecided",
    };
    pullDistance = 0;
    pullArmed = false;
    monthDrag = 0;
    monthDragging = false;
  }

  function onHomeTouchMove(e: TouchEvent) {
    if (!gesture || isScanning || anyModalOpen || e.touches.length !== 1) return;
    const dx = e.touches[0].clientX - gesture.x;
    const dy = e.touches[0].clientY - gesture.y;
    if (gesture.axis === "undecided") {
      if (Math.abs(dx) < 12 && Math.abs(dy) < 12) return;
      gesture.axis = Math.abs(dx) > Math.abs(dy) ? "h" : "v";
    }
    if (gesture.axis === "h") {
      monthDragging = true;
      monthDrag = Math.max(-96, Math.min(96, dx));
      if (e.cancelable) e.preventDefault();
      return;
    }
    if (window.scrollY > 4 && pullDistance === 0) return;
    if (dy <= 8) {
      pullDistance = 0;
      pullArmed = false;
      return;
    }
    pullDistance = Math.min(148, (dy - 8) * 0.5);
    pullArmed = pullDistance >= PULL_THRESHOLD;
    if (e.cancelable) e.preventDefault();
  }

  function onHomeTouchEnd() {
    if (gesture?.axis === "h") {
      const dx = monthDrag;
      gesture = null;
      monthDragging = false;
      monthDrag = 0;
      if (dx <= -48) void changePeriod(1);
      else if (dx >= 48) void changePeriod(-1);
      return;
    }
    const shouldScan = pullArmed && !isScanning;
    gesture = null;
    pullDistance = 0;
    pullArmed = false;
    if (shouldScan) void triggerSlipScan();
  }

  function onHomeTouchCancel() {
    gesture = null;
    monthDragging = false;
    monthDrag = 0;
    pullDistance = 0;
    pullArmed = false;
  }

  function openOnboarding(firstRun: boolean) {
    onboardingLanguage = firstRun ? deviceLanguage() : settings.language;
    if (firstRun) markOnboardingPending();
    showOnboarding = true;
  }

  function closeOnboarding() {
    markOnboardingDone();
    showOnboarding = false;
    activeTab = "home";
    settingsSubPage = null;
  }

  // First run: show the welcome tour unless this install already has data.
  $effect(() => {
    if (firstRunState === "done" || firstRunState === "returning") {
      if (firstRunState === "returning") markOnboardingDone();
      return;
    }
    void (async () => {
      try {
        const [s, h, months] = await Promise.all([
          api.getSettings(),
          api.getHomeData(0),
          api.listScannedMonths(),
        ]);
        settings = s;
        const hasData = h.dayGroups.length > 0 || h.futureRecords.length > 0 || months.length > 0;
        if (firstRunState === "new" && hasData) {
          markOnboardingDone();
          return;
        }
      } catch (e) {
        console.warn("onboarding: could not check existing data", e);
      }
      openOnboarding(true);
    })();
  });

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
    scannedMonths = await api.listScannedMonths();
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

  function monthStartSec(offset: number): number {
    const start = new Date();
    start.setDate(1);
    start.setMonth(start.getMonth() + offset);
    start.setHours(0, 0, 0, 0);
    return Math.floor(start.getTime() / 1000);
  }

  function monthLabel(offset: number): string {
    const start = new Date();
    start.setDate(1);
    start.setMonth(start.getMonth() + offset);
    return start.toLocaleDateString(lang === "th" ? "th-TH" : "en-US", {
      month: "short",
      year: "2-digit",
    });
  }

  function monthKey(offset: number): string {
    const start = new Date();
    start.setDate(1);
    start.setMonth(start.getMonth() + offset);
    return `${start.getFullYear()}-${String(start.getMonth() + 1).padStart(2, "0")}`;
  }

  function formatMonthKey(key: string): string {
    const [year, month] = key.split("-").map(Number);
    if (!year || !month) return key;
    return new Date(year, month - 1, 1).toLocaleDateString(lang === "th" ? "th-TH" : "en-US", {
      month: "short",
      year: "2-digit",
    });
  }

  function monthKeysFromSince(sinceSec: number): string[] {
    const cursor = new Date(sinceSec * 1000);
    cursor.setDate(1);
    cursor.setHours(0, 0, 0, 0);
    const end = new Date();
    end.setDate(1);
    end.setHours(0, 0, 0, 0);
    const keys: string[] = [];
    while (cursor <= end && keys.length < 36) {
      keys.push(`${cursor.getFullYear()}-${String(cursor.getMonth() + 1).padStart(2, "0")}`);
      cursor.setMonth(cursor.getMonth() + 1);
    }
    return keys;
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
    if (!scannedMonths.includes(monthKey(periodOffset))) {
      scanStartPrompt = true;
      return;
    }
    await startSlipScan(monthStartSec(periodOffset));
  }

  async function startSlipScan(sinceSec: number) {
    if (!window.MooJotAndroid) return;
    scanSinceSec = sinceSec;
    scanStartPrompt = false;
    isScanning = true;
    scanBanner = lang === "th" ? "กำลังสแกนสลิปใหม่..." : "Scanning new slips...";
    const [keywords, knownIds] = await Promise.all([
      api.getAlbumScanKeywords(),
      api.listKnownSlipMediaIds(),
    ]);
    window.MooJotAndroid.scanBankSlips(
      JSON.stringify(keywords),
      settings.allowSharedAlbum,
      sinceSec,
      JSON.stringify(knownIds)
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
      if (monthPicker) {
        monthSheet.current?.();
        return "handled";
      }
      if (scanStartPrompt) {
        scanSheet.current?.();
        return "handled";
      }
      if (pdfPrompt) {
        pdfPrompt = null;
        pdfPassword = "";
        return "handled";
      }
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
        filterSheet.current?.();
        return "handled";
      }
      if (showFabMenu) {
        showFabMenu = false;
        return "handled";
      }
      if (showOnboarding) {
        onboardingBack?.();
        return "handled";
      }
      if (activeTab === "settings" && settingsSubPage !== null) {
        settingsSubPage =
          settingsSubPage === "terms" || settingsSubPage === "privacy" ? "about" : null;
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
      // During onboarding, granting access shouldn't start a scan behind it.
      if (detail?.granted && !showOnboarding) triggerSlipScan();
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

    const onScanBatch = (ev: Event) => {
      const detail = (
        ev as CustomEvent<{
          candidates?: SlipScanCandidate[];
          seenIds?: string[];
          done?: boolean;
        }>
      ).detail;
      scanChain = scanChain
        .then(async () => {
          const seen = detail?.seenIds ?? [];
          if (seen.length > 0) await api.rememberScannedMedia(seen);
          const candidates = detail?.candidates ?? [];
          if (candidates.length > 0) {
            const report = await api.importSlipCandidates(candidates);
            scanBanner =
              lang === "th"
                ? `นำเข้า ${report.imported} · ซ้ำ ${report.duplicates}`
                : `Imported ${report.imported} · duplicates ${report.duplicates}`;
            await refreshPeriodData();
            if (report.imported > 0) {
              celebrateMascot(
                `Imported ${report.imported} new slip(s)!`,
                `นำเข้าสลิปใหม่ ${report.imported} รายการแล้ว!`
              );
            }
          }
          if (detail?.done) {
            const covered = monthKeysFromSince(scanSinceSec);
            if (covered.length > 0) {
              await api.rememberScannedMonths(covered);
              scannedMonths = [...new Set([...covered, ...scannedMonths])].sort().reverse();
            }
            isScanning = false;
            if (!candidates.length) {
              scanBanner = lang === "th" ? "ไม่มีสลิปใหม่" : "No new slips";
            }
            setTimeout(() => (scanBanner = null), 4000);
          }
        })
        .catch((error) => {
          isScanning = false;
          scanBanner = String(error);
        });
    };

    const onSharedPdf = (ev: Event) => {
      const detail = (ev as CustomEvent<{ pdfBase64?: string }>).detail;
      const pdfBase64 = detail?.pdfBase64;
      if (pdfBase64) void importPdfBase64(pdfBase64, null);
    };

    window.addEventListener("moojot:scan-candidates", onCandidates);
    window.addEventListener("moojot:scan-batch", onScanBatch);
    window.addEventListener("moojot:shared-slips", onSharedSlips);
    window.addEventListener("moojot:permission", onPerm);
    window.addEventListener("moojot:scan-error", onErr);
    window.addEventListener("moojot:backfill-done", onBackfill);
    window.addEventListener("moojot:shared-pdf", onSharedPdf);
    return () => {
      window.removeEventListener("moojot:scan-candidates", onCandidates);
      window.removeEventListener("moojot:scan-batch", onScanBatch);
      window.removeEventListener("moojot:shared-slips", onSharedSlips);
      window.removeEventListener("moojot:permission", onPerm);
      window.removeEventListener("moojot:scan-error", onErr);
      window.removeEventListener("moojot:backfill-done", onBackfill);
      window.removeEventListener("moojot:shared-pdf", onSharedPdf);
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

  function commitQueuedDelete(id: string) {
    deleteChain = deleteChain
      .then(async () => {
        await api.deleteRecord(id);
      })
      .catch(() => {});
  }

  async function flushDeletes() {
    await deleteChain;
    if (undoDelete === null) await refreshPeriodData();
  }

  function queueSwipeDelete(rec: RecordItem) {
    if (!home) return;
    if (undoDelete) {
      clearTimeout(undoDelete.timer);
      commitQueuedDelete(undoDelete.recordId);
      undoDelete = null;
    }
    const snapshot = {
      home: $state.snapshot(home),
      summary: summary ? $state.snapshot(summary) : null,
      compare: compare ? $state.snapshot(compare) : null,
    };
    home = omitRecord(home, rec.id);
    if (summary) summary = omitFromSummary(summary, rec, summaryKind);
    if (compare && rec.kind === summaryKind) {
      compare = omitFromCompare(compare, rec.amountSatang);
    }
    openSwipeId = null;
    const timer = setTimeout(() => {
      undoDelete = null;
      commitQueuedDelete(rec.id);
      void flushDeletes();
    }, 4500);
    undoDelete = {
      recordId: rec.id,
      home: snapshot.home,
      summary: snapshot.summary,
      compare: snapshot.compare,
      timer,
    };
  }

  function undoSwipeDelete() {
    if (!undoDelete) return;
    clearTimeout(undoDelete.timer);
    home = undoDelete.home;
    summary = undoDelete.summary;
    compare = undoDelete.compare;
    undoDelete = null;
  }

  async function importPdfBase64(pdfBase64: string, password: string | null) {
    pdfBusy = true;
    try {
      const report = await api.importStatementPdf(pdfBase64, password);
      if (report.needsPassword || report.wrongPassword) {
        pdfPrompt = { pdfBase64, wrong: report.wrongPassword };
        pdfPassword = "";
        return;
      }
      pdfPrompt = null;
      pdfPassword = "";
      await refreshPeriodData();
      activeTab = "home";
      scanBanner =
        report.imported === 0 && report.message
          ? report.message
          : lang === "th"
            ? `นำเข้า ${report.imported} รายการ · ซ้ำ ${report.duplicates}`
            : `Imported ${report.imported} · duplicates ${report.duplicates}`;
      setTimeout(() => (scanBanner = null), 5000);
    } catch (error) {
      scanBanner = error instanceof Error ? error.message : t(lang, "error");
      setTimeout(() => (scanBanner = null), 4000);
    } finally {
      pdfBusy = false;
    }
  }
</script>

<div class="min-h-screen max-w-full overflow-x-clip bg-[var(--bg-canvas)] text-[var(--text-primary)] flex flex-col">
  {#if activeTab === "home"}
    <header
      class="fixed inset-x-0 top-0 z-40 text-white transition-transform duration-200 ease-out {stickyVisible
        ? 'translate-y-0'
        : '-translate-y-full pointer-events-none'}"
      style="padding-top: env(safe-area-inset-top); background: linear-gradient(180deg, var(--hero-from), var(--hero-to));"
      aria-hidden={!stickyVisible}
    >
      <div class="flex items-center justify-between gap-3 px-4 py-3 {hand === 'left' ? 'flex-row-reverse' : ''}">
        <div class="min-w-0">
          <button
            type="button"
            class="truncate text-left text-sm font-extrabold"
            onclick={openMonthPicker}
          >
            {home?.period.label ?? "..."}
          </button>
          <div class="truncate text-xs font-semibold text-white/85">
            {t(lang, "totalSpending")} {formatBaht(home?.totalSpendingSatang ?? 0, true)}
          </div>
        </div>
        <button
          type="button"
          class="inline-flex shrink-0 items-center gap-1.5 rounded-full px-3.5 py-1.5 text-xs font-extrabold"
          style="background-color: var(--hero-btn-bg); color: var(--hero-btn-text);"
          onclick={() => (activeTab = "summary")}
        >
          <PieChart class="size-3.5" />
          <span>{t(lang, "summary")}</span>
        </button>
      </div>
    </header>

    <main
      bind:this={homeMain}
      class="flex-1 overflow-x-clip pb-28 touch-pan-y"
      ontouchstart={onHomeTouchStart}
      ontouchend={onHomeTouchEnd}
      ontouchcancel={onHomeTouchCancel}
    >
      <div
        bind:this={heroEl}
        role="group"
        class="relative overflow-hidden shadow-xl"
        style="background: linear-gradient(180deg, var(--hero-from), var(--hero-to)); border-radius: 0 0 28px 28px;"
      >
        <svg class="pointer-events-none absolute inset-0 h-full w-full" viewBox="0 0 360 228" preserveAspectRatio="xMidYMax slice" aria-hidden="true">
          <path d="M0 150 C90 118 150 176 230 146 C290 126 330 160 360 140 L360 228 L0 228 Z" fill="#ffffff" opacity="0.2" />
          <path d="M0 184 C100 158 170 206 250 178 C300 160 330 186 360 172 L360 228 L0 228 Z" fill="#ffffff" opacity="0.34" />
        </svg>

        <div class="relative z-10 flex min-h-[228px] flex-col px-4 pb-3" style="padding-top: max(env(safe-area-inset-top), 28px);">
          {#if pullDistance > 8 && !isScanning}
            <div class="flex items-end justify-center overflow-hidden" style="height: {Math.round(pullDistance)}px;">
              <p class="pb-2 text-center text-xs font-bold text-white">
                {pullArmed
                  ? lang === "th"
                    ? "ปล่อยเพื่อสแกน · เลื่อนขึ้นเพื่อยกเลิก"
                    : "Release to scan · slide up to cancel"
                  : lang === "th"
                    ? "ดึงค้างไว้เพื่อสแกนสลิป"
                    : "Pull and hold to scan slips"}
              </p>
            </div>
          {/if}
          <div class="flex items-start justify-between gap-3 {hand === 'left' ? 'flex-row-reverse' : ''}">
            <div class="min-w-0 {hand === 'left' ? 'text-right' : ''}">
              <div class="text-[11px] font-semibold text-white/85">
                {t(lang, "totalSpending")}
              </div>
              <div class="text-2xl font-extrabold leading-tight text-white">
                {formatBaht(home?.totalSpendingSatang ?? 0, true)}
              </div>
              {#if isScanning}
                <div class="mt-1 text-[11px] font-bold text-white">
                  {lang === "th" ? "กำลังสแกน..." : "Scanning..."}
                </div>
              {/if}
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="flex size-9 items-center justify-center rounded-full bg-white text-[#18181b] shadow-sm"
                aria-label={t(lang, "search")}
                onclick={() => {
                  showSearch = true;
                  searchQuery = "";
                  searchResults = [];
                }}
              >
                <Search class="size-4" />
              </button>
              <button
                type="button"
                class="relative flex size-9 items-center justify-center rounded-full bg-white text-[#18181b] shadow-sm"
                aria-label={t(lang, "filterRecords")}
                onclick={() => (showFilterSheet = true)}
              >
                <Wallet class="size-4" />
                {#if filterAccountIds.length > 0}
                  <span class="absolute right-0.5 top-0.5 size-2 rounded-full bg-[var(--brand)]"></span>
                {/if}
              </button>
            </div>
          </div>

          {#if scanBanner || mascotSpeech}
            <div
              class="pointer-events-none absolute top-[40%] z-20 {hand === 'right' ? 'left-[34%] right-4' : 'left-4 right-[34%]'}"
            >
              <div class="relative mx-auto w-fit max-w-[168px] rounded-2xl bg-white px-3 py-1.5 text-left text-[11px] font-bold leading-snug text-[#18181b] shadow-lg">
                {scanBanner || mascotSpeech}
                <span
                  class="absolute -bottom-1 size-2.5 rotate-45 bg-white {hand === 'right' ? 'left-3' : 'right-3'}"
                ></span>
              </div>
            </div>
          {/if}

          <div class="mt-auto flex items-end justify-between gap-2 {hand === 'right' ? 'flex-row-reverse' : ''}">
            <div class="flex min-w-0 items-center gap-1 pb-2">
              <button
                type="button"
                class="flex size-8 items-center justify-center rounded-full bg-white text-[#18181b] shadow-sm"
                aria-label="Previous period"
                onclick={() => changePeriod(-1)}
              >
                <ChevronLeft class="size-4" />
              </button>
              <button
                type="button"
                class="truncate rounded-full px-3 py-1 text-xs font-bold {periodOffset === 0
                  ? 'bg-[var(--brand)] text-white'
                  : 'bg-white text-[#18181b]'}"
                onclick={openMonthPicker}
              >
                {home?.period.label ?? "..."}
              </button>
              <button
                type="button"
                class="flex size-8 items-center justify-center rounded-full bg-white text-[#18181b] shadow-sm"
                aria-label="Next period"
                onclick={() => changePeriod(1)}
              >
                <ChevronRight class="size-4" />
              </button>
            </div>
            <div class="relative shrink-0">
              <MooMascot
                mood={isScanning || pullArmed ? "scan" : mascotMood}
                size="hero"
                onTap={handleMascotTap}
                alt="MooJot mascot"
              />
            </div>
          </div>
        </div>
      </div>

      <div class="overflow-x-clip">
      <div
        class="max-w-full"
        style="transform: translate3d({monthDrag}px, 0, 0); transition: {monthDragging ? 'none' : 'transform 180ms ease'};"
      >
      <!-- Collapsible Future / Recurring Records Drawer -->
      {#if home && home.futureRecords.length > 0}
        <div class="mx-4 mt-4 rounded-2xl bg-[var(--bg-card)] border border-[var(--border-subtle)] overflow-hidden shadow-xs">
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
      <div class="mt-4 space-y-3">
        {#if !home || home.dayGroups.length === 0}
          <div class="mx-4 rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-8 text-center space-y-3 flex flex-col items-center shadow-xs">
            <EmptyMonthArt alt={t(lang, "noRecords")} />
            <div class="text-sm font-bold text-[var(--text-primary)]">{t(lang, "noRecords")}</div>
            <p class="text-xs text-[var(--text-muted)]">{t(lang, "emptyHint")}</p>
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
                    <SwipeRecordRow
                      open={openSwipeId === rec.id}
                      deleteLabel={t(lang, "delete")}
                      onOpen={() => (openSwipeId = rec.id)}
                      onClose={() => {
                        if (openSwipeId === rec.id) openSwipeId = null;
                      }}
                      onDelete={() => queueSwipeDelete(rec)}
                      onActivate={() => {
                        editingRecord = rec;
                        recordLightboxOpen = false;
                        showRecordSheet = true;
                      }}
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
                    </SwipeRecordRow>
                  {/each}
                </div>
              </div>
            </div>
          {/each}
        {/if}
      </div>
      </div>
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
      {hand}
      bind:subPage={settingsSubPage}
      onReplayOnboarding={() => openOnboarding(false)}
      onChangeTheme={(nextTheme) => {
        theme = nextTheme;
      }}
      onChangeHand={(nextHand) => {
        hand = nextHand;
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
      onPickStatement={(pdfBase64) => importPdfBase64(pdfBase64, null)}
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
  {#if undoDelete}
    <div
      class="fixed inset-x-4 z-40 flex items-center justify-between gap-3 rounded-2xl bg-[var(--text-primary)] px-4 py-3 text-[var(--bg-canvas)] shadow-xl"
      style="bottom: calc(env(safe-area-inset-bottom) + 84px);"
      role="status"
    >
      <span class="text-sm font-semibold">{t(lang, "recordDeleted")}</span>
      <button
        type="button"
        class="text-sm font-extrabold text-[var(--brand)]"
        onclick={undoSwipeDelete}
      >
        {t(lang, "undo")}
      </button>
    </div>
  {/if}

  {#if monthPicker}
    <ThumbSheet
      title={lang === "th" ? "ไปที่เดือน" : "Go to month"}
      labelId="month-sheet-title"
      closeLabel={t(lang, "close")}
      close={monthSheet}
      onClose={() => (monthPicker = false)}
    >
      <button
        type="button"
        class="w-full rounded-xl bg-[var(--brand)] py-2.5 text-sm font-bold text-white"
        onclick={() => goToPeriod(0)}
      >
        {lang === "th" ? "เดือนนี้" : "This month"}
      </button>
      <div class="mt-2 flex flex-col gap-2">
        {#each [1, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11, -12] as offset (offset)}
          <button
            type="button"
            class="rounded-xl py-2.5 text-sm font-bold {offset === periodOffset
              ? 'border border-[var(--brand)] bg-[var(--brand-soft)] text-[var(--brand-text)]'
              : 'border border-[var(--border-subtle)] bg-[var(--bg-card)] text-[var(--text-primary)]'}"
            onclick={() => goToPeriod(offset)}
          >
            {monthLabel(offset)}
          </button>
        {/each}
      </div>
    </ThumbSheet>
  {/if}

  {#if scanStartPrompt}
    <ThumbSheet
      title={t(lang, "scanFrom")}
      labelId="scan-sheet-title"
      closeLabel={t(lang, "close")}
      close={scanSheet}
      onClose={() => (scanStartPrompt = false)}
    >
      <p class="text-xs text-[var(--text-muted)]">
        {lang === "th"
          ? "แตะเดือนแรกที่จะรวม สลิปที่สแกนแล้วจะถูกข้าม"
          : "Tap the first month to include. Slips already scanned are skipped."}
      </p>
      <div class="mt-4 flex flex-col gap-2">
        <button
          type="button"
          class="rounded-xl bg-[var(--brand)] py-2.5 text-sm font-bold text-white"
          onclick={() => startSlipScan(monthStartSec(periodOffset))}
        >
          {home?.period.label ?? monthLabel(periodOffset)}
        </button>
        <button
          type="button"
          class="rounded-xl border border-[var(--border-subtle)] py-2.5 text-sm font-bold text-[var(--text-primary)]"
          onclick={() => startSlipScan(monthStartSec(periodOffset - 3))}
        >
          {lang === "th"
            ? `3 เดือนจาก ${monthLabel(periodOffset)}`
            : `3 months from ${monthLabel(periodOffset)}`}
        </button>
        <button
          type="button"
          class="rounded-xl border border-[var(--border-subtle)] py-2.5 text-sm font-bold text-[var(--text-primary)]"
          onclick={() => startSlipScan(monthStartSec(periodOffset - 12))}
        >
          {lang === "th"
            ? `1 ปีจาก ${monthLabel(periodOffset)}`
            : `1 year from ${monthLabel(periodOffset)}`}
        </button>
      </div>
      <div class="mt-4">
        <div class="text-xs font-bold text-[var(--text-primary)]">
          {lang === "th" ? "สแกนแล้ว" : "Already scanned"}
        </div>
        {#if scannedMonths.length === 0}
          <p class="mt-1 text-xs text-[var(--text-muted)]">
            {lang === "th" ? "ยังไม่มี" : "None yet"}
          </p>
        {:else}
          <div class="mt-2 flex flex-wrap gap-1.5">
            {#each scannedMonths as key (key)}
              <span class="rounded-full bg-[var(--brand-soft)] px-2.5 py-1 text-[11px] font-bold text-[var(--brand-text)]">
                {formatMonthKey(key)}
              </span>
            {/each}
          </div>
        {/if}
      </div>
    </ThumbSheet>
  {/if}

  {#if pdfPrompt}
    <div class="fixed inset-0 z-50 flex items-end justify-center bg-black/50 p-4">
      <form
        class="w-full max-w-md rounded-3xl bg-[var(--bg-card)] p-5 shadow-xl"
        onsubmit={(event) => {
          event.preventDefault();
          if (!pdfPrompt || pdfBusy) return;
          void importPdfBase64(pdfPrompt.pdfBase64, pdfPassword);
        }}
      >
        <h2 class="text-base font-extrabold text-[var(--text-primary)]">{t(lang, "statementPassword")}</h2>
        <p class="mt-1 text-xs text-[var(--text-muted)]">{t(lang, "statementPasswordHint")}</p>
        {#if pdfPrompt.wrong}
          <p class="mt-2 text-xs font-semibold text-rose-500">{t(lang, "wrongPassword")}</p>
        {/if}
        <input
          type="password"
          autocomplete="off"
          bind:value={pdfPassword}
          class="mt-4 w-full rounded-xl border border-[var(--border-strong)] bg-[var(--bg-input)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--brand)]"
        />
        <div class="mt-4 flex gap-2">
          <button
            type="button"
            class="flex-1 rounded-xl border border-[var(--border-subtle)] py-2.5 text-sm font-bold text-[var(--text-secondary)]"
            onclick={() => {
              pdfPrompt = null;
              pdfPassword = "";
            }}
          >
            {t(lang, "cancel")}
          </button>
          <button
            type="submit"
            class="flex-1 rounded-xl bg-[var(--brand)] py-2.5 text-sm font-bold text-white disabled:opacity-60"
            disabled={pdfBusy || pdfPassword.trim() === ""}
          >
            {t(lang, "confirm")}
          </button>
        </div>
      </form>
    </div>
  {/if}

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

  {#if showFilterSheet}
    <ThumbSheet
      title={t(lang, "filterRecords")}
      labelId="filter-sheet-title"
      closeLabel={t(lang, "close")}
      close={filterSheet}
      onClose={() => (showFilterSheet = false)}
    >
      {#snippet actions()}
        <button
          type="button"
          onclick={async () => {
            filterAccountIds = [];
            await refreshPeriodData();
            filterSheet.current?.();
          }}
          class="text-sm font-bold text-[var(--brand)]"
        >
          {t(lang, "reset")}
        </button>
      {/snippet}
      <div class="space-y-1">
        {#each accounts as acc}
          {@const checked = filterAccountIds.includes(acc.id)}
          <label class="flex items-center justify-between py-2">
            <div class="flex items-center gap-3">
              <span class="size-4 rounded-full" style="background-color: {acc.color};"></span>
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
      {#snippet footer()}
        <button
          type="button"
          onclick={async () => {
            await refreshPeriodData();
            filterSheet.current?.();
          }}
          class="w-full rounded-full bg-[var(--brand)] py-3 text-sm font-bold text-white active:bg-[var(--brand-deep)]"
        >
          {t(lang, "apply")}
        </button>
      {/snippet}
    </ThumbSheet>
  {/if}
</div>

{#if showOnboarding}
  <Onboarding
    {settings}
    {theme}
    {hand}
    initialLanguage={onboardingLanguage}
    bind:backHandler={onboardingBack}
    onChangeTheme={(next) => (theme = next)}
    onChangeHand={(next) => (hand = next)}
    onSaveSettings={async (next) => {
      await api.updateSettings(next);
      settings = next;
      await refreshPeriodData();
    }}
    onFinish={closeOnboarding}
  />
{/if}
