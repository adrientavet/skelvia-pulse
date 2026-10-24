// Prépare la publication d'une version, après `npm run tauri build` sous
// Windows avec la clé de signature des mises à jour.
//
// Écrit dans `publication/` : les deux installeurs (noms sans espace) et
// `latest.json`, le fichier que SKELVIA Pulse lit pour savoir s'il existe une
// nouvelle version. Écrit aussi `notes-de-version.md` (texte de la page GitHub).
//
// Usage : node scripts/preparer-publication.mjs <étiquette, ex. v0.2.0> <dépôt, ex. adrientavet/skelvia-pulse>
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [tag, repo] = process.argv.slice(2);
const stop = (message) => {
  console.error(`Erreur : ${message}`);
  process.exit(1);
};
if (!tag || !repo) stop("usage : node scripts/preparer-publication.mjs v0.2.0 propriétaire/dépôt");

const version = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")).version;
if (tag !== `v${version}`) stop(`l'étiquette ${tag} ne correspond pas à la version ${version} de src-tauri/tauri.conf.json.`);
for (const [file, found] of [
  ["package.json", JSON.parse(readFileSync("package.json", "utf8")).version],
  ["Cargo.toml", /^version = "([^"]+)"/m.exec(readFileSync("Cargo.toml", "utf8"))?.[1]],
]) {
  if (found !== version) stop(`${file} indique la version ${found}, au lieu de ${version}.`);
}

// Nouveautés de cette version, lues dans NOUVEAUTES.md (section « ## 0.2.0 »).
const lines = readFileSync("NOUVEAUTES.md", "utf8").split("\n");
const start = lines.findIndex((l) => l.trim() === `## ${version}`);
if (start < 0) stop(`NOUVEAUTES.md n'a pas de section « ## ${version} ».`);
const end = lines.findIndex((l, i) => i > start && l.startsWith("## "));
const notes = lines.slice(start + 1, end < 0 ? undefined : end).join("\n").trim();
if (!notes) stop(`la section « ## ${version} » de NOUVEAUTES.md est vide.`);

const bundle = "target/release/bundle";
const find = (dir, suffix) => {
  const files = existsSync(join(bundle, dir)) ? readdirSync(join(bundle, dir)).filter((f) => f.startsWith(`SKELVIA Pulse_${version}_`) && f.endsWith(suffix)) : [];
  if (files.length !== 1) stop(`installeur ${suffix} de la version ${version} introuvable dans ${bundle}/${dir}.`);
  const file = join(bundle, dir, files[0]);
  if (!existsSync(`${file}.sig`)) stop(`signature ${file}.sig absente : la clé TAURI_SIGNING_PRIVATE_KEY était-elle fournie ?`);
  return { file, signature: readFileSync(`${file}.sig`, "utf8").trim() };
};
const nsis = find("nsis", "-setup.exe");
const msi = find("msi", ".msi");

const out = "publication";
rmSync(out, { recursive: true, force: true });
mkdirSync(out);
const names = { nsis: `skelvia-pulse_${version}_x64-setup.exe`, msi: `skelvia-pulse_${version}_x64_fr-FR.msi` };
copyFileSync(nsis.file, join(out, names.nsis));
copyFileSync(msi.file, join(out, names.msi));

const url = (name) => `https://github.com/${repo}/releases/download/${tag}/${name}`;
const entry = (name, signature) => ({ url: url(name), signature });
const manifest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms: {
    // Chaque installation se met à jour avec le même type d'installeur.
    "windows-x86_64-nsis": entry(names.nsis, nsis.signature),
    "windows-x86_64-msi": entry(names.msi, msi.signature),
    "windows-x86_64": entry(names.nsis, nsis.signature),
  },
};
writeFileSync(join(out, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
writeFileSync(
  "notes-de-version.md",
  `${notes}\n\n**Installer** : téléchargez \`${names.nsis}\` ci-dessous (conseillé, sans droits administrateur). Le fichier \`.msi\` sert au déploiement par un informaticien.\n\nDéjà installé ? SKELVIA Pulse propose la mise à jour dans sa pastille.\n`,
);
console.log(`Publication de la version ${version} prête dans ${out}/ :`);
for (const f of readdirSync(out)) console.log(`  ${f}`);
