# SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
# SPDX-License-Identifier: GPL-3.0-or-later
#
# This file is part of gdvm.
#
# gdvm is free software: you can redistribute it and/or modify it under the
# terms of the GNU General Public License as published by the Free Software
# Foundation, either version 3 of the License, or (at your option) any later
# version.
#
# gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
# A PARTICULAR PURPOSE. See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program. If not, see <https://www.gnu.org/licenses/>.

hello = Bonjour le monde !

help-about = Gestionnaire de Versions Godot
help-help = Afficher l'aide (voir un résumé avec '-h')
help-help-command = Afficher ce message ou l'aide de la(des) sous-commande(s) donnée(s)
help-gdvm-version = Afficher la version du Gestionnaire de Versions Godot

help-install = Installer une nouvelle version de Godot
help-run = Exécuter une version spécifique de Godot
help-show = Afficher le chemin de l'exécutable pour la version de Godot indiquée
help-cache-path = Afficher le chemin de l'archive de téléchargement en cache pour la version de Godot indiquée
help-link = Lier l'exécutable d'une version de Godot à un chemin spécifié
help-list = Lister toutes les versions installées de Godot
help-remove = Supprimer une version installée de Godot

help-branch = La branche (stable, beta, alpha, ou personnalisée).
help-csharp = [obsolète] Utiliser la version de Godot avec le support C#. Utilisez plutôt le spécificateur de variante « csharp » (ex. csharp:4.4).
help-run-csharp-long = { help-csharp }
help-version = La version à installer (ex. 4, csharp:4.4, stable, latest).
help-version-long =
    { help-version }

    Format : [variante:]version_ou_mot_clé

    Mots-clés : « latest » correspond à la version la plus récente. Par défaut, cela n'inclut que les versions stables, mais les pré-publications peuvent être incluses avec le drapeau --pre.

    Variantes : préfixez avec un nom de variante et deux-points, ex. « csharp:4.4 » pour la version C#.

    Exemples : 4.4 installera la dernière version stable de Godot 4.4. Si seules des versions de pré-publication existent, la dernière version de pré-publication sera installée. 4.3-rc installera la dernière version candidate de Godot 4.3, etc.
help-version-installed = La version installée (ex. 4.2 ou 4.2-stable).

help-search = Lister les versions disponibles depuis le registre
help-filter = Chaîne optionnelle pour filtrer les tags de versions
help-include-pre = Inclure les versions de pré-publication (rc, beta, dev)
help-cache-only = Utiliser uniquement les informations de versions en cache sans interroger le registre
help-limit = Nombre de versions à lister, par défaut 10. Utilisez 0 pour lister toutes
help-clear-cache = Vide le cache des versions
help-refresh = Actualiser le cache des versions depuis le registre
help-refresh-flag = Actualiser le cache des versions avant d'exécuter cette commande

help-prune = Supprimer les installations et les archives en cache qui ne sont plus utilisées
help-prune-long = { help-prune }

    Par défaut, prune supprime les installations qui n'ont pas été utilisées depuis un certain temps ainsi que les archives de téléchargement en cache devenues trop anciennes, tout en préservant toute installation encore référencée par un lien. L'installation définie comme défaut n'est jamais supprimée, quels que soient les drapeaux fournis. Le seuil d'ancienneté est configurable avec « gdvm config set prune.max-age-days <jours> » (par défaut { $default_days } jours).
help-prune-all = Supprimer toutes les installations et archives en cache quel que soit leur âge. Les installations encore référencées par un lien actif sont conservées sauf si --force est également fourni.
help-prune-force = Ignorer les liens, afin que les installations référencées uniquement par un lien puissent aussi être supprimées.
help-prune-dry-run = Afficher ce qui serait supprimé sans rien supprimer.

prune-dry-run-header = Les éléments suivants seraient supprimés (simulation) :
prune-removed-header = Éléments supprimés :
prune-installs-header = Installations :
prune-archives-header = Archives en cache :
prune-nothing-dry-run = Rien ne serait supprimé.
prune-nothing-removed = Rien à supprimer ; tout est utilisé ou dans le seuil d'ancienneté.
prune-preserved-by-link = { $count } installation(s) conservée(s) car encore référencée(s) par un lien.
prune-freed = Environ { $size } libéré(s).
prune-would-free = Environ { $size } seraient libéré(s).

help-force = Forcer la réinstallation même si la version est déjà installée.
help-redownload = Retélécharger la version même si elle est déjà présente dans le cache.
help-yes = Ignorer la confirmation de suppression
help-link-version = La version à lier. Si elle n'est pas fournie, la version est résolue en fonction du répertoire courant ou de la version par défaut.
help-link-path = Le chemin où le lien ou la copie sera créé, par exemple «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    } ».
help-link-force = Écraser le lien existant s'il existe
help-link-copy = Copier l'exécutable au lieu de créer un lien

cached-zip-stored = L'archive de cette version de Godot a été mise en cache.
using-cached-zip = Utilisation de l'archive de version en cache, téléchargement ignoré.
warning-cache-metadata-reset = Index de version en cache invalide ou corrompu. Remise à zéro.
cache-files-removed = Les fichiers du cache ont été supprimés avec succès.
cache-metadata-removed = Les métadonnées de cache ont été supprimées avec succès.
error-cache-metadata-empty = Erreur : Les métadonnées de cache sont vides, les versions doivent être récupérées.
no-cache-files-found = Aucun fichier de cache trouvé.
no-cache-metadata-found = Aucune métadonnée de cache trouvée.
gdvm-toml-malformed = {"\u001b"}[33mAvertissement : gdvm.toml à { $path } ignoré, car il n'a pas pu être analysé : { $error }{"\u001b"}[0m

help-console = Exécuter Godot avec la console attachée. Par défaut false sur Windows, true sur les autres plateformes.

help-default = Gérer la version par défaut
help-default-version = La version à définir par défaut (ex. 4.2 ou 4.2-stable).
no-default-set = Aucune version par défaut définie. Exécutez « gdvm use <version> » pour définir une version par défaut système, ou « gdvm pin <version> » pour définir une version par défaut pour le répertoire courant.

installing-version = Installation de la version {$version}
installed-success = {$version} installée avec succès

warning-prerelease = {"\u001b"}[33mAvertissement : Vous installez une version de pré-publication ({$branch}).{"\u001b"}[0m
warning-deprecated-csharp-flag = {"\u001b"}[33mAvertissement : Le drapeau --csharp est obsolète. Utilisez le spécificateur de variante "csharp" à la place (ex. csharp:4.4).{"\u001b"}[0m

force-reinstalling-version = Réinstallation forcée de la version {$version}.

auto-installing-version = Installation automatique de la version { $version }

no-versions-installed = Aucune version installée.
installed-versions = Versions installées de Godot :
removed-version = Version {$version} supprimée
removing-version = Suppression de la version {$version}

force-redownload = Retéléchargement forcé de la version {$version}.
operation-downloading-url = Téléchargement de {$url}...
operation-download-complete = Téléchargement terminé.
operation-extracting = Extraction...
operation-extract-complete = Extraction terminée.

unsupported-platform = Plateforme non prise en charge
unsupported-architecture = Architecture non prise en charge

verifying-checksum = Vérification de la somme de contrôle...
checksum-verified = Somme de contrôle vérifiée.
error-checksum-mismatch = Incompatibilité de somme de contrôle pour le fichier { $file }
error-invalid-sha-length = Longueur SHA invalide { $length }
warning-sha-sums-missing = Fichiers de somme de contrôle introuvables pour cette version. Vérification ignorée.

error-find-user-dirs = Échec de la recherche des répertoires utilisateur.

fetching-releases = Récupération des versions...
releases-fetched = Versions récupérées.
error-fetching-releases = Erreur lors de la récupération des versions : { $error }
warning-fetching-releases-using-cache = Erreur lors de la récupération des versions : { $error }. Utilisation des versions en cache à la place.

error-version-not-found = Version introuvable.
error-archive-not-cached = Aucune archive en cache trouvée pour {$version}. Installez-la d'abord pour remplir le cache.
error-multiple-versions-found = Plusieurs versions correspondent à votre demande :

running-version = Exécution de la version {$version}
link-created = Lien créé de {$version} vers {$path}
copy-created = Copie de {$version} vers {$path} effectuée
no-matching-releases = Aucune version correspondante trouvée.
available-releases = Versions disponibles :
cache-cleared = Cache vidé avec succès.
cache-refreshed = Cache actualisé avec succès.

version-already-installed = Version {$version} déjà installée.
godot-executable-not-found = Exécutable Godot introuvable pour la version {$version}.
error-link-exists = Le chemin {$path} existe déjà. Utilisez --force pour écraser.
error-link-symlink = Échec de la création du lien de {$link} vers {$target} : {$error}
error-link-copy = Échec de la copie de l'exécutable : {$error}

error-no-stable-releases-found = Aucune version stable trouvée.

error-starting-godot = Échec du démarrage de Godot : { $error }

confirm-remove = Êtes-vous sûr de vouloir supprimer cette version ? (oui/non) :
confirm-yes = oui
remove-cancelled = Suppression annulée.

default-set-success = {$version} définie avec succès comme version par défaut de Godot.
default-unset-success = Version par défaut de Godot supprimée avec succès.
provide-version-or-unset = Veuillez fournir une version à définir par défaut ou 'unset' pour supprimer la version par défaut.

error-open-zip = Échec de l'ouverture du fichier ZIP { $path } : { $error }
error-read-zip = Échec de la lecture de l'archive ZIP { $path } : { $error }
error-access-file = Échec de l'accès au fichier à l'index { $index } : { $error }
error-reopen-zip = Échec de la réouverture du fichier ZIP { $path } : { $error }
error-invalid-file-name = Nom de fichier invalide dans l'archive ZIP
error-create-dir = Échec de la création du répertoire { $path } : { $error }
error-create-file = Échec de la création du fichier { $path } : { $error }
error-read-zip-file = Échec de la lecture du fichier ZIP { $file } : { $error }
error-write-file = Échec de l'écriture du fichier { $path } : { $error }
error-strip-prefix = Erreur lors de la suppression du préfixe : { $error }
error-set-permissions = Échec de la définition des permissions pour { $path } : { $error }
error-create-symlink-windows = Impossible de créer le lien symbolique. Veuillez vous assurer que le {"\u001b"}]8;;ms-settings:developers{"\u001b"}\Mode Développeur{"\u001b"}]8;;{"\u001b"}\ est activé ou exécutez en tant qu'administrateur.

help-upgrade = Mettre à jour gdvm vers la dernière version
help-upgrade-major = Autoriser la mise à jour entre versions majeures
upgrade-starting = Démarrage de la mise à jour de gdvm...
upgrade-downloading-latest = Téléchargement de la dernière version de gdvm...
upgrade-complete = gdvm a été mis à jour avec succès !
upgrade-not-needed = gdvm est déjà à la dernière version : { $version }.
upgrade-current-version-newer = La version actuelle de gdvm ({ $current }) est plus récente que la dernière version disponible ({ $latest }). Aucune mise à jour nécessaire.
upgrade-failed = Échec de la mise à jour : { $error }
upgrade-download-failed = Échec du téléchargement de la mise à jour : { $error }
upgrade-file-create-failed = Échec de la création du fichier de mise à jour : { $error }
upgrade-file-write-failed = Échec de l'écriture du fichier de mise à jour : { $error }
upgrade-install-dir-failed = Échec de la création du répertoire d'installation : { $error }
upgrade-rename-failed = Échec du renommage de l'exécutable actuel : { $error }
upgrade-replace-failed = Échec du remplacement de l'exécutable par le nouveau : { $error }
checking-updates = Vérification des mises à jour de gdvm...
upgrade-available = 💡 Une nouvelle version de gdvm est disponible : {$version}. Exécutez « gdvm upgrade » pour mettre à jour.
upgrade-available-major = 💡 Une mise à jour de version majeure de gdvm est disponible : {$version}. Exécutez « gdvm upgrade -m » pour mettre à jour.
upgrade-available-both = 💡 Une nouvelle version de gdvm est disponible : {$minor_version}. Une mise à jour de version majeure est également disponible : {$major_version}. Exécutez « gdvm upgrade » pour mettre à jour dans la version majeure actuelle, ou « gdvm upgrade -m » pour mettre à jour vers la dernière version.

help-pin = Épingler une version de Godot au répertoire courant.
help-pin-long = { help-pin }

    Cela créera un fichier gdvm.toml dans le répertoire courant avec la version épinglée. Lorsque vous exécutez « gdvm run » dans ce répertoire ou dans l'un de ses sous-répertoires, la version épinglée sera utilisée au lieu de la version par défaut.

    Ceci est utile lorsque vous voulez utiliser une version spécifique de Godot pour un projet sans changer la version par défaut du système.

    Actuellement, cela écrit aussi le fichier .gdvmrc hérité pour la compatibilité avec les anciennes versions de gdvm. Cela sera supprimé dans une future version, il est donc recommandé de passer au nouveau format gdvm.toml et de supprimer le fichier .gdvmrc s'il existe.

    Vous pouvez désactiver l'écriture du fichier .gdvmrc avec le drapeau --no-legacy.
help-pin-version = La version à épingler
help-no-legacy = Ne pas écrire le fichier de compatibilité hérité .gdvmrc
pinned-success = Version {$version} épinglée avec succès dans gdvm.toml
error-pin-version-not-found = Impossible d'épingler la version {$version}
pin-subcommand-description = Définir ou mettre à jour gdvm.toml avec la version demandée

error-file-not-found = Fichier introuvable. Il peut ne pas exister sur le serveur.
error-download-failed = Échec du téléchargement dû à une erreur inattendue : { $error }
error-ensure-godot-binaries-failed = Échec de l'assurance des binaires Godot.
    Erreur : { $error }.
    Essayez de supprimer { $path } puis exécutez gdvm à nouveau.

error-failed-reading-project-godot = Échec de la lecture de project.godot, impossible de déterminer automatiquement la version du projet.
warning-using-project-version = Utilisation de la version { $version } définie dans project.godot.

warning-project-version-mismatch =
    {"\u001b"}[33mAvertissement : La version définie dans project.godot ne correspond pas à la version { $pinned ->
        [1] épinglée
        *[0] demandée
    }. Ouvrir le projet avec la version { $pinned ->
        [1] épinglée
        *[0] demandée
    } peut écraser le fichier de projet.{"\u001b"}[0m

    { $pinned ->
        [1] Version du projet : { $project_version }
            Version épinglée :  { $requested_version }
        *[0] Version du projet :   { $project_version }
             Version demandée : { $requested_version }
    }

error-project-version-mismatch = {"\u001b"}[31m{ $pinned ->
        [1] Si vous êtes sûr de vouloir exécuter le projet avec la version épinglée, exécutez {"\u001b"}[0m« gdvm run --force »{"\u001b"}[31m. Sinon, mettez à jour la version épinglée dans .gdvmrc pour correspondre à la version du projet, ou supprimez le fichier .gdvmrc pour utiliser la version du projet.
        *[0] Si vous êtes sûr de vouloir exécuter le projet avec la version demandée, exécutez {"\u001b"}[0m« gdvm run --force <version> »{"\u001b"}[31m.
    }{"\u001b"}[0m
warning-project-version-mismatch-force = {"\u001b"}[33mIgnoration de l'invite de confirmation et continuation avec la version { $pinned ->
        [1] épinglée
        *[0] demandée
    } {"\u001b"}[0m({ $requested_version }){"\u001b"}[33m.{"\u001b"}[0m

help-run-args = Arguments supplémentaires à passer à l'exécutable Godot (ex. -- path/to/project.godot).
help-run-force =
    Forcer l'exécution du projet avec la version demandée ou épinglée même si elle ne correspond pas à la version du projet.
help-run-force-long =
    { help-run-force }

    Si vous faites cela, la version demandée ou épinglée de Godot peut écraser le fichier de projet. Si vous épinglez des versions, il est plutôt recommandé de mettre à jour la version épinglée dans .gdvmrc pour correspondre à la version du projet, ou de supprimer le fichier .gdvmrc pour utiliser la version du projet.

help-config = Gérer la configuration gdvm
help-config-get = Obtenir une valeur de configuration
help-config-set = Définir une valeur de configuration
help-config-unset = Supprimer une valeur de configuration
help-config-list = Lister toutes les valeurs de configuration
help-config-key = La clé de configuration (ex., github.token)
help-config-value = La valeur à définir pour la clé de configuration
help-config-unset-key = La clé de configuration à supprimer (ex., github.token)
help-config-show-sensitive = Rendre visible les valeurs de configuration sensibles
help-config-available = Lister toutes les clés de configuration disponibles et leurs valeurs, y compris les valeurs par défaut
warning-setting-sensitive = {"\u001b"}[33mAvertissement : Vous définissez une valeur sensible qui sera stockée en texte brut dans votre répertoire personnel.{"\u001b"}[0m
config-set-prompt = Veuillez entrer la valeur pour { $key } :
error-reading-input = Erreur lors de la lecture de l'entrée
config-set-success = Configuration mise à jour avec succès.
config-unset-success = Clé de configuration { $key } supprimée avec succès.
config-key-not-set = Clé de configuration non définie.
error-unknown-config-key = Clé de configuration inconnue.
error-invalid-config-value = Valeur invalide pour la clé de configuration { $key }.
error-invalid-config-subcommand = Sous-commande de configuration invalide. Utilisez « get », « set », ou « list ».
error-parse-config = Échec de l'analyse du fichier de configuration : { $error }
error-parse-config-using-default = {"\u001b"}[33mUtilisation des valeurs de configuration par défaut.{"\u001b"}[0m
error-github-api = Erreur de l'API GitHub : { $error }
error-github-rate-limit = Limite d'utilisation de l'API GitHub dépassée.

  Pour résoudre cela, veuillez créer un jeton d'accès personnel sur GitHub en visitant https://github.com/settings/tokens.

  Cliquez sur « Generate new token », sélectionnez uniquement les permissions minimales requises (ex. public_repo), puis définissez le jeton via la variable d'environnement GITHUB_TOKEN ou en exécutant :

    gdvm config set github.token

  Note : Le jeton sera stocké en texte brut dans votre répertoire personnel. Veuillez vous assurer de le garder sécurisé.
  Il est recommandé de régulièrement examiner et faire tourner vos jetons pour des raisons de sécurité.

error-copy-file-failed = Échec de la copie du fichier : { $error }
error-move-file-failed = Échec du déplacement du fichier : { $error }
error-user-dir-not-found = Impossible de créer le raccourci : répertoire utilisateur introuvable
error-desktop-not-found = Impossible de créer le raccourci : répertoire Bureau introuvable
help-launch-shortcut = Crée un raccourci pour un accès rapide aux instances de Godot en cours d'exécution (sur le bureau et dans le menu Démarrer sous Windows ; dans le répertoire ~/.local/share/applications sous Linux ; dans le dossier Applications sous macOS).
error-base-dir-not-found = Dossier de base non trouvé
help-registry = Gérer les registres depuis lesquels installer des builds de Godot
help-registry-add = Ajouter un registre
help-registry-remove = Supprimer un registre
help-registry-list = Lister les registres configurés
help-registry-refresh = Actualiser le cache d'un registre ou de tous
help-registry-name = Le nom du registre
help-registry-url = L'URL du registre. Peut être une URL http(s):// ou file://.

registry-added = Registre { $registry } ajouté ({ $url }).
registry-removed = Registre { $registry } supprimé.
registry-list-header = Registres configurés :
registry-tag-official = officiel
registry-error = Erreur de registre : { $error }

error-invalid-registry-subcommand = Sous-commande de registre invalide. Utilisez « add », « remove », « list » ou « refresh ».
registry-trust-warning = {"\u001b"}[33m{ $registry } ({ $url }) est un registre personnalisé, pas le registre officiel. gdvm vérifie que les téléchargements correspondent à ce que le registre annonce, mais il ne peut pas savoir s'ils sont sûrs à exécuter. Ne l'utilisez que si vous faites confiance à la personne qui le gère.{"\u001b"}[0m
registry-trust-prompt = Faites-vous confiance à ce registre et voulez-vous continuer ? (oui/non) :
registry-trust-bypass = {"\u001b"}[1;31mVérification de confiance ignorée pour { $registry } ({ $url }) parce que vous avez utilisé --yes. gdvm ne peut pas savoir si ses fichiers sont sûrs à exécuter. Petite pause ; appuyez sur Ctrl+C maintenant pour arrêter.{"\u001b"}[0m
registry-trust-aborted = Annulé : registre non approuvé.
registry-project-override-conflict = {"\u001b"}[33mLe fichier gdvm.toml du projet redéfinit le registre { $registry } (votre configuration : { $machine_url }) en { $project_url }. La définition du projet prévaut.{"\u001b"}[0m

help-registry-init = Initialiser un nouveau répertoire de registre
help-registry-add-build = Ajouter un build à un registre
help-registry-remove-build = Supprimer un build d'un registre
help-registry-validate = Valider un répertoire de registre
help-registry-dir = Le répertoire du registre
help-registry-init-name = Le nom du registre. Par défaut le nom du répertoire.

help-registry-build-version = L'étiquette de version, p. ex. 4.4-stable.
help-registry-build-variant = Le nom de la variante. Par défaut « default ».
help-registry-build-platform = La clé de plateforme, p. ex. linux-x86_64.
help-registry-build-file = Chemin de l'archive de build à hacher
help-registry-build-store = Copier l'archive dans le registre et enregistrer une URL relative
help-registry-build-url = L'URL où l'archive sera servie (si --store n'est pas utilisé)
help-registry-build-sha512 = Le SHA-512 de l'archive, au lieu de le calculer. Nécessite --size.
help-registry-build-size = La taille de l'archive en octets, au lieu de la mesurer. Nécessite --sha512.

registry-init-success = Registre { $name } initialisé dans { $path }.
registry-build-added = Build { $version } ajouté pour { $platform }.
registry-build-removed = Build { $version } supprimé.
registry-build-downloading = Téléchargement de { $url } pour calculer sa taille et son SHA-512…
registry-build-warn-local-hash = {"\u001b"}[33mHachage du fichier local en supposant qu'il correspond à { $url }. gdvm ne télécharge pas l'URL pour le vérifier.{"\u001b"}[0m
registry-build-warn-unverified = {"\u001b"}[33mUtilisation du SHA-512 et de la taille que vous avez fournis sans télécharger l'artefact pour les vérifier. Vérifiez qu'ils sont corrects.{"\u001b"}[0m
registry-build-warn-explicit-store = {"\u001b"}[33mUtilisation du SHA-512 et/ou de la taille que vous avez fournis au lieu de mesurer l'archive stockée.{"\u001b"}[0m
registry-build-sha-mismatch = Le SHA-512 fourni ({ $expected }) ne correspond pas à l'artefact ({ $actual }).
registry-build-size-mismatch = La taille fournie ({ $expected }) ne correspond pas à l'artefact ({ $actual }).
registry-validate-ok = Le registre est valide ({ $count } artefacts vérifiés).
registry-validate-failed = Échec de la validation du registre :
