hello = Hei, Verda!

help-about = Godot-versjonsbehandlaren
help-help = Vis hjelp (se et sammendrag med '-h')
help-gdvm-version = Vis versjonen av Godot-versjonsbehandleren

help-install = Installer en ny Godot-versjon
help-run = Kjør en spesifikk Godot-versjon
help-list = List alle installerte Godot-versjoner
help-remove = Fjern en installert Godot-versjon

help-branch = Greina (stable, beta, alpha eller tilpassa).
help-csharp = Bruk Godot-versjonen med C#-støtte.
help-run-csharp-long = Kjør Godot-versjonen med C#-støtte.

    Ved å gi en verdi, overskrider du standardversjonen satt med «use». Ellers
    brukes standardversjonen. Med andre ord, hvis du setter en standardversjon med
    «use --csharp», kan du prøve å kjøre den samme versjon men uten C#-støtte med
    «run --csharp false». Det kan imidlertid ikke fungere som forventet hvis
    versjonen uten C#-støtte ikke er installert. (Bare kjør «install» for å
    installere den.)
help-version = Versjonen som skal installeres (f.eks. 4), eller stable for den siste stabile versjonen.
help-version-long =
    Versjonen som skal installeres (f.eks. 4), eller stable for den siste stabile
    versjonen.

    Eksempler: 4.4 vil installere den siste stabile utgivelsen av Godot 4.4. Hvis
    bare forhåndsversjoner finnes, vil den siste forhåndsversjonen bli installert.
    4.3-rc vil installere den siste utgivelsen av Godot 4.3, osv.
help-version-installed = Den installerte versjonen (f.eks. 4.2 eller 4.2-stable).

help-search = List fjerne utgivelser fra godot-builds
help-filter = Valgfri streng for å filtrere utgivelsestagger
help-include-pre = Inkluder forhåndsversjoner (rc, beta, dev)
help-cache-only = Bruk kun hurtigbufret utgivelsesinformasjon uten å spørre GitHub-APIet
help-limit = Antall utgivelser som skal vises, standard er 10. Bruk 0 for å vise alle
help-clear-cache = Tøm gdvm-utgivelseshurtigbufferet

help-force = Tving installasjon på nytt selv om versjonen allerede er installert.
help-redownload = Last ned versjonen på nytt selv om den allerede er lasta ned i hurtigbufferet.

help-yes = Hopp over bekreftelsesprompt for fjerning

cached-zip-stored = Lagra Godot-utgivelsesarkivet i hurtigbufferet.
using-cached-zip = Bruker hurtigbufret utgivelsesarkiv, hopper over nedlasting.
warning-cache-metadata-reset = Hurtigbufferindeksen for utgivelser er ugyldig eller korrupt. Tilbakestiller.
cache-files-removed = Hurtigbufferfilene ble fjernet.
cache-metadata-removed = Hurtigbuffermetadataene ble fjernet.
error-cache-metadata-empty = Feil: Hurtigbuffermetadataen er tom, må hente utgivelser først.
no-cache-files-found = Ingen hurtigbufferfiler funnet.
no-cache-metadata-found = Ingen hurtigbuffermetadata funnet.

help-console = Kjør Godot med konsoll tilkobla. Standard er false på Windows, true på andre plattformer.

help-default = Administrer standardversjonen
help-default-version = Versjonen som skal settes som standard (f.eks. 4.2 eller 4.2-stable).

help-help-command = Vis denne meldingen eller hjelpa for de gitte underkommandoene

installing-version = Installerer versjon {$version}
installed-success = Installerte {$version} vellykka.

unsupported-platform = Plattforma støttes ikke
unsupported-architecture = Arkitekturen støttes ikke

verifying-checksum = Verifiserer sjekksum...
checksum-verified = Sjekksum verifisert.
error-checksum-mismatch = Sjekksumfeil for filen { $file }
warning-sha-sums-missing = Sjekksumfiler ble ikke funnet for denne utgivelsen. Hopper over verifisering.

error-find-user-dirs = Klarte ikke å finne brukermappene.

fetching-releases = Henter utgivelser...
releases-fetched = Utgivelser henta.

warning-prerelease = Advarsel: Du installerer en forhåndsversjon ({$branch}).

no-versions-installed = Ingen versjoner installert.
installed-versions = Installerte Godot-versjoner:
removed-version = Fjerna versjonen {$version}
removing-version = Fjerner versjon {$version}

force-reinstalling-version = Tvinger installasjon av versjon {$version} på nytt.

force-redownload = Tvinger nedlasting av versjon {$version} på nytt.
operation-downloading-url = Laster ned {$url}...
operation-download-complete = Nedlasting fullført.
operation-extracting = Pakker ut...
operation-extract-complete = Pakking fullført.

error-version-not-found = Versjonen ble ikke funnet.
error-multiple-versions-found = Flere versjoner samsvarer med forespørselen:

error-invalid-godot-version = Ugyldig Godot-versjonsformat. Forventede formater: x, x.y, x.y.z, x.y.z.w eller x.y.z-tag.
error-invalid-remote-version = Ugyldig fjern Godot-versjonsformat. Forventede formater: x, x.y, x.y.z, x.y.z.w, x.y.z-tag eller «stable».

running-version = Kjører versjon {$version}
no-matching-releases = Ingen samsvarende utgivelser funna.
error-no-stable-releases-found = Ingen stabile versjoner funnet.
available-releases = Tilgjengelige utgivelser:
cache-cleared = Hurtigbufferen ble tømt.

version-already-installed = Versjon {$version} er allerede installert.
godot-executable-not-found = Godot-kjørbar fil ble ikke funnet for versjon {$version}.

confirm-remove = Er du sikker på at du vil fjerne denne versjonen? (ja/nei):
confirm-yes = ja
remove-cancelled = Fjerning avbrutt.

default-set-success = Standardversjon {$version} er satt.
default-unset-success = Standardversjonen er fjerna.
provide-version-or-unset = Vennligst oppgi en versjon for å sette som standard eller «unset» for å fjerne standardversjonen.
no-default-set = Ingen standardversjon er satt.

error-starting-godot = Kunne ikke starte Godot: { $error }

error-open-zip = Kunne ikke åpne ZIP-fila { $path }: { $error }
error-read-zip = Kunne ikke lese ZIP-arkivet { $path }: { $error }
error-access-file = Kunne ikke få tilgang til fila ved indeks { $index }: { $error }
error-reopen-zip = Kunne ikke åpne ZIP-fila på nytt { $path }: { $error }
error-invalid-file-name = Ugyldig filnavn i ZIP-arkivet
error-create-dir = Kunne ikke opprette katalogen { $path }: { $error }
error-create-file = Kunne ikke opprette fila { $path }: { $error }
error-read-zip-file = Kunne ikke lese fra ZIP-fila { $file }: { $error }
error-write-file = Kunne ikke skrive til fila { $path }: { $error }
error-strip-prefix = Kunne ikke fjerne prefiks: { $error }
error-set-permissions = Kunne ikke sette tillatelser for { $path }: { $error }

auto-installing-version = Automatisk installasjon av versjon { $version }
