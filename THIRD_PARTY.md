# Composants de tiers et attributions

SKELVIA Pulse s'appuie sur des logiciels libres. Merci à leurs auteurs.

## ActivityWatch (MPL-2.0)

- **Ce qui est utilisé** : les bibliothèques `aw-server`, `aw-datastore` et `aw-models` du projet [aw-server-rust](https://github.com/ActivityWatch/aw-server-rust) (serveur local et base de données SQLite), en dépendance, **sans modification**, à la révision `1d101d4c41b7c434f0e06e89bfceb8abe30ba364`.
- **Licence** : Mozilla Public License 2.0, texte complet dans [`LICENSES/MPL-2.0.txt`](LICENSES/MPL-2.0.txt).
- **Code source** : disponible à l'adresse ci-dessus, à la révision indiquée (obligation de la MPL-2.0, section 3.2, pour la distribution sous forme exécutable). Aucun fichier d'ActivityWatch n'est copié ni modifié dans ce dépôt. Si un fichier d'ActivityWatch venait à être modifié, il devrait rester sous MPL-2.0 et sa version modifiée être publiée.
- **Auteurs** : Erik Bjäreholt, Johan Bjäreholt et les contributeurs d'ActivityWatch (https://activitywatch.net).
- **Extension navigateur** : « ActivityWatch Web Watcher » ([aw-watcher-web](https://github.com/ActivityWatch/aw-watcher-web), MPL-2.0) n'est **pas** distribuée avec SKELVIA Pulse. L'utilisateur l'installe lui-même depuis le Chrome Web Store ou Firefox Add-ons.
- **Collecteur Windows** (`crates/pulse-collector`) : écrit pour SKELVIA Pulse. Il utilise les mêmes fonctions de Windows que les watchers d'ActivityWatch (`GetForegroundWindow`, `GetLastInputInfo`), sans reprendre leur code (écrit en Python).

## Coucou (MIT)

Le principe d'interface de la pastille (une petite « île » en haut de l'écran qui s'agrandit au survol, des « pills » pour les états) s'inspire de [Coucou](https://github.com/louis-cfm/coucou) de Louis Raillé. **Aucun code, aucun nom, aucune image, aucun son et aucun média de Coucou n'est repris.** Les noms « Coucou » et « Mochi », le personnage, l'icône, les sons et les médias de Coucou appartiennent à Louis Raillé (voir son fichier `LICENSE-ASSETS.md`).

Si du code de Coucou est repris un jour, il faudra conserver sa mention de copyright MIT (« Copyright (c) 2026 Louis Raillé ») et la reproduire ici.

## Polices (SIL Open Font License 1.1)

Incluses dans l'application et dans les rapports HTML (sous-ensemble « latin »), sans requête vers Google Fonts.

- **Bricolage Grotesque** : Copyright 2022 The Bricolage Grotesque Project Authors (https://github.com/ateliertriay/bricolage). Licence : [`LICENSES/OFL-1.1-BricolageGrotesque.txt`](LICENSES/OFL-1.1-BricolageGrotesque.txt).
- **DM Sans** : Copyright 2014 The DM Sans Project Authors (https://github.com/googlefonts/dm-fonts). Licence : [`LICENSES/OFL-1.1-DMSans.txt`](LICENSES/OFL-1.1-DMSans.txt).

Licence vérifiée dans les fichiers `OFL.txt` du dépôt [google/fonts](https://github.com/google/fonts).

## Tauri et autres bibliothèques

- **Tauri** (https://tauri.app) : MIT ou Apache-2.0, au choix.
- **Rocket**, **windows-rs**, **serde**, **chrono** et la plupart des bibliothèques Rust : MIT ou Apache-2.0.
- **Vite**, **TypeScript**, **@tauri-apps/api** : MIT ou Apache-2.0.

La liste exacte des bibliothèques et de leurs versions figure dans `Cargo.lock` et `package-lock.json`. Un relevé fait avec `cargo metadata` ne montre aucune licence GPL. On y trouve, outre MIT et Apache-2.0 : MPL-2.0 (quelques bibliothèques, utilisées sans modification), BSD-3-Clause, ISC, Zlib, Unicode-3.0, Unlicense. Une liste générée automatiquement avec les textes complets (par exemple avec `cargo about`) reste à produire avant la première version publique.
