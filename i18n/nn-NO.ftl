hello = Hei, Verda!

help-about = Godot-versjonsbehandlaren
help-help = Vis hjelp (sjå eit samandrag med '-h')
help-gdvm-version = Vis versjonen av Godot-versjonsbehandlaren

help-install = Installer ein ny Godot-versjon
help-run = Køyr ein spesifikk Godot-versjon
help-list = List alle installerte Godot-versjonar
help-remove = Fjern ein installert Godot-versjon

help-branch = Greina (stable, beta, alpha eller tilpassa).
help-csharp = Bruk Godot-versjonen med C#-støtte.
help-run-csharp-long = Køyr Godot-versjonen med C#-støtte.

    Ved å gjeva ein verdi, overskridar du standardversjonen sett med «use». Elles
    vert standardversjonen brukt. Med andre ord, om du set ein standardversjon med
    «use --csharp», kan du prøva å køyra den same versjon men utan C#-støtte med
    «run --csharp false». Det kan likevel ikkje fungera som forventa om versjonen
    utan C#-støtte ikkje er installert. (Berre køyr «install» for å installere han.)
help-version = Versjonen som skal installerast (t.d. 4), eller «stable» for den siste stabile versjonen.
help-version-long =
    Versjonen som skal installerast (t.d. 4), eller «stable» for den siste stabile
    versjonen.

    Døme: 4.4 vil installere den siste stabile utgjevinga av Godot 4.4. Om berre
    førhandsversjonar finst, vil den siste førhandsversjonen verta installert.
    4.3-rc vil installere den siste utgjevinga av Godot 4.3, osb.
help-version-installed = Den installerte versjonen (t.d. 4.2 eller 4.2-stable).

help-search = List tilgjengelege utgjevingar frå godot-builds
help-filter = Valfri streng for å filtrere utgjevingstagg
help-include-pre = Inkluder førhandsversjonar (rc, beta, dev)
help-cache-only = Bruk berre hurtigbufra utgjevingsinformasjon utan å spørja GitHub-APIet
help-limit = Talet på utgjevingar som skal visast, standard er 10. Bruk 0 for å vise alle
help-clear-cache = Tøm gdvm-utgjevingshurtigbufferet

help-force = Tving installasjon på nytt sjølv om versjonen allereie er installert.
help-redownload = Last ned versjonen på nytt sjølv om den allereie er lasta ned i hurtigbufferet.
help-yes = Hopp over bekreftelsesprompt for fjerning

cached-zip-stored = Lagra Godot-utgjevingsarkivet i hurtigbufferet.
using-cached-zip = Brukar hurtigbufra utgjevingsarkiv, hoppar over nedlasting.

help-console = Køyr Godot med konsoll tilkopla. Standard er false på Windows, true på andre plattformer.

help-default = Administrer standardversjonen
help-default-version = Versjonen som skal setjast som standard (t.d. 4.2 eller 4.2-stable).

help-help-command = Vis denne meldinga eller hjelpa for dei gjeve underkommandoane

installing-version = Installerer versjon {$version}
installed-success = Installerte {$version} vellukka.

unsupported-platform = Plattforma er ikkje støtta
unsupported-architecture = Arkitekturen er ikkje støtta

verifying-checksum = Verifiserer sjekksum...
checksum-verified = Sjekksum verifisert.
error-checksum-mismatch = Sjekksumfeil for fila { $file }

error-find-user-dirs = Klarte ikkje å finne brukarmappene.

fetching-releases = Hentar utgjevingar...
releases-fetched = Utgjevingar henta.

warning-prerelease = Åtvaring: Du installerer ein førhandsverversjon ({$branch}).

no-versions-installed = Ingen versjonar installert.
installed-versions = Installerte Godot-versjonar:
removed-version = Fjerna versjonen {$version}
removing-version = Fjernar versjon {$version}

force-reinstalling-version = Tvingar installasjon av versjon {$version} på nytt.

force-redownload = Tvingar nedlasting av versjon {$version} på nytt.
operation-downloading-url = Lastar ned {$url}...
operation-download-complete = Nedlasting fullført.
operation-extracting = Pakkar ut...
operation-extract-complete = Utpakking fullført.

error-version-not-found = Versjonen vart ikkje funne. (Gløymde du å spesifisere --csharp?)
error-multiple-versions-found = Fleire versjonar samsvarar med førespurnaden:

error-invalid-godot-version = Ugyldig Godot-versjonsformat. Forventa formater: x, x.y, x.y.z, x.y.z.w eller x.y.z-tag.
error-invalid-remote-version = Ugyldig fjern Godot-versjonsformat. Forventa formater: x, x.y, x.y.z, x.y.z.w, x.y.z-tag eller «stable».

error-no-stable-releases-found = Ingen stabile utgivelser funne.

running-version = Køyrer versjon {$version}
no-matching-releases = Ingen samsvarande utgjevingar funne.
available-releases = Tilgjengelege utgjevingar:
cache-cleared = Hurtigbufferet vart tømt.

version-already-installed = Versjon {$version} er allereie installert.
godot-executable-not-found = Godot-køyrberr fil vart ikkje funne for versjon {$version}.
warning-cache-metadata-reset = Hurtigbufferindeksen for utgjevingar er ugyldig eller korrupt. Tilbakestiller.
cache-files-removed = Hurtigbufferfilene vart fjerna.
cache-metadata-removed = Hurtigbuffermetadataene vart fjerna.
error-cache-metadata-empty = Feil: Hurtigbuffermetadataen er tom, må hente utgjevingar først.
no-cache-files-found = Ingen hurtigbufferfil funne.
no-cache-metadata-found = Ingen hurtigbuffermetadata funne.

confirm-remove = Er du sikker på at du vil fjerne denne versjonen? (ja/nei):
confirm-yes = ja
remove-cancelled = Fjerning avbroten.

default-set-success = Standardversjon {$version} er sett.
default-unset-success = Standardversjonen er fjerna.
provide-version-or-unset = Vennligst oppgjeva ein versjon for å setja som standard eller «unset» for å fjerne standardversjonen.
no-default-set = Ingen standardversjon er sett.

error-starting-godot = Kunne ikkje starte Godot: { $error }

error-open-zip = Kunne ikkje opne ZIP-fila { $path }: { $error }
error-read-zip = Kunne ikkje lese ZIP-arkivet { $path }: { $error }
error-access-file = Kunne ikkje få tilgang til fila ved indeks { $index }: { $error }
error-reopen-zip = Kunne ikkje opne ZIP-fila på nytt { $path }: { $error }
error-invalid-file-name = Ugyldig filnamn i ZIP-arkivet
error-create-dir = Kunne ikkje opprette katalogen { $path }: { $error }
error-create-file = Kunne ikkje opprette fila { $path }: { $error }
error-read-zip-file = Kunne ikkje lese frå ZIP-fila { $file }: { $error }
error-write-file = Kunne ikkje skrive til fila { $path }: { $error }
error-strip-prefix = Kunne ikkje fjerne prefiks: { $error }
error-set-permissions = Kunne ikkje setje tillatingar for { $path }: { $error }
