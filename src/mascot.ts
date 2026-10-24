/*
 * Mascotte SKELVIA, dessinée en code (SVG). Le nom et le dessin de la
 * mascotte sont réservés : voir TRADEMARKS.md.
 *
 * Un rond noir cerclé d'un fin trait de sa couleur, éclairé de l'intérieur
 * par des points en demi-teinte (gros au bord, petits vers le centre).
 * - Au travail : points vifs, yeux ronds blancs, halo extérieur de points.
 * - En veille : points très pâles, yeux fermés, pas de halo.
 *
 * Dans SKELVIA Pulse : « au travail » quand l'enregistrement tourne, « en
 * veille » pendant la pause ou l'inactivité.
 */

const NS = "http://www.w3.org/2000/svg";
const CX = 100;
const CY = 100;
const DOUX = "cubic-bezier(0.22, 1, 0.36, 1)";
export const BLEU = "#2A63F5";

type Point = { x: number; y: number; r: number; o: number };

function pointsInterieurs(): Point[] {
  const pts: Point[] = [];
  for (let x = 24; x <= 176; x += 8) {
    for (let y = 24; y <= 176; y += 8) {
      const d = Math.hypot(x - CX, y - CY);
      if (d < 31 || d > 74) continue;
      const t = (d - 31) / (74 - 31);
      pts.push({ x, y, r: 0.45 + 2.6 * t, o: 0.17 + 0.8 * t });
    }
  }
  return pts;
}

function pointsHalo(): Point[] {
  const pts: Point[] = [];
  for (let x = -29; x <= 229; x += 9) {
    for (let y = -29; y <= 229; y += 9) {
      const d = Math.hypot(x - CX, y - CY);
      if (d < 82 || d > 122) continue;
      const u = 1 - (d - 82) / 40;
      pts.push({ x, y, r: 0.5 + 2.8 * u, o: 0.14 + 0.4 * u });
    }
  }
  return pts;
}

const INTERIEUR = pointsInterieurs();
const HALO = pointsHalo();

function el<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string | number>): SVGElementTagNameMap[K] {
  const e = document.createElementNS(NS, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v));
  return e;
}

export interface Mascotte {
  svg: SVGSVGElement;
  /** Passe en veille (yeux fermés, points pâles) ou au travail. */
  setVeille(veille: boolean): void;
}

/**
 * Crée la mascotte. `simple` : version pour les petites tailles (moins de
 * 48 px), lueur pleine et gros yeux, sinon les points font une tache.
 */
export function creerMascotte(opts: { couleur?: string; simple?: boolean; titre?: string } = {}): Mascotte {
  const couleur = opts.couleur ?? BLEU;
  const simple = opts.simple ?? false;
  const svg = el("svg", { viewBox: simple ? "0 0 200 200" : "-30 -30 260 260" });
  if (opts.titre) {
    svg.setAttribute("role", "img");
    svg.setAttribute("aria-label", opts.titre);
  } else {
    svg.setAttribute("aria-hidden", "true");
  }

  const yeux: SVGCircleElement[] = [];
  const styleOeil = (e: SVGCircleElement) => {
    e.style.transformBox = "fill-box";
    e.style.transformOrigin = "center";
    e.style.transition = `transform 420ms ${DOUX}, fill 420ms ${DOUX}`;
    yeux.push(e);
  };

  let halo: SVGGElement | null = null;
  let interieur: SVGGElement | null = null;
  let cercle: SVGCircleElement;
  let stop: SVGStopElement | null = null;

  if (simple) {
    const id = `lueur-${Math.random().toString(36).slice(2)}`;
    const defs = el("defs", {});
    const grad = el("radialGradient", { id, cx: "50%", cy: "50%", r: "50%" });
    grad.append(el("stop", { offset: "45%", "stop-color": "#151517" }));
    stop = el("stop", { offset: "100%", "stop-color": couleur, "stop-opacity": 0.75 });
    grad.append(stop);
    defs.append(grad);
    svg.append(defs);
    cercle = el("circle", { cx: 100, cy: 100, r: 92, fill: `url(#${id})`, stroke: couleur, "stroke-width": 8 });
    svg.append(cercle);
    for (const cx of [72, 128]) {
      const o = el("circle", { cx, cy: 100, r: 16, fill: "#fff" });
      styleOeil(o);
      svg.append(o);
    }
  } else {
    halo = el("g", {});
    halo.style.transformOrigin = "100px 100px";
    halo.style.transition = `opacity 600ms ${DOUX}, transform 600ms ${DOUX}`;
    for (const p of HALO) halo.append(el("circle", { cx: p.x, cy: p.y, r: p.r.toFixed(2), fill: couleur, "fill-opacity": p.o.toFixed(2) }));
    svg.append(halo);
    cercle = el("circle", { cx: CX, cy: CY, r: 80, fill: "#151517", stroke: couleur, "stroke-width": 2 });
    cercle.style.transition = `stroke-opacity 600ms ${DOUX}`;
    svg.append(cercle);
    interieur = el("g", {});
    interieur.style.transition = `opacity 600ms ${DOUX}`;
    for (const p of INTERIEUR) interieur.append(el("circle", { cx: p.x, cy: p.y, r: p.r.toFixed(2), fill: couleur, "fill-opacity": p.o.toFixed(2) }));
    svg.append(interieur);
    for (const cx of [78, 122]) {
      const o = el("circle", { cx, cy: 101.5, r: 10, fill: "#fff" });
      styleOeil(o);
      svg.append(o);
    }
  }

  const setVeille = (veille: boolean) => {
    for (const o of yeux) {
      o.setAttribute("fill", veille ? couleur : "#fff");
      o.style.transform = veille ? "scale(1.15, 0.35)" : "none";
    }
    if (halo) {
      halo.style.opacity = veille ? "0" : "1";
      halo.style.transform = veille ? "scale(0.94)" : "none";
    }
    if (interieur) interieur.style.opacity = veille ? "0.3" : "1";
    if (!simple) cercle.style.strokeOpacity = veille ? "0.17" : "0.55";
    if (stop) stop.setAttribute("stop-opacity", veille ? "0.25" : "0.75");
  };
  setVeille(false);
  return { svg, setVeille };
}
