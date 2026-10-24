# Tester sur Windows

SKELVIA Pulse a été écrit et vérifié sous Linux : le code se compile, les tests automatiques passent, et le collecteur Windows est vérifié par le compilateur pour la cible Windows. **Il n'a encore jamais tourné sur un vrai PC Windows.** Cette page explique comment l'essayer et quoi vérifier.

## 1. Obtenir l'installeur

Le dépôt contient un workflow GitHub Actions (`.github/workflows/ci.yml`) qui compile l'installeur sur une machine Windows de GitHub :

1. Sur GitHub, ouvrez l'onglet **Actions** du dépôt, puis le dernier passage réussi de « CI ».
2. En bas de la page, section **Artifacts**, téléchargez **`skelvia-pulse-windows`** (il faut être connecté à GitHub).
3. Décompressez le fichier zip. Vous y trouvez :
   - `SKELVIA Pulse_0.1.0_x64-setup.exe` (installeur conseillé, sans droits administrateur) ;
   - `SKELVIA Pulse_0.1.0_x64_fr-FR.msi` (installeur MSI, pour un déploiement par un informaticien).

Vous pouvez aussi lancer la compilation à la main depuis l'onglet Actions (« Run workflow »).

## 2. Liste de vérifications

Cochez au fur et à mesure et notez ce qui ne va pas.

### Installation
- [ ] L'avertissement SmartScreen s'affiche et « Informations complémentaires » puis « Exécuter quand même » fonctionne. Noter le libellé exact du lien.
- [ ] L'installation se termine sans demander de mot de passe administrateur (fichier `.exe`).
- [ ] La fenêtre d'accueil s'affiche, en français, aux couleurs SKELVIA.

### Premier lancement
- [ ] Avant de cliquer sur « J'ai compris, commencer » : pas de pastille, et rien n'apparaît dans Aujourd'hui.
- [ ] Après le clic : la pastille apparaît en haut au centre de l'écran, avec « Enregistre ».
- [ ] L'icône près de l'horloge est visible (parfois dans la flèche « icônes cachées » de la barre des tâches).

### Mesure
- [ ] Travaillez 10 minutes dans Excel, Outlook, un PDF : le temps apparaît dans Aujourd'hui (mis à jour chaque minute).
- [ ] Le nom des applications est lisible (Excel, Outlook, Adobe Acrobat…).
- [ ] Laissez le PC sans toucher la souris plus de 3 minutes : la pastille passe en « en veille » (yeux fermés) et ce temps n'est pas compté.
- [ ] Verrouillez la session (touche Windows + L) pendant 2 minutes : ce temps n'est pas compté.
- [ ] Applications du Microsoft Store (Courrier, Calculatrice…) : leur vrai nom apparaît, et pas « ApplicationFrameHost ».
- [ ] Une application lancée « en tant qu'administrateur » : son nom apparaît-il ? (cas connu comme délicat)

### Pastille
- [ ] Au survol, elle s'agrandit et montre le temps du jour et « Ouvrir le détail ».
- [ ] Le bouton Pause passe à « En pause » (mascotte en veille), et plus rien n'est compté.
- [ ] Elle reste au-dessus des autres fenêtres sans gêner (vérifier avec une fenêtre en plein écran, et sur un écran avec mise à l'échelle 125 % ou 150 %).
- [ ] Avec deux écrans : au départ, elle se place sur l'écran principal.
- [ ] Glisser la pastille (en la tenant par la mascotte ou le texte, pas par le bouton) la déplace n'importe où, y compris sur un autre écran.
- [ ] Après avoir quitté puis relancé SKELVIA Pulse, elle revient à la position choisie.
- [ ] Posée en bas de l'écran, elle s'ouvre vers le haut au survol, sans sortir de l'écran.
- [ ] « Remettre la pastille en haut de l'écran » (Réglages, ou menu de l'icône) la ramène en haut au centre.

### Navigateur
- [ ] Installez l'extension via les boutons « Pour Chrome », « Pour Edge » ou « Pour Firefox ». Noter les étapes réelles (surtout pour Edge).
- [ ] Après quelques minutes de navigation, la carte « Sites web » apparaît.
- [ ] En pause, la navigation n'est pas comptée.
- [ ] Un onglet de navigation privée n'apparaît pas (si l'extension y est autorisée).

### Rapport
- [ ] « Télécharger mon rapport » écrit le fichier dans Téléchargements et l'ouvre dans le navigateur.
- [ ] Le rapport s'affiche correctement sans internet (couper le Wi-Fi et rouvrir le fichier).
- [ ] L'impression (Ctrl + P) donne une page lisible sur fond blanc.

### Réglages et effacement
- [ ] Ajouter une exclusion (par exemple « excel ») : Excel disparaît des chiffres, y compris pour le passé.
- [ ] « Effacer la dernière heure », « Effacer aujourd'hui », « Tout effacer » fonctionnent (double clic de confirmation).
- [ ] « Démarrer avec Windows » : après redémarrage du PC, la pastille et l'icône reviennent, sans ouvrir la fenêtre principale.
- [ ] Fermer la fenêtre principale ne coupe pas l'enregistrement ; « Quitter » dans le menu de l'icône l'arrête.

### Désinstallation
- [ ] Paramètres > Applications > SKELVIA Pulse > Désinstaller fonctionne.
- [ ] Le dossier `%LOCALAPPDATA%\SKELVIA Pulse` reste en place (comportement prévu) et peut être supprimé à la main.

## 3. En cas de problème

Notez la version de Windows (touche Windows, tapez « winver »), ce que vous avez fait, et ce qui s'est passé. Une capture d'écran aide. Ouvrez une « issue » sur le dépôt ou transmettez ces informations à SKELVIA.

## Pour un développeur : lancer en mode développement sous Windows

Prérequis : [prérequis Tauri 2 pour Windows](https://v2.tauri.app/start/prerequisites/) (outils de compilation Microsoft C++, WebView2), Rust et Node.js 20 ou plus.

```powershell
npm ci
npm run tauri dev
```
