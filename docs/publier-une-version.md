# Publier une version

Une version publiée, c'est une page de téléchargement GitHub (« Release ») avec :

- l'installeur `skelvia-pulse_<version>_x64-setup.exe` (conseillé) ;
- l'installeur `skelvia-pulse_<version>_x64_fr-FR.msi` (déploiement par un informaticien) ;
- `latest.json`, le petit fichier que SKELVIA Pulse lit pour proposer la mise à jour dans sa pastille.

Tout est fait par le workflow `.github/workflows/publication.yml`, déclenché par une étiquette Git `v<version>`.

## Une fois pour toutes : les réglages du dépôt

Les libellés de GitHub ci-dessous sont ceux connus à la rédaction : à vérifier, ils changent parfois.

### 1. La clé des mises à jour

SKELVIA Pulse n'installe une mise à jour que si elle est signée avec la clé privée de SKELVIA. La clé publique correspondante est inscrite dans `src-tauri/tauri.conf.json` (`plugins > updater > pubkey`).

- **Clé privée** : dans le dépôt GitHub, *Settings* > *Secrets and variables* > *Actions* > *New repository secret*, nom **`TAURI_SIGNING_PRIVATE_KEY`**, valeur : le contenu complet du fichier `.key`.
- **Gardez une copie** de ce fichier dans un endroit sûr (gestionnaire de mots de passe, par exemple). GitHub ne permet pas de relire un secret.
- **Ne la publiez jamais**, ne la mettez jamais dans le dépôt.

Si la clé est perdue : générez-en une nouvelle (`npx tauri signer generate -w skelvia-pulse.key`), remplacez la clé publique dans `tauri.conf.json` et le secret. Les PC déjà installés ne pourront pas installer la version suivante tout seuls : il faudra la réinstaller une fois à la main.

Si la clé a fuité : même démarche, au plus vite. Pour diffuser une fausse mise à jour, il faudrait aussi pouvoir publier sur la page de téléchargement : protégez le compte GitHub (double authentification).

La clé actuelle n'a pas de mot de passe. Si vous en ajoutez un, créez aussi le secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

### 2. Où sont publiés les installeurs

L'application lit `latest.json` à l'adresse inscrite dans `src-tauri/tauri.conf.json` (`plugins > updater > endpoints`). Cette adresse doit être **publique** : un PC ne peut pas lire un dépôt privé.

- **Si ce dépôt est public** : rien d'autre à régler. Le workflow publie ici avec le jeton automatique de GitHub.
- **Si les installeurs sont publiés dans un autre dépôt, public** (le code restant privé) :
  1. créez ce dépôt avec au moins un fichier (un README), sinon GitHub ne peut pas y créer de page de téléchargement ;
  2. créez un jeton : *Settings* (de votre compte) > *Developer settings* > *Personal access tokens* > *Fine-grained tokens* > *Generate new token*, accès au seul dépôt des installeurs, droit *Contents : Read and write* ;
  3. dans ce dépôt-ci : secret **`RELEASES_TOKEN`** (le jeton) et variable **`RELEASES_REPO`** (par exemple `adrientavet/skelvia-pulse-releases`) dans *Secrets and variables* > *Actions* ;
  4. le jeton expire : notez la date et renouvelez-le avant.

## À chaque version

1. Décrivez les nouveautés dans `NOUVEAUTES.md`, sous un titre `## 0.2.1` (phrases simples, pour un dirigeant). Ce texte est repris sur la page de téléchargement.
2. Changez le numéro de version, le même dans les trois fichiers :
   - `src-tauri/tauri.conf.json` (`"version"`) ;
   - `Cargo.toml` (`[workspace.package] version`) ;
   - `package.json` (`"version"`), puis `npm install --package-lock-only` et `cargo update -w` pour les fichiers de verrouillage.
3. Vérifiez : `cargo test --workspace` et `npm run build`.
4. Enregistrez et publiez :

   ```sh
   git commit -am "Version 0.2.1"
   git tag v0.2.1
   git push origin main v0.2.1
   ```

5. Suivez le workflow « Publication » dans l'onglet *Actions*. Il refuse de publier si l'étiquette ne correspond pas à la version, si `NOUVEAUTES.md` n'a pas la section, ou si la clé manque.

Les PC qui ont SKELVIA Pulse verront « mise à jour » dans la pastille au plus tard 6 heures après, ou au prochain démarrage.

Le numéro de version doit toujours augmenter (`0.2.0` → `0.2.1` → `0.3.0`) : l'application ignore une version égale ou plus ancienne.

## Ce qui n'est pas encore vérifié

- Le parcours complet de mise à jour sur un vrai PC Windows (voir [Tester sur Windows](tester-sur-windows.md)).
- La réaction de SmartScreen et des antivirus quand l'installeur est lancé par l'application (l'installeur n'étant pas signé par un certificat de signature de code).
