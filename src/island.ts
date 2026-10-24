// La pastille : une toute petite barre (blip et temps du jour), toujours
// visible, qui ne change pas au survol. Horizontale en haut et en bas de
// l'écran, verticale à gauche et à droite. Un clic l'ouvre : temps du jour,
// état, pause, mise à jour éventuelle, et un bouton pour ouvrir
// l'application. Un autre clic, Échap ou un clic ailleurs la referme.
// Cliquer, maintenir et glisser la barre la déplace ; lâchée, elle se colle
// au bord de l'écran le plus proche.
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, dureeCourte, texteMiseAJour, type IslandLayout, type Status } from "./api";
import { h } from "./dom";
import { creerMascotte } from "./mascot";

const root = document.getElementById("island")!;
const mascotte = creerMascotte({ simple: true, titre: "Blip SKELVIA" });

// La barre.
const mini = h("span", { class: "mini" }, "");
const pastilleMaj = h("span", { class: "dot", hidden: true, "aria-hidden": "true" });
const barre = h(
  "div",
  {
    class: "bar",
    role: "button",
    tabindex: "0",
    "aria-expanded": "false",
    title: "Cliquez pour ouvrir. Cliquez, maintenez et glissez pour déplacer.",
  },
  mascotte.svg,
  mini,
  pastilleMaj,
);

// Le panneau, visible une fois la barre ouverte.
const total = h("div", { class: "total" }, "");
const etat = h("span", { class: "etat" }, "");
const explication = h("p", {}, "");
const titreMaj = h("b", {}, "");
const texteMaj = h("p", {}, "");
const boutonMaj = h("button", { type: "button", class: "primary" }, "Mettre à jour");
const blocMaj = h("div", { class: "update", hidden: true }, titreMaj, texteMaj, h("div", { class: "actions" }, boutonMaj));
const iconePause = '<svg viewBox="0 0 12 12" aria-hidden="true"><rect x="2" y="1.5" width="2.6" height="9" rx="1" fill="currentColor"/><rect x="7.4" y="1.5" width="2.6" height="9" rx="1" fill="currentColor"/></svg>';
const iconeReprendre = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3 1.8v8.4c0 .5.5.8.9.5l6.4-4.2a.6.6 0 0 0 0-1L3.9 1.3c-.4-.3-.9 0-.9.5z" fill="currentColor"/></svg>';
const boutonPause = h("button", { type: "button" });
const boutonOuvrir = h("button", { type: "button", class: "primary" }, "Ouvrir l'application");
const panel = h(
  "div",
  { class: "panel" },
  h("div", { class: "head" }, total, etat),
  explication,
  blocMaj,
  h("div", { class: "actions" }, boutonPause, boutonOuvrir),
);

root.append(barre, panel);

let status: Status | null = null;
let vertical = false;

function texteBarre(s: Status): string {
  return s.paused ? "Pause" : dureeCourte(s.todaySecs);
}

function afficher(s: Status) {
  status = s;
  mascotte.setVeille(s.paused || s.idle);
  mini.textContent = texteBarre(s);
  total.textContent = `${dureeCourte(s.todaySecs)} aujourd'hui`;
  etat.textContent = s.paused ? "En pause" : s.idle ? "En veille" : "Enregistre";
  etat.className = `etat${s.paused || s.idle ? " off" : ""}`;
  explication.textContent = s.paused
    ? "Rien n'est enregistré jusqu'à ce que vous repreniez."
    : s.idle
      ? "Souris et clavier immobiles : ce temps n'est pas compté."
      : "Temps compté quand la souris ou le clavier servent.";
  boutonPause.innerHTML = s.paused ? `${iconeReprendre}Reprendre` : `${iconePause}Pause`;
  boutonPause.className = s.paused ? "primary" : "";
  boutonOuvrir.className = s.paused ? "" : "primary";
  afficherMiseAJour(s);
}

function afficherMiseAJour(s: Status) {
  const u = s.update;
  pastilleMaj.hidden = u === null;
  blocMaj.hidden = u === null;
  explication.hidden = u !== null;
  barre.title =
    u === null
      ? "Cliquez pour ouvrir. Cliquez, maintenez et glissez pour déplacer."
      : `Nouvelle version ${u.version} disponible : cliquez pour l'installer.`;
  if (u === null) return;
  const enCours = u.step === "telechargement" || u.step === "installation";
  titreMaj.textContent = u.step === "erreur" ? "Mise à jour non installée" : `Nouvelle version ${u.version}`;
  texteMaj.textContent =
    u.step === "disponible" ? "SKELVIA Pulse se ferme quelques secondes, puis se relance. Vos mesures sont gardées." : texteMiseAJour(u);
  boutonMaj.textContent = u.step === "erreur" ? "Réessayer" : enCours ? "En cours…" : "Mettre à jour";
  boutonMaj.toggleAttribute("disabled", enCours);
}

// Forme donnée par l'application : bord, orientation, place de la barre.
function appliquer(l: IslandLayout | null) {
  if (!l) return;
  vertical = l.edge === "gauche" || l.edge === "droite";
  root.dataset.edge = l.edge;
  root.classList.toggle("vertical", vertical);
  root.style.setProperty("--bar-left", `${l.barLeft}px`);
  root.style.setProperty("--bar-top", `${l.barTop}px`);
  // Barre verticale : panneau centré sur elle, sans sortir de la fenêtre.
  const milieu = l.barTop + 4 + 34;
  const haut = Math.min(Math.max(milieu - panel.offsetHeight / 2, 4), window.innerHeight - panel.offsetHeight - 4);
  root.style.setProperty("--panel-top", `${Math.max(haut, 4)}px`);
}

// Ouvrir et refermer.
let ouverte = false;
let retrecir: number | undefined;
let quitter: number | undefined;

async function ouvrir() {
  if (ouverte) return;
  ouverte = true;
  window.clearTimeout(retrecir);
  barre.setAttribute("aria-expanded", "true");
  const l = await api.islandExpand(true);
  if (!ouverte) return;
  appliquer(l);
  root.classList.add("expanded");
}

function fermer() {
  if (!ouverte) return;
  ouverte = false;
  window.clearTimeout(quitter);
  barre.setAttribute("aria-expanded", "false");
  root.classList.remove("expanded");
  // On attend la fin de l'animation avant de rétrécir la fenêtre.
  retrecir = window.setTimeout(() => api.islandExpand(false).then(appliquer), 220);
}

// Un clic sur la barre l'ouvre ou la referme. Cliquer, maintenir et glisser
// la déplace : le glissement commence quand la souris a bougé de quelques
// pixels. Lâchée, elle se colle au bord le plus proche, et sa place est
// gardée, même après un redémarrage.
const SEUIL_GLISSEMENT = 4;
let appui: { x: number; y: number } | null = null;

barre.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  e.preventDefault();
  appui = { x: e.screenX, y: e.screenY };
});
window.addEventListener("mousemove", (e) => {
  if (!appui) return;
  if ((e.buttons & 1) === 0) {
    appui = null;
    return;
  }
  if (Math.abs(e.screenX - appui.x) + Math.abs(e.screenY - appui.y) >= SEUIL_GLISSEMENT) {
    appui = null;
    glisser();
  }
});
window.addEventListener("mouseup", (e) => {
  if (!appui || e.button !== 0) return;
  appui = null;
  if (ouverte) fermer();
  else ouvrir();
});
barre.addEventListener("keydown", (e) => {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    if (ouverte) fermer();
    else ouvrir();
  }
});

// Ouverte, elle se referme d'abord : on ne déplace que la petite barre.
// Une fois lâchée, l'application la colle au bord de l'écran le plus proche.
async function glisser() {
  if (ouverte) {
    ouverte = false;
    window.clearTimeout(retrecir);
    window.clearTimeout(quitter);
    barre.setAttribute("aria-expanded", "false");
    root.classList.remove("expanded");
    appliquer(await api.islandExpand(false));
  }
  getCurrentWindow().startDragging();
}

// Elle se referme d'elle-même : Échap, clic dans une autre fenêtre, ou
// souris partie depuis quelques secondes.
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") fermer();
});
window.addEventListener("blur", fermer);
document.body.addEventListener("mouseleave", () => {
  window.clearTimeout(quitter);
  if (ouverte) quitter = window.setTimeout(fermer, 3000);
});
document.body.addEventListener("mouseenter", () => window.clearTimeout(quitter));

boutonPause.addEventListener("click", () => {
  if (status) api.setPaused(!status.paused);
});
boutonOuvrir.addEventListener("click", () => {
  fermer();
  api.showMain();
});
boutonMaj.addEventListener("click", () => {
  // Les erreurs arrivent aussi par l'état : la pastille les affiche.
  api.installUpdate().catch(() => {});
});

api.onIslandLayout(appliquer);
api.islandLayout().then(appliquer);
api.status().then(afficher);
api.onStatus(afficher);
