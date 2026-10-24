// Fenêtre principale : premier lancement, puis Aujourd'hui, Rapport, Réglages.
import { api, texteMiseAJour, type Status } from "./api";
import { h } from "./dom";
import { creerMascotte } from "./mascot";
import { vueAujourdhui } from "./views/today";
import { vuePremierLancement } from "./views/firstrun";
import { vueRapport } from "./views/report";
import { vueReglages } from "./views/settings";

const app = document.getElementById("app")!;

type Page = "today" | "report" | "settings";
const PAGES: { id: Page; label: string }[] = [
  { id: "today", label: "Aujourd'hui" },
  { id: "report", label: "Rapport" },
  { id: "settings", label: "Réglages" },
];

let page: Page = "today";
let nettoyer: (() => void) | null = null;

function shell(status: Status) {
  // Moins de 48 px : version simplifiée, sinon les points font une tache.
  const mascotte = creerMascotte({ simple: true, titre: "Mascotte SKELVIA" });
  const content = h("main", { class: "content" });
  const nav = h("nav", { class: "nav", "aria-label": "Pages" });
  const etat = h("b", {});
  const detail = h("p", {});
  const bouton = h("button", { type: "button" });
  // Nouvelle version disponible : un clic l'installe.
  const titreMaj = h("b", {});
  const texteMaj = h("p", {});
  const boutonMaj = h("button", { type: "button", class: "primary", onclick: () => api.installUpdate().catch(() => {}) }, "Mettre à jour");
  const blocMaj = h("div", { class: "status update", role: "status", hidden: true }, titreMaj, texteMaj, boutonMaj);

  const afficherEtat = (s: Status) => {
    mascotte.setVeille(s.paused || s.idle);
    etat.textContent = s.paused ? "En pause" : s.idle ? "Enregistre (en veille)" : "Enregistre";
    detail.textContent = s.paused
      ? "Rien n'est enregistré en ce moment."
      : s.idle
        ? "Souris et clavier immobiles : ce temps n'est pas compté."
        : "Seule la fenêtre au premier plan est mesurée.";
    bouton.textContent = s.paused ? "Reprendre" : "Mettre en pause";
    bouton.className = s.paused ? "primary" : "";
    const u = s.update;
    blocMaj.hidden = u === null;
    if (u) {
      const enCours = u.step === "telechargement" || u.step === "installation";
      titreMaj.textContent = u.step === "erreur" ? "Mise à jour non installée" : `Nouvelle version ${u.version}`;
      texteMaj.textContent =
        u.step === "disponible" ? "SKELVIA Pulse se ferme quelques secondes, puis se relance. Vos mesures sont gardées." : texteMiseAJour(u);
      boutonMaj.textContent = u.step === "erreur" ? "Réessayer" : enCours ? "En cours…" : "Mettre à jour";
      boutonMaj.toggleAttribute("disabled", enCours);
    }
    current = s;
  };
  let current = status;
  bouton.addEventListener("click", () => api.setPaused(!current.paused));

  const ouvrir = (p: Page) => {
    page = p;
    nettoyer?.();
    nettoyer = null;
    for (const b of nav.querySelectorAll("button")) b.setAttribute("aria-current", b.dataset.page === p ? "page" : "false");
    content.replaceChildren();
    if (p === "today") nettoyer = vueAujourdhui(content, current);
    if (p === "report") vueRapport(content);
    if (p === "settings") vueReglages(content, current);
    content.scrollTop = 0;
  };
  for (const p of PAGES) {
    const b = h("button", { type: "button", onclick: () => ouvrir(p.id) }, p.label);
    b.dataset.page = p.id;
    nav.append(b);
  }

  app.replaceChildren(
    h(
      "div",
      { class: "shell" },
      h(
        "aside",
        { class: "side" },
        h("div", { class: "brand" }, mascotte.svg, h("b", {}, "SKELVIA Pulse", h("span", {}, "Tout reste sur ce PC"))),
        nav,
        h("div", { class: "status", role: "status" }, etat, detail, bouton),
        blocMaj,
      ),
      content,
    ),
  );
  afficherEtat(status);
  ouvrir(page);
  api.onStatus(afficherEtat);
}

async function demarrer() {
  const status = await api.status();
  if (!status.firstRunDone) {
    vuePremierLancement(app, async () => shell(await api.status()));
  } else {
    shell(status);
  }
}

demarrer();
