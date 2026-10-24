// Écran du premier lancement : ce qui est enregistré, ce qui ne l'est
// jamais, et comment garder la main. Rien n'est enregistré avant l'accord.
import { api } from "../api";
import { h } from "../dom";
import { creerMascotte } from "../mascot";

const li = (t: string) => h("li", {}, t);

export function vuePremierLancement(root: HTMLElement, ensuite: () => void) {
  const mascotte = creerMascotte({ titre: "Mascotte SKELVIA" });
  const autostart = h("input", { type: "checkbox", id: "autostart", checked: true });
  const commencer = h("button", { type: "button", class: "primary" }, "J'ai compris, commencer");
  commencer.addEventListener("click", async () => {
    commencer.disabled = true;
    await api.completeFirstRun(autostart.checked);
    ensuite();
  });

  root.replaceChildren(
    h(
      "div",
      { class: "welcome" },
      h("div", { class: "head" }, mascotte.svg, h("h1", {}, "Bienvenue dans SKELVIA Pulse")),
      h(
        "p",
        { class: "lead" },
        "SKELVIA Pulse vous montre où part votre temps sur ce PC : quelles applications, quels sites, quels types de tâches. Tout reste sur ce PC. Rien n'est envoyé, ni à SKELVIA, ni à personne.",
      ),
      h(
        "div",
        { class: "cols" },
        h(
          "section",
          { class: "card" },
          h("h3", {}, "Ce qui est enregistré"),
          h(
            "ul",
            {},
            li("Le nom de l'application au premier plan (Excel, Outlook…)."),
            li("Le titre de sa fenêtre, par exemple le nom du fichier ouvert. Il peut contenir le sujet d'un mail ou le nom d'un client."),
            li("Le site consulté (adresse et titre de la page), seulement si vous ajoutez l'extension au navigateur."),
            li("La durée, et si la souris ou le clavier ont servi : oui ou non."),
          ),
        ),
        h(
          "section",
          { class: "card" },
          h("h3", {}, "Ce qui n'est jamais enregistré"),
          h(
            "ul",
            {},
            li("Aucune capture d'écran."),
            li("Aucune touche tapée, aucun texte saisi, aucun clic."),
            li("Aucun contenu de vos mails, documents ou pages web."),
            li("Les fenêtres ouvertes en arrière-plan."),
            li("Rien n'est envoyé sur internet : pas de compte, pas de serveur SKELVIA."),
          ),
        ),
        h(
          "section",
          { class: "card" },
          h("h3", {}, "Vous gardez la main"),
          h(
            "ul",
            {},
            li("La pastille en haut de l'écran montre toujours l'état, avec un bouton Pause."),
            li("Dans Réglages, excluez une application ou un site : rien ne sera enregistré, et ce qui l'avait été est effacé."),
            li("Effacez la dernière heure, la journée ou tout, à tout moment."),
            li("Pour tout supprimer : désinstallez SKELVIA Pulse, puis supprimez le dossier « %LOCALAPPDATA%\\SKELVIA Pulse »."),
          ),
        ),
      ),
      h(
        "p",
        { class: "faint", style: "margin-top:20px;max-width:720px" },
        "Vous installez SKELVIA Pulse sur le PC d'un salarié ? Prévenez-le avant, expliquez-lui ce qui est mesuré et pourquoi, et faites valider cet usage (RGPD, CNIL). Le guide d'installation en donne les points principaux ; ce n'est pas un avis juridique.",
      ),
      h(
        "div",
        { class: "go" },
        commencer,
        h("label", { class: "check", for: "autostart" }, autostart, h("span", {}, "Démarrer SKELVIA Pulse avec Windows")),
      ),
    ),
  );
}
