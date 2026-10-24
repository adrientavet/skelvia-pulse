// Écran « Aujourd'hui » : temps par type de tâche, par application, par
// site, et jour par jour sur 7 ou 30 jours.
import { api, duree, type Range, type Status, type Summary } from "../api";
import { bars, h, pct } from "../dom";

const PERIODES: { id: Range; label: string }[] = [
  { id: "today", label: "Aujourd'hui" },
  { id: "week", label: "7 jours" },
  { id: "month", label: "30 jours" },
];

let periode: Range = "today";

function carteExtension(): HTMLElement {
  return h(
    "section",
    { class: "card notice" },
    h("h3", {}, "Voir aussi les sites web consultés"),
    h(
      "p",
      { class: "muted" },
      "Ajoutez à votre navigateur l'extension gratuite « ActivityWatch Web Watcher » : c'est l'extension officielle d'ActivityWatch, le moteur libre utilisé par SKELVIA Pulse. Elle n'envoie ses données qu'à ce PC. Sans elle, seul le titre de la fenêtre du navigateur est connu.",
    ),
    h(
      "div",
      { class: "actions" },
      h("button", { type: "button", onclick: () => api.openExtensionPage("chrome") }, "Pour Chrome"),
      h("button", { type: "button", onclick: () => api.openExtensionPage("edge") }, "Pour Edge"),
      h("button", { type: "button", onclick: () => api.openExtensionPage("firefox") }, "Pour Firefox"),
    ),
    h(
      "p",
      { class: "faint", style: "margin-top:10px" },
      "La page s'ouvre dans votre navigateur par défaut : cliquez sur « Ajouter ». Sous Edge, acceptez les extensions d'autres boutiques si Edge le demande. Sous Firefox, l'extension vous demande votre accord une fois.",
    ),
  );
}

function rendu(root: HTMLElement, s: Summary, label: string, status: Status) {
  const total = s.active_secs;
  const blocs: HTMLElement[] = [];

  if (status.paused) {
    blocs.push(h("section", { class: "card notice" }, h("h3", {}, "Enregistrement en pause"), h("p", { class: "muted" }, "Rien n'est enregistré tant que vous n'avez pas repris l'enregistrement.")));
  }
  if (status.serverMessage) {
    blocs.push(h("section", { class: "card notice" }, h("h3", {}, "Extension navigateur indisponible"), h("p", { class: "muted" }, status.serverMessage)));
  }

  const jours = s.days.filter((d) => d.secs >= 60).length;
  blocs.push(
    h(
      "div",
      { class: "hero" },
      h("div", { class: "card" }, h("div", { class: "k" }, "Temps actif sur l'ordinateur"), h("div", { class: "v" }, total < 30 ? "0 min" : duree(total))),
      periode !== "today" && jours > 0
        ? h("div", { class: "card" }, h("div", { class: "k" }, "Moyenne par jour avec activité"), h("div", { class: "v" }, duree(total / jours)))
        : null,
    ) as HTMLElement,
  );

  if (total < 60) {
    blocs.push(
      h(
        "section",
        { class: "card" },
        h("h2", {}, "Pas encore de chiffres"),
        h("p", { class: "muted" }, "Travaillez normalement : le temps apparaîtra ici au fil de la journée. Seul le temps où la souris ou le clavier servent est compté."),
      ),
    );
  } else {
    const maxCat = s.by_category[0]?.secs ?? 1;
    blocs.push(
      h(
        "section",
        { class: "card" },
        h("h2", {}, "Par type de tâche"),
        bars(
          s.by_category.map((c) => ({
            label: c.label,
            value: duree(c.secs),
            ratio: c.secs / maxCat,
            meta: `${pct(c.secs, total)}${c.apps.length ? " · " + c.apps.map((a) => a.label).join(", ") : ""}`,
          })),
        ),
        h("p", { class: "faint", style: "margin-top:14px" }, "Les types de tâche viennent de règles simples que vous pouvez modifier dans Réglages."),
      ),
    );

    const maxApp = s.by_app[0]?.secs ?? 1;
    const colonnes: HTMLElement[] = [
      h(
        "section",
        { class: "card" },
        h("h2", {}, "Par application"),
        bars(s.by_app.slice(0, 10).map((a) => ({ label: a.label, value: duree(a.secs), ratio: a.secs / maxApp, meta: pct(a.secs, total) }))),
      ),
    ];
    if (s.by_site.length) {
      const maxSite = s.by_site[0].secs;
      colonnes.push(
        h(
          "section",
          { class: "card" },
          h("h2", {}, "Sites web"),
          bars(s.by_site.slice(0, 10).map((a) => ({ label: a.label, value: duree(a.secs), ratio: a.secs / maxSite, meta: pct(a.secs, total) }))),
        ),
      );
    }
    if (s.top_items.length) {
      colonnes.push(
        h(
          "section",
          { class: "card" },
          h("h2", {}, "Documents et pages"),
          h("ul", { class: "list" }, ...s.top_items.slice(0, 10).map((i) => h("li", {}, h("span", {}, i.label), h("span", {}, duree(i.secs))))),
        ),
      );
    }
    blocs.push(h("div", { class: "grid2" }, ...colonnes));

    if (periode !== "today" && s.days.length > 1) {
      const max = Math.max(...s.days.map((d) => d.secs), 1);
      blocs.push(
        h(
          "section",
          { class: "card" },
          h("h2", {}, "Jour par jour"),
          h(
            "div",
            { class: "days" },
            ...s.days.map((d) => {
              const date = new Date(`${d.date}T12:00:00`);
              const court = date.toLocaleDateString("fr-FR", { weekday: "short", day: "numeric" });
              const barre = h("i", {});
              barre.style.height = `${(d.secs / max) * 100}%`;
              return h("div", { class: "day", title: `${date.toLocaleDateString("fr-FR", { dateStyle: "full" })} : ${duree(d.secs)}` }, barre, h("span", {}, court));
            }),
          ),
        ),
      );
    }
  }

  if (!s.has_web_data && status.serverOk) blocs.push(carteExtension());

  const choix = h(
    "div",
    { class: "segmented", role: "group", "aria-label": "Période" },
    ...PERIODES.map((p) =>
      h(
        "button",
        {
          type: "button",
          "aria-pressed": p.id === periode ? "true" : "false",
          onclick: () => {
            periode = p.id;
            charger(root, status);
          },
        },
        p.label,
      ),
    ),
  );

  root.replaceChildren(
    h("div", { class: "page-head" }, h("div", {}, h("h1", {}, "Où part votre temps"), h("p", { class: "muted" }, label)), choix),
    ...blocs,
  );
}

async function charger(root: HTMLElement, status: Status) {
  const { summary, periodLabel } = await api.summary(periode);
  rendu(root, summary, periodLabel, status);
}

/** Affiche l'écran et le met à jour chaque minute. Renvoie la fonction d'arrêt. */
export function vueAujourdhui(root: HTMLElement, status: Status): () => void {
  let courant = status;
  charger(root, courant);
  const minuterie = window.setInterval(() => charger(root, courant), 60_000);
  let arret: (() => void) | null = null;
  api.onStatus((s) => {
    const change = s.paused !== courant.paused;
    courant = s;
    if (change) charger(root, courant);
  }).then((u) => (arret = u));
  return () => {
    window.clearInterval(minuterie);
    arret?.();
  };
}
