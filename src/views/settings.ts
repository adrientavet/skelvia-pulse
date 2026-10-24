// Écran « Réglages » : délai d'inactivité, titres, exclusions, règles de
// regroupement, extension navigateur, effacement des données.
import { api, type Criterion, type Exclusion, type Rule, type Settings, type Status } from "../api";
import { h } from "../dom";

const CRITERES: { id: Criterion; label: string }[] = [
  { id: "titre", label: "Mot dans le titre" },
  { id: "site", label: "Site web" },
  { id: "app", label: "Application" },
];
const nomCritere = (c: Criterion) => CRITERES.find((x) => x.id === c)?.label ?? c;

function selectCritere(valeur: Criterion): HTMLSelectElement {
  return h("select", {}, ...CRITERES.map((c) => h("option", { value: c.id, selected: c.id === valeur }, c.label)));
}

/** Bouton qui demande une seconde confirmation avant d'agir. */
function boutonConfirme(texte: string, confirmation: string, action: () => Promise<void>, classe = "danger"): HTMLButtonElement {
  const b = h("button", { type: "button", class: classe }, texte);
  let arme = false;
  let minuterie: number | undefined;
  b.addEventListener("click", async () => {
    if (!arme) {
      arme = true;
      b.textContent = confirmation;
      minuterie = window.setTimeout(() => {
        arme = false;
        b.textContent = texte;
      }, 5000);
      return;
    }
    window.clearTimeout(minuterie);
    b.disabled = true;
    await action();
    b.disabled = false;
    arme = false;
    b.textContent = texte;
  });
  return b;
}

export async function vueReglages(root: HTMLElement, status: Status) {
  let s: Settings = await api.settings();
  const autostartActif = await api.autostart().catch(() => false);
  const message = h("p", { class: "saved", role: "status" });
  const dire = (t: string) => {
    message.textContent = t;
    window.setTimeout(() => (message.textContent === t ? (message.textContent = "") : null), 4000);
  };
  const enregistrer = async (msg = "Réglage enregistré.") => {
    s = await api.saveSettings(s);
    dire(msg);
  };

  // Délai d'inactivité.
  const delai = h(
    "select",
    { id: "delai" },
    ...[1, 2, 3, 5, 10, 15].map((m) =>
      h("option", { value: m * 60, selected: s.idle_timeout_secs === m * 60 }, m === 3 ? "3 minutes (par défaut)" : `${m} minute${m > 1 ? "s" : ""}`),
    ),
  );
  if (![1, 2, 3, 5, 10, 15].some((m) => m * 60 === s.idle_timeout_secs)) {
    delai.append(h("option", { value: s.idle_timeout_secs, selected: true }, `${Math.round(s.idle_timeout_secs / 60)} minutes`));
  }
  delai.addEventListener("change", () => {
    s.idle_timeout_secs = Number(delai.value);
    enregistrer();
  });

  const titres = h("input", { type: "checkbox", id: "titres", checked: s.record_titles });
  titres.addEventListener("change", () => {
    s.record_titles = titres.checked;
    enregistrer();
  });
  const demarrage = h("input", { type: "checkbox", id: "demarrage", checked: autostartActif });
  demarrage.addEventListener("change", async () => {
    try {
      await api.setAutostart(demarrage.checked);
      dire("Réglage enregistré.");
    } catch (e) {
      dire(`Réglage impossible : ${e}`);
    }
  });

  // Exclusions.
  const listeExclusions = h("div", { class: "chips" });
  const dessinerExclusions = () => {
    listeExclusions.replaceChildren(
      ...(s.exclusions.length
        ? s.exclusions.map((e, i) =>
            h(
              "span",
              { class: "chip" },
              e.value,
              h("small", {}, nomCritere(e.criterion)),
              h(
                "button",
                {
                  type: "button",
                  class: "ghost",
                  "aria-label": `Retirer l'exclusion ${e.value}`,
                  onclick: async () => {
                    s.exclusions.splice(i, 1);
                    await enregistrer("Exclusion retirée.");
                    dessinerExclusions();
                  },
                },
                "Retirer",
              ),
            ),
          )
        : [h("span", { class: "faint" }, "Aucune exclusion pour l'instant.")]),
    );
  };
  const typeExclusion = selectCritere("site");
  const valeurExclusion = h("input", { type: "text", placeholder: "par exemple : ma-banque.fr, KeePass, Médical" });
  const ajouterExclusion = h("button", { type: "button" }, "Exclure");
  ajouterExclusion.addEventListener("click", async () => {
    const value = valeurExclusion.value.trim();
    if (!value) return;
    const ex: Exclusion = { criterion: typeExclusion.value as Criterion, value };
    s.exclusions.push(ex);
    valeurExclusion.value = "";
    await enregistrer("Exclusion ajoutée. Ce qui avait déjà été enregistré pour elle est en cours d'effacement.");
    dessinerExclusions();
  });
  dessinerExclusions();

  // Règles de regroupement.
  const corpsRegles = h("tbody", {});
  const categories = h("datalist", { id: "categories" });
  let brouillon: Rule[] = s.rules.map((r) => ({ ...r }));
  const dessinerRegles = () => {
    categories.replaceChildren(...[...new Set(brouillon.map((r) => r.category))].map((c) => h("option", { value: c })));
    corpsRegles.replaceChildren(
      ...brouillon.map((r, i) => {
        const cat = h("input", { type: "text", value: r.category, list: "categories", "aria-label": "Type de tâche" });
        cat.addEventListener("input", () => (r.category = cat.value));
        const crit = selectCritere(r.criterion);
        crit.setAttribute("aria-label", "Critère");
        crit.addEventListener("change", () => (r.criterion = crit.value as Criterion));
        const val = h("input", { type: "text", value: r.value, "aria-label": "Valeur" });
        val.addEventListener("input", () => (r.value = val.value));
        const deplacer = (d: number) => () => {
          const j = i + d;
          if (j < 0 || j >= brouillon.length) return;
          [brouillon[i], brouillon[j]] = [brouillon[j], brouillon[i]];
          dessinerRegles();
        };
        return h(
          "tr",
          {},
          h("td", {}, cat),
          h("td", {}, crit),
          h("td", {}, val),
          h(
            "td",
            { class: "tools" },
            h("button", { type: "button", class: "ghost", title: "Monter", "aria-label": "Monter la règle", onclick: deplacer(-1) }, "↑"),
            h("button", { type: "button", class: "ghost", title: "Descendre", "aria-label": "Descendre la règle", onclick: deplacer(1) }, "↓"),
            h(
              "button",
              {
                type: "button",
                class: "ghost",
                title: "Supprimer",
                "aria-label": "Supprimer la règle",
                onclick: () => {
                  brouillon.splice(i, 1);
                  dessinerRegles();
                },
              },
              "✕",
            ),
          ),
        );
      }),
    );
  };
  dessinerRegles();

  const ajouterRegle = h("button", { type: "button" }, "Ajouter une règle");
  ajouterRegle.addEventListener("click", () => {
    brouillon.unshift({ category: "", criterion: "titre", value: "" });
    dessinerRegles();
    (corpsRegles.querySelector("input") as HTMLInputElement | null)?.focus();
  });
  const enregistrerRegles = h("button", { type: "button", class: "primary" }, "Enregistrer les règles");
  enregistrerRegles.addEventListener("click", async () => {
    s.rules = brouillon.filter((r) => r.category.trim() && r.value.trim());
    await enregistrer("Règles enregistrées. Les chiffres sont recalculés avec ces règles, y compris pour les jours passés.");
    brouillon = s.rules.map((r) => ({ ...r }));
    dessinerRegles();
  });
  const reinitialiser = boutonConfirme("Revenir aux règles proposées", "Confirmer : remplacer mes règles", async () => {
    brouillon = await api.defaultRules();
    dessinerRegles();
    dire("Règles proposées rétablies. Cliquez sur « Enregistrer les règles » pour les garder.");
  }, "");

  const effacer = (scope: "hour" | "today" | "all", ok: string) => async () => {
    await api.erase(scope);
    dire(ok);
  };

  const blocs: (HTMLElement | null)[] = [
    h("div", { class: "page-head" }, h("div", {}, h("h1", {}, "Réglages"), h("p", { class: "muted" }, "Aucun réglage n'est obligatoire. Tout est enregistré sur ce PC.")), message),
    status.serverMessage ? h("section", { class: "card notice" }, h("h3", {}, "Extension navigateur indisponible"), h("p", { class: "muted" }, status.serverMessage)) : null,
    h(
      "section",
      { class: "card" },
      h("h2", {}, "Mesure"),
      h(
        "div",
        { class: "field" },
        h("label", { for: "delai" }, h("b", {}, "Délai d'inactivité")),
        delai,
        h("span", { class: "faint" }, "Sans souris ni clavier pendant ce délai, le temps n'est plus compté, même si des fenêtres restent ouvertes. Un délai plus long compte mieux la lecture de longs documents."),
      ),
      h(
        "label",
        { class: "check field", for: "titres" },
        titres,
        h("span", {}, h("b", {}, "Enregistrer le titre des fenêtres"), h("br"), h("span", { class: "faint" }, "Le titre indique le document ou la page (par exemple « Devis Martin.xlsx »). Il peut contenir le nom d'un client ou le sujet d'un mail. Sans titre, le regroupement ne se fait que sur les applications et les sites.")),
      ),
      h(
        "label",
        { class: "check field", for: "demarrage" },
        demarrage,
        h("span", {}, h("b", {}, "Démarrer avec Windows"), h("br"), h("span", { class: "faint" }, "SKELVIA Pulse démarre discrètement : la pastille et l'icône près de l'horloge restent visibles.")),
      ),
    ),
    h(
      "section",
      { class: "card" },
      h("h2", {}, "Exclusions"),
      h("p", { class: "muted" }, "Ce qui correspond à une exclusion n'est jamais enregistré, et ce qui l'avait déjà été est effacé. Utile pour une banque, un gestionnaire de mots de passe, un dossier personnel."),
      listeExclusions,
      h("div", { class: "inline-form" }, typeExclusion, valeurExclusion, ajouterExclusion),
    ),
    h(
      "section",
      { class: "card" },
      h("h2", {}, "Règles de regroupement"),
      h(
        "p",
        { class: "muted" },
        "Chaque règle range le temps dans un type de tâche. Les règles sont lues de haut en bas : la première qui correspond l'emporte. « Mot dans le titre » cherche un mot qui commence ainsi (« facture » trouve aussi « Factures »). Le temps qu'aucune règle ne range apparaît dans « Autre ».",
      ),
      h("div", { class: "actions" }, ajouterRegle, enregistrerRegles, reinitialiser),
      categories,
      h(
        "table",
        { class: "rules", style: "margin-top:14px" },
        h("thead", {}, h("tr", {}, h("th", {}, "Type de tâche"), h("th", {}, "Critère"), h("th", {}, "Valeur"), h("th", {}, ""))),
        corpsRegles,
      ),
    ),
    h(
      "section",
      { class: "card" },
      h("h2", {}, "Extension navigateur"),
      h("p", { class: "muted" }, "Facultative. Elle indique le site consulté dans l'onglet actif. Elle n'envoie ses données qu'à ce PC, et SKELVIA Pulse les refuse pendant la pause. La navigation privée n'est pas comptée et ce qu'elle envoie est effacé."),
      h(
        "div",
        { class: "actions" },
        h("button", { type: "button", onclick: () => api.openExtensionPage("chrome") }, "Pour Chrome"),
        h("button", { type: "button", onclick: () => api.openExtensionPage("edge") }, "Pour Edge"),
        h("button", { type: "button", onclick: () => api.openExtensionPage("firefox") }, "Pour Firefox"),
      ),
    ),
    h(
      "section",
      { class: "card" },
      h("h2", {}, "Effacer mes données"),
      h("p", { class: "muted" }, "L'effacement est définitif. Un événement en cours au moment choisi est effacé en entier."),
      h(
        "div",
        { class: "actions" },
        boutonConfirme("Effacer la dernière heure", "Confirmer : effacer la dernière heure", effacer("hour", "La dernière heure est effacée.")),
        boutonConfirme("Effacer aujourd'hui", "Confirmer : effacer aujourd'hui", effacer("today", "La journée est effacée.")),
        boutonConfirme("Tout effacer", "Confirmer : tout effacer", effacer("all", "Toutes les données enregistrées sont effacées.")),
      ),
      h(
        "p",
        { class: "faint", style: "margin-top:14px" },
        "Vos données sont dans le dossier « %LOCALAPPDATA%\\SKELVIA Pulse ». Pour tout supprimer, y compris les réglages : désinstallez SKELVIA Pulse, puis supprimez ce dossier. ",
        h("button", { type: "button", class: "ghost", onclick: () => api.openDataFolder() }, "Ouvrir le dossier"),
      ),
    ),
    h(
      "section",
      { class: "card" },
      h("h2", {}, "À propos"),
      h(
        "p",
        { class: "muted" },
        "SKELVIA Pulse est un logiciel libre et gratuit (licence MIT), proposé par SKELVIA. Il repose sur le moteur libre ActivityWatch (licence MPL-2.0). Le nom SKELVIA, le logo et la mascotte sont réservés.",
      ),
    ),
  ];
  root.replaceChildren(...blocs.filter((b): b is HTMLElement => b !== null));
}
