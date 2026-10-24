# Fonctionnement technique

## Vue d'ensemble

```
┌──────────────────────────── PC de l'utilisateur ─────────────────────────────┐
│                                                                              │
│  Windows ──(fenêtre au premier plan, heure de la dernière action)──┐         │
│                                                                    ▼         │
│  ┌────────────── skelvia-pulse.exe (un seul programme) ─────────────────┐    │
│  │ pulse-collector  : lecture Windows, une fois par seconde             │    │
│  │ collector.rs     : actif / inactif, exclusions, pause                │    │
│  │ aw-datastore     : base SQLite (format ActivityWatch)                │    │
│  │ aw-server        : serveur local 127.0.0.1:5600 (pour l'extension)   │◄───┼── extension navigateur
│  │ pulse-core       : calculs, règles, rapport HTML                     │    │   (onglet actif)
│  │ Tauri + TypeScript : pastille, fenêtre principale, icône             │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  %LOCALAPPDATA%\SKELVIA Pulse\  donnees.sqlite, reglages.json                │
│  Téléchargements\               Rapport SKELVIA Pulse AAAA-MM-JJ.html        │
└──────────────────────────────────────────────────────────────────────────────┘
        Seule connexion vers l'extérieur : lecture de latest.json sur GitHub
        (mises à jour, désactivable, aucune mesure envoyée).
```

## Pourquoi cette architecture

ActivityWatch fournit sous Windows des « watchers » écrits en Python et empaquetés avec PyInstaller, même dans sa version Tauri (`aw-tauri`). Les embarquer imposerait une chaîne de compilation Python + Rust + Node et plusieurs programmes à surveiller. SKELVIA Pulse garde donc :

- le **serveur et la base d'ActivityWatch** (`aw-server-rust`, MPL-2.0), utilisés sans modification comme bibliothèques Rust, comme le fait `aw-tauri`. Les données restent au format ActivityWatch et l'extension officielle fonctionne telle quelle ;
- un **collecteur maison en Rust** (`crates/pulse-collector`, une centaine de lignes) qui appelle les mêmes fonctions Windows qu'ActivityWatch : `GetForegroundWindow`, `GetWindowTextW`, `QueryFullProcessImageNameW`, `GetLastInputInfo`, et `OpenInputDesktop` pour la session verrouillée.

Résultat : un seul programme, un seul installeur, sans Python.

## Les modules

| Dossier | Rôle | Tests |
|---|---|---|
| `crates/pulse-core` | Réglages, règles, calcul du temps actif, rapport HTML, blip (mascotte) en SVG. Aucune dépendance système. | `cargo test -p pulse-core` |
| `crates/pulse-collector` | Lecture Windows (fenêtre, inactivité, verrouillage). Ne renvoie rien hors Windows. | `cargo check -p pulse-collector --target x86_64-pc-windows-msvc` |
| `src-tauri` | Application : boucle de collecte, base, serveur local, commandes, icône, pastille. | `cargo test -p skelvia-pulse` |
| `src` | Interface TypeScript sans framework (pastille, Aujourd'hui, Rapport, Réglages, premier lancement). | `npm run typecheck` |

## Règles de calcul

- **Fenêtre** : un événement `{app, title}` par seconde, fusionné avec le précédent s'il est identique (même principe que l'outil `heartbeat` d'ActivityWatch). Rien n'est écrit pendant la pause, l'inactivité, ni pour une fenêtre exclue.
- **Activité** : la plage « active » va de son début à la **dernière action** de la souris ou du clavier. Quand le délai d'inactivité est dépassé, l'inactivité commence à cette dernière action : le délai d'attente n'est pas compté comme du travail. Au verrouillage, l'inactivité commence tout de suite.
- **Temps compté** = intersection des fenêtres au premier plan et des plages actives (`pulse-core/src/stats.rs`).
- **Navigateur** : pour Chrome, Edge, Firefox, Brave et Opera, le temps est découpé selon l'onglet actif envoyé par l'extension (seau `aw-watcher-web-<navigateur>_<PC>`). Les onglets de navigation privée sont retirés.
- **Exclusions** : appliquées avant l'écriture pour les fenêtres, et par un effacement chaque minute pour les données de l'extension (le serveur ne peut pas lire leur contenu avant de les écrire). La fenêtre de navigateur qui recouvre surtout un onglet effacé est effacée aussi. Ajouter une exclusion efface aussi le passé.
- **Pause** : le collecteur n'écrit plus rien, et le serveur répond « 503 » aux envois de l'extension (`server.rs`, testé).

## Sécurité et confidentialité

- Le serveur n'écoute que sur `127.0.0.1` : il n'est pas joignable depuis le réseau.
- Comme dans ActivityWatch, un autre programme du même PC peut lire l'API locale. ActivityWatch limite ce que les extensions de navigateur peuvent lire (`extension_cors.rs`), mais n'importe quel programme lancé sur le PC peut interroger `127.0.0.1:5600`. C'est une limite connue, à garder en tête.
- Politique de sécurité du contenu (CSP) stricte dans les fenêtres : aucun script ni ressource extérieure.
- Le rapport HTML n'a aucune ressource extérieure (polices incluses). Le seul lien est celui, facultatif, vers l'audit SKELVIA.
- **Mises à jour** (`updater.rs`, extension officielle `tauri-plugin-updater`) : 30 secondes après le démarrage puis toutes les 6 heures, si l'option est active et l'écran d'accueil accepté, lecture de `https://github.com/<dépôt>/releases/latest/download/latest.json`. L'adresse est fixe : ni version ni identifiant. L'installeur n'est téléchargé qu'après un clic, et il est refusé si sa signature ne correspond pas à la clé publique inscrite dans `tauri.conf.json` (`plugins > updater > pubkey`). Sous Windows, l'application se ferme, l'installeur tourne en mode « passif » (barre de progression seulement), puis relance l'application. Publication : voir [Publier une version](publier-une-version.md).

## Données sur le disque

| Fichier | Contenu |
|---|---|
| `%LOCALAPPDATA%\SKELVIA Pulse\donnees.sqlite` | Base ActivityWatch : seaux et événements. |
| `%LOCALAPPDATA%\SKELVIA Pulse\reglages.json` | Réglages (délai, exclusions, règles, pause). |
| `%LOCALAPPDATA%\SKELVIA Pulse\identifiant-installation` | Identifiant aléatoire demandé par le serveur ActivityWatch. Ne quitte pas le PC. |
| `%LOCALAPPDATA%\fr.skelvia.pulse\` | Cache d'affichage WebView2 (créé par Windows). |
