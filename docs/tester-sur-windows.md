# Tester sur Windows

SKELVIA Pulse a été écrit et vérifié sous Linux : le code se compile, les tests automatiques passent, et le collecteur Windows est vérifié par le compilateur pour la cible Windows. **Il n'a encore jamais tourné sur un vrai PC Windows.** Cette page explique comment l'essayer et quoi vérifier.

## 1. Obtenir l'installeur

Le dépôt contient un workflow GitHub Actions (`.github/workflows/ci.yml`) qui compile l'installeur sur une machine Windows de GitHub :

1. Sur GitHub, ouvrez l'onglet **Actions** du dépôt, puis le dernier passage réussi de « CI ».
2. En bas de la page, section **Artifacts**, téléchargez **`skelvia-pulse-windows`** (il faut être connecté à GitHub).
3. Décompressez le fichier zip. Vous y trouvez :
   - `SKELVIA Pulse_<version>_x64-setup.exe` (installeur conseillé, sans droits administrateur) ;
   - `SKELVIA Pulse_<version>_x64_fr-FR.msi` (installeur MSI, pour un déploiement par un informaticien).

Ces installeurs d'essai fonctionnent normalement. Les versions publiées sur la page de téléchargement (« Releases ») sont les mêmes, avec en plus les fichiers signés qui servent aux mises à jour automatiques.

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
- [ ] Avant ce clic, aucune vérification de mise à jour n'a lieu.
- [ ] L'icône près de l'horloge est visible (parfois dans la flèche « icônes cachées » de la barre des tâches).

### Mesure
- [ ] Travaillez 10 minutes dans Excel, Outlook, un PDF : le temps apparaît dans Aujourd'hui (mis à jour chaque minute).
- [ ] Le nom des applications est lisible (Excel, Outlook, Adobe Acrobat…).
- [ ] Laissez le PC sans toucher la souris plus de 3 minutes : la pastille passe en « en veille » (yeux fermés) et ce temps n'est pas compté.
- [ ] Verrouillez la session (touche Windows + L) pendant 2 minutes : ce temps n'est pas compté.
- [ ] Applications du Microsoft Store (Courrier, Calculatrice…) : leur vrai nom apparaît, et pas « ApplicationFrameHost ».
- [ ] Une application lancée « en tant qu'administrateur » : son nom apparaît-il ? (cas connu comme délicat)

### Pastille
- [ ] C'est une toute petite barre (mascotte et temps du jour). Passer la souris dessus ne change rien.
- [ ] Un clic l'ouvre : temps du jour, état, Pause, « Ouvrir l'application ». Un autre clic, Échap, ou un clic dans une autre fenêtre la referme.
- [ ] « Ouvrir l'application » ouvre la fenêtre principale.
- [ ] Le bouton Pause passe à « Reprendre » (mascotte en veille, « En pause » dans la barre), et plus rien n'est compté.
- [ ] Cliquer, maintenir et glisser la barre la déplace, y compris sur un autre écran, sans l'ouvrir.
- [ ] Lâchée, elle se colle au bord le plus proche : en haut, en bas (au-dessus de la barre des tâches, pas dessous), à gauche, à droite, et reste à l'endroit choisi le long de ce bord.
- [ ] À gauche et à droite, elle devient verticale ; ouverte, son panneau s'affiche à côté, à sa hauteur. En haut et en bas, elle est horizontale, sans espace vide.
- [ ] Ouverte près du bas de l'écran, elle se déploie vers le haut, sans sortir de l'écran. Contre le bord gauche ou droit, le panneau reste sur l'écran.
- [ ] Elle reste au-dessus des autres fenêtres sans gêner (vérifier avec une fenêtre en plein écran, et sur un écran avec mise à l'échelle 125 % ou 150 %).
- [ ] Après avoir quitté puis relancé SKELVIA Pulse, elle revient à sa place.
- [ ] « Remettre la pastille en haut de l'écran » (Réglages, ou menu de l'icône) la ramène en haut au centre.
- [ ] Pour un gaucher qui a inversé les boutons de la souris : le collage au bord se fait bien au relâchement.

### Mises à jour
Ces vérifications demandent deux versions publiées (par exemple 0.2.0 installée, puis 0.2.1 publiée).
- [ ] Dans Réglages > Mises à jour, la version installée est affichée, et « Vérifier maintenant » répond (« dernière version » ou « version … disponible »).
- [ ] Après publication d'une version plus récente : au plus tard au redémarrage de SKELVIA Pulse (vérification 30 secondes après le démarrage), la pastille affiche « mise à jour ».
- [ ] Un clic sur « mise à jour » : téléchargement (pourcentage dans la pastille), fermeture, petite fenêtre de progression de l'installeur, puis relance automatique. Noter si SmartScreen ou un antivirus intervient.
- [ ] Après la relance : la nouvelle version est affichée dans Réglages, les mesures d'avant sont toujours là, la pastille est à sa place.
- [ ] Sans internet : « Vérifier maintenant » affiche un message clair, sans blocage.
- [ ] Option décochée : plus de vérification automatique.

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
