# SKELVIA Pulse

**Voir où part votre temps sur votre PC. Sans rien envoyer.**

SKELVIA Pulse est un petit logiciel gratuit et libre pour Windows 10 et 11. Il tourne en arrière-plan et mesure, **sur votre PC uniquement**, le temps passé dans chaque application, sur chaque site et dans chaque document. Il range ce temps par type de tâche (devis, factures, mails, plans et CCTP, recherche de fournisseurs…) et vous donne un rapport clair, par exemple : « 6 h par semaine sur les mails, 3 h sur les devis, 2 h sur des PDF de CCTP ».

Il est proposé par [SKELVIA](https://skelvia.fr), qui installe des assistants IA administratifs dans les TPE et PME du bâtiment.

> ⚠️ Version 0.1, en cours de test. Elle n'a pas encore été essayée sur un vrai PC Windows : voir [Tester sur Windows](docs/tester-sur-windows.md).

---

## En bref

- **Tout reste sur votre PC.** Pas de compte, pas de serveur SKELVIA, aucune télémétrie. Le rapport est un fichier que vous gardez, imprimez ou transmettez, comme vous voulez.
- **Pas d'espionnage.** Pas de capture d'écran, pas de touches tapées, pas de contenu de mails ou de documents.
- **Toujours visible.** Une pastille, que vous placez où vous voulez sur l'écran, montre l'état (« Enregistre » ou « En pause ») avec un bouton Pause. Une icône reste près de l'horloge de Windows.
- **Aucun réglage obligatoire.** On installe, on lit l'écran d'accueil, c'est parti.

## Installer

1. Téléchargez le fichier **`SKELVIA Pulse_0.1.0_x64-setup.exe`** (lien donné par SKELVIA, ou page « Releases » de ce dépôt quand elle sera publique).
2. Double-cliquez dessus. L'installation se fait pour votre compte Windows seulement et ne demande pas de mot de passe administrateur.
3. **Windows affiche sans doute un avertissement bleu « Windows a protégé votre ordinateur »** (SmartScreen). C'est normal pour un logiciel récent qui n'est pas encore signé numériquement : voir [l'avertissement SmartScreen](#lavertissement-smartscreen) ci-dessous.
4. Au premier lancement, un écran explique ce qui est enregistré et ce qui ne l'est pas. Rien n'est enregistré avant que vous cliquiez sur « J'ai compris, commencer ».

Si votre navigateur signale le téléchargement comme « peu fréquent », choisissez « Conserver ». Le libellé exact dépend du navigateur.

Windows 10 et 11 utilisent un composant de Microsoft, WebView2, pour afficher les fenêtres. Il est déjà présent sur la plupart des PC. S'il manque, l'installeur le télécharge depuis Microsoft : c'est la seule connexion à internet de l'installation.

### L'avertissement SmartScreen

Comme SKELVIA Pulse n'est pas encore signé numériquement, Windows peut afficher :

> **Windows a protégé votre ordinateur**

Pour continuer :

1. cliquez sur **« Informations complémentaires »** (selon la version de Windows, le lien peut s'appeler « Plus d'infos ») ;
2. vérifiez que l'éditeur affiché est « Éditeur inconnu » et que le nom du fichier est bien celui que vous avez téléchargé ;
3. cliquez sur **« Exécuter quand même »**.

Ne faites cela que pour un fichier téléchargé depuis une source que vous connaissez (le lien donné par SKELVIA ou ce dépôt). Le code source est public : n'importe qui peut vérifier ce que fait le logiciel.

## Ce qui est enregistré

| Enregistré | Exemple |
|---|---|
| L'application **au premier plan** | Excel, Outlook, Acrobat |
| Le **titre de sa fenêtre** | « Devis Martin toiture.xlsx » |
| Le **site** de l'onglet actif, si vous ajoutez l'extension au navigateur | boamp.fr, et le titre de la page |
| La **durée**, et si la souris ou le clavier ont servi (oui ou non) | 12 min, actif |

Le titre d'une fenêtre peut contenir des informations sensibles (le sujet d'un mail, le nom d'un client). Vous pouvez arrêter d'enregistrer les titres, ou exclure des applications et des sites (voir plus bas).

## Ce qui n'est jamais enregistré

- Aucune capture d'écran.
- Aucune touche tapée, aucun texte saisi, aucun clic, aucun mouvement de souris. Windows indique seulement l'heure de la dernière action : c'est ce qui permet de savoir si vous êtes actif.
- Aucun contenu de vos mails, documents ou pages web.
- Les fenêtres ouvertes en arrière-plan : seule celle que vous utilisez compte.
- La navigation privée : non comptée, et ce que l'extension en envoie est effacé.
- Rien n'est envoyé sur internet.

## Comment le temps est compté

- Seul compte le temps où **une fenêtre est au premier plan** et où **la souris ou le clavier ont servi** depuis moins de **3 minutes** (réglable : 1, 2, 3, 5, 10 ou 15 minutes).
- Passé ce délai, le temps n'est plus compté, même si des fenêtres restent ouvertes. Le délai d'attente lui-même n'est pas compté non plus : l'inactivité démarre à la dernière action.
- Session Windows verrouillée : le temps n'est pas compté (détection à confirmer pendant les tests sur Windows).
- La lecture d'un long document sans toucher la souris peut donc être un peu sous-estimée. Un délai plus long corrige en partie cet effet.

## Les écrans

1. **La pastille** : mascotte, état, bouton Pause. Au survol, elle montre le temps compté aujourd'hui. Elle apparaît en haut au centre de l'écran ; **faites-la glisser à la souris pour la placer où vous voulez**, elle garde sa place même après un redémarrage. Pour la remettre en haut : Réglages, ou le menu de l'icône près de l'horloge.
2. **Aujourd'hui** : temps par type de tâche, par application, par site, et jour par jour sur 7 ou 30 jours.
3. **Rapport** : bouton « Télécharger mon rapport ». Un fichier HTML d'une page, lisible sans internet, est écrit dans votre dossier Téléchargements. Par défaut, il ne contient pas le nom de vos documents (ils peuvent contenir des noms de clients) : c'est une case à cocher.
4. **Réglages** : délai d'inactivité, titres, exclusions, règles de regroupement, extension navigateur, effacement des données.

## Les types de tâche

Le temps est rangé par des **règles simples**, modifiables dans Réglages :

- **mot dans le titre** : « devis » range « Devis Martin.xlsx » dans *Devis et chiffrage* ;
- **site web** : « pointp.fr » range ce site dans *Recherche de fournisseurs* ;
- **application** : « outlook » range Outlook dans *Mails*.

Les règles sont lues de haut en bas, la première qui correspond l'emporte. Le temps qu'aucune règle ne range apparaît dans « Autre ». Aucune intelligence artificielle extérieure n'est utilisée : tout se fait sur le PC. Les règles proposées au départ sont des exemples à adapter à votre entreprise.

## Voir les sites web (facultatif)

Pour connaître le site consulté (et pas seulement le titre de la fenêtre du navigateur), ajoutez l'extension gratuite **« ActivityWatch Web Watcher »**. C'est l'extension officielle d'ActivityWatch, le moteur libre sur lequel repose SKELVIA Pulse. Les boutons « Pour Chrome », « Pour Edge » et « Pour Firefox » de l'application ouvrent la bonne page :

- **Chrome** : [Chrome Web Store](https://chromewebstore.google.com/detail/activitywatch-web-watcher/nglaklhklhcoonedhgnpgddginnjdadi), puis « Ajouter à Chrome » ;
- **Edge** : la même page du Chrome Web Store. Edge peut demander d'autoriser les extensions provenant d'autres boutiques (comportement à confirmer sur votre version d'Edge) ;
- **Firefox** : [Firefox Add-ons](https://addons.mozilla.org/fr/firefox/addon/aw-watcher-web/). L'extension demande votre accord une fois.

L'extension envoie ses données seulement à votre PC (adresse `127.0.0.1`, port 5600). Pendant la pause, SKELVIA Pulse les refuse.

Si ActivityWatch est déjà installé sur le PC, il occupe ce port : l'extension enverra alors ses données à ActivityWatch et non à SKELVIA Pulse. L'application l'indique dans Réglages. Le temps par application reste mesuré normalement.

## Garder la main : pause, exclusions, effacement

- **Pause** : bouton de la pastille, de la fenêtre principale ou de l'icône près de l'horloge. Rien n'est enregistré pendant la pause, et la pause est conservée au redémarrage du PC.
- **Exclusions** (Réglages) : une application, un site ou un mot du titre. Ce qui correspond n'est **jamais** enregistré, et ce qui l'avait été est **effacé**. Pour les sites, l'effacement se fait dans la minute qui suit, et la fenêtre du navigateur correspondante est effacée aussi.
- **Ne pas enregistrer les titres** (Réglages) : seuls les noms d'applications et de sites sont gardés.
- **Effacer** (Réglages) : la dernière heure, aujourd'hui, ou tout.

### Tout effacer, définitivement

1. Dans Réglages, cliquez sur « Tout effacer ». Si vous le souhaitez, décochez aussi « Démarrer avec Windows ».
2. Désinstallez SKELVIA Pulse : *Paramètres* > *Applications* > *SKELVIA Pulse* > *Désinstaller*.
3. Supprimez le dossier **`%LOCALAPPDATA%\SKELVIA Pulse`** (tapez ce chemin dans la barre d'adresse de l'Explorateur de fichiers). Il contient la base de données et les réglages.
4. Supprimez aussi, s'il existe, le dossier **`%LOCALAPPDATA%\fr.skelvia.pulse`**. Il contient le cache d'affichage de Windows (WebView2), pas vos mesures.

## Installer SKELVIA Pulse sur le PC d'un salarié

C'est possible, mais un dirigeant qui l'installe sur le PC d'un salarié a, selon toute vraisemblance, des obligations d'**information** et de **proportionnalité** (RGPD, recommandations de la CNIL, Code du travail). Lisez [Usage en entreprise](docs/usage-en-entreprise.md) et faites valider votre projet. Ce document n'est pas un avis juridique.

## Pour aller plus loin

- [Usage en entreprise (RGPD, CNIL)](docs/usage-en-entreprise.md)
- [Tester sur Windows](docs/tester-sur-windows.md)
- [Fonctionnement technique](docs/architecture.md)
- [Contribuer](CONTRIBUTING.md)

## Licence

- Code : [licence MIT](LICENSE).
- Nom SKELVIA, logo et mascotte : réservés, voir [TRADEMARKS.md](TRADEMARKS.md).
- Composants de tiers (ActivityWatch sous MPL-2.0, polices sous SIL OFL 1.1, Tauri…) : voir [THIRD_PARTY.md](THIRD_PARTY.md).

---

Pour analyser vos chiffres avec Adrien, vous pouvez [réserver un audit de 45 minutes gratuit](https://skelvia.fr/#reserver). Sans engagement : SKELVIA Pulse reste gratuit et complet sans cela.
