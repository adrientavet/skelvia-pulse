// La pastille en haut de l'écran : mascotte, état, bouton pause. Toujours
// visible. Au survol, elle s'agrandit et montre le temps compté aujourd'hui.
// Quand une nouvelle version existe, elle l'indique : un clic l'installe.
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, dureeCourte, texteMiseAJour, type Status } from "./api";
import { h } from "./dom";
import { creerMascotte } from "./mascot";

const root = document.getElementById("island")!;
const mascotte = creerMascotte({ simple: true, titre: "Mascotte SKELVIA" });

const etat = h("b", {}, "Enregistre");
const sousEtat = h("span", {}, "");
// Remplace la ligne d'état quand une mise à jour est prête : un clic l'installe.
const lienMaj = h("button", { type: "button", class: "maj", hidden: true, title: "Installer la nouvelle version" }, "mise à jour");
const iconePause = '<svg viewBox="0 0 12 12" aria-hidden="true"><rect x="2" y="1.5" width="2.6" height="9" rx="1" fill="currentColor"/><rect x="7.4" y="1.5" width="2.6" height="9" rx="1" fill="currentColor"/></svg>';
const iconeReprendre = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3 1.8v8.4c0 .5.5.8.9.5l6.4-4.2a.6.6 0 0 0 0-1L3.9 1.3c-.4-.3-.9 0-.9.5z" fill="currentColor"/></svg>';
const bouton = h("button", { type: "button" });

const total = h("div", { class: "total" }, "0 min");
const explication = h("p", {}, "");
const astuce = h("p", { class: "hint" }, "Glissez la pastille pour la placer où vous voulez.");

const titreMaj = h("b", {}, "");
const texteMaj = h("p", {}, "");
const boutonMaj = h("button", { type: "button", class: "primary" }, "Mettre à jour");
const blocMaj = h("div", { class: "update", hidden: true }, titreMaj, texteMaj);
const boutonDetail = h("button", { type: "button", onclick: () => api.showMain() }, "Ouvrir le détail");

const panel = h(
  "div",
  { class: "panel" },
  total,
  explication,
  blocMaj,
  h("div", { class: "actions" }, boutonMaj, boutonDetail),
  astuce,
);

const pilule = h("div", { class: "pill", title: "Glissez pour déplacer la pastille" }, mascotte.svg, h("div", { class: "state" }, etat, sousEtat, lienMaj), bouton);
root.append(pilule, panel);

// Glisser la pilule (ailleurs que sur un bouton) déplace la pastille. La
// position est gardée par l'application, même après un redémarrage.
pilule.addEventListener("mousedown", (e) => {
  if (e.button !== 0 || (e.target as HTMLElement).closest("button")) return;
  e.preventDefault();
  getCurrentWindow().startDragging();
});

let status: Status | null = null;

function afficher(s: Status) {
  status = s;
  mascotte.setVeille(s.paused || s.idle);
  if (s.paused) {
    etat.textContent = "En pause";
    sousEtat.textContent = "aucune mesure";
    bouton.innerHTML = `${iconeReprendre}Reprendre`;
    bouton.setAttribute("aria-label", "Reprendre l'enregistrement");
    bouton.className = "primary";
  } else {
    etat.textContent = "Enregistre";
    sousEtat.textContent = s.idle ? "en veille" : `aujourd'hui ${dureeCourte(s.todaySecs)}`;
    bouton.innerHTML = `${iconePause}Pause`;
    bouton.setAttribute("aria-label", "Mettre l'enregistrement en pause");
    bouton.className = "";
  }
  total.textContent = `${dureeCourte(s.todaySecs)} aujourd'hui`;
  explication.textContent = s.paused
    ? "L'enregistrement est arrêté jusqu'à ce que vous le repreniez."
    : s.idle
      ? "Souris et clavier immobiles : ce temps n'est pas compté."
      : "Temps compté quand la souris ou le clavier servent.";
  afficherMiseAJour(s);
}

function afficherMiseAJour(s: Status) {
  const u = s.update;
  const enCours = u !== null && (u.step === "telechargement" || u.step === "installation");
  root.classList.toggle("has-update", u !== null);
  blocMaj.hidden = u === null;
  explication.hidden = u !== null;
  astuce.hidden = u !== null;
  boutonMaj.hidden = u === null;
  boutonDetail.className = u === null ? "primary" : "";
  sousEtat.hidden = u !== null;
  lienMaj.hidden = u === null;
  if (u === null) return;
  titreMaj.textContent = u.step === "erreur" ? "Mise à jour non installée" : `Nouvelle version ${u.version}`;
  texteMaj.textContent =
    u.step === "disponible"
      ? "SKELVIA Pulse se ferme quelques secondes, puis se relance. Vos mesures sont gardées."
      : texteMiseAJour(u);
  lienMaj.textContent =
    u.step === "telechargement" && u.progress !== null ? `mise à jour ${Math.round(u.progress * 100)} %` : u.step === "installation" ? "installation…" : "mise à jour";
  boutonMaj.textContent = u.step === "erreur" ? "Réessayer" : enCours ? "En cours…" : "Mettre à jour";
  boutonMaj.toggleAttribute("disabled", enCours);
  lienMaj.toggleAttribute("disabled", enCours);
}

function installer() {
  // Les erreurs arrivent aussi par l'état : la pastille les affiche.
  api.installUpdate().catch(() => {});
}
boutonMaj.addEventListener("click", installer);
lienMaj.addEventListener("click", installer);

bouton.addEventListener("click", () => {
  if (status) api.setPaused(!status.paused);
});

// Survol : on agrandit la fenêtre, puis on la rétrécit un peu après la sortie.
let fermeture: number | undefined;
let retrecir: number | undefined;
document.body.addEventListener("mouseenter", () => {
  window.clearTimeout(fermeture);
  window.clearTimeout(retrecir);
  if (!root.classList.contains("expanded")) {
    root.classList.add("expanded");
    // Posée en bas de l'écran, elle s'ouvre vers le haut.
    api.islandExpand(true).then((versLeHaut) => root.classList.toggle("up", versLeHaut));
  }
});
document.body.addEventListener("mouseleave", () => {
  window.clearTimeout(fermeture);
  fermeture = window.setTimeout(() => {
    root.classList.remove("expanded");
    retrecir = window.setTimeout(() => {
      api.islandExpand(false);
      root.classList.remove("up");
    }, 300);
  }, 350);
});

api.status().then(afficher);
api.onStatus(afficher);
