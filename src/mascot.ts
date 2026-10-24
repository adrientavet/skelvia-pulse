/*
 * Le blip SKELVIA (la mascotte) : dessin à plat qui donne l'impression du volume,
 * même modèle que la planche technique et que skelvia.fr, tiré par blip3d.ts.
 * Le nom et le dessin du blip sont réservés : voir TRADEMARKS.md.
 *
 * Un galet noir couvert de petites billes de sa couleur qui suivent sa courbe,
 * un fin trait de contour, deux yeux posés sur la surface.
 * - Au travail : billes vives, yeux ronds blancs, billes en orbite autour.
 * - En veille : billes à 30 %, yeux fermés de sa couleur, plus d'orbite.
 *
 * Dans SKELVIA Pulse : « au travail » quand l'enregistrement tourne, « en
 * veille » pendant la pause ou l'inactivité.
 */
import { blip } from "./blip3d";

const NS = "http://www.w3.org/2000/svg";
export const BLEU = "#2A63F5";

export interface Mascotte {
  svg: SVGSVGElement;
  /** Passe en veille (yeux fermés, billes pâles) ou au travail. */
  setVeille(veille: boolean): void;
}

/**
 * Crée le blip. `simple` : version pour les petites tailles (moins de 48 px),
 * sphère lisse et gros yeux, sans billes (sinon elles font une tache).
 * Dessin fixe (24/10/2026, le soir) : il n'est redessiné qu'au passage
 * veille / travail, jamais en continu.
 */
export function creerMascotte(opts: { couleur?: string; simple?: boolean; titre?: string } = {}): Mascotte {
  const couleur = opts.couleur ?? BLEU;
  const simple = opts.simple ?? false;
  const svg = document.createElementNS(NS, "svg");
  svg.setAttribute("viewBox", blip({ couleur, simple }).viewBox);
  svg.setAttribute("overflow", "visible");
  if (opts.titre) {
    svg.setAttribute("role", "img");
    svg.setAttribute("aria-label", opts.titre);
  } else {
    svg.setAttribute("aria-hidden", "true");
  }
  const setVeille = (veille: boolean) => {
    svg.innerHTML = blip({ couleur, simple, eveil: veille ? 0 : 1 }).contenu;
  };
  setVeille(false);
  return { svg, setVeille };
}