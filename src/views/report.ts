// Écran « Rapport » : un fichier HTML d'une page, lisible sans internet,
// enregistré dans le dossier Téléchargements.
import { api, type Range } from "../api";
import { h } from "../dom";

export function vueRapport(root: HTMLElement) {
  const periode = h(
    "select",
    { id: "periode" },
    h("option", { value: "week", selected: true }, "Les 7 derniers jours"),
    h("option", { value: "month" }, "Les 30 derniers jours"),
    h("option", { value: "today" }, "Aujourd'hui"),
  );
  const sites = h("input", { type: "checkbox", id: "sites", checked: true });
  const documents = h("input", { type: "checkbox", id: "documents" });
  const audit = h("input", { type: "checkbox", id: "audit", checked: true });
  const resultat = h("p", { class: "saved", role: "status", style: "margin-top:12px" });

  const bouton = h("button", { type: "button", class: "primary" }, "Télécharger mon rapport");
  bouton.addEventListener("click", async () => {
    bouton.disabled = true;
    resultat.textContent = "Création du rapport…";
    try {
      const chemin = await api.saveReport(periode.value as Range, documents.checked, sites.checked, audit.checked);
      resultat.textContent = `Rapport enregistré : ${chemin}. Il s'ouvre dans votre navigateur.`;
    } catch (e) {
      resultat.textContent = `Le rapport n'a pas pu être créé : ${e}`;
    } finally {
      bouton.disabled = false;
    }
  });

  const option = (input: HTMLInputElement, titre: string, aide: string) =>
    h("label", { class: "check field", for: input.id }, input, h("span", {}, titre, h("br"), h("span", { class: "faint" }, aide)));

  root.replaceChildren(
    h("div", { class: "page-head" }, h("div", {}, h("h1", {}, "Mon rapport"), h("p", { class: "muted" }, "Une page claire, à garder, imprimer ou transmettre. Vous décidez de ce que vous en faites."))),
    h(
      "section",
      { class: "card" },
      h("div", { class: "field" }, h("label", { for: "periode" }, h("b", {}, "Période")), periode),
      option(sites, "Ajouter la liste des sites web consultés", "Seulement le nom du site (par exemple boamp.fr), pas les pages."),
      option(documents, "Ajouter le nom des documents et des pages les plus longs", "Utile pour vous, mais ces noms peuvent contenir des noms de clients ou des sujets de mails. Désactivé par défaut."),
      option(audit, "Ajouter en bas du rapport le lien vers l'audit gratuit de SKELVIA", "Une ligne discrète pour réserver, si vous le souhaitez, 45 minutes avec Adrien pour analyser ces chiffres."),
      h("div", { class: "actions", style: "margin-top:18px" }, bouton),
      resultat,
    ),
    h(
      "section",
      { class: "card" },
      h("h3", {}, "Ce que contient le rapport"),
      h(
        "p",
        { class: "muted" },
        "Le temps actif, le temps par type de tâche (avec une moyenne par semaine sur 7 ou 30 jours), par application, et la méthode de calcul. Le fichier est créé sur ce PC, dans le dossier Téléchargements. SKELVIA Pulse ne l'envoie nulle part.",
      ),
    ),
  );
}
