# Contribuer à SKELVIA Pulse

Merci de votre intérêt. Les contributions sont bienvenues : corrections, tests sur Windows, nouvelles règles de regroupement utiles au bâtiment, améliorations de l'interface.

## Les principes à respecter

Toute contribution doit respecter ces règles, sans exception :

1. **Tout reste sur le PC.** Aucune requête réseau vers l'extérieur, aucune télémétrie, aucun compte, aucune bibliothèque qui en ajoute. Seule exception : la vérification des mises à jour (`src-tauri/src/updater.rs`), qui ne lit qu'un fichier public, n'envoie aucune mesure et peut être coupée.
2. **Le minimum de données.** Application, titre de fenêtre, domaine, durée, actif ou non. Jamais de capture d'écran, de touches, de clics, de contenu de documents ou de mails.
3. **Toujours visible.** La pastille et l'icône restent visibles ; la pause reste à un clic. Pas de mode caché.
4. **Simple pour un dirigeant non technicien.** Aucun réglage obligatoire, pas de jargon dans l'interface.

## Règles d'écriture de l'interface

- Français, **vouvoiement**.
- Pas de tiret long ; des phrases courtes.
- Le mot « assistant », jamais « employé » ni « agent ».
- **Aucun chiffre inventé** : l'application n'affiche que des mesures.

## Préparer son poste

- [Rust](https://rustup.rs) stable, [Node.js](https://nodejs.org) 20 ou plus.
- Sous Windows : [prérequis Tauri 2](https://v2.tauri.app/start/prerequisites/).
- Sous Linux (pour développer et tester la logique) : `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`.

```sh
npm ci
npm run tauri dev        # lancer l'application (la mesure ne fonctionne que sous Windows)
```

## Vérifier avant de proposer une modification

```sh
cargo test --workspace                                              # tous les tests Rust
cargo check -p pulse-collector --target x86_64-pc-windows-msvc      # collecteur Windows
npm run build                                                       # types TypeScript et interface
```

Pour le collecteur Windows depuis Linux : `rustup target add x86_64-pc-windows-msvc` (vérification seulement, sans exécution).

La CI GitHub (`.github/workflows/ci.yml`) lance ces vérifications et compile les installeurs Windows.

## Organisation du code

Voir [docs/architecture.md](docs/architecture.md).

## Licence des contributions

En proposant une contribution, vous acceptez qu'elle soit publiée sous [licence MIT](LICENSE). N'ajoutez pas de code sous une licence incompatible. Si vous reprenez du code d'un autre projet, gardez sa mention de copyright et ajoutez-la à [THIRD_PARTY.md](THIRD_PARTY.md). Le nom SKELVIA, le logo et les blips (la mascotte) restent réservés ([TRADEMARKS.md](TRADEMARKS.md)).
