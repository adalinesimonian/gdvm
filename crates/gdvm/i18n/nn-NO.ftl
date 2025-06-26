hello = Hei, Verda!

help-about = Godot-versjonsbehandlaren
help-help = Vis hjelp (sjå eit samandrag med '-h')
help-help-command = Vis denne meldinga eller hjelpa for dei gjeve underkommandoane
help-gdvm-version = Vis versjonen av Godot-versjonsbehandlaren

help-install = Installer ein ny Godot-versjon
help-run = Køyr ein spesifikk Godot-versjon
help-list = List alle installerte Godot-versjonar
help-remove = Fjern ein installert Godot-versjon

help-branch = Greina (stable, beta, alpha eller tilpassa).
help-csharp = Bruk Godot-versjonen med C#-støtte.
help-run-csharp-long = { help-csharp }

    Ved å gjeva ein verdi, overskridar du standardversjonen sett med «use». Elles vert standardversjonen brukt. Med andre ord, om du set ein standardversjon med «use --csharp», kan du prøva å køyra den same versjon men utan C#-støtte med «run --csharp false». Det kan likevel ikkje fungera som forventa om versjonen utan C#-støtte ikkje er installert. (Berre køyr «install» for å installere han.)
help-version = Versjonen som skal installerast (t.d. 4), eller «stable» for den siste stabile versjonen.
help-version-long =
    { help-version }

    Døme: 4.4 vil installere den siste stabile utgjevinga av Godot 4.4. Om berre førhandsversjonar finst, vil den siste førhandsversjonen verta installert. 4.3-rc vil installere den siste utgjevinga av Godot 4.3, osb.
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
warning-cache-metadata-reset = Hurtigbufferindeksen for utgjevingar er ugyldig eller korrupt. Tilbakestiller.
cache-files-removed = Hurtigbufferfilene vart fjerna.
cache-metadata-removed = Hurtigbuffermetadataene vart fjerna.
error-cache-metadata-empty = Feil: Hurtigbuffermetadataen er tom, må hente utgjevingar først.
no-cache-files-found = Ingen hurtigbufferfil funne.
no-cache-metadata-found = Ingen hurtigbuffermetadata funne.

help-console = Køyr Godot med konsoll tilkopla. Standard er false på Windows, true på andre plattformer.

help-default = Administrer standardversjonen
help-default-version = Versjonen som skal setjast som standard (t.d. 4.2 eller 4.2-stable).
no-default-set = Ingen standardversjon er sett. Køyr «gdvm use <version>» for å setja ein  standardversjon systemomfattande, eller «gdvm pin <version>» for å setja ein  standardversjon for den gjeldende mappa.

installing-version = Installerer versjon {$version}
installed-success = Installerte {$version} vellukka.

warning-prerelease = Åtvaring: Du installerer ein førhandsverversjon ({$branch}).

force-reinstalling-version = Tvingar installasjon av versjon {$version} på nytt.

auto-installing-version = Automatisk installasjon av versjon { $version }

no-versions-installed = Ingen versjonar installert.
installed-versions = Installerte Godot-versjonar:
removed-version = Fjerna versjonen {$version}
removing-version = Fjernar versjon {$version}

force-redownload = Tvingar nedlasting av versjon {$version} på nytt.
operation-downloading-url = Lastar ned {$url}...
operation-download-complete = Nedlasting fullført.
operation-extracting = Pakkar ut...
operation-extract-complete = Utpakking fullført.

unsupported-platform = Plattforma er ikkje støtta
unsupported-architecture = Arkitekturen er ikkje støtta

verifying-checksum = Verifiserer sjekksum...
checksum-verified = Sjekksum verifisert.
error-checksum-mismatch = Sjekksumfeil for fila { $file }
error-invalid-sha-length = Ugyldig SHA-lengde { $length }
warning-sha-sums-missing = Sjekksumfiler vart ikkje funne for denne utgjevinga. Hoppar over verifisering.

error-find-user-dirs = Klarte ikkje å finne brukarmappene.

fetching-releases = Hentar utgjevingar...
releases-fetched = Utgjevingar henta.
error-fetching-releases = Feil ved henting av utgjevingar: { $error }
warning-fetching-releases-using-cache = Feil ved henting av utgjevingar: { $error }. Brukar hurtigbuffer i staden.

error-version-not-found = Versjonen vart ikkje funne.
error-multiple-versions-found = Fleire versjonar samsvarar med førespurnaden:

error-invalid-godot-version = Ugyldig Godot-versjonsformat. Forventa formater: x, x.y, x.y.z, x.y.z.w eller x.y.z-tag.
error-invalid-remote-version = Ugyldig fjern Godot-versjonsformat. Forventa formater: x, x.y, x.y.z, x.y.z.w, x.y.z-tag eller «stable».

running-version = Køyrer versjon {$version}
no-matching-releases = Ingen samsvarande utgjevingar funne.
available-releases = Tilgjengelege utgjevingar:
cache-cleared = Hurtigbufferet vart tømt.

version-already-installed = Versjon {$version} er allereie installert.
godot-executable-not-found = Godot-køyrberr fil vart ikkje funne for versjon {$version}.

error-no-stable-releases-found = Ingen stabile utgivelser funne.

error-starting-godot = Kunne ikkje starte Godot: { $error }

confirm-remove = Er du sikker på at du vil fjerne denne versjonen? (ja/nei):
confirm-yes = ja
remove-cancelled = Fjerning avbroten.

default-set-success = Standardversjon {$version} er sett.
default-unset-success = Standardversjonen er fjerna.
provide-version-or-unset = Vennligst oppgjeva ein versjon for å setja som standard eller «unset» for å fjerne standardversjonen.

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
error-create-symlink-windows = Kunne ikkje laga symlink. Kontroller at {"\u001b"}]8;;ms-settings:developers{"\u001b"}\utviklarmodus{"\u001b"}]8;;{"\u001b"}\ er aktivert eller kør som administrator.

help-upgrade = Oppgrader gdvm til nyaste versjon
help-upgrade-major = Tillat oppgradering på tvers av hovudversjonar
upgrade-starting = Startar oppgradering av gdvm...
upgrade-downloading-latest = Lastar ned nyaste gdvm...
upgrade-complete = gdvm vart oppgradert!
upgrade-not-needed = gdvm er allereie på siste versjon: { $version }.
upgrade-current-version-newer = Den noverande gdvm-versjonen ({ $current }) er nyare enn den siste tilgjengelege versjonen ({ $latest }). Inga oppgradering nødvendig.
upgrade-failed = Oppgradering feila: { $error }
upgrade-download-failed = Nedlasting av oppgradering feila: { $error }
upgrade-file-create-failed = Klarte ikkje å opprette oppgraderingsfila: { $error }
upgrade-file-write-failed = Klarte ikkje å skrive til oppgraderingsfila: { $error }
upgrade-install-dir-failed = Klarte ikkje å opprette installasjonskatalogen: { $error }
upgrade-rename-failed = Klarte ikkje å endre namn på den noverande køyrberre fila: { $error }
upgrade-replace-failed = Klarte ikkje å erstatte den køyrberre fila med den nye: { $error }
checking-updates = Sjekkar etter oppdateringar til gdvm...
upgrade-available = 💡 Ein ny versjon av gdvm er tilgjengeleg: {$version}. Køyr «gdvm upgrade» for å oppdatere.
upgrade-available-major = 💡 Ei hovudversjonsoppdatering av gdvm er tilgjengeleg: {$version}. Køyr «gdvm upgrade -m» for å oppdatere.
upgrade-available-both = 💡 Ein ny versjon av gdvm er tilgjengeleg: {$minor_version}. Ei hovudversjonsoppdatering er òg tilgjengeleg: {$major_version}. Køyr «gdvm upgrade» for å oppdatere innan gjeldande hovudversjon, eller «gdvm upgrade -m» for å oppgradere til siste versjon.

help-pin = Fest ein versjon av Godot til gjeldande katalog.
help-pin-long = { help-pin }

    Dette vil opprette ei .gdvmrc-fil i gjeldande mappe med den festa versjonen. Når du køyrer «gdvm run» i denne katalogen eller nokon av underkatalogane, vil den festa versjonen verta bruka i staden for standardversjonen.

    Dette er nyttig når du vil bruke ein spesifikk versjon av Godot for eit prosjekt utan å endre standardversjonen systemomfattande.
help-pin-version = Versjonen som skal festast
pinned-success = Versjon {$version} vart festa i .gdvmrc
error-pin-version-not-found = Kan ikkje feste versjon {$version}
pin-subcommand-description = Set eller oppdater .gdvmrc med ønskja versjon

error-file-not-found = Fil vart ikkje funnen. Ho finst kanskje ikkje på tenaren.
error-download-failed = Nedlasting feila på grunn av ein uventa feil: { $error }
error-ensure-godot-binaries-failed = Kunne ikkje sikre Godot-køyrberre filer.
    Feil: { $error }.
    Prøv å slette { $path } og køyre gdvm på nytt.

error-failed-reading-project-godot = Kunne ikkje lese project.godot, kan ikkje automatisk bestemme prosjektversjonen.
warning-using-project-version = Brukar versjon { $version } definert i project.godot.

warning-project-version-mismatch =
    {"\u001b"}[33mÅtvaring: Versjonen definert i project.godot samsvarer ikkje med den { $pinned ->
        [1] festa
        *[0] ynskte
    } versjonen. Opning av prosjektet med den { $pinned ->
        [1] festa
        *[0] ynskte
    } versjonen kan overskrive prosjektfila.{"\u001b"}[0m

    { $pinned ->
        [1] Prosjektversjon: { $project_version }
            Festa versjon:   { $requested_version }
        *[0] Prosjektversjon:   { $project_version }
             Førespurd versjon: { $requested_version }
    }

error-project-version-mismatch =
    {"\u001b"}[31m{ $pinned ->
        [1] Om du er sikker på at du vil køyre prosjektet med den festa versjonen, køyr {"\u001b"}[0mgdvm run --force{"\u001b"}[31m. Elles, oppdater den festa versjonen i .gdvmrc for å samsvara med prosjektversjonen, eller fjern .gdvmrc-fila for å bruke prosjektversjonen.
        *[0] Om du er sikker på at du vil køyre prosjektet med den ynskte versjonen, køyr {"\u001b"}[0mgdvm run --force <version>{"\u001b"}[31m.
    }{"\u001b"}[0m
warning-project-version-mismatch-force = {"\u001b"}[33mHoppar over bekreftelsesprompt og held fram med den { $pinned ->
        [1] festa
        *[0] ynskte
    } versjonen {"\u001b"}[0m({ $requested_version }){"\u001b"}[33m.{"\u001b"}[0m

help-run-args = Tilleggsargument som skal sendast til Godot-køyrbar fil (t.d. -- path/to/project.godot).
help-run-force =
    Tving køyring av prosjektet med den ynskte eller festa versjonen sjølv om han ikkje samsvarar med prosjektversjonen.
help-run-force-long =
    { help-run-force }

    Viss du gjer dette, kan den ynskte eller festa versjonen av Godot overskriva prosjektfila. Viss du festar versjonar, er det tilrådd å i staden oppdatere den festa versjonen i .gdvmrc for å samsvara med prosjektversjonen, eller fjerne .gdvmrc-fila for å bruke prosjektversjonen.

help-config = Administrer gdvm-konfigurasjon
help-config-get = Hent ein konfigurasjonsverdi
help-config-set = Set ein konfigurasjonsverdi
help-config-unset = Fjern ein konfigurasjonsverdi
help-config-list = List alle konfigurasjonsverdiar
help-config-key = Konfigurasjonsnykelen (t.d. github.token)
help-config-value = Verdien som skal setjast for konfigurasjonsnykelen
help-config-unset-key = Konfigurasjonsnykelen som skal fjernast (t.d. github.token)
help-config-show-sensitive = Vis sensitive konfigurasjonsverdiar i klartekst
help-config-available = List alle tilgjengelege konfigurasjonsnyklar og verdiar, inkludert standardverdiar
warning-setting-sensitive = {"\u001b"}[33mÅtvaring: Du set ein sensitiv verdi som vil verta lagra i klartekst i heimemappa di.{"\u001b"}[0m
config-set-prompt = Ver vennleg og oppgjev verdien for { $key }:
error-reading-input = Feil ved lesing av inndata
config-set-success = Konfigurasjonen vart oppdatert.
config-unset-success = Konfigurasjonsnykelen { $key } vart fjerna vellukka.
config-key-not-set = Konfigurasjonsnykel ikkje sett.
error-unknown-config-key = Ukjend konfigurasjonsnykel.
error-invalid-config-subcommand = Ugyldig config-underkommando. Bruk «get», «set» eller «list».
error-parse-config = Kunne ikkje tolke konfigurasjonsfila: { $error }
error-parse-config-using-default = {"\u001b"}[33mBrukar standard konfigurasjonsverdiar.{"\u001b"}[0m
error-github-api = GitHub API-feil: { $error }
error-github-rate-limit = GitHub API-rategrense overskriden.

  For å løyse dette, ver venleg og opprett ein personleg tilgangstoken på GitHub ved å vitja https://github.com/settings/tokens.

  Klikk på «Generate new token», vel berre dei minimale løyva som krevst (t.d. public_repo), og set deretter tokenet via miljøvariabelen GITHUB_TOKEN eller ved å køyre:

    gdvm config set github.token

  Merk: Tokenet vil verta lagra i klartekst i heimemappa di. Ver venleg å sørgje for at du held det sikkert.
  Det er tilrådd å regelmessig gjennomgå og rotere tokena dine for tryggleiksføremål.
