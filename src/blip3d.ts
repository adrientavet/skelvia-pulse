/*
 * Le blip SKELVIA en 3D, dessiné en SVG (source unique, 24/10/2026).
 *
 * Même modèle que la planche technique (vsl-motion/mochi-3d) : un galet noir
 * (ellipsoïde 1 : 0,94 : 0,72) couvert de petites billes de sa couleur qui
 * suivent sa courbe, un fin trait de contour, deux yeux posés sur la surface
 * et, quand il travaille, des billes qui tournent autour comme des météorites
 * autour d'une planète. Le calcul 3D est fait ici, le rendu est un simple SVG :
 * pas de WebGL, donc autant de blips qu'on veut sur une page, à toutes les tailles.
 *
 * Copies (ne pas modifier une copie seule, repartir d'ici) :
 *   website-skelvia/src/components/btp/blip3d.ts
 *   skelvia-app/src/composants/blip3d.js, skelvia-crm/src/espace/blip3d.js (copies JS)
 *   skelvia-pulse/src/blip3d.ts
 *
 * 24/10/2026, le soir : dessin FIXE partout (le site ramait quand le blip était recalculé
 * 30 fois par seconde). On le calcule une fois, de face ; la vie (regard, clin, veille)
 * se fait par-dessus en CSS, grâce aux groupes blip-orbites, blip-points, blip-contour,
 * blip-lueur et à l'option yeux: false (le composant dessine alors les yeux lui-même).
 */

export type OptionsBlip = {
  couleur: string;
  /** 0 = en veille, 1 = au travail (les valeurs entre deux servent aux transitions). */
  eveil?: number;
  /** Rotation gauche-droite et haut-bas, en radians (regard qui suit la souris). */
  lacet?: number;
  tangage?: number;
  /** Temps en secondes : fait tourner les billes en orbite. */
  temps?: number;
  /** Clin d'œil en cours, de 0 à 1. */
  clin?: number;
  /** Petites tailles (moins de 48 px) : sphère lisse et gros yeux, sans billes. */
  simple?: boolean;
  /** Billes en orbite (au travail). Faux pour une icône. */
  billes?: boolean;
  /** Le carré « S. » du logo, en bas à droite. */
  maya?: boolean;
  titre?: string;
  /** Faux : pas d'yeux, le composant les dessine lui-même (regard, clin). */
  yeux?: boolean;
  /** Taille en pixels écrite sur la balise (sinon elle remplit son conteneur). */
  taille?: number;
};

type V = [number, number, number];
const A = 1, B = 0.94, P = 0.72; // proportions du galet
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const f = (v: number) => v.toFixed(2);

// Billes sur la surface : spirale de Fibonacci, rien autour du regard (moins de 23° de l'avant),
// plus grosses et plus vives vers le bord (taille plafonnée à 70° : le bord ne fait pas un anneau plein).
const SURFACE: { d: V; r: number; o: number }[] = [];
{
  const N = 820, or = Math.PI * (3 - Math.sqrt(5));
  for (let i = 0; i < N; i++) {
    const z = 1 - (2 * (i + 0.5)) / N, rr = Math.sqrt(1 - z * z);
    const th = Math.acos(z);
    if (th < 0.4) continue;
    const k = Math.min(1, (th - 0.4) / 0.82);
    SURFACE.push({ d: [Math.cos(or * i) * rr, Math.sin(or * i) * rr, z], r: (0.5 + 1.9 * k) / 80, o: 0.17 + 0.8 * k });
  }
}

// Billes en orbite : tirage fixe, orbites proches d'un axe incliné, plus lentes au loin.
type Orbite = { axe: V; h: number; perp: V; ray: number; rp: number; phase: number; vit: number; taille: number; eclat: number };
const ORBITES: Orbite[] = [];
{
  let g = 7;
  const al = () => (g = (g * 16807) % 2147483647) / 2147483647;
  const norm = (v: V): V => { const n = Math.hypot(...v); return [v[0] / n, v[1] / n, v[2] / n]; };
  const base = norm([0.25, 1, 0.18]);
  for (let i = 0; i < 110; i++) {
    const axe = norm([base[0] + (al() - 0.5) * 0.9, base[1] + (al() - 0.5) * 0.9, base[2] + (al() - 0.5) * 0.9]);
    const ray = 1.08 + 0.5 * Math.pow(al(), 1.3), h = (al() - 0.5) * 0.5 * ray;
    const q: V = [al() - 0.5, al() - 0.5, al() - 0.5];
    const dot = q[0] * axe[0] + q[1] * axe[1] + q[2] * axe[2];
    const perp = norm([q[0] - dot * axe[0], q[1] - dot * axe[1], q[2] - dot * axe[2]]);
    ORBITES.push({ axe, h, perp, ray, rp: Math.sqrt(ray * ray - h * h), phase: al() * Math.PI * 2, vit: 0.45 / Math.pow(ray, 1.5),
      taille: 0.008 + 0.034 * Math.pow(al(), 2.4), eclat: 0.55 + 0.6 * al() });
  }
}

/** Tourne autour de l'axe donné (formule de Rodrigues). */
function autour(v: V, k: V, a: number): V {
  const c = Math.cos(a), s = Math.sin(a), d = (k[0] * v[0] + k[1] * v[1] + k[2] * v[2]) * (1 - c);
  return [
    v[0] * c + (k[1] * v[2] - k[2] * v[1]) * s + k[0] * d,
    v[1] * c + (k[2] * v[0] - k[0] * v[2]) * s + k[1] * d,
    v[2] * c + (k[0] * v[1] - k[1] * v[0]) * s + k[2] * d,
  ];
}
/** Lacet (autour de l'axe vertical) puis tangage (autour de l'axe horizontal). */
function tourne([x, y, z]: V, lacet: number, tangage: number): V {
  const cy = Math.cos(lacet), sy = Math.sin(lacet);
  [x, z] = [x * cy + z * sy, -x * sy + z * cy];
  const cp = Math.cos(tangage), sp = Math.sin(tangage);
  return [x, y * cp - z * sp, y * sp + z * cp];
}

let compteur = 0;

/** Le blip en balise SVG complète. Repère : centre (100 ; 100), rayon 80 (92 en version simple). */
export function svgBlip(o: OptionsBlip): string {
  const { viewBox, contenu } = blip(o);
  const dim = o.taille ? ` width="${o.taille}" height="${o.taille}"` : "";
  const aria = o.titre ? ` role="img" aria-label="${o.titre.replace(/"/g, "&quot;")}"` : ` aria-hidden="true"`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}"${dim}${aria} focusable="false" style="display:block;width:100%;height:auto;overflow:visible">${contenu}</svg>`;
}

/** Le cadre et l'intérieur du SVG, pour un composant qui garde sa propre balise <svg> (classe, taille). */
export function blip(o: OptionsBlip): { viewBox: string; contenu: string } {
  const c = o.couleur, r = Math.max(0, Math.min(1, o.eveil ?? 1)), lacet = o.lacet ?? 0, tangage = o.tangage ?? 0;
  const simple = !!o.simple, R = simple ? 92 : 80, C = 100, id = `b${(compteur++).toString(36)}`;
  const parts: string[] = [];
  // Silhouette : le galet vu sous l'angle donné.
  const ex = Math.hypot(A * Math.cos(lacet), P * Math.sin(lacet)), ey = Math.hypot(B * Math.cos(tangage), P * Math.sin(tangage));
  const corps = `cx="${C}" cy="${C}" rx="${f(R * ex)}" ry="${f(R * ey)}"`;
  // Billes en orbite, derrière puis devant le corps.
  const derriere: string[] = [], devant: string[] = [];
  if (!simple && o.billes !== false && r > 0.02) {
    const sortie = 1 - Math.pow(1 - r, 3), t = o.temps ?? 0;
    for (const m of ORBITES) {
      let v: V = [m.perp[0] * m.rp + m.axe[0] * m.h, m.perp[1] * m.rp + m.axe[1] * m.h, m.perp[2] * m.rp + m.axe[2] * m.h];
      v = autour(v, m.axe, m.phase + m.vit * t);
      const k = lerp(0.9 / m.ray, 1, sortie);
      const el = `<circle cx="${f(C + R * v[0] * k)}" cy="${f(C - R * v[1] * k)}" r="${f(R * m.taille * sortie)}" fill="${c}" fill-opacity="${f(Math.min(1, m.eclat))}"/>`;
      (v[2] < 0 ? derriere : devant).push(el);
    }
  }
  parts.push(
    `<defs>`,
    // Volume : plus clair en haut à gauche, d'où vient la lumière.
    `<radialGradient id="${id}c" cx="36%" cy="30%" r="78%"><stop offset="0" stop-color="#34343c"/><stop offset=".45" stop-color="#16161a"/><stop offset="1" stop-color="#08080a"/></radialGradient>`,
    `<radialGradient id="${id}s" cx="50%" cy="50%" r="50%"><stop offset="0" stop-color="#fff" stop-opacity=".55"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></radialGradient>`,
    `<radialGradient id="${id}l" cx="50%" cy="50%" r="50%"><stop offset=".62" stop-color="${c}" stop-opacity="0"/><stop offset="1" stop-color="${c}" stop-opacity="${f(lerp(0.18, 0.55, r))}"/></radialGradient>`,
    `</defs>`,
    `<g class="blip-orbites">`, ...derriere, `</g>`,
    `<ellipse ${corps} fill="url(#${id}c)"/>`,
  );
  if (simple) {
    // En petit : la lumière de sa couleur au bord de la sphère, à la place des billes.
    parts.push(`<ellipse class="blip-lueur" ${corps} fill="url(#${id}l)"/>`);
  } else {
    parts.push(`<g class="blip-points">`);
    for (const s of SURFACE) {
      const p = tourne([A * s.d[0], B * s.d[1], P * s.d[2]], lacet, tangage);
      const n = tourne([s.d[0] / A, s.d[1] / B, s.d[2] / P], lacet, tangage);
      const nz = n[2] / Math.hypot(...n);
      if (nz <= 0.02) continue;
      const op = s.o * lerp(0.3, 1, r) * Math.pow(nz, 0.35);
      parts.push(`<circle cx="${f(C + R * p[0])}" cy="${f(C - R * p[1])}" r="${f(R * s.r)}" fill="${c}" fill-opacity="${f(op)}"/>`);
    }
    parts.push(`</g>`);
  }
  // Reflet de la lumière, puis le fin trait de contour.
  parts.push(
    `<ellipse cx="${f(C - R * 0.36 * ex)}" cy="${f(C - R * 0.5 * ey)}" rx="${f(R * 0.3)}" ry="${f(R * 0.19)}" transform="rotate(-28 ${f(C - R * 0.36 * ex)} ${f(C - R * 0.5 * ey)})" fill="url(#${id}s)"/>`,
    `<ellipse class="blip-contour" ${corps} fill="none" stroke="${c}" stroke-width="${simple ? 7 : 2}" stroke-opacity="${f(lerp(simple ? 0.35 : 0.17, simple ? 0.9 : 0.55, r))}"/>`,
  );
  // Les yeux, posés sur la courbe : ils suivent la rotation et s'écrasent en veille ou en clignant.
  const h = lerp(0.35, 1, r) * (1 - 0.88 * (o.clin ?? 0));
  for (const sx of o.yeux === false ? [] : [-1, 1]) {
    const x0 = sx * 0.275, y0 = -0.019, z0 = P * Math.sqrt(1 - x0 * x0 - (y0 / B) ** 2);
    const p = tourne([x0, y0, z0], lacet, tangage);
    const n = tourne([x0 / (A * A), y0 / (B * B), z0 / (P * P)], lacet, tangage);
    const nn = Math.hypot(...n), nx = n[0] / nn, ny = n[1] / nn, nz = n[2] / nn;
    if (nz <= 0.05) continue;
    const re = R * (simple ? 0.17 : 0.125);
    const ex2 = simple ? sx * 0.3 * R : R * p[0], ey2 = simple ? 0 : -R * p[1];
    parts.push(`<ellipse cx="${f(C + ex2)}" cy="${f(C + ey2)}" rx="${f(re * lerp(1.15, 1, r) * Math.sqrt(1 - nx * nx))}" ry="${f(re * h * Math.sqrt(1 - ny * ny))}" fill="${r > 0.5 ? "#fff" : c}"/>`);
  }
  if (devant.length) parts.push(`<g class="blip-orbites">`, ...devant, `</g>`);
  if (o.maya && !simple) {
    parts.push(`<rect x="140" y="140" width="46" height="46" rx="11" fill="#0B0B0C" stroke="#fff" stroke-opacity=".55" stroke-width="1.2"/>`,
      `<text x="163" y="172" text-anchor="middle" font-family="Bricolage Grotesque, DM Sans, sans-serif" font-weight="800" font-size="26" fill="#fff">S.</text>`);
  }
  return { viewBox: simple ? "0 0 200 200" : "-30 -30 260 260", contenu: parts.join("") };
}
