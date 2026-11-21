// Bilingual UI dictionary. `I18nKey` is derived from the English map so the
// Thai map is type-checked for completeness at compile time.

const en = {
  // App / navigation
  appName: "MooJot",
  home: "Home",
  summary: "Summary",
  compare: "Compare",
  budgets: "Budgets",
  myMoo: "My Moo",
  search: "Search",
  settings: "Settings",

  // Records
  filterRecords: "Filter Records",
  addRecord: "Add Record",
  editRecord: "Edit Record",
  recordDetails: "Record Details",
  spending: "Spending",
  income: "Income",
  transfer: "Transfer",
  transfers: "Transfers",
  futureRecords: "Future Records",
  needsReview: "Needs Review",
  recurringRecord: "Recurring Record",
  recurring: "Recurring",
  allRecords: "All Records",
  noRecords: "No records yet",
  emptyHint: "Pull down to scan a slip, or tap + to add one.",
  noResults: "No results found",

  // Slips / OCR
  scanESlips: "Scan E-Slips",
  scan: "Scan",
  scanAgain: "Scan Again",
  slip: "Slip",
  slips: "Slips",
  ocrSettings: "OCR Settings",
  aksonOcr: "AksonOCR",
  mlKitLatin: "ML Kit Latin",
  englishOcrEngine: "English OCR Engine",
  aksonOcrApiKey: "AksonOCR API Key",
  aksonOcrModel: "AksonOCR Model",
  allowSharedAlbum: "Allow Shared Album",

  // Organisation
  categories: "Categories",
  tags: "Tags",
  accounts: "Accounts",
  calendarSettings: "Calendar Settings",
  monthStartDay: "Month Start Day",
  weekStartDay: "Week Start Day",
  defaultView: "Default View",
  language: "Language",
  ownerNames: "Owner Names",
  sharedAlbum: "Shared Album",

  // Data / export
  exportCsv: "Export CSV",
  exportJson: "Export JSON",
  export: "Export",
  import: "Import",
  backup: "Backup",
  restore: "Restore",

  // Fields
  amount: "Amount",
  date: "Date",
  time: "Time",
  account: "Account",
  category: "Category",
  tag: "Tag",
  payee: "Payee",
  note: "Note",
  currency: "Currency",
  source: "Source",
  kind: "Type",
  status: "Status",

  // Periods / ranges
  monthly: "Monthly",
  biweekly: "Biweekly",
  weekly: "Weekly",
  today: "Today",
  thisWeek: "This Week",
  thisMonth: "This Month",
  thisPeriod: "This Period",
  lastPeriod: "Last Period",
  startDate: "Start Date",
  endDate: "End Date",
  period: "Period",

  // Summary / compare / budgets
  totalSpending: "Total Spending",
  totalIncome: "Total Income",
  totalTransfer: "Total Transfers",
  remaining: "Remaining",
  byCategory: "By Category",
  byTag: "By Tag",
  average: "Average",
  current: "Current",
  versusAverage: "vs. Average",
  limit: "Limit",
  spent: "Spent",
  spentPct: "Spent %",
  daysRemaining: "Days Remaining",
  alertThreshold: "Alert Threshold",
  dailyPace: "Daily Pace",
  addBudget: "Add Budget",
  editBudget: "Edit Budget",

  // Streaks / stats
  syncStreak: "Sync Streak",
  categorizeStreak: "Categorize Streak",
  bestStreak: "Best Streak",
  days: "days",
  day: "day",
  jottedToday: "Jotted Today",

  // Source labels
  manual: "Manual",
  slipAuto: "Slip (Auto)",
  slipShare: "Slip (Share)",
  pdfStatement: "PDF Statement",
  importStatement: "Import card statement",
  statementPassword: "Statement password",
  statementPasswordHint: "Used once to open this PDF. It is not saved.",
  wrongPassword: "That password did not open the PDF.",
  recordDeleted: "Record deleted",
  undo: "Undo",

  // Status labels
  ok: "OK",
  needsReviewShort: "Review",
  failed: "Failed",
  reviewed: "Reviewed",
  unreviewed: "Unreviewed",

  // Actions
  save: "Save",
  cancel: "Cancel",
  delete: "Delete",
  edit: "Edit",
  close: "Close",
  confirm: "Confirm",
  add: "Add",
  remove: "Remove",
  select: "Select",
  selectAll: "Select All",
  clear: "Clear",
  apply: "Apply",
  reset: "Reset",
  retry: "Retry",
  done: "Done",
  back: "Back",
  next: "Next",
  refresh: "Refresh",
  share: "Share",
  copy: "Copy",

  // Dialogs / confirmations
  deleteConfirm: "Delete this record?",
  unsavedChanges: "You have unsaved changes.",
  noCategory: "Uncategorized",
  selectCategory: "Select a category",
  other: "Other",

  // Slip scan report
  scanned: "Scanned",
  imported: "Imported",
  duplicates: "Duplicates",
  skippedNonSlip: "Skipped (Not Slips)",
  aksonCalls: "AksonOCR Calls",
  scanComplete: "Scan Complete",
  noSlipsFound: "No new slips found",

  // Misc
  loading: "Loading...",
  error: "Something went wrong",
  empty: "Nothing here yet",
  all: "All",
  searchPlaceholder: "Search payees, notes, amounts...",
} as const;

export type I18nKey = keyof typeof en;

const th: Record<I18nKey, string> = {
  // App / navigation
  appName: "MooJot",
  home: "หน้าแรก",
  summary: "สรุป",
  compare: "เปรียบเทียบ",
  budgets: "งบประมาณ",
  myMoo: "มูของฉัน",
  search: "ค้นหา",
  settings: "ตั้งค่า",

  // Records
  filterRecords: "กรองรายการ",
  addRecord: "เพิ่มรายการ",
  editRecord: "แก้ไขรายการ",
  recordDetails: "รายละเอียดรายการ",
  spending: "รายจ่าย",
  income: "รายรับ",
  transfer: "โอนเงิน",
  transfers: "การโอนเงิน",
  futureRecords: "รายการในอนาคต",
  needsReview: "รอตรวจสอบ",
  recurringRecord: "รายการประจำ",
  recurring: "ประจำ",
  allRecords: "รายการทั้งหมด",
  noRecords: "ยังไม่มีรายการ",
  emptyHint: "ดึงลงเพื่อสแกนสลิป หรือกด + เพื่อจดรายการ",
  noResults: "ไม่พบผลลัพธ์",

  // Slips / OCR
  scanESlips: "สแกนสลิป",
  scan: "สแกน",
  scanAgain: "สแกนอีกครั้ง",
  slip: "สลิป",
  slips: "สลิป",
  ocrSettings: "ตั้งค่า OCR",
  aksonOcr: "AksonOCR",
  mlKitLatin: "ML Kit Latin",
  englishOcrEngine: "เครื่องมือ OCR ภาษาอังกฤษ",
  aksonOcrApiKey: "คีย์ API AksonOCR",
  aksonOcrModel: "โมเดล AksonOCR",
  allowSharedAlbum: "อนุญาตอัลบั้มแชร์",

  // Organisation
  categories: "หมวดหมู่",
  tags: "ป้ายกำกับ",
  accounts: "บัญชี",
  calendarSettings: "ตั้งค่าปฏิทิน",
  monthStartDay: "วันเริ่มเดือน",
  weekStartDay: "วันเริ่มสัปดาห์",
  defaultView: "มุมมองเริ่มต้น",
  language: "ภาษา",
  ownerNames: "ชื่อเจ้าของ",
  sharedAlbum: "อัลบั้มแชร์",

  // Data / export
  exportCsv: "ส่งออก CSV",
  exportJson: "ส่งออก JSON",
  export: "ส่งออก",
  import: "นำเข้า",
  backup: "สำรองข้อมูล",
  restore: "กู้คืนข้อมูล",

  // Fields
  amount: "จำนวนเงิน",
  date: "วันที่",
  time: "เวลา",
  account: "บัญชี",
  category: "หมวดหมู่",
  tag: "ป้ายกำกับ",
  payee: "ผู้รับเงิน",
  note: "หมายเหตุ",
  currency: "สกุลเงิน",
  source: "แหล่งที่มา",
  kind: "ประเภท",
  status: "สถานะ",

  // Periods / ranges
  monthly: "รายเดือน",
  biweekly: "รายครึ่งเดือน",
  weekly: "รายสัปดาห์",
  today: "วันนี้",
  thisWeek: "สัปดาห์นี้",
  thisMonth: "เดือนนี้",
  thisPeriod: "งวดนี้",
  lastPeriod: "งวดก่อน",
  startDate: "วันเริ่มต้น",
  endDate: "วันสิ้นสุด",
  period: "งวด",

  // Summary / compare / budgets
  totalSpending: "รายจ่ายรวม",
  totalIncome: "รายรับรวม",
  totalTransfer: "ยอดโอนรวม",
  remaining: "คงเหลือ",
  byCategory: "ตามหมวดหมู่",
  byTag: "ตามป้ายกำกับ",
  average: "เฉลี่ย",
  current: "ปัจจุบัน",
  versusAverage: "เทียบค่าเฉลี่ย",
  limit: "ขีดจำกัด",
  spent: "ใช้ไปแล้ว",
  spentPct: "ใช้ไป (%)",
  daysRemaining: "วันคงเหลือ",
  alertThreshold: "เกณฑ์แจ้งเตือน",
  dailyPace: "อัตรารายวัน",
  addBudget: "เพิ่มงบประมาณ",
  editBudget: "แก้ไขงบประมาณ",

  // Streaks / stats
  syncStreak: "ต่อเนื่องการซิงก์",
  categorizeStreak: "ต่อเนื่องการจัดหมวด",
  bestStreak: "สถิติต่อเนื่อง",
  days: "วัน",
  day: "วัน",
  jottedToday: "จดวันนี้",

  // Source labels
  manual: "บันทึกเอง",
  slipAuto: "สลิป (อัตโนมัติ)",
  slipShare: "สลิป (แชร์)",
  pdfStatement: "ใบแจ้งยอด PDF",
  importStatement: "นำเข้าใบแจ้งยอดบัตร",
  statementPassword: "รหัสใบแจ้งยอด",
  statementPasswordHint: "ใช้เปิด PDF ครั้งนี้เท่านั้น และไม่ถูกบันทึก",
  wrongPassword: "รหัสนี้เปิด PDF ไม่ได้",
  recordDeleted: "ลบรายการแล้ว",
  undo: "เลิกทำ",

  // Status labels
  ok: "ปกติ",
  needsReviewShort: "ตรวจสอบ",
  failed: "ล้มเหลว",
  reviewed: "ตรวจสอบแล้ว",
  unreviewed: "ยังไม่ตรวจสอบ",

  // Actions
  save: "บันทึก",
  cancel: "ยกเลิก",
  delete: "ลบ",
  edit: "แก้ไข",
  close: "ปิด",
  confirm: "ยืนยัน",
  add: "เพิ่ม",
  remove: "ลบออก",
  select: "เลือก",
  selectAll: "เลือกทั้งหมด",
  clear: "ล้าง",
  apply: "ใช้",
  reset: "รีเซ็ต",
  retry: "ลองใหม่",
  done: "เสร็จ",
  back: "ย้อนกลับ",
  next: "ถัดไป",
  refresh: "รีเฟรช",
  share: "แชร์",
  copy: "คัดลอก",

  // Dialogs / confirmations
  deleteConfirm: "ลบรายการนี้หรือไม่?",
  unsavedChanges: "มีการเปลี่ยนแปลงที่ยังไม่ได้บันทึก",
  noCategory: "ไม่ระบุหมวดหมู่",
  selectCategory: "เลือกหมวดหมู่",
  other: "อื่นๆ",

  // Slip scan report
  scanned: "สแกนแล้ว",
  imported: "นำเข้าแล้ว",
  duplicates: "รายการซ้ำ",
  skippedNonSlip: "ข้าม (ไม่ใช่สลิป)",
  aksonCalls: "การเรียก AksonOCR",
  scanComplete: "สแกนเสร็จแล้ว",
  noSlipsFound: "ไม่พบสลิปใหม่",

  // Misc
  loading: "กำลังโหลด...",
  error: "เกิดข้อผิดพลาด",
  empty: "ยังไม่มีอะไรที่นี่",
  all: "ทั้งหมด",
  searchPlaceholder: "ค้นหาผู้รับเงิน หมายเหตุ จำนวนเงิน...",
};

const dictionaries: Record<"en" | "th", Record<I18nKey, string>> = { en, th };

/** Look up a translated UI string. */
export function t(lang: "en" | "th", key: I18nKey): string {
  return dictionaries[lang][key];
}
