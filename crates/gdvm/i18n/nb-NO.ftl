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

hello = Hei, Verda!

help-about = Godot-versjonsbehandlaren
help-help = Vis hjelp (se et sammendrag med '-h')
help-help-command = Vis denne meldinga eller hjelpa for de gitte underkommandoene
help-gdvm-version = Vis versjonen av Godot-versjonsbehandleren

help-install = Installer en ny Godot-versjon
help-run = Kjør en spesifikk Godot-versjon
help-show = Vis stien til den kjørbare fila for den angitte Godot-versjonen
help-link = Opprett ei lenke frå ein Godot-versjon si kjørbar fil til  til en angitt sti
help-list = List alle installerte Godot-versjoner
help-remove = Fjern en installert Godot-versjon

help-branch = Greina (stable, beta, alpha eller tilpassa).
help-csharp = [avvikla] Bruk Godot-versjonen med C#-støtte. Bruk variantspesifikatoren «csharp» i stedet (f.eks. csharp:4.4).
help-run-csharp-long = { help-csharp }
help-version = Versjonen som skal installeres (f.eks. 4, csharp:4.4, stable, latest).
help-version-long =
    { help-version }

    Format: [variant:]versjon_eller_nøkkelord

    Nøkkelord: «latest» løser til den nyeste versjonen. Som standard inkluderer dette bare stabile utgivelser, men forhåndsversjoner kan inkluderes med --pre-flagget.

    Varianter: prefiks med et variantnavn og kolon, f.eks. «csharp:4.4» for C#-versjonen.

    Eksempler: 4.4 vil installere den siste stabile utgivelsen av Godot 4.4. Hvis bare forhåndsversjoner finnes, vil den siste forhåndsversjonen bli installert. 4.3-rc vil installere den siste utgivelsen av Godot 4.3, osv.
help-version-installed = Den installerte versjonen (f.eks. 4.2 eller 4.2-stable).

help-search = List tilgjengelige utgivelser fra registeret
help-filter = Valgfri streng for å filtrere utgivelsestagger
help-include-pre = Inkluder forhåndsversjoner (rc, beta, dev)
help-cache-only = Bruk bare bufra utgivelsesinformasjon uten å spørre registeret
help-limit = Antall utgivelser som skal vises, standard er 10. Bruk 0 for å vise alle
help-clear-cache = Tøm utgivelsescachen
help-refresh = Oppdater utgivelsescachen fra registeret
help-refresh-flag = Oppdater utgivelsescachen før denne kommandoen kjøres

help-force = Tving installasjon på nytt selv om versjonen allerede er installert.
help-redownload = Last ned versjonen på nytt selv om den allerede er lasta ned i cachen.
help-yes = Hopp over bekreftelsesprompt for fjerning
help-link-version = Versjonen som skal lenkes. Hvis den ikke oppgis, blir versjonen løst basert på gjeldende mappe eller standardversjonen.
help-link-path = Stien der lenka eller kopien skal opprettes, f.eks. «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }».
help-link-force = Overskriv eksisterende lenke hvis den finnes
help-link-copy = Kopier kjørbar i stedet for å lage lenke

cached-zip-stored = Lagra Godot-utgivelsesarkivet i cachen.
using-cached-zip = Bruker cachet utgivelsesarkiv, hopper over nedlasting.
warning-cache-metadata-reset = Cache-indeksen for utgivelser er ugyldig eller korrupt. Tilbakestiller.
cache-files-removed = Cache-filene ble fjerna.
cache-metadata-removed = Cache-metadataet ble fjerna.
error-cache-metadata-empty = Feil: Cache-metadataet er tomt, må hente utgivelser først.
no-cache-files-found = Ingen cache-filer funnet.
no-cache-metadata-found = Ingen cache-metadata funnet.

help-console = Kjør Godot med konsoll tilkobla. Standard er false på Windows, true på andre plattformer.

help-default = Administrer standardversjonen
help-default-version = Versjonen som skal settes som standard (f.eks. 4.2 eller 4.2-stable).
no-default-set = Ingen standardversjon er satt. Kjør «gdvm use <version>» for å sette en standardversjon systemomfattende, eller «gdvm pin <version>» for å sette en standardversjon for den gjeldende mappa.

installing-version = Installerer versjon {$version}
installed-success = Installerte {$version} vellykka.

warning-prerelease = {"\u001b"}[33mAdvarsel: Du installerer en forhåndsversjon ({$branch}).{"\u001b"}[0m
warning-deprecated-csharp-flag = {"\u001b"}[33mAdvarsel: Flagget --csharp er avvikla. Bruk "csharp"-variantspesifikatoren i stedet (f.eks. csharp:4.4).{"\u001b"}[0m

force-reinstalling-version = Tvinger installasjon av versjon {$version} på nytt.

auto-installing-version = Automatisk installasjon av versjon { $version }

no-versions-installed = Ingen versjoner installerte.
installed-versions = Installerte Godot-versjoner:
removed-version = Fjerna versjonen {$version}
removing-version = Fjerner versjon {$version}

force-redownload = Tvinger nedlasting av versjon {$version} på nytt.
operation-downloading-url = Laster ned {$url}...
operation-download-complete = Nedlasting fullført.
operation-extracting = Pakker ut...
operation-extract-complete = Utpakking fullført.

unsupported-platform = Plattforma støttes ikke
unsupported-architecture = Arkitekturen støttes ikke

verifying-checksum = Verifiserer sjekksum...
checksum-verified = Sjekksum verifisert.
error-checksum-mismatch = Sjekksumfeil for fila { $file }
error-invalid-sha-length = Ugyldig SHA-lengde { $length }
warning-sha-sums-missing = Sjekksumfiler ble ikke funnet for denne utgivelsen. Hopper over verifisering.

error-find-user-dirs = Klarte ikke å finne brukermappene.

fetching-releases = Henter utgivelser...
releases-fetched = Utgivelser henta.
error-fetching-releases = Feil ved henting av utgivelser: { $error }
warning-fetching-releases-using-cache = Feil ved henting av utgivelser: { $error }. Bruker hurtigbuffer i stedet.

error-version-not-found = Versjonen ble ikke funnet.
error-multiple-versions-found = Flere versjoner samsvarer med forespørselen:

running-version = Kjører versjon {$version}
link-created = Lenket {$version} til {$path}
copy-created = Kopierte {$version} til {$path}
no-matching-releases = Ingen samsvarende utgivelser funna.
available-releases = Tilgjengelige utgivelser:
cache-cleared = Cachen ble tømt.
cache-refreshed = Cachen ble oppdatert.

version-already-installed = Versjon {$version} er allerede installert.
godot-executable-not-found = Godot-kjørbar fil ble ikke funnet for versjon {$version}.
error-link-exists = Stien {$path} finnes allerede. Bruk --force for å overskrive.
error-link-symlink = Klarte ikke å opprette lenke fra {$link} til {$target}: {$error}
error-link-copy = Klarte ikke å kopiere kjørbar: {$error}
error-link-godotsharp-target = Klarte ikke å finne GodotSharp-målsti.
error-link-godotsharp-missing = GodotSharp-katalogen mangler ved siden av den løste kjørbaren.

error-no-stable-releases-found = Ingen stabile versjoner funnet.

error-starting-godot = Kunne ikke starte Godot: { $error }

confirm-remove = Er du sikker på at du vil fjerne denne versjonen? (ja/nei):
confirm-yes = ja
remove-cancelled = Fjerning avbrutt.

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

help-upgrade = Oppgrader gdvm til nyeste versjon
help-upgrade-major = Tillat oppgradering på tvers av hovedversjoner
upgrade-starting = Starter oppgradering av gdvm...
upgrade-downloading-latest = Laster ned nyeste gdvm...
upgrade-complete = gdvm ble oppgradert!
upgrade-not-needed = gdvm er allerede på siste versjon: { $version }.
upgrade-current-version-newer = Den nåværende gdvm-versjonen ({ $current }) er nyere enn den siste tilgjengelige versjonen ({ $latest }). Ingen oppgradering nødvendig.
upgrade-failed = Oppgradering mislyktes: { $error }
upgrade-download-failed = Nedlasting av oppgradering mislyktes: { $error }
upgrade-file-create-failed = Klarte ikke å opprette oppgraderingsfila: { $error }
upgrade-file-write-failed = Klarte ikke å skrive til oppgraderingsfila: { $error }
upgrade-install-dir-failed = Klarte ikke å opprette installasjonskatalogen: { $error }
upgrade-rename-failed = Klarte ikke å endre navnet på den nåværende kjørbare fila: { $error }
upgrade-replace-failed = Klarte ikke å erstatte den kjørbare fila med den nye: { $error }
checking-updates = Sjekker etter oppdateringer til gdvm...
upgrade-available = 💡 En ny versjon av gdvm er tilgjengelig: {$version}. Kjør «gdvm upgrade» for å oppdatere.
upgrade-available-major = 💡 Ei hovedversjonsoppdatering av gdvm er tilgjengelig: {$version}. Kjør «gdvm upgrade -m» for å oppdatere.
upgrade-available-both = 💡 En ny versjon av gdvm er tilgjengelig: {$minor_version}. Ei hovedversjonsoppdatering er også tilgjengelig: {$major_version}. Kjør «gdvm upgrade» for å oppdatere innen gjeldende hovedversjon, eller «gdvm upgrade -m» for å oppgradere til aller siste versjon.

help-pin = Fest en versjon av Godot til gjeldende mappe.
help-pin-long = { help-pin }

    Dette vil opprette en gdvm.toml-fil i gjeldende mappe med den festa versjonen. Når du kjører «gdvm run» i denne katalogen eller noen av underkatalogene, vil den festa versjonen brukes i stedet for standardversjonen.

    Dette er nyttig når du vil bruke en spesifikk versjon av Godot for et prosjekt uten å endre standardversjonen systemomfattende.

    Dette skriver foreløpig også den eldre .gdvmrc-fila for kompatibilitet med eldre versjoner av gdvm. Dette vil bli fjerna i en framtidig utgivelse, så det anbefales å gå over til det nye gdvm.toml-formatet og fjerne .gdvmrc-fila hvis den finnes.

    Du kan deaktivere skriving av en .gdvmrc-fil med --no-legacy-flagget.
help-pin-version = Versjonen som skal festes
help-no-legacy = Ikke skriv den eldre .gdvmrc-kompatibilitetsfila
pinned-success = Versjon {$version} ble festet i gdvm.toml
error-pin-version-not-found = Kan ikke feste versjon {$version}
pin-subcommand-description = Sett eller oppdater gdvm.toml med forespurt versjon

error-file-not-found = Fil ble ikke funnet. Den finnes kanskje ikke på serveren.
error-download-failed = Nedlasting mislyktes på grunn av en uventa feil: { $error }
error-ensure-godot-binaries-failed = Kunne ikke forsikre Godot-kjørbare filer.
    Feil: { $error }.
    Prøv å slette { $path } og kjør gdvm på nytt.

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
        [1] Hvis du er sikker på at du vil kjøre prosjektet med den festa versjonen, kjør {"\u001b"}[0mgdvm run --force{"\u001b"}[31m. Ellers oppdater den festa versjonen i .gdvmrc for å samsvare med prosjektversjonen, eller fjern .gdvmrc-fila for å bruke prosjektversjonen.
        *[0] Hvis du er sikker på at du vil kjøre prosjektet med den forespurte versjonen, kjør {"\u001b"}[0mgdvm run --force <version>{"\u001b"}[31m.
    }{"\u001b"}[0m
warning-project-version-mismatch-force = {"\u001b"}[33mHopper over bekreftelsesprompt og fortsetter med den { $pinned ->
        [1] festa
        *[0] forespurte
    } versjonen {"\u001b"}[0m({ $requested_version }){"\u001b"}[33m.{"\u001b"}[0m

help-run-args = Tilleggsargumenter som skal sendes til Godot-kjørbar fil (f.eks. -- path/to/project.godot).
help-run-force =
    Tving kjøring av prosjektet med den forespurte eller festa versjonen selv om den ikke samsvarer med prosjektversjonen.
help-run-force-long =
    Tving kjøring av prosjektet med den forespurte eller festa versjonen selv om den ikke samsvarer med prosjektversjonen.

    Hvis du gjør dette, kan den forespurte eller festa versjonen av Godot overskrive prosjektfila. Hvis du fester versjoner, anbefales det i stedet å oppdatere den festa versjonen i .gdvmrc for å samsvare med prosjektversjonen, eller fjerne .gdvmrc-fila for å bruke prosjektversjonen.

help-config = Administrer gdvm-konfigurasjon
help-config-get = Hent en konfigurasjonsverdi
help-config-set = Sett en konfigurasjonsverdi
help-config-unset = Fjern en konfigurasjonsverdi
help-config-list = List alle konfigurasjonsverdier
help-config-key = Konfigurasjonsnøkkelen (f.eks. github.token)
help-config-value = Verdien som skal settes for konfigurasjonsnøkkelen
help-config-unset-key = Konfigurasjonsnøkkelen som skal fjernes (f.eks. github.token)
help-config-show-sensitive = Vis sensitive konfigurasjonsverdier i klartekst
help-config-available = List alle tilgjengelige konfigurasjonsnøkler og verdier, inkludert standardverdier
warning-setting-sensitive = {"\u001b"}[33mAdvarsel: Du setter en sensitiv verdi som vil lagres i klartekst i hjemmemappa di.{"\u001b"}[0m
config-set-prompt = Vennligst oppgi verdien for { $key }:
error-reading-input = Feil ved lesing av inndata
config-set-success = Konfigurasjonen ble oppdatert.
config-unset-success = Konfigurasjonsnøkkelen { $key } ble fjernet vellykket.
config-key-not-set = Konfigurasjonsnøkkel ikke satt.
error-unknown-config-key = Ukjent konfigurasjonsnøkkel.
error-invalid-config-subcommand = Ugyldig config-underkommando. Bruk "get", "set" eller "list".
error-parse-config = Kunne ikke tolke konfigurasjonsfila: { $error }
error-parse-config-using-default = {"\u001b"}[33mBruker standard konfigurasjonsverdier.{"\u001b"}[0m
error-github-api = GitHub API-feil: { $error }
error-github-rate-limit = GitHub API si rate-begrensing overskredet.

  For å løse dette, vennligst opprett et personlig tilgangstoken på GitHub ved å besøke https://github.com/settings/tokens.

  Klikk på "Generate new token", velg kun de minimale tillatelsene som kreves (f.eks. public_repo), og sett deretter tokenet via miljøvariabelen GITHUB_TOKEN eller ved å kjøre:

    gdvm config set github.token

  Merk: Tokenet vil bli lagret i klartekst i hjemmekatalogen din. Vennligst sørg for at du holder det sikkert.
  Det anbefales å regelmessig gjennomgå og rotere tokenene dine for sikkerhetsformål.

error-copy-file-failed = Kunne ikke kopiere filen: { $error }
error-move-file-failed = Kunne ikke flytte filen: { $error }
error-user-dir-not-found = Kunne ikke opprette snarvei: Brukerkatalogen ble ikke funnet
error-desktop-not-found = Kunne ikke opprette snarvei: Skrivebordskatalogen ble ikke funnet
warning-shortcut-macos-not-supported = For øyeblikket støttes ikke oppretting av snarveier i MacOS, så snarveien vil ikke bli opprettet.