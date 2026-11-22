<script lang="ts">
  import { untrack } from "svelte";
  import {
    ScanLine,
    ImagePlus,
    Calendar,
    Download,
    Plus,
    Check,
    Globe,
    Sparkles,
    WandSparkles,
    ShieldCheck,
    Palette,
    ChevronRight,
    ArrowLeft,
    Trash2,
    Building2,
    Database,
    FileText,
    Info,
    Scale,
    GitFork,
    ChartNoAxesColumn,
  } from "@lucide/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import CategoryIcon from "./CategoryIcon.svelte";
  import LegalText from "./LegalText.svelte";
  import MooMascot, { type MascotMood } from "./MooMascot.svelte";
  import termsMarkdown from "../../../docs/terms-of-service.md?raw";
  import privacyMarkdown from "../../../docs/privacy-policy.md?raw";
  import { t } from "$lib/i18n";
  import type { AppTheme, HandMode } from "$lib/theme";
  import type { Account, AppSettings, Category, Tag } from "$lib/types";

  export type SettingsSubPage =
    | null
    | "appearance"
    | "slips"
    | "categories"
    | "calendar"
    | "data"
    | "about"
    | "terms"
    | "privacy";

  const GITHUB_URL = "https://github.com/trajectonomics/moojot";

  let {
    settings,
    accounts,
    categories,
    tags,
    isScanning,
    mascotMood = "idle",
    theme = "pink-white",
    hand = "right",
    shareStats = false,
    subPage = $bindable<SettingsSubPage>(null),
    onChangeTheme,
    onChangeHand,
    onChangeShareStats,
    onSaveSettings,
    onToggleAccount,
    onScanNow,
    onPickPhoto,
    onPickStatement,
    onReparseSlips,
    onOpenRules,
    onCreateCategory,
    onDeleteCategory,
    onCreateTag,
    onToggleTag,
    onExportCsv,
    onExportJson,
    onReplayOnboarding,
  }: {
    settings: AppSettings;
    accounts: Account[];
    categories: Category[];
    tags: Tag[];
    isScanning: boolean;
    mascotMood?: MascotMood;
    theme?: AppTheme;
    hand?: HandMode;
    /** Anonymous usage stats switch (opt-in, off by default). See $lib/analytics. */
    shareStats?: boolean;
    subPage?: SettingsSubPage;
    onChangeTheme: (next: AppTheme) => void;
    onChangeHand: (next: HandMode) => void;
    onChangeShareStats: (on: boolean) => void;
    onSaveSettings: (next: AppSettings) => Promise<void>;
    onToggleAccount: (id: string, enabled: boolean) => Promise<void>;
    onScanNow: () => Promise<void>;
    onPickPhoto: () => void;
    onPickStatement: (pdfBase64: string) => void;
    onReparseSlips: () => Promise<void>;
    onOpenRules: () => void;
    onCreateCategory: (input: {
      nameEn: string;
      nameTh: string;
      kind: "spending" | "income";
      icon: string;
      color: string;
    }) => Promise<void>;
    onDeleteCategory: (id: string) => Promise<void>;
    onCreateTag: (name: string) => Promise<void>;
    onToggleTag: (id: string, disabled: boolean) => Promise<void>;
    onExportCsv: () => Promise<void>;
    onExportJson: () => Promise<void>;
    onReplayOnboarding?: () => void;
  } = $props();

  const initialSettings = untrack(() => settings);
  let draft = $state<AppSettings>({
    ...initialSettings,
    ownerNames: [...initialSettings.ownerNames],
  });
  let ownerNamesCsv = $state(initialSettings.ownerNames.join(", "));
  let editingOwners = $state(false);
  let ownerSavedToast = $state(false);
  let savedToast = $state(false);

  let newTagName = $state("");
  let newCatEn = $state("");
  let newCatTh = $state("");
  let newCatKind = $state<"spending" | "income">("spending");
  let newCatIcon = $state<string>("coffee");
  let newCatColor = $state<string>("#FF4D94");
  let catSavedToast = $state(false);
  let catError = $state("");
  let deletingCatId = $state<string | null>(null);
  let showAllBanks = $state(false);
  let statementInput = $state<HTMLInputElement | null>(null);

  const CAT_COLORS = [
    "#FF4D94",
    "#FF6B1A",
    "#22c55e",
    "#06b6d4",
    "#a855f7",
    "#eab308",
    "#3b82f6",
    "#ef4444",
  ];

  const CAT_ICONS = [
    "coffee",
    "utensils",
    "car",
    "shopping-bag",
    "shopping-basket",
    "shirt",
    "film",
    "gamepad-2",
    "music",
    "home",
    "heart-pulse",
    "dumbbell",
    "paw-print",
    "gift",
    "plane",
    "graduation-cap",
    "book-open",
    "briefcase",
    "trending-up",
    "credit-card",
    "smartphone",
    "wallet",
    "sparkles",
    "star",
  ];

  $effect(() => {
    const incoming = settings;
    if (!editingOwners) {
      ownerNamesCsv = incoming.ownerNames.join(", ");
    }
    draft = {
      ...incoming,
      ownerNames: [...incoming.ownerNames],
    };
  });

  // Scroll to top cleanly when navigating into/out of a settings sub-page
  $effect(() => {
    void subPage;
    if (typeof window !== "undefined") {
      window.scrollTo({ top: 0, behavior: "instant" });
    }
  });

  const lang = $derived(settings.language);
  const bankAccounts = $derived(accounts.filter((a) => a.kind !== "cash"));
  const enabledBankCount = $derived(bankAccounts.filter((b) => b.enabled).length);
  const sortedCategories = $derived([
    ...categories.filter((c) => !c.isSystem),
    ...categories.filter((c) => c.isSystem),
  ]);

  function parseOwnerCsv(raw: string): string[] {
    return raw
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
  }

  async function saveOwnerNames() {
    editingOwners = false;
    const names = parseOwnerCsv(ownerNamesCsv);
    ownerNamesCsv = names.join(", ");
    await onSaveSettings({ ...draft, ownerNames: names });
    ownerSavedToast = true;
    setTimeout(() => (ownerSavedToast = false), 2200);
  }

  async function saveAllSettings() {
    const names = parseOwnerCsv(ownerNamesCsv);
    ownerNamesCsv = names.join(", ");
    await onSaveSettings({ ...draft, ownerNames: names });
    savedToast = true;
    setTimeout(() => (savedToast = false), 1800);
  }

  async function addTag() {
    if (!newTagName.trim()) return;
    await onCreateTag(newTagName.trim());
    newTagName = "";
  }

  async function addCategory() {
    const en = newCatEn.trim() || newCatTh.trim();
    const th = newCatTh.trim() || newCatEn.trim();
    if (!en) {
      catError = lang === "th" ? "กรุณากรอกชื่อหมวดหมู่" : "Enter a category name";
      return;
    }
    catError = "";
    try {
      await onCreateCategory({
        nameEn: en,
        nameTh: th,
        kind: newCatKind,
        icon: newCatIcon,
        color: newCatColor,
      });
      newCatEn = "";
      newCatTh = "";
      catSavedToast = true;
      setTimeout(() => (catSavedToast = false), 2000);
    } catch (e) {
      catError = String(e);
    }
  }

  async function removeCategory(id: string) {
    if (deletingCatId) return;
    deletingCatId = id;
    try {
      await onDeleteCategory(id);
    } finally {
      deletingCatId = null;
    }
  }

  const subPageTitle = $derived.by(() => {
    switch (subPage) {
      case "appearance":
        return lang === "th" ? "ธีมและภาษา" : "Appearance & Theme";
      case "slips":
        return lang === "th" ? "สแกนสลิปและธนาคาร" : "Bank Slips & Auto-Scan";
      case "categories":
        return lang === "th" ? "หมวดหมู่และป้ายกำกับ" : "Categories & Tags";
      case "calendar":
        return t(lang, "calendarSettings");
      case "data":
        return lang === "th" ? "สำรองข้อมูลและ OCR" : "Backup & Local OCR";
      case "about":
        return lang === "th" ? "เกี่ยวกับ" : "About";
      case "terms":
        return lang === "th" ? "ข้อกำหนดการใช้" : "Terms";
      case "privacy":
        return lang === "th" ? "ความเป็นส่วนตัว" : "Privacy";
      default:
        return "";
    }
  });

  function closeSubPage() {
    if (subPage === "terms" || subPage === "privacy") subPage = "about";
    else subPage = null;
  }

  async function openGitHub() {
    try {
      await openUrl(GITHUB_URL);
    } catch {
      window.open(GITHUB_URL, "_blank", "noopener");
    }
  }
</script>

<div
  class="bg-[var(--bg-canvas)] {subPage === null
    ? 'min-h-screen pb-28'
    : 'fixed inset-0 z-10 flex flex-col overflow-hidden'}"
>
  {#if subPage === null}
    <!-- Clean Main My Moo Hub -->
    <header
      class="text-white px-5 pb-5 rounded-b-3xl shadow-lg"
      style="background: linear-gradient(135deg, var(--hero-from), var(--hero-to)); padding-top: max(env(safe-area-inset-top), 40px);"
    >
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-2.5 min-w-0">
          <MooMascot mood={isScanning ? "scan" : mascotMood} size="md" alt="MooJot" />
          <div class="min-w-0">
            <h1 class="text-2xl font-extrabold tracking-tight">
              {lang === "th" ? "สวัสดี มนุษย์หมู!" : "Hello, Moo Human!"}
            </h1>
            <p class="text-xs font-medium text-white/85 mt-0.5">
              {lang === "th"
                ? "โอเพนซอร์ส 100% · ข้อมูลอยู่ในเครื่องคุณ"
                : "100% Open Source · Local-First"}
            </p>
          </div>
        </div>
      </div>
    </header>

    <div class="px-4 mt-4 space-y-4">
      <!-- Quick Slip Actions -->
      <div class="grid grid-cols-2 gap-2.5">
        <button
          type="button"
          onclick={onScanNow}
          disabled={isScanning}
          class="flex items-center justify-center gap-2 py-3 px-3 rounded-2xl bg-[var(--brand)] text-white text-xs font-bold shadow-sm disabled:opacity-50 active:bg-[var(--brand-deep)]"
        >
          <ScanLine class="size-4" />
          <span>{isScanning ? t(lang, "loading") : t(lang, "scanESlips")}</span>
        </button>
        <button
          type="button"
          onclick={onPickPhoto}
          class="flex items-center justify-center gap-2 py-3 px-3 rounded-2xl text-xs font-bold shadow-sm active:opacity-85"
          style="background-color: var(--btn-contrast-bg); color: var(--btn-contrast-text);"
        >
          <ImagePlus class="size-4" />
          <span>{lang === "th" ? "เลือกรูปสลิป" : "Pick Slip Photo"}</span>
        </button>
      </div>

      <!-- Clean Grouped Settings Navigation Menu -->
      <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] overflow-hidden divide-y divide-[var(--border-subtle)] shadow-xs">
        <!-- 1. Appearance & Theme -->
        <button
          type="button"
          onclick={() => (subPage = "appearance")}
          class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
              <Palette class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">
                {lang === "th" ? "ธีมสีและหน้าตา" : "Appearance & Theme"}
              </div>
              <div class="text-xs text-[var(--text-muted)] truncate">
                {theme === "pink-white"
                  ? lang === "th" ? "ชมพู & ขาว (ค่าเริ่มต้น)" : "Pink & White (Default)"
                  : lang === "th" ? "ดำ & ส้ม (โหมดมืด)" : "Black & Orange (Dark)"}
              </div>
            </div>
          </div>
          <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
        </button>

        <!-- 2. Bank Slips & Auto-Scan -->
        <button
          type="button"
          onclick={() => (subPage = "slips")}
          class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
              <Building2 class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">
                {lang === "th" ? "สแกนสลิปและธนาคาร" : "Bank Slips & Auto-Scan"}
              </div>
              <div class="text-xs text-[var(--text-muted)] truncate">
                {lang === "th"
                  ? `${enabledBankCount}/${bankAccounts.length} ธนาคาร · อัลบั้มแชร์ · ชื่อเจ้าของ`
                  : `${enabledBankCount}/${bankAccounts.length} banks · Shared album · Owner names`}
              </div>
            </div>
          </div>
          <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
        </button>

        <!-- 3. Auto-Categorize Rules -->
        <button
          type="button"
          onclick={onOpenRules}
          class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
              <WandSparkles class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">
                {lang === "th" ? "กฎจัดหมวดหมู่อัตโนมัติ" : "Auto-Categorize Rules"}
              </div>
              <div class="text-xs text-[var(--text-muted)] truncate">
                {lang === "th"
                  ? "ฟรี 247 ร้านค้าไทย + กฎที่คุณกำหนดเอง"
                  : "Free 247 built-in rules + learned & custom"}
              </div>
            </div>
          </div>
          <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
        </button>

        <!-- 4. Categories & Tags -->
        <button
          type="button"
          onclick={() => (subPage = "categories")}
          class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
              <Sparkles class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">
                {lang === "th" ? "หมวดหมู่และป้ายกำกับ" : "Categories & Tags"}
              </div>
              <div class="text-xs text-[var(--text-muted)] truncate">
                {lang === "th"
                  ? `${categories.length} หมวดหมู่ · ${tags.length} ป้ายกำกับ`
                  : `${categories.length} categories · ${tags.length} tags`}
              </div>
            </div>
          </div>
          <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
        </button>

        <!-- 5. Calendar & Payday Cycle -->
        <button
          type="button"
          onclick={() => (subPage = "calendar")}
          class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
              <Calendar class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">
                {t(lang, "calendarSettings")}
              </div>
              <div class="text-xs text-[var(--text-muted)] truncate">
                {lang === "th"
                  ? `รอบเดือนเริ่มวันที่ ${settings.monthStartDay}`
                  : `Month cycle starts day ${settings.monthStartDay}`}
              </div>
            </div>
          </div>
          <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
        </button>

        <!-- 6. Backup & Local OCR -->
        <button
          type="button"
          onclick={() => (subPage = "data")}
          class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-emerald-500/15 text-emerald-500 flex items-center justify-center shrink-0">
              <Database class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">
                {lang === "th" ? "สำรองข้อมูลและ OCR ออฟไลน์" : "Backup & Local OCR"}
              </div>
              <div class="text-xs text-[var(--text-muted)] truncate">
              {lang === "th"
                ? "OCR ในเครื่อง · ส่งออก CSV / JSON"
                : "On-device OCR · Export CSV / JSON"}
              </div>
            </div>
          </div>
          <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
        </button>

        <!-- 7. Anonymous usage stats (opt-in) -->
        <label class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left">
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
              <ChartNoAxesColumn class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">{t(lang, "shareStats")}</div>
              <div class="text-xs text-[var(--text-muted)]">{t(lang, "shareStatsHint")}</div>
            </div>
          </div>
          <input
            type="checkbox"
            checked={shareStats}
            onchange={(e) => onChangeShareStats(e.currentTarget.checked)}
            class="size-5 shrink-0 accent-[var(--brand)]"
          />
        </label>

        <!-- 8. About -->
        <button
          type="button"
          onclick={() => (subPage = "about")}
          class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
              <Info class="size-4" />
            </span>
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)]">
                {lang === "th" ? "เกี่ยวกับ" : "About"}
              </div>
              <div class="text-xs text-[var(--text-muted)] truncate">
                {lang === "th"
                  ? "ข้อกำหนด · ความเป็นส่วนตัว · GitHub"
                  : "Terms · Privacy · GitHub"}
              </div>
            </div>
          </div>
          <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
        </button>

        {#if onReplayOnboarding}
          <!-- 9. Replay the first-run welcome tour -->
          <button
            type="button"
            onclick={onReplayOnboarding}
            class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
                <Sparkles class="size-4" />
              </span>
              <div class="min-w-0">
                <div class="text-sm font-bold text-[var(--text-primary)]">{t(lang, "onbReplay")}</div>
                <div class="text-xs text-[var(--text-muted)] truncate">{t(lang, "onbReplayHint")}</div>
              </div>
            </div>
            <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
          </button>
        {/if}
      </section>
    </div>
  {:else}
    <!-- Focused Sub-Page Header seamlessly extending behind status bar -->
    <header
      class="flex shrink-0 items-center gap-2 px-3 pb-3 text-white shadow-sm"
      style="background: linear-gradient(135deg, var(--hero-from), var(--hero-to)); padding-top: max(env(safe-area-inset-top), 38px);"
    >
      <button
        type="button"
        onclick={closeSubPage}
        class="size-10 flex items-center justify-center rounded-full active:bg-black/15"
        aria-label={t(lang, "back")}
      >
        <ArrowLeft class="size-6" />
      </button>
      <h2 class="flex-1 text-base font-extrabold tracking-tight">{subPageTitle}</h2>
    </header>

    <div class="min-h-0 flex-1 space-y-4 overflow-y-auto px-4 pt-4 pb-28">
      {#if subPage === "appearance"}
        <!-- Theme Picker -->
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-3 shadow-xs">
          <h3 class="text-sm font-bold text-[var(--text-primary)] flex items-center gap-2">
            <Palette class="size-4 text-[var(--brand)]" />
            <span>{lang === "th" ? "ธีมสีของแอป" : "App Color Theme"}</span>
          </h3>
          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              onclick={() => onChangeTheme("pink-white")}
              class="flex items-center gap-2.5 rounded-2xl p-3.5 border text-left transition-all {theme === 'pink-white'
                ? 'border-[#FF4D94] bg-[#FF4D94]/10 ring-1 ring-[#FF4D94]'
                : 'border-[var(--border-subtle)] bg-[var(--bg-input)]'}"
            >
              <span class="flex -space-x-1.5 shrink-0">
                <span class="size-5 rounded-full bg-[#FF4D94] ring-2 ring-white"></span>
                <span class="size-5 rounded-full bg-white ring-2 ring-pink-200"></span>
              </span>
              <div class="min-w-0">
                <div class="text-xs font-bold text-[var(--text-primary)] truncate">
                  {lang === "th" ? "ชมพู & ขาว" : "Pink & White"}
                </div>
                <div class="text-[10px] text-[var(--text-muted)]">
                  {lang === "th" ? "ค่าเริ่มต้น" : "Default"}
                </div>
              </div>
            </button>

            <button
              type="button"
              onclick={() => onChangeTheme("orange-black")}
              class="flex items-center gap-2.5 rounded-2xl p-3.5 border text-left transition-all {theme === 'orange-black'
                ? 'border-[#FF6B1A] bg-[#FF6B1A]/15 ring-1 ring-[#FF6B1A]'
                : 'border-[var(--border-subtle)] bg-[var(--bg-input)]'}"
            >
              <span class="flex -space-x-1.5 shrink-0">
                <span class="size-5 rounded-full bg-[#FF6B1A] ring-2 ring-[#0a0a0c]"></span>
                <span class="size-5 rounded-full bg-[#0a0a0c] ring-2 ring-zinc-600"></span>
              </span>
              <div class="min-w-0">
                <div class="text-xs font-bold text-[var(--text-primary)] truncate">
                  {lang === "th" ? "ดำ & ส้ม" : "Black & Orange"}
                </div>
                <div class="text-[10px] text-[var(--text-muted)]">
                  {lang === "th" ? "โหมดมืด" : "Dark mode"}
                </div>
              </div>
            </button>
          </div>
        </section>

        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-3 shadow-xs">
          <h3 class="text-sm font-bold text-[var(--text-primary)]">
            {lang === "th" ? "มือที่ใช้" : "Hand"}
          </h3>
          <p class="text-[11px] text-[var(--text-muted)]">
            {lang === "th"
              ? "ปุ่มอยู่ฝั่งมือที่คุณใช้"
              : "Buttons sit on the hand you use."}
          </p>
          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              onclick={() => onChangeHand("left")}
              class="py-2.5 rounded-xl text-xs font-bold border {hand === 'left'
                ? 'bg-[var(--brand)] text-white border-[var(--brand)]'
                : 'bg-[var(--bg-input)] text-[var(--text-primary)] border-[var(--border-subtle)]'}"
            >
              {lang === "th" ? "มือซ้าย" : "Left hand"}
            </button>
            <button
              type="button"
              onclick={() => onChangeHand("right")}
              class="py-2.5 rounded-xl text-xs font-bold border {hand === 'right'
                ? 'bg-[var(--brand)] text-white border-[var(--brand)]'
                : 'bg-[var(--bg-input)] text-[var(--text-primary)] border-[var(--border-subtle)]'}"
            >
              {lang === "th" ? "มือขวา" : "Right hand"}
            </button>
          </div>
        </section>

        <!-- Language Picker -->
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-3 shadow-xs">
          <h3 class="text-sm font-bold text-[var(--text-primary)] flex items-center gap-2">
            <Globe class="size-4 text-[var(--brand)]" />
            <span>{lang === "th" ? "ภาษา" : "Language"}</span>
          </h3>
          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              onclick={async () => {
                draft.language = "th";
                await saveAllSettings();
              }}
              class="py-2.5 rounded-xl text-xs font-bold border {draft.language === 'th'
                ? 'bg-[var(--brand)] text-white border-[var(--brand)]'
                : 'bg-[var(--bg-input)] text-[var(--text-primary)] border-[var(--border-subtle)]'}"
            >
              ภาษาไทย
            </button>
            <button
              type="button"
              onclick={async () => {
                draft.language = "en";
                await saveAllSettings();
              }}
              class="py-2.5 rounded-xl text-xs font-bold border {draft.language === 'en'
                ? 'bg-[var(--brand)] text-white border-[var(--brand)]'
                : 'bg-[var(--bg-input)] text-[var(--text-primary)] border-[var(--border-subtle)]'}"
            >
              English
            </button>
          </div>
        </section>

      {:else if subPage === "slips"}
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-4 shadow-xs">
          <button
            type="button"
            onclick={onReparseSlips}
            class="w-full py-2.5 px-3 rounded-xl bg-[var(--brand-soft)] border border-[var(--brand-border)] text-[var(--brand-text)] text-xs font-bold"
          >
            {lang === "th" ? "ประมวลผล + แนบสลิปเดิมใหม่ทั้งหมด" : "Re-parse & Attach Existing Slips"}
          </button>

          <button
            type="button"
            onclick={() => {
              if (window.MooJotAndroid?.pickStatementPdf) {
                window.MooJotAndroid.pickStatementPdf();
                return;
              }
              statementInput?.click();
            }}
            class="w-full py-2.5 px-3 rounded-xl border border-[var(--border-strong)] text-[var(--text-primary)] text-xs font-bold inline-flex items-center justify-center gap-2"
          >
            <FileText class="size-4 text-[var(--brand)]" />
            {t(lang, "importStatement")}
          </button>
          <input
            bind:this={statementInput}
            type="file"
            accept="application/pdf"
            class="hidden"
            onchange={(event) => {
              const file = event.currentTarget.files?.[0];
              event.currentTarget.value = "";
              if (!file) return;
              const reader = new FileReader();
              reader.onload = () => {
                const raw = String(reader.result ?? "");
                const comma = raw.indexOf(",");
                onPickStatement(comma >= 0 ? raw.slice(comma + 1) : raw);
              };
              reader.readAsDataURL(file);
            }}
          />

          <!-- Shared album opt-in -->
          <label class="flex items-center justify-between gap-3 pt-1">
            <div>
              <div class="text-xs font-bold text-[var(--text-primary)]">{t(lang, "allowSharedAlbum")}</div>
              <div class="text-[11px] text-[var(--text-muted)]">
                {lang === "th"
                  ? "ปิดไว้ = ไม่อ่าน Pictures/DCIM · เปิด = ตรวจขนาด + บังคับต้องมี BOT Slip QR ก่อนอ่านเสมอ"
                  : "OFF = never reads Pictures/DCIM · ON = strict slip aspect + mandatory BOT Slip QR gate"}
              </div>
            </div>
            <input
              type="checkbox"
              bind:checked={draft.allowSharedAlbum}
              onchange={saveAllSettings}
              class="size-5 accent-[var(--brand)]"
            />
          </label>

          <!-- Self-transfer Owner Name Matcher -->
          <div class="pt-3 border-t border-[var(--border-subtle)] space-y-2">
            <div class="flex items-center justify-between gap-2">
              <label class="text-xs font-bold text-[var(--text-primary)] block" for="owner-names">
                {t(lang, "ownerNames")} ({lang === "th" ? "ตรวจจับโอนเข้าบัญชีตัวเอง" : "self-transfer matcher"})
              </label>
              {#if ownerSavedToast}
                <span class="text-[11px] font-bold text-emerald-500">
                  {lang === "th" ? "บันทึกแล้ว ✓" : "Saved ✓"}
                </span>
              {/if}
            </div>
            <div class="flex gap-2">
              <input
                id="owner-names"
                type="text"
                bind:value={ownerNamesCsv}
                onfocus={() => (editingOwners = true)}
                onblur={saveOwnerNames}
                onkeydown={(e) => {
                  if (e.key === "Enter") {
                    e.currentTarget.blur();
                  }
                }}
                placeholder="MR. SOMCHAI, สมชาย"
                class="flex-1 min-w-0 rounded-xl bg-[var(--bg-input)] border border-[var(--border-strong)] px-3 py-2 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] outline-none focus:border-[var(--brand)]"
              />
              <button
                type="button"
                onclick={saveOwnerNames}
                class="px-3.5 py-2 rounded-xl bg-[var(--brand)] text-white text-xs font-bold shrink-0 active:bg-[var(--brand-deep)]"
              >
                {t(lang, "save")}
              </button>
            </div>
            {#if settings.ownerNames.length > 0}
              <div class="flex flex-wrap gap-1.5">
                {#each settings.ownerNames as owner}
                  <span class="rounded-full bg-[var(--brand-soft)] border border-[var(--brand-border)] px-2.5 py-0.5 text-[11px] font-semibold text-[var(--brand-text)]">
                    {owner}
                  </span>
                {/each}
              </div>
            {/if}
          </div>

          <!-- 20 Thai Banks -->
          <div class="pt-3 border-t border-[var(--border-subtle)] space-y-2">
            <button
              type="button"
              onclick={() => (showAllBanks = !showAllBanks)}
              class="text-xs font-bold text-[var(--brand-text)]"
            >
              {showAllBanks
                ? lang === "th" ? "ซ่อนรายชื่อธนาคาร ▲" : "Hide 20 supported banks ▲"
                : lang === "th" ? `เลือกธนาคารที่ต้องการสแกน (${enabledBankCount}/20) ▼` : `Configure all 20 Thai banks (${enabledBankCount}/20) ▼`}
            </button>
            {#if showAllBanks}
              <div class="grid grid-cols-1 gap-1.5 max-h-72 overflow-y-auto pr-1">
                {#each bankAccounts as acc}
                  <label class="flex items-center justify-between rounded-xl bg-[var(--bg-input)] border border-[var(--border-subtle)] px-3 py-2">
                    <div class="flex items-center gap-2.5">
                      <span
                        class="size-3 rounded-full shrink-0"
                        style="background-color: {acc.color};"
                      ></span>
                      <span class="text-xs text-[var(--text-primary)]">
                        {lang === "th" ? acc.nameTh : acc.nameEn}
                      </span>
                    </div>
                    <input
                      type="checkbox"
                      checked={acc.enabled}
                      onchange={(e) => onToggleAccount(acc.id, e.currentTarget.checked)}
                      class="size-4 accent-[var(--brand)]"
                    />
                  </label>
                {/each}
              </div>
            {/if}
          </div>
        </section>

      {:else if subPage === "categories"}
        <!-- Create Custom Category with Icon + Color Picker -->
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-3.5 shadow-xs">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-bold text-[var(--text-primary)] flex items-center gap-2">
              <Plus class="size-4 text-[var(--brand)]" />
              <span>{lang === "th" ? "เพิ่มหมวดหมู่ใหม่" : "Add New Category"}</span>
            </h3>
            {#if catSavedToast}
              <span class="text-[11px] font-bold text-emerald-500">
                {lang === "th" ? "เพิ่มแล้ว ✓" : "Added ✓"}
              </span>
            {/if}
          </div>

          <!-- Live Preview -->
          <div class="flex items-center gap-3 rounded-2xl bg-[var(--bg-input)] border border-[var(--border-subtle)] px-3.5 py-2.5">
            <CategoryIcon icon={newCatIcon} color={newCatColor} size="md" />
            <div class="min-w-0">
              <div class="text-sm font-bold text-[var(--text-primary)] truncate">
                {lang === "th"
                  ? newCatTh.trim() || newCatEn.trim() || "ชื่อหมวดหมู่ใหม่"
                  : newCatEn.trim() || newCatTh.trim() || "New Category Name"}
              </div>
              <div class="text-[11px] text-[var(--text-muted)]">
                {newCatKind === "income" ? t(lang, "income") : t(lang, "spending")}
              </div>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-2">
            <input
              type="text"
              bind:value={newCatEn}
              placeholder={lang === "th" ? "ชื่ออังกฤษ (ไม่บังคับ)" : "Category (EN)"}
              class="rounded-xl bg-[var(--bg-input)] border border-[var(--border-strong)] px-3 py-2 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] outline-none focus:border-[var(--brand)]"
            />
            <input
              type="text"
              bind:value={newCatTh}
              placeholder={lang === "th" ? "ชื่อหมวด (ไทย)" : "Thai name (optional)"}
              class="rounded-xl bg-[var(--bg-input)] border border-[var(--border-strong)] px-3 py-2 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] outline-none focus:border-[var(--brand)]"
            />
          </div>

          <!-- Color Picker -->
          <div class="space-y-1.5">
            <div class="text-[11px] font-semibold text-[var(--text-muted)]">
              {lang === "th" ? "เลือกสี" : "Pick color"}
            </div>
            <div class="flex flex-wrap items-center gap-2.5">
              {#each CAT_COLORS as hex}
                <button
                  type="button"
                  onclick={() => (newCatColor = hex)}
                  aria-label="Color {hex}"
                  class="size-7 rounded-full transition-transform {newCatColor === hex
                    ? 'ring-2 ring-[var(--text-primary)] scale-110'
                    : 'opacity-75'}"
                  style="background-color: {hex};"
                ></button>
              {/each}
            </div>
          </div>

          <!-- Icon Picker Grid -->
          <div class="space-y-1.5">
            <div class="text-[11px] font-semibold text-[var(--text-muted)]">
              {lang === "th" ? "เลือกไอคอน" : "Pick icon"}
            </div>
            <div class="grid grid-cols-6 gap-2">
              {#each CAT_ICONS as ic}
                <button
                  type="button"
                  onclick={() => (newCatIcon = ic)}
                  aria-label="Icon {ic}"
                  class="p-1.5 rounded-xl flex items-center justify-center transition-all {newCatIcon === ic
                    ? 'bg-[var(--brand-soft)] ring-2 ring-[var(--brand)] scale-105'
                    : 'bg-[var(--bg-input)] border border-[var(--border-subtle)]'}"
                >
                  <CategoryIcon icon={ic} color={newCatColor} size="sm" />
                </button>
              {/each}
            </div>
          </div>

          <div class="flex gap-2 pt-1">
            <select
              bind:value={newCatKind}
              class="rounded-xl bg-[var(--bg-input)] border border-[var(--border-strong)] px-3 py-2 text-xs text-[var(--text-primary)] outline-none"
            >
              <option value="spending">{t(lang, "spending")}</option>
              <option value="income">{t(lang, "income")}</option>
            </select>
            <button
              type="button"
              onclick={addCategory}
              disabled={!newCatEn.trim() && !newCatTh.trim()}
              class="flex-1 py-2.5 rounded-xl bg-[var(--brand)] text-white text-xs font-bold flex items-center justify-center gap-1 disabled:opacity-40 active:bg-[var(--brand-deep)]"
            >
              <Plus class="size-4" />
              <span>{t(lang, "add")}</span>
            </button>
          </div>
          {#if catError}
            <p class="text-xs text-rose-500">{catError}</p>
          {/if}
        </section>

        <!-- Existing Categories List with Delete Button -->
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-2.5 shadow-xs">
          <h3 class="text-sm font-bold text-[var(--text-primary)]">
            {t(lang, "categories")} ({categories.length})
          </h3>
          <div class="space-y-2">
            {#each sortedCategories as c (c.id)}
              <div class="flex items-center justify-between gap-3 rounded-2xl bg-[var(--bg-input)] border border-[var(--border-subtle)] px-3 py-2">
                <div class="flex items-center gap-2.5 min-w-0">
                  <CategoryIcon icon={c.icon} color={c.color} size="sm" />
                  <div class="min-w-0">
                    <div class="text-xs font-bold text-[var(--text-primary)] truncate">
                      {lang === "th" ? c.nameTh : c.nameEn}
                    </div>
                    <div class="text-[10px] text-[var(--text-muted)]">
                      {c.kind === "income" ? t(lang, "income") : t(lang, "spending")}
                      {!c.isSystem ? (lang === "th" ? " · กำหนดเอง" : " · Custom") : ""}
                    </div>
                  </div>
                </div>
                <button
                  type="button"
                  onclick={() => removeCategory(c.id)}
                  disabled={deletingCatId === c.id}
                  aria-label="Delete {c.nameEn}"
                  class="size-8 rounded-full flex items-center justify-center text-[var(--text-muted)] hover:text-rose-500 active:bg-rose-500/15 disabled:opacity-40 shrink-0"
                >
                  <Trash2 class="size-4" />
                </button>
              </div>
            {/each}
          </div>
        </section>

        <!-- #Tags Section -->
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-3 shadow-xs">
          <h3 class="text-sm font-bold text-[var(--text-primary)]"># {t(lang, "tags")} ({tags.length})</h3>
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={newTagName}
              placeholder="#bigspending"
              class="flex-1 rounded-xl bg-[var(--bg-input)] border border-[var(--border-strong)] px-3 py-2 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] outline-none focus:border-[var(--brand)]"
            />
            <button
              type="button"
              onclick={addTag}
              class="px-4 py-2 rounded-xl bg-[var(--brand)] text-white text-xs font-bold active:bg-[var(--brand-deep)]"
            >
              {t(lang, "add")}
            </button>
          </div>
          {#if tags.length > 0}
            <div class="flex flex-wrap gap-2">
              {#each tags as tg}
                <button
                  type="button"
                  onclick={() => onToggleTag(tg.id, !tg.disabled)}
                  class="px-3 py-1 rounded-full text-xs font-semibold {tg.disabled
                    ? 'bg-[var(--bg-elevated)] text-[var(--text-muted)] line-through'
                    : 'bg-[var(--brand-soft)] text-[var(--brand-text)] border border-[var(--brand-border)]'}"
                >
                  #{tg.name}
                </button>
              {/each}
            </div>
          {/if}
        </section>

      {:else if subPage === "calendar"}
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-3 shadow-xs">
          <div class="flex items-center justify-between gap-3">
            <span class="text-xs font-medium text-[var(--text-secondary)]">
              {t(lang, "monthStartDay")} (1–28)
            </span>
            <input
              type="number"
              min="1"
              max="28"
              bind:value={draft.monthStartDay}
              class="w-20 rounded-xl bg-[var(--bg-input)] border border-[var(--border-strong)] px-3 py-1.5 text-center text-sm text-[var(--text-primary)] outline-none focus:border-[var(--brand)]"
            />
          </div>
          <button
            type="button"
            onclick={saveAllSettings}
            class="w-full py-2.5 rounded-xl bg-[var(--brand)] text-white font-bold text-xs flex items-center justify-center gap-1.5 active:bg-[var(--brand-deep)]"
          >
            <Check class="size-4" />
            <span>{savedToast ? (lang === "th" ? "บันทึกแล้ว ✓" : "Saved ✓") : t(lang, "save")}</span>
          </button>
        </section>

      {:else if subPage === "data"}
        <!-- Local OCR Status -->
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-2.5 shadow-xs">
          <h3 class="text-sm font-bold text-[var(--text-primary)] flex items-center gap-2">
            <ShieldCheck class="size-4 text-emerald-500" />
            <span>{lang === "th" ? "OCR ในเครื่อง" : "On-device OCR"}</span>
          </h3>
          <p class="text-xs text-[var(--text-secondary)] leading-relaxed">
            {lang === "th"
              ? "อ่านสลิปภาษาไทยด้วย AksonOCR-Lite (ONNX 4.3 MB, Apache-2.0) และภาษาอังกฤษด้วย ML Kit Latin บนเครื่อง ขั้นตอนนี้ไม่อัปโหลดรูป ถ้าในฐานข้อมูลมีคีย์ AksonOCR อยู่แล้ว สลิปที่ยังต้องอ่านภาษาไทยอาจถูกส่งไปที่ AksonOCR"
              : "Reads Thai with AksonOCR-Lite (4.3 MB ONNX, Apache-2.0) and English with ML Kit Latin on the phone. Those steps do not upload the photo. If an AksonOCR API key is already stored in the local database, a slip that still needs Thai text can be sent to AksonOCR."}
          </p>
          <div class="flex flex-wrap gap-2 text-[11px] font-semibold">
            <span class="rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-300 px-2.5 py-0.5">Thai: AksonOCR-Lite ONNX</span>
            <span class="rounded-full bg-[var(--bg-elevated)] text-[var(--text-primary)] px-2.5 py-0.5">EN: ML Kit Latin</span>
            <span class="rounded-full bg-[var(--brand-soft)] text-[var(--brand-text)] px-2.5 py-0.5">QR: ZXing BOT TLV</span>
          </div>
        </section>

        <!-- Free Export CSV & JSON -->
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 space-y-3 shadow-xs">
          <h3 class="text-sm font-bold text-[var(--text-primary)] flex items-center gap-2">
            <Download class="size-4 text-[var(--brand)]" />
            <span>{t(lang, "export")} & {t(lang, "backup")}</span>
          </h3>
          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              onclick={onExportCsv}
              class="py-2.5 rounded-xl bg-[var(--brand)] text-white text-xs font-bold active:bg-[var(--brand-deep)]"
            >
              {t(lang, "exportCsv")}
            </button>
            <button
              type="button"
              onclick={onExportJson}
              class="py-2.5 rounded-xl bg-[var(--bg-elevated)] border border-[var(--border-subtle)] text-[var(--text-primary)] text-xs font-bold"
            >
              {t(lang, "exportJson")}
            </button>
          </div>
        </section>

      {:else if subPage === "about"}
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] overflow-hidden divide-y divide-[var(--border-subtle)] shadow-xs">
          <button
            type="button"
            onclick={() => (subPage = "terms")}
            class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
                <Scale class="size-4" />
              </span>
              <div class="min-w-0">
                <div class="text-sm font-bold text-[var(--text-primary)]">
                  {lang === "th" ? "ข้อกำหนดการใช้" : "Terms and conditions"}
                </div>
                <div class="text-xs text-[var(--text-muted)] truncate">
                  {lang === "th" ? "ฉบับภาษาอังกฤษมีผลบังคับ" : "English text controls"}
                </div>
              </div>
            </div>
            <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
          </button>
          <button
            type="button"
            onclick={() => (subPage = "privacy")}
            class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
                <ShieldCheck class="size-4" />
              </span>
              <div class="min-w-0">
                <div class="text-sm font-bold text-[var(--text-primary)]">
                  {lang === "th" ? "นโยบายความเป็นส่วนตัว" : "Privacy policy"}
                </div>
                <div class="text-xs text-[var(--text-muted)] truncate">
                  {lang === "th" ? "บัญชีอยู่ในเครื่องคุณ" : "The ledger stays on your phone"}
                </div>
              </div>
            </div>
            <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
          </button>
          <button
            type="button"
            onclick={openGitHub}
            class="w-full px-4 py-3.5 flex items-center justify-between gap-3 text-left active:opacity-75"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span class="size-9 rounded-2xl bg-[var(--brand-soft)] text-[var(--brand)] flex items-center justify-center shrink-0">
                <GitFork class="size-4" />
              </span>
              <div class="min-w-0">
                <div class="text-sm font-bold text-[var(--text-primary)]">GitHub</div>
                <div class="text-xs text-[var(--text-muted)] truncate">trajectonomics/moojot</div>
              </div>
            </div>
            <ChevronRight class="size-4 text-[var(--text-muted)] shrink-0" />
          </button>
        </section>
        <p class="px-1 text-[11px] leading-relaxed text-[var(--text-muted)]">
          {lang === "th"
            ? "MooJot 1.0.0 เผยแพร่โดย Trajectonomics ภายใต้สัญญาอนุญาต MIT ข้อความกฎหมายฉบับภาษาอังกฤษเป็นฉบับที่มีผล"
            : "MooJot 1.0.0 is published by Trajectonomics under the MIT License. The English legal text is the official version."}
        </p>

      {:else if subPage === "terms"}
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 shadow-xs">
          <LegalText markdown={termsMarkdown} />
        </section>

      {:else if subPage === "privacy"}
        <section class="rounded-3xl bg-[var(--bg-card)] border border-[var(--border-subtle)] p-4 shadow-xs">
          <LegalText markdown={privacyMarkdown} />
        </section>
      {/if}
    </div>
  {/if}
</div>
