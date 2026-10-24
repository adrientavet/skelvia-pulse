// Petit outil pour construire l'interface sans framework : le code reste
// court et facile à relire pour un contributeur.

type Child = Node | string | null | undefined | false;
type Attrs = Record<string, string | number | boolean | EventListener | undefined>;

export function h<K extends keyof HTMLElementTagNameMap>(tag: K, attrs: Attrs = {}, ...children: Child[]): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (v === undefined || v === false) continue;
    if (k.startsWith("on") && typeof v === "function") {
      e.addEventListener(k.slice(2).toLowerCase(), v);
    } else if (k === "class") {
      e.className = String(v);
    } else if (v === true) {
      e.setAttribute(k, "");
    } else {
      e.setAttribute(k, String(v));
    }
  }
  for (const c of children) if (c) e.append(c);
  return e;
}

/** Barres horizontales, triées, valeur écrite à côté de chaque barre. */
export function bars(rows: { label: string; value: string; ratio: number; meta?: string; tooltip?: string }[]): HTMLElement {
  return h(
    "div",
    { class: "bars" },
    ...rows.map((r) => {
      const fill = h("div", { class: "fill" });
      fill.style.width = `${Math.max(0, Math.min(1, r.ratio)) * 100}%`;
      return h(
        "div",
        { class: "bar-row", title: r.tooltip ?? `${r.label} : ${r.value}` },
        h("span", { class: "label" }, r.label),
        h("span", { class: "value" }, r.value),
        h("div", { class: "track" }, fill),
        r.meta ? h("span", { class: "meta" }, r.meta) : null,
      );
    }),
  );
}

export function pct(part: number, total: number): string {
  if (total <= 0) return "0 %";
  const p = (part / total) * 100;
  return p < 1 ? "moins de 1 %" : `${Math.round(p)} %`;
}
