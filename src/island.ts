// La pastille en haut de l'écran : mascotte, état, bouton pause. Toujours
// visible. Au survol, elle s'agrandit et montre le temps compté aujourd'hui.
import { api, dureeCourte, type Status } from "./api";
import { h } from "./dom";
import { creerMascotte } from "./mascot";

const root = document.getElementById("island")!;
const mascotte = creerMascotte({ simple: true, titre: "Mascotte SKELVIA" });

const etat = h("b", {}, "Enregistre");
const sousEtat = h("span", {}, "");
const iconePause = '<svg viewBox="0 0 12 12" aria-hidden="true"><rect x="2" y="1.5" width="2.6" height="9" rx="1" fill="currentColor"/><rect x="7.4" y="1.5" width="2.6" height="9" rx="1" fill="currentColor"/></svg>';
const iconeReprendre = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3 1.8v8.4c0 .5.5.8.9.5l6.4-4.2a.6.6 0 0 0 0-1L3.9 1.3c-.4-.3-.9 0-.9.5z" fill="currentColor"/></svg>';
const bouton = h("button", { type: "button" });

const total = h("div", { class: "total" }, "0 min");
const explication = h("p", {}, "");
const panel = h(
  "div",
  { class: "panel" },
  total,
  explication,
  h("div", { class: "actions" }, h("button", { type: "button", class: "primary", onclick: () => api.showMain() }, "Ouvrir le détail")),
);

root.append(h("div", { class: "pill" }, mascotte.svg, h("div", { class: "state" }, etat, sousEtat), bouton), panel);

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
}

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
    api.islandExpand(true);
    root.classList.add("expanded");
  }
});
document.body.addEventListener("mouseleave", () => {
  window.clearTimeout(fermeture);
  fermeture = window.setTimeout(() => {
    root.classList.remove("expanded");
    retrecir = window.setTimeout(() => api.islandExpand(false), 300);
  }, 350);
});

api.status().then(afficher);
api.onStatus(afficher);
