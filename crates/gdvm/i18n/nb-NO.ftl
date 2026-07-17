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

-gdvm = gdvm
-gdvm-toml = gdvm.toml
-gdvmrc = .gdvmrc
-godot = Godot
size-display =
    { $unit ->
        [b] { NUMBER($value, maximumFractionDigits: 0) } B
        [kib] { NUMBER($value, maximumFractionDigits: 1) } KiB
        [mib] { NUMBER($value, maximumFractionDigits: 1) } MiB
        [gib] { NUMBER($value, maximumFractionDigits: 1) } GiB
       *[tib] { NUMBER($value, maximumFractionDigits: 1) } TiB
    }

help-about = { -godot }-versjonsbehandlaren
help-help = Vis hjelp (se et sammendrag med '-h')
help-gdvm-version = Vis versjonen av { -godot }-versjonsbehandleren

help-install = Installer en ny { -godot }-versjon
help-run = Kjør en spesifikk { -godot }-versjon
help-show = Vis stien til den kjørbare fila for den angitte { -godot }-versjonen
help-cache-path = Vis stien til nedlastingsarkivet i cachen for den oppgitte { -godot }-versjonen
help-link = Opprett ei lenke frå ein { -godot }-versjon si kjørbar fil til  til en angitt sti
help-list = List alle installerte { -godot }-versjoner
help-remove = Fjern en installert { -godot }-versjon
help-csharp = [avvikla] Bruk { -godot }-versjonen med C#-støtte. Bruk variantspesifikatoren «csharp» i stedet (f.eks. csharp:4.4).
help-run-csharp-long = { help-csharp }
help-version = Versjonen som skal installeres (f.eks. 4, csharp:4.4, stable, latest).
help-version-long =
    { help-version }

    Format: [variant:]versjon_eller_nøkkelord

    Hvis en avsluttende * er til stede, treffer den den nyeste utgava med samme prefiks, f.eks. treffer «4.7-dev*» 4.7-dev1, 4.7-dev2 osv.

    Nøkkelord: «latest» løser til den nyeste versjonen. Som standard inkluderer dette bare stabile utgivelser, men forhåndsversjoner kan inkluderes med --pre-flagget.

    Varianter: prefiks med et variantnavn og kolon, f.eks. «csharp:4.4» for C#-versjonen.

    Eksempler: 4.4 vil installere den siste stabile utgivelsen av { -godot } 4.4. Hvis bare forhåndsversjoner finnes, vil den siste forhåndsversjonen bli installert. 4.3-rc* vil installere den siste utgivelsen av { -godot } 4.3, osv.
help-version-installed = Den installerte versjonen (f.eks. 4.2 eller 4.2-stable).

help-search = List tilgjengelige utgivelser fra registeret
help-filter = Valgfri streng for å filtrere utgivelsestagger
help-filter-deprecated = [avvikla] Valgfri streng for å filtrere utgivelsestagger. Bruk posisjonsargumentet for filter i stedet.
help-include-pre = Inkluder forhåndsversjoner (rc, beta, dev)
help-cache-only = Bruk bare bufra utgivelsesinformasjon uten å spørre registeret
help-limit = Antall utgivelser som skal vises, standard er 10. Bruk 0 for å vise alle
help-clear-cache = Tøm utgivelsescachen
help-refresh = Oppdater utgivelsescachen fra registeret
help-refresh-flag = Oppdater utgivelsescachen før denne kommandoen kjøres

help-prune = Fjern installasjoner og cacha arkiv som ikke lenger er i bruk
help-prune-long = { help-prune }

    Som standard fjerner prune installasjoner som ikke har vært brukt på en stund, og cacha nedlastingsarkiv som har blitt for gamle, mens installasjoner som fortsatt har en aktiv lenke inn i seg blir bevart. Installasjonen som er satt som standard blir aldri fjerna, uansett hvilke flagg som gis. Aldersgrensa kan settes med «{ -gdvm } config set prune.max-age-days <dager>» (standard { $default_days } dager).
help-prune-all = Fjern alle installasjoner og cacha arkiv uavhengig av alder. Installasjoner som fortsatt har en aktiv lenke beholdes med mindre --force også er gitt.
help-prune-force = Ignorer lenker, slik at installasjoner som bare er referert av en lenke også kan fjernes.
help-prune-dry-run = Vis hva som ville blitt fjerna uten å slette noe.

prune-dry-run-header = Følgende ville blitt fjerna (tørrkjøring):
prune-removed-header = Fjerna følgende:
prune-installs-header = Installasjoner:
prune-archives-header = Cacha arkiv:
prune-nothing-dry-run = Ingenting ville blitt fjerna.
prune-nothing-removed = Ingenting å fjerne; alt er i bruk eller innenfor aldersgrensa.
prune-preserved-by-link =
    { $count ->
        [one] Beholdt { $count } installasjon som fortsatt er referert av en lenke.
       *[other] Beholdt { $count } installasjoner som fortsatt er refererte av en lenke.
    }
prune-freed = Frigjorde omtrent { size-display }.
prune-would-free = Ville frigjort omtrent { size-display }.
prune-item = - { $label } ({ size-display })
prune-interrupted-header = Fjerna rester etter avbrutte nedlastinger og installasjoner:
warning-broken-install-reinstalling = Den installerte { $version } mangler den kjørbare fila, installerer den på nytt.

help-force = Tving installasjon på nytt selv om versjonen allerede er installert.
help-redownload = Last ned versjonen på nytt selv om den allerede er lasta ned i cachen.
help-yes = Hopp over bekreftelsesprompt for fjerning
help-remove-yes-deprecated = [avvikla] Dette flagget gjør ingenting og vil bli fjerna i ei fremtidig utgave.
help-link-version = Versjonen som skal lenkes. Hvis den ikke oppgis, blir versjonen løst basert på gjeldende mappe eller standardversjonen.
help-link-path = Stien der lenka eller kopien skal opprettes, f.eks. «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }».
help-link-force = Overskriv eksisterende lenke hvis den finnes
help-link-copy = Kopier kjørbar i stedet for å lage lenke
no-cache-files-found = Ingen cache-filer funnet.
no-cache-metadata-found = Ingen cache-metadata funnet.
gdvm-toml-malformed = ignorerer { -gdvm-toml } på { $path } fordi den ikke kunne tolkes: { $error }

help-console = Kjør { -godot } med konsoll tilkobla. Standard er false på Windows, true på andre plattformer.

help-default = Administrer standardversjonen
help-default-version = Versjonen som skal settes som standard (f.eks. 4.2 eller 4.2-stable).
no-default-set = Ingen standardversjon er satt. Kjør «{ -gdvm } use <version>» for å sette en standardversjon systemomfattende, eller «{ -gdvm } pin <version>» for å sette en standardversjon for den gjeldende mappa.

warning-prerelease = Du installerer en forhåndsversjon ({$branch}).
warning-deprecated-csharp-flag = Flagget --csharp er avvikla. Bruk "csharp"-variantspesifikatoren i stedet (f.eks. csharp:4.4).

label-error = Feil:
label-note = Merknad:
label-warning = Advarsel:
progress-rate = { size-display }/s
progress-eta-remaining = { $time } igjen
progress-fraction = { $done }/{ $total }
status-downloading = Laster ned
status-extracting = Pakker ut
status-fetching = Henter
status-installed = Installert
status-installing = Installerer
status-removed = Fjerna
status-removing = Fjerner
status-running = Kjører
status-cleared = Tømt
status-refreshed = Oppdatert
status-skipped = Hoppa over
status-upgraded = Oppgradert
status-upgrading = Oppgraderer
status-verifying = Verifiserer
subject-cached-archive = bufra arkiv
subject-cache = cache
subject-cache-files = cache-filer
subject-cache-metadata = cache-metadata
subject-releases = utgivelser
subject-update-manifest = oppdateringsmanifest
upgrade-target = { -gdvm } { $version }

auto-installing-version = Automatisk installasjon av versjon { $version }

no-versions-installed = Ingen versjoner installerte.
installed-versions = Installerte { -godot }-versjoner:
progress-eta =
    { $magnitude ->
        [seconds] { $secs }s
        [minutes] { $mins }m { $secs }s
       *[hours] { $hours }t { $mins }m
    }

unsupported-platform = Plattforma støttes ikke
unsupported-architecture = Arkitekturen støttes ikke
error-checksum-mismatch = Sjekksumfeil for fila { $file }
error-invalid-sha-length = Ugyldig SHA-lengde { $length }
error-size-mismatch = Størrelsesavvik for fila { $file }: forventa { $expected } byte, fikk { $actual } byte.
error-insecure-url = Nekter å hente { $url } over ei ukryptert tilkobling. Bare https://- og file://-URL-er er tillatte. Sett miljøvariabelen GDVM_ALLOW_INSECURE_URLS for å tillate ukrypterte http://-URL-er.
error-insecure-redirect = Nekter å følge ei omdirigering fra https:// til en ukryptert http://-URL. Sett miljøvariabelen GDVM_ALLOW_INSECURE_URLS for å tillate ukrypterte http://-URL-er.
error-response-not-utf8 = Svaret fra { $url } er ikke gyldig UTF-8: { $error }
error-response-too-large = Svaret fra { $url } overskrider den maksimale tillatte størrelsen på { $limit } byte.
error-too-many-redirects = For mange omdirigeringer.
error-config-invalid-number = Ugyldig verdi for { $key }: { $value } (forventet et tall)
error-config-unknown-key = Ukjent konfigurasjonsnøkkel: { $key }
error-invalid-path = Ugyldig sti: { $path }
error-publish-missing-manifest = registry.json mangler
error-publish-no-such-version = ingen slik versjon: { $version }
error-publish-store-or-url-required = enten --store eller --url må oppgis
error-publish-store-requires-file = --store krever en lokal --file
error-publish-url-requires-integrity = --url krever enten en lokal --file eller eksplisitte --sha512 og --size
error-publish-already-initialized = Registeret er allerede initialisert på { $path }
error-publish-archive-not-found = Arkiv ikke funnet: { $path }
error-publish-no-such-platform = Ingen slik plattform { $platform } for varianten { $variant }
error-publish-no-such-variant = Ingen slik variant: { $variant }
error-publish-invalid-segment = Ugyldig { $what }: { $value }
error-registry-fetch-failed = Klarte ikke å hente { $url }: HTTP { $status }
error-registry-fetch-release-failed = Klarte ikke å hente utgivelsesmetadata
error-registry-invalid-name = Ugyldig registernavn: { $name }
error-registry-missing-index = Registeret «{ $name }» mangler index.json
error-registry-missing-manifest = Registeret «{ $name }» mangler registry.json
error-registry-not-configured = Registeret «{ $name }» er ikke konfigurert
error-registry-parse-index = Klarte ikke å tolke indeksen for «{ $name }»: { $error }
error-registry-parse-manifest = Klarte ikke å tolke manifestet for «{ $name }»: { $error }
error-registry-unknown = Ukjent register «{ $name }»
error-registry-unsupported-url-scheme = Registerets URL-skjema støttes ikke: { $url }
error-spec-empty-registry = Tomt registernavn i «{ $input }»
error-spec-empty-variant = Tomt variantnavn i «{ $input }»
error-spec-empty-version = Tom versjon i «{ $input }»
error-system-time = Systemtiden er før UNIX-epoken
error-unrecognized-version-format = Ukjent versjonsformat: { $input }
error-non-interactive-trust = Kan ikke spørre om å stole på registeret «{ $registry }» ({ $url }) i ei økt som ikke er interaktiv. Send --yes for å stole på det eksplisitt.
error-non-interactive-value = Kan ikke be om en verdi for «{ $key }» i ei økt som ikke er interaktiv. Send verdien som et argument i stedet.
error-registry-unsupported-schema = Registeret «{ $registry }» oppgir en skjemaversjon som ikke støttes: { $schema }.
label-caused-by = Forårsaka av:
label-error-coded = Feil { $code }:
error-wildcard-position = Jokertegnet (*) kan bare stå på slutten av utgivelsestaggen, f.eks. 4.7-dev* (fikk { $input }).
hint-try-wildcard = Ingen utgivelse har taggen { $requested }, men det finnes lignende tagger, der den nyeste er { $newest }. Prøv { $suggestion } for å treffe dem.
download-retrying = Nedlastinga ble avbrutt, prøver på nytt (forsøk { $attempt } av { $max })...
download-resuming = Gjenopptar avbrutt nedlasting ({ size-display } allerede lasta ned).
warning-resume-verification-failed = Den gjenopptatte nedlastinga samsvarte ikke med forventa kontrollsum, laster den ned på nytt fra bunnen av.
lock-waiting = Venter på at en annen { -gdvm }-prosess skal bli ferdig (lås: { $resource })...
prune-skipped-error = Hopper over { $item }: { $error }
prune-skipped-in-use = Hopper over { $item }: den er i bruk av en annen { -gdvm }-prosess.

error-find-user-dirs = Klarte ikke å finne brukermappene.
warning-fetching-releases-using-cache = Feil ved henting av utgivelser: { $error }. Bruker hurtigbuffer i stedet.

error-version-not-found = Versjonen ble ikke funnet.
error-archive-not-cached = Fant ingen arkiv i cachen for {$version}. Installer den først for å fylle cachen.
error-multiple-versions-found = Flere versjoner samsvarer med forespørselen:
link-created = Lenket {$version} til {$path}
copy-created = Kopierte {$version} til {$path}
no-matching-releases = Ingen samsvarende utgivelser funna.
available-releases = Tilgjengelige utgivelser:

version-already-installed = Versjon {$version} er allerede installert.
godot-executable-not-found = { -godot }-kjørbar fil ble ikke funnet for versjon {$version}.
error-link-exists = Stien {$path} finnes allerede. Bruk --force for å overskrive.
error-link-symlink = Klarte ikke å opprette lenke fra {$link} til {$target}: {$error}
error-link-copy = Klarte ikke å kopiere kjørbar: {$error}

error-no-stable-releases-found = Ingen stabile versjoner funnet.

error-starting-godot = Kunne ikke starte { -godot }: { $error }
confirm-yes = ja

default-set-success = Standardversjon {$version} er satt.
default-unset-success = Standardversjonen er fjerna.
provide-version-or-unset = Vennligst oppgi en versjon for å sette som standard eller «unset» for å fjerne standardversjonen.

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
error-create-symlink-windows = Kunne ikke opprette symlink. Kontroller at {"\u001b"}]8;;ms-settings:developers{"\u001b"}\utviklermodus{"\u001b"}]8;;{"\u001b"}\ er aktivert eller kjør som administrator.

help-upgrade = Oppgrader { -gdvm } til nyeste versjon
help-upgrade-major = Tillat oppgradering på tvers av hovedversjoner
help-upgrade-pre = Oppgrader til nyeste forhåndsutgivelse
upgrade-not-needed = { -gdvm } er allerede på siste versjon: { $version }.
upgrade-current-version-newer = Den nåværende { -gdvm }-versjonen ({ $current }) er nyere enn den siste tilgjengelige versjonen ({ $latest }). Ingen oppgradering nødvendig.
upgrade-download-failed = Nedlasting av oppgradering mislyktes: { $error }
upgrade-install-dir-failed = Klarte ikke å opprette installasjonskatalogen: { $error }
upgrade-rename-failed = Klarte ikke å endre navnet på den nåværende kjørbare fila: { $error }
upgrade-replace-failed = Klarte ikke å erstatte den kjørbare fila med den nye: { $error }
upgrade-no-binary = Ingen { -gdvm }-binærfil er tilgjengelig for versjon { $version } og målet { $target }.
upgrade-checksum-required = Utgivelsesmanifestet inneholder ingen sjekksum for denne { -gdvm }-binærfila. Nekter å oppgradere.
error-fetching-gdvm-releases = Feil ved henting av { -gdvm }-utgivelser: { $error }
error-parsing-gdvm-releases = Feil ved tolking av { -gdvm }-utgivelser: { $error }
error-unsupported-gdvm-schema = Skjemaversjon for { -gdvm }-utgivelsesmanifestet støttes ikke: { $schema }. Prøv å oppgradere { -gdvm } manuelt.
upgrade-available = 💡 En ny versjon av { -gdvm } er tilgjengelig: {$version}. Kjør «{ -gdvm } upgrade» for å oppdatere.
upgrade-available-major = 💡 Ei hovedversjonsoppdatering av { -gdvm } er tilgjengelig: {$version}. Kjør «{ -gdvm } upgrade -m» for å oppdatere.
upgrade-available-both = 💡 En ny versjon av { -gdvm } er tilgjengelig: {$minor_version}. Ei hovedversjonsoppdatering er også tilgjengelig: {$major_version}. Kjør «{ -gdvm } upgrade» for å oppdatere innen gjeldende hovedversjon, eller «{ -gdvm } upgrade -m» for å oppgradere til aller siste versjon.
upgrade-prerelease-available = 💡 En nyere forhåndsutgivelse av { -gdvm } er tilgjengelig. Kjør "{ -gdvm } upgrade --pre" for å installere den.

help-pin = Fest en versjon av { -godot } til gjeldende mappe.
help-pin-long = { help-pin }

    Dette vil opprette en { -gdvm-toml }-fil i gjeldende mappe med den festa versjonen. Når du kjører «{ -gdvm } run» i denne katalogen eller noen av underkatalogene, vil den festa versjonen brukes i stedet for standardversjonen.

    Dette er nyttig når du vil bruke en spesifikk versjon av { -godot } for et prosjekt uten å endre standardversjonen systemomfattende.

    Dette skriver foreløpig også den eldre { -gdvmrc }-fila for kompatibilitet med eldre versjoner av { -gdvm }. Dette vil bli fjerna i en framtidig utgivelse, så det anbefales å gå over til det nye { -gdvm-toml }-formatet og fjerne { -gdvmrc }-fila hvis den finnes.

    Du kan deaktivere skriving av en { -gdvmrc }-fil med --no-legacy-flagget.
help-pin-version = Versjonen som skal festes
help-no-legacy = Ikke skriv den eldre { -gdvmrc }-kompatibilitetsfila
pinned-success = Versjon {$version} ble festet i { -gdvm-toml }
error-pin-version-not-found = Kan ikke feste versjon {$version}

error-file-not-found = Fil ble ikke funnet. Den finnes kanskje ikke på serveren.
error-download-failed = Nedlasting mislyktes på grunn av en uventa feil: { $error }
error-ensure-godot-binaries-failed = Kunne ikke forsikre { -godot }-kjørbare filer.
    Feil: { $error }.
    Prøv å slette { $path } og kjør { -gdvm } på nytt.

error-post-upgrade-action-failed = Trinnet { $id } mislyktes etter oppgraderinga.
    Feil: { $error }.
    { -gdvm }-installasjonen din kan være ufullstendig. Prøv å kjøre { -gdvm } på nytt.

error-failed-reading-project-godot = Kunne ikke lese project.godot, kan ikke automatisk bestemme prosjektversjonen.
warning-using-project-version = Bruker versjon { $version } definert i project.godot.

warning-project-version-mismatch =
    {"\u001b"}[33mAdvarsel: Versjonen definert i project.godot samsvarer ikke med den { $pinned ->
        [1] festede
        *[0] forespurte
    } versjonen. Åpning av prosjektet med den { $pinned ->
        [1] festede
        *[0] forespurte
    } versjonen kan overskrive prosjektfila.{"\u001b"}[0m

    { $pinned ->
        [1] Prosjektversjon: { $project_version }
            Festet versjon:  { $requested_version }
        *[0] Prosjektversjon:   { $project_version }
             Forespurt versjon: { $requested_version }
    }

error-project-version-mismatch = {"\u001b"}[31m{ $pinned ->
        [1] Hvis du er sikker på at du vil kjøre prosjektet med den festa versjonen, kjør {"\u001b"}[0m{ -gdvm } run --force{"\u001b"}[31m. Ellers oppdater den festa versjonen i { -gdvmrc } for å samsvare med prosjektversjonen, eller fjern { -gdvmrc }-fila for å bruke prosjektversjonen.
        *[0] Hvis du er sikker på at du vil kjøre prosjektet med den forespurte versjonen, kjør {"\u001b"}[0m{ -gdvm } run --force <version>{"\u001b"}[31m.
    }{"\u001b"}[0m

help-run-args = Tilleggsargumenter som skal sendes til { -godot }-kjørbar fil (f.eks. -- path/to/project.godot).
help-run-force =
    Tving kjøring av prosjektet med den forespurte eller festa versjonen selv om den ikke samsvarer med prosjektversjonen.
help-run-force-long =
    Tving kjøring av prosjektet med den forespurte eller festa versjonen selv om den ikke samsvarer med prosjektversjonen.

    Hvis du gjør dette, kan den forespurte eller festa versjonen av { -godot } overskrive prosjektfila. Hvis du fester versjoner, anbefales det i stedet å oppdatere den festa versjonen i { -gdvmrc } for å samsvare med prosjektversjonen, eller fjerne { -gdvmrc }-fila for å bruke prosjektversjonen.

help-config = Administrer { -gdvm }-konfigurasjon
help-format = Utdataformat: text (standard) eller json
help-info = Vis detaljert informasjon om en installert versjon
info-default =
    { $value ->
        [1] { confirm-yes }
       *[0] { info-no }
    }
    .label = Standard:
info-executable = { $path }
    .label = Kjørbar fil:
info-install-path = { $path }
    .label = Installasjonssti:
info-last-used = { $timestamp }
    .label = Sist brukt:
info-no = nei
info-registry = { $registry }
    .label = Register:
info-size = { size-display }
    .label = Størrelse på disk:
info-variant = { $variant }
    .label = Variant:
info-version = { $version }
    .label = Versjon:
help-completions = Generer skript for skallfullføring
help-completions-shell = Skallet det skal genereres fullføringer for
help-config-get = Hent en konfigurasjonsverdi
help-config-set = Sett en konfigurasjonsverdi
help-config-unset = Fjern en konfigurasjonsverdi
help-config-list = List alle konfigurasjonsverdier
help-config-key = Konfigurasjonsnøkkelen (f.eks. prune.max-age-days)
help-config-value = Verdien som skal settes for konfigurasjonsnøkkelen
help-config-unset-key = Konfigurasjonsnøkkelen som skal fjernes (f.eks. prune.max-age-days)
help-config-show-sensitive = Vis sensitive konfigurasjonsverdier i klartekst
help-config-available = List alle tilgjengelige konfigurasjonsnøkler og verdier, inkludert standardverdier
warning-setting-sensitive = Du setter en sensitiv verdi som vil lagres i klartekst i hjemmemappa di.
config-set-prompt = Vennligst oppgi verdien for { $key }:
error-reading-input = Feil ved lesing av inndata
config-set-success = Konfigurasjonen ble oppdatert.
config-unset-success = Konfigurasjonsnøkkelen { $key } ble fjernet vellykket.
config-key-not-set = Konfigurasjonsnøkkel ikke satt.
config-key-not-set-value = <ikke satt>
error-unknown-config-key = Ukjent konfigurasjonsnøkkel.
error-invalid-config-subcommand = Ugyldig config-underkommando. Bruk "get", "set" eller "list".
error-parse-config = Kunne ikke tolke konfigurasjonsfila: { $error }
error-parse-config-using-default = Bruker standard konfigurasjonsverdier.

help-registry = Administrer registre å installere { -godot }-bygg fra
help-registry-add = Legg til et register
help-registry-remove = Fjern et register
help-registry-list = List opp konfigurerte registre
help-registry-refresh = Oppdater hurtigbufferen for ett eller alle registre
help-registry-name = Registernavnet
help-registry-url = Register-URL-en. Kan være en http(s)://- eller file://-URL.

registry-added = La til registeret { $registry } ({ $url }).
registry-removed = Fjernet registeret { $registry }.
registry-list-header = Konfigurerte registre:
registry-tag-official = offisielt

error-invalid-registry-subcommand = Ugyldig register-underkommando. Bruk «add», «remove», «list» eller «refresh».
registry-trust-warning = { $registry } ({ $url }) er et egendefinert register, ikke det offisielle. { -gdvm } sjekker at nedlastinger stemmer med det registeret oppgir, men kan ikke vite om de er trygge å kjøre. Installer fra det bare hvis du stoler på de som driver det.
registry-trust-prompt = Stoler du på dette registeret og vil fortsette? (ja/nei):
registry-trust-bypass = {"\u001b"}[1;31mHopper over tillitssjekken for { $registry } ({ $url }) fordi du brukte --yes. { -gdvm } kan ikke vite om filene er trygge å kjøre. Tar en kort pause; trykk Ctrl+C nå for å stoppe.{"\u001b"}[0m
registry-trust-aborted = Avbrutt: registeret er ikke betrodd.
registry-project-override-conflict = Prosjektets { -gdvm-toml } omdefinerer registeret { $registry } (din konfigurasjon: { $machine_url }) som { $project_url }. Prosjektets definisjon har forrang.

help-registry-init = Initialiser en ny registermappe
help-registry-add-build = Legg til et bygg i et register
help-registry-remove-build = Fjern et bygg fra et register
help-registry-validate = Valider en registermappe
help-registry-dir = Registermappen
help-registry-init-name = Registernavnet. Standard er mappenavnet.

help-registry-build-version = Versjonsetiketten, f.eks. 4.4-stable.
help-registry-build-variant = Variantnavnet. Standard er «default».
help-registry-build-platform = Plattformnøkkelen, f.eks. linux-x86_64.
help-registry-build-file = Sti til byggarkivet som skal hashes
help-registry-build-store = Kopier arkivet inn i registeret og registrer en relativ URL.
help-registry-build-url = URL-en der arkivet skal serveres (når --store ikke brukes).
help-registry-build-sha512 = Arkivets SHA-512, i stedet for å beregne det. Krever --size.
help-registry-build-size = Arkivets størrelse i byte, i stedet for å måle det. Krever --sha512.

registry-init-success = Initialiserte registeret { $name } i { $path }.
registry-build-added = La til bygget { $version } for { $platform }.
registry-build-removed = Fjerna bygget { $version }.
registry-build-warn-local-hash = Hasher den lokale fila og antar at den samsvarer med { $url }. { -gdvm } laster ikke ned URL-en for å verifisere den.
registry-build-warn-unverified = Bruker SHA-512 og størrelsen du oppga uten å laste ned artefakten for å verifisere dem. Kontroller at de er riktige.
registry-build-warn-explicit-store = Bruker SHA-512 og/eller størrelsen du oppga i stedet for å måle det lagra arkivet.
registry-build-sha-mismatch = Oppgitt SHA-512 ({ $expected }) samsvarer ikke med artefakten ({ $actual }).
registry-build-size-mismatch = Oppgitt størrelse ({ $expected }) samsvarer ikke med artefakten ({ $actual }).
registry-validate-ok =
    { $count ->
        [one] Registeret er gyldig ({ $count } artefakt kontrollert).
       *[other] Registeret er gyldig ({ $count } artefakter kontrollert).
    }
registry-validate-failed = Validering av registeret mislyktes:
