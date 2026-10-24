// Appels à la partie Rust de l'application. Tout se passe sur le PC.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Criterion = "app" | "site" | "titre";
export interface Rule { category: string; criterion: Criterion; value: string }
export interface Exclusion { criterion: Criterion; value: string }
export interface Settings {
  idle_timeout_secs: number;
  record_titles: boolean;
  exclusions: Exclusion[];
  rules: Rule[];
  first_run_done: boolean;
  paused: boolean;
}

export interface Status {
  paused: boolean;
  idle: boolean;
  todaySecs: number;
  serverOk: boolean;
  serverMessage: string | null;
  firstRunDone: boolean;
}

export interface Entry { label: string; secs: number }
export interface CategoryEntry extends Entry { apps: Entry[]; items: Entry[] }
export interface Summary {
  from: string;
  to: string;
  active_secs: number;
  by_category: CategoryEntry[];
  by_app: Entry[];
  by_site: Entry[];
  top_items: Entry[];
  days: { date: string; secs: number }[];
  has_web_data: boolean;
}
export type Range = "today" | "week" | "month";

export const api = {
  status: () => invoke<Status>("get_status"),
  setPaused: (paused: boolean) => invoke<void>("set_paused", { paused }),
  settings: () => invoke<Settings>("get_settings"),
  defaultRules: () => invoke<Rule[]>("default_rules"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  summary: (range: Range) => invoke<{ summary: Summary; periodLabel: string }>("get_summary", { range }),
  saveReport: (range: Range, includeItems: boolean, includeSites: boolean, includeAudit: boolean) =>
    invoke<string>("save_report", { range, includeItems, includeSites, includeAudit }),
  erase: (scope: "hour" | "today" | "all") => invoke<void>("erase", { scope }),
  completeFirstRun: (autostart: boolean) => invoke<void>("complete_first_run", { autostart }),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  autostart: () => invoke<boolean>("get_autostart"),
  openExtensionPage: (browser: "chrome" | "edge" | "firefox") => invoke<void>("open_extension_page", { browser }),
  openDataFolder: () => invoke<void>("open_data_folder"),
  openAuditPage: () => invoke<void>("open_audit_page"),
  showMain: () => invoke<void>("show_main_window"),
  islandExpand: (expanded: boolean) => invoke<void>("island_expand", { expanded }),
  onStatus: (cb: (s: Status) => void): Promise<UnlistenFn> => listen<Status>("pulse://status", (e) => cb(e.payload)),
};

/** « 6 h 05 », « 45 min », « moins d'une minute » (même règle que le rapport). */
export function duree(secs: number): string {
  if (secs < 30) return "moins d'une minute";
  const totalMin = Math.round(secs / 60);
  const h = Math.floor(totalMin / 60);
  const m = totalMin % 60;
  if (h === 0) return `${m} min`;
  if (m === 0) return `${h} h`;
  return `${h} h ${String(m).padStart(2, "0")}`;
}

export function dureeCourte(secs: number): string {
  return secs < 60 ? "0 min" : duree(secs);
}
