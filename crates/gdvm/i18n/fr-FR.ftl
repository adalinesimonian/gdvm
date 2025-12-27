hello = Bonjour le monde !

help-about = Gestionnaire de Versions Godot
help-help = Afficher l'aide (voir un résumé avec '-h')
help-help-command = Afficher ce message ou l'aide de la(des) sous-commande(s) donnée(s)
help-gdvm-version = Afficher la version du Gestionnaire de Versions Godot

help-install = Installer une nouvelle version de Godot
help-run = Exécuter une version spécifique de Godot
help-show = Afficher le chemin de l'exécutable pour la version de Godot indiquée
help-link = Lier l'exécutable d'une version de Godot à un chemin spécifié
help-list = Lister toutes les versions installées de Godot
help-remove = Supprimer une version installée de Godot

help-branch = La branche (stable, beta, alpha, ou personnalisée).
help-csharp = Utiliser la version de Godot avec le support C#.
help-run-csharp-long = { help-csharp }

    Si spécifié, la valeur remplace la version par défaut définie avec « use ». Sinon, la version par défaut est utilisée. En d'autres termes, si vous définissez une version par défaut avec « use --csharp », vous pouvez essayer d'exécuter la même version mais sans le support C# avec « run --csharp false ». Cependant, cela peut ne pas fonctionner comme attendu si la version sans support C# n'est pas installée. (Exécutez simplement « install » pour l'installer.)
help-version = La version à installer (ex. 4), ou « stable » pour la dernière version stable.
help-version-long =
    { help-version }

    Exemples : 4.4 installera la dernière version stable de Godot 4.4. Si seules des versions de pré-publication existent, dans ce cas, la dernière version de pré-publication sera installée. 4.3-rc installera la dernière version candidate de Godot 4.3, etc.
help-version-installed = La version installée (ex. 4.2 ou 4.2-stable).

help-search = Lister les versions disponibles depuis le registre
help-filter = Chaîne optionnelle pour filtrer les tags de versions
help-include-pre = Inclure les versions de pré-publication (rc, beta, dev)
help-cache-only = Utiliser uniquement les informations de versions en cache sans interroger le registre
help-limit = Nombre de versions à lister, par défaut 10. Utilisez 0 pour lister toutes
help-clear-cache = Vide le cache des versions
help-refresh = Actualiser le cache des versions depuis le registre
help-refresh-flag = Actualiser le cache des versions avant d'exécuter cette commande

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

help-console = Exécuter Godot avec la console attachée. Par défaut false sur Windows, true sur les autres plateformes.

help-default = Gérer la version par défaut
help-default-version = La version à définir par défaut (ex. 4.2 ou 4.2-stable).
no-default-set = Aucune version par défaut définie. Exécutez « gdvm use <version> » pour définir une version par défaut système, ou « gdvm pin <version> » pour définir une version par défaut pour le répertoire courant.

installing-version = Installation de la version {$version}
installed-success = {$version} installée avec succès

warning-prerelease = {"\u001b"}[33mAvertissement : Vous installez une version de pré-publication ({$branch}).{"\u001b"}[0m

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
error-multiple-versions-found = Plusieurs versions correspondent à votre demande :

error-invalid-godot-version = Format de version Godot invalide. Formats attendus : x, x.y, x.y.z, x.y.z.w, x.y.z-tag.
error-invalid-remote-version = Format de version Godot distante invalide. Formats attendus : x, x.y, x.y.z, x.y.z.w, x.y.z-tag, ou « stable ».

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
error-link-symlink = Échec de la création du lien : {$error}
error-link-copy = Échec de la copie de l'exécutable : {$error}
error-link-godotsharp-target = Impossible de déterminer le chemin cible GodotSharp.
error-link-godotsharp-missing = Le répertoire GodotSharp est manquant à côté de l'exécutable résolu.

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

    Cela créera un fichier .gdvmrc dans le répertoire courant avec la version épinglée. Lorsque vous exécutez « gdvm run » dans ce répertoire ou dans l'un de ses sous-répertoires, la version épinglée sera utilisée au lieu de la version par défaut.

    Ceci est utile lorsque vous voulez utiliser une version spécifique de Godot pour un projet sans changer la version par défaut du système.
help-pin-version = La version à épingler
pinned-success = Version {$version} épinglée avec succès dans .gdvmrc
error-pin-version-not-found = Impossible d'épingler la version {$version}
pin-subcommand-description = Définir ou mettre à jour .gdvmrc avec la version demandée

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

