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
  island_position: [number, number] | null;
  check_updates: boolean;
}

export interface Status {
  paused: boolean;
  idle: boolean;
  todaySecs: number;
  serverOk: boolean;
  serverMessage: string | null;
  firstRunDone: boolean;
  update: UpdateStatus | null;
}

/** Nouvelle version trouvée, et où en est son installation. */
export interface UpdateStatus {
  version: string;
  notes: string | null;
  step: "disponible" | "telechargement" | "installation" | "erreur";
  /** De 0 à 1, si la taille est connue. */
  progress: number | null;
  error: string | null;
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
  /** Renvoie vrai si la pastille s'ouvre vers le haut (posée en bas de l'écran). */
  islandExpand: (expanded: boolean) => invoke<boolean>("island_expand", { expanded }),
  resetIsland: () => invoke<void>("reset_island"),
  version: () => invoke<string>("app_version"),
  /** Renvoie le message à afficher (« Vous avez la dernière version »…). */
  checkUpdates: () => invoke<string>("check_updates_now"),
  /** Sous Windows, l'application se ferme puis se relance toute seule. */
  installUpdate: () => invoke<void>("install_update"),
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

/** Texte court de l'état d'une mise à jour, commun à la pastille et à la fenêtre. */
export function texteMiseAJour(u: UpdateStatus): string {
  switch (u.step) {
    case "telechargement":
      return u.progress === null ? "Téléchargement de la mise à jour…" : `Téléchargement de la mise à jour… ${Math.round(u.progress * 100)} %`;
    case "installation":
      return "Installation : SKELVIA Pulse se ferme puis se relance.";
    case "erreur":
      return u.error ?? "La mise à jour n'a pas pu être installée.";
    default:
      return `La version ${u.version} est prête à être installée.`;
  }
}
