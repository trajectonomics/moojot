// The screens the screenshot test captures. One PNG per scene, named after it,
// in tests/screenshots/baseline/.
//
// A scene is:
//   name     file name of the baseline (kebab-case, unique)
//   fixture  options for fixtureScript() in fixture.mjs: lang, theme, hand, data, onboarding
//   ready    CSS selector that must be visible before `run` starts (default: a Home record row)
//   run      async (page) => {...} steps that bring the app to the screen to capture
//
// `page` offers click(selector), clickText(text, selector?), fill(selector, value),
// waitFor(selector), waitForText(text, selector?) and evaluate(js). Text matching is
// exact, on the element's whole visible text, and skips hidden elements.
//
// To add a scene: add an entry, then run `pnpm test:screenshots:update <name>`
// and look at the new PNG before committing it.

const DARK = { theme: "orange-black" };
// A first run on Android with photo access already allowed.
const FIRST_RUN_ANDROID = { data: "empty", onboarding: "pending", android: "granted" };
// The first scan covers this month and the two before it (FIRST_SCAN_MONTHS): from
// 1 January 2026, 00:00 in Bangkok, because the fixture's clock is 18 March 2026.
const FIRST_SCAN_SINCE = Date.parse("2026-01-01T00:00:00+07:00") / 1000;
const THAI = { lang: "th" };

// Bottom navigation: Home, Summary, My Moo. The buttons have no labels, and
// "Summary" also names a pill inside the Summary tab, so pick them by position.
const NAV = { home: 1, summary: 2, settings: 3 };
const openTab = (page, tab) => page.click(`nav.fixed button:nth-of-type(${NAV[tab]})`);

const openSummary = async (page) => {
  await openTab(page, "summary");
  await page.waitFor('button[aria-label="Previous"]');
};

const openSettings = async (page) => {
  await openTab(page, "settings");
  await page.waitForText("Appearance & Theme", "button *");
};

export const scenes = [
  // ------------------------------------------------------------------ home ---
  { name: "home" },
  { name: "home-dark", fixture: DARK },
  { name: "home-thai", fixture: THAI },
  { name: "home-left-hand", fixture: { hand: "left" } },
  {
    name: "home-empty-month",
    fixture: { data: "empty" },
    ready: 'svg[aria-label="No records yet"]',
  },
  {
    name: "home-scrolled",
    run: async (page) => {
      await page.evaluate("window.scrollTo(0, 420)");
      await page.waitFor('[aria-label="More actions"]');
    },
  },
  {
    name: "home-add-menu",
    run: async (page) => {
      await page.click('[aria-label="More actions"]');
      await page.waitFor('[aria-label="Close menu"]');
    },
  },
  {
    name: "home-month-picker",
    run: async (page) => {
      await page.click('[aria-label="Previous period"] + button');
      await page.waitFor('[role="dialog"][aria-labelledby="month-sheet-title"]');
    },
  },
  {
    name: "home-filter",
    run: async (page) => {
      await page.click('[aria-label="Filter Records"]');
      await page.waitFor('[role="dialog"][aria-labelledby="filter-sheet-title"]');
    },
  },
  {
    name: "search-results",
    run: async (page) => {
      await page.click('[aria-label="Search"]');
      await page.fill('input[placeholder^="Search"]', "st");
      await page.waitForText("Fuel Station", "button *");
    },
  },

  // ---------------------------------------------------------------- record ---
  {
    name: "record-add",
    run: async (page) => {
      await page.clickText("Add Record");
      await page.waitForText("Paste");
    },
  },
  {
    name: "record-add-dark",
    fixture: DARK,
    run: async (page) => {
      await page.clickText("Add Record");
      await page.waitForText("Paste");
    },
  },
  {
    name: "record-edit",
    run: async (page) => {
      await page.click("[data-swipe-row] button.relative");
      await page.waitFor('input[placeholder="Payee"]');
    },
  },

  // --------------------------------------------------------------- summary ---
  { name: "summary", run: openSummary },
  { name: "summary-dark", fixture: DARK, run: openSummary },
  { name: "summary-thai", fixture: THAI, run: openSummary },
  {
    name: "summary-compare",
    run: async (page) => {
      await openSummary(page);
      await page.clickText("Compare");
    },
  },
  {
    name: "summary-budgets",
    run: async (page) => {
      await openSummary(page);
      await page.clickText("Budgets");
      await page.waitForText("Add Budget");
    },
  },

  // -------------------------------------------------------------- settings ---
  { name: "settings", run: openSettings },
  // On Android My Moo also has the app lock switch (#79).
  { name: "settings-app-lock", fixture: { android: "granted" }, run: openSettings },
  // On Android My Moo also has the daily reminder (#80), here switched on.
  { name: "settings-reminder", fixture: { android: "granted", reminder: true }, run: openSettings },
  { name: "settings-dark", fixture: DARK, run: openSettings },
  {
    name: "settings-appearance",
    run: async (page) => {
      await openSettings(page);
      await page.clickText("Appearance & Theme", "button *");
      await page.waitFor('button[aria-label="Back"]');
    },
  },
  {
    name: "settings-categories",
    run: async (page) => {
      await openSettings(page);
      await page.clickText("Categories & Tags", "button *");
      await page.waitFor('button[aria-label="Delete Food"]');
    },
  },
  {
    name: "settings-rules",
    run: async (page) => {
      await openSettings(page);
      await page.clickText("Auto-Categorize Rules", "button *");
      await page.waitFor('button[aria-label="Delete rule"]');
    },
  },
  {
    name: "settings-about",
    run: async (page) => {
      await openSettings(page);
      await page.clickText("About", "button *");
      await page.waitForText("Open-source licenses", "button *");
    },
  },

  // -------------------------------------------------------------- app lock ---
  // The opt-in lock (#79) after the person closed Android's unlock prompt.
  { name: "app-lock", fixture: { android: "granted", appLock: true }, ready: "#lock-title" },
  { name: "app-lock-dark", fixture: { ...DARK, android: "granted", appLock: true }, ready: "#lock-title" },
  { name: "app-lock-thai", fixture: { ...THAI, android: "granted", appLock: true }, ready: "#lock-title" },

  // ------------------------------------------------------------ onboarding ---
  {
    name: "onboarding-welcome",
    fixture: { data: "empty", onboarding: "pending" },
    ready: "#onb-title-0",
  },
  {
    name: "onboarding-private",
    fixture: { data: "empty", onboarding: "pending" },
    ready: "#onb-title-0",
    run: async (page) => {
      await page.click('button[aria-label="Go to step 2"]');
      await page.waitFor("#onb-title-1");
    },
  },
  {
    name: "onboarding-setup",
    fixture: { data: "empty", onboarding: "pending" },
    ready: "#onb-title-0",
    run: async (page) => {
      await page.click('button[aria-label="Go to step 4"]');
      await page.waitFor("#onb-title-3");
    },
  },
  {
    // The last step says the first scan starts on "Let's go" (#47).
    name: "onboarding-done-scan",
    fixture: FIRST_RUN_ANDROID,
    ready: "#onb-title-0",
    run: async (page) => {
      await page.click('button[aria-label="Go to step 6"]');
      await page.waitFor("#onb-title-5");
    },
  },
  {
    // Finishing the first tour starts the scan by itself (#47).
    name: "onboarding-first-scan",
    fixture: FIRST_RUN_ANDROID,
    ready: "#onb-title-0",
    run: async (page) => {
      await page.click('button[aria-label="Go to step 6"]');
      await page.waitFor("#onb-title-5");
      await page.clickText("Let's go");
      await page.waitForText("Scanning new slips...", "*");
      await page.evaluate(`(() => {
        const scans = window.__moojotScans;
        if (scans.length !== 1 || scans[0].sinceSec !== ${FIRST_SCAN_SINCE}) {
          throw new Error("expected one scan since ${FIRST_SCAN_SINCE}, got " + JSON.stringify(scans));
        }
      })()`);
    },
  },
];
