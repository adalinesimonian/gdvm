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
help-help = Vis hjelp (sjå eit samandrag med '-h')
help-gdvm-version = Vis versjonen av { -godot }-versjonsbehandlaren

help-install = Installer ein ny { -godot }-versjon
help-run = Køyr ein spesifikk { -godot }-versjon
help-show = Vis stien til den køyrberre fila for den gjevne { -godot }-versjonen
help-cache-path = Vis stigen til nedlastingsarkivet i cachen for den oppgjevne { -godot }-versjonen
help-link = Opprett ei lenkje frå ein { -godot }-versjon si køyrbare fil til ein oppgjeven stig
help-list = List alle installerte { -godot }-versjonar
help-remove = Fjern ein installert { -godot }-versjon
help-csharp = [avvikla] Bruk { -godot }-versjonen med C#-støtte. Bruk variantspesifikatoren «csharp» i staden (t.d. csharp:4.4).
help-run-csharp-long = { help-csharp }
help-version = Versjonen som skal installerast (t.d. 4, csharp:4.4, stable, latest).
help-version-long =
    { help-version }

    Format: [variant:]versjon_eller_nykelord

    Viss ein avsluttande * er til stades, treffer han den nyaste utgåva med same prefiks, t.d. treffer «4.7-dev*» 4.7-dev1, 4.7-dev2 osb.

    Nøkkelord: «latest» løyser til den nyaste versjonen. Som standard inkluderer dette berre stabile utgjevingar, men førehandsversjonar kan inkluderast med --pre-flagget.

    Variantar: prefiks med eit variantnamn og kolon, t.d. «csharp:4.4» for C#-versjonen.

    Døme: 4.4 vil installere den siste stabile utgjevinga av { -godot } 4.4. Viss berre førehandsversjonar finst, vil den siste førehandsversjonen verta installert. 4.3-rc* vil installere den siste utgjevinga av { -godot } 4.3, osb.
help-version-installed = Den installerte versjonen (t.d. 4.2 eller 4.2-stable).

help-search = List tilgjengelege utgjevingar frå registeret
help-filter = Valfri streng for å filtrere utgjevingstaggar
help-filter-deprecated = [avvikla] Valfri streng for å filtrere utgjevingstaggar. Bruk posisjonsargumentet for filter i staden.
help-include-pre = Inkluder førehandsversjonar (rc, beta, dev)
help-cache-only = Bruk berre bufra utgjevingsinformasjon utan å spørja registeret
help-limit = Talet på utgjevingar som skal visast, standard er 10. Bruk 0 for å vise alle
help-clear-cache = Tøm utgjevingscachen
help-refresh = Oppdater utgjevingscachen frå registeret
help-refresh-flag = Oppdater utgjevingscachen før denne kommandoen vert køyrd

help-prune = Fjern installasjonar og cacha arkiv som ikkje lenger er i bruk
help-prune-long = { help-prune }

    Som standard fjernar prune installasjonar som ikkje har vore bruka på ei stund, og cacha nedlastingsarkiv som har vorte for gamle, medan installasjonar som framleis har ei lenkje inn i seg vert tekne vare på. Installasjonen som er sett som standard vert aldri fjerna, uansett kva flagg som vert gjeve. Aldersgrensa kan setjast med «{ -gdvm } config set prune.max-age-days <dagar>» (standard { $default_days } dagar).
help-prune-all = Fjern alle installasjonar og cacha arkiv uavhengig av alder. Installasjonar som framleis har ei aktiv lenkje vert tekne vare på med mindre --force òg er gjeve.
help-prune-force = Ignorer lenkjer, slik at installasjonar som berre er refererte av ei lenkje òg kan fjernast.
help-prune-dry-run = Vis kva som ville vorte fjerna utan å sletta noko.
prune-nothing-dry-run = Ingenting ville vorte fjerna.
prune-nothing-removed = Ingenting å fjerna; alt er i bruk eller innanfor aldersgrensa.
prune-preserved-by-link =
    { $count ->
        [one] Tok vare på { $count } installasjon som framleis er referert av ei lenkje.
       *[other] Tok vare på { $count } installasjonar som framleis er refererte av ei lenkje.
    }
warning-broken-install-reinstalling = Den installerte { $version } manglar den køyrberre fila, installerer han på nytt.

help-force = Tving installasjon på nytt sjølv om versjonen alt er installert.
help-redownload = Last ned versjonen på nytt sjølv om han alt er lasta ned i cachen.
help-yes = Hopp over stadfestingsprompt for fjerning
help-remove-yes-deprecated = [avvikla] Dette flagget gjer ingenting og vil verta fjerna i ei framtidig utgåve.
help-link-version = Versjonen som skal lenkjast. Viss ho ikkje vert oppgjeven, vert versjonen løyst basert på gjeldande mappe eller standardversjonen.
help-link-path = Stien der lenkja eller kopien skal opprettast, t.d. «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }».
help-link-force = Overskriv eksisterande lenkje om ho finst
help-link-copy = Kopier køyrbar i staden for å lage lenkje
no-cache-files-found = Ingen cache-filer funne.
no-cache-metadata-found = Inkje cache-metadata funne.
gdvm-toml-malformed = ignorerer { -gdvm-toml } på { $path } fordi han ikkje kunne tolkast: { $error }

help-diagnose = Kontroller installasjonen og rapporter tilstanden.
diagnose-base-dir = { -gdvm }-mappe: { $path }
diagnose-healthy = Ingen problem funne.
diagnose-install-broken = { $version } manglar den køyrberre fila. Køyr «{ -gdvm } install» for han for å installere på nytt.
diagnose-install-ok = { $version } kan køyrast.
diagnose-partial-downloads =
    { $count ->
        [one] { $count } avbroten nedlasting i cachen; ho vert teken opp att automatisk, eller «{ -gdvm } prune» fjernar ho.
       *[other] { $count } avbrotne nedlastingar i cachen; dei vert tekne opp att automatisk, eller «{ -gdvm } prune» fjernar dei.
    }
diagnose-path-missing = { $path } er ikkje i PATH; godot-shimen vil ikkje vere tilgjengeleg ved namn.
diagnose-path-ok = bin-mappa er i PATH.
diagnose-shim-missing = Shimen «{ $name }» manglar eller er ikkje køyrberr. Reinstallering av { -gdvm } skriv han på nytt.
diagnose-shim-ok = Shimen «{ $name }» er installert og køyrberr.

help-console = Køyr { -godot } med konsoll tilkopla. Standard er false på Windows, true på andre plattformer.

help-default = Administrer standardversjonen
help-default-version = Versjonen som skal setjast som standard (t.d. 4.2 eller 4.2-stable).
no-default-set = Ingen standardversjon er sett. Køyr «{ -gdvm } use <version>» for å setja ein standardversjon systemomfattande, eller «{ -gdvm } pin <version>» for å setja ein standardversjon for den gjeldende mappa.

warning-prerelease = Du installerer ein førehandsversjon ({$branch}).
warning-deprecated-csharp-flag = Flagget --csharp er avvikla. Bruk "csharp"-variantspesifikatoren i staden (t.d. csharp:4.4).

label-error = Feil:
label-note = Merknad:
label-warning = Åtvaring:
progress-rate = { size-display }/s
progress-eta-remaining = { $time } att
progress-fraction = { $done }/{ $total }
status-downloading = Lastar ned
status-extracting = Pakkar ut
status-fetching = Hentar
status-installed = Installert
status-installing = Installerer
status-removed = Fjerna
status-healthy = Frisk
status-ok = OK
prune-item-detail = { $label } ({ size-display })
status-freed = Frigjort
status-pruned = Rydda
status-would-free = Ville frigjort
status-would-prune = Ville rydda
status-removing = Fjernar
status-running = Køyrer
status-cleared = Tømd
status-refreshed = Oppdatert
status-skipped = Hoppa over
status-upgraded = Oppgradert
status-upgrading = Oppgraderer
status-verifying = Verifiserer
subject-cached-archive = bufra arkiv
subject-cache = cache
subject-cache-files = cache-filer
subject-cache-metadata = cache-metadata
subject-releases = utgjevingar
subject-update-manifest = oppdateringsmanifest
upgrade-target = { -gdvm } { $version }

auto-installing-version = Automatisk installasjon av versjon { $version }

no-versions-installed = Ingen versjonar installerte.
installed-versions = Installerte { -godot }-versjonar:
progress-eta =
    { $magnitude ->
        [seconds] { $secs }s
        [minutes] { $mins }m { $secs }s
       *[hours] { $hours }t { $mins }m
    }

unsupported-platform = Plattforma er ikkje støtta
unsupported-architecture = Arkitekturen er ikkje støtta
error-checksum-mismatch = Sjekksumfeil for fila { $file }
error-invalid-sha-length = Ugyldig SHA-lengde { $length }
error-size-mismatch = Storleiksavvik for fila { $file }: forventa { $expected } byte, fekk { $actual } byte.
error-insecure-url = Nektar å hente { $url } over ei ukryptert tilkopling. Berre https://- og file://-URL-ar er tillatne. Set miljøvariabelen GDVM_ALLOW_INSECURE_URLS for å tillate ukrypterte http://-URL-ar.
error-insecure-redirect = Nektar å fylgje ei omdirigering frå https:// til ein ukryptert http://-URL. Set miljøvariabelen GDVM_ALLOW_INSECURE_URLS for å tillate ukrypterte http://-URL-ar.
error-response-not-utf8 = Svaret frå { $url } er ikkje gyldig UTF-8.
error-response-too-large = Svaret frå { $url } overskrid den maksimale tillatne storleiken på { $limit } byte.
error-too-many-redirects = For mange omdirigeringar.
error-config-invalid-number = Ugyldig verdi for { $key }: { $value } (venta eit tal)
error-config-unknown-key = Ukjend konfigurasjonsnøkkel: { $key }
error-config-path-empty = Stien kan ikkje vere tom.
error-config-path-file = Stien peikar til ei fil, ikkje ein mappe: { $path }
error-config-path-reserved = Stien er reservert for gdvm sine interne delar: { $path }
error-config-path-overlap = Konfigurerte stiar må ikkje overlappe: { $key }
error-invalid-path = Ugyldig sti: { $path }
error-publish-missing-manifest = registry.json manglar
error-publish-no-such-version = ingen slik versjon: { $version }
error-publish-store-or-url-required = anten --store eller --url må oppgjevast
error-publish-store-requires-file = --store krev ein lokal --file
error-publish-url-requires-integrity = --url krev anten ein lokal --file eller eksplisitte --sha512 og --size
error-publish-already-initialized = Registeret er alt initialisert på { $path }
error-publish-archive-not-found = Arkiv ikkje funne: { $path }
error-publish-no-such-platform = Inga slik plattform { $platform } for varianten { $variant }
error-publish-no-such-variant = Ingen slik variant: { $variant }
error-publish-invalid-segment = Ugyldig { $what }: { $value }
error-registry-fetch-failed = Klarte ikkje å hente { $url }: HTTP { $status }
error-registry-fetch-release-failed = Klarte ikkje å hente utgjevingsmetadata
error-registry-invalid-name = Ugyldig registernamn: { $name }
error-registry-missing-index = Registeret «{ $name }» manglar index.json
error-registry-missing-manifest = Registeret «{ $name }» manglar registry.json
error-registry-not-configured = Registeret «{ $name }» er ikkje konfigurert
error-registry-parse-index = Klarte ikkje å tolke indeksen for «{ $name }».
error-registry-parse-manifest = Klarte ikkje å tolke manifestet for «{ $name }».
error-registry-unknown = Ukjend register «{ $name }»
error-registry-unsupported-url-scheme = URL-skjemaet til registeret er ikkje støtta: { $url }
error-spec-empty-registry = Tomt registernamn i «{ $input }»
error-spec-empty-variant = Tomt variantnamn i «{ $input }»
error-spec-empty-version = Tom versjon i «{ $input }»
error-system-time = Systemtida er før UNIX-epoken
error-unrecognized-version-format = Ukjent versjonsformat: { $input }
error-diagnose-problems = { $count } problem funne.
error-non-interactive-trust = Kan ikkje spørja om å stole på registeret «{ $registry }» ({ $url }) i ei økt som ikkje er interaktiv. Send --yes for å stole på det eksplisitt.
error-non-interactive-value = Kan ikkje bede om ein verdi for «{ $key }» i ei økt som ikkje er interaktiv. Send verdien som eit argument i staden.
error-registry-unsupported-schema = Registeret «{ $registry }» oppgjev ein skjemaversjon som ikkje er stødd: { $schema }.
label-caused-by = Forårsaka av:
label-error-coded = Feil { $code }:
error-wildcard-position = Jokerteiknet (*) kan berre stå på slutten av utgjevingstaggen, t.d. 4.7-dev* (fekk { $input }).
hint-try-wildcard = Inga utgjeving har taggen { $requested }, men det finst liknande taggar, der den nyaste er { $newest }. Prøv { $suggestion } for å treffe dei.
download-retrying = Nedlastinga vart avbroten, prøver på nytt (forsøk { $attempt } av { $max })...
download-resuming = Tek opp att avbroten nedlasting ({ size-display } alt lasta ned).
warning-resume-verification-failed = Den oppattekne nedlastinga samsvarte ikkje med venta kontrollsum, lastar ho ned på nytt frå botnen av.
lock-waiting = Ventar på at ein annan { -gdvm }-prosess skal verta ferdig (lås: { $resource })...
prune-skipped-error = Hoppar over { $item }: { $error }
prune-skipped-in-use = Hoppar over { $item }: han er i bruk av ein annan { -gdvm }-prosess.

error-find-user-dirs = Klarte ikkje å finne brukarmappene.
warning-fetching-releases-using-cache = Feil ved henting av utgjevingar: { $error }. Brukar hurtigbuffer i staden.

error-version-not-found = Versjonen vart ikkje funnen.
error-archive-not-cached = Fann ikkje noko arkiv i cachen for {$version}. Installer han fyrst for å fylle cachen.
error-multiple-versions-found = Fleire versjonar samsvarar med førespurnaden:
    {$list}
link-created = Lenkja {$version} til {$path}
copy-created = Kopierte {$version} til {$path}
no-matching-releases = Ingen samsvarande utgjevingar funne.
available-releases = Tilgjengelege utgjevingar:

version-already-installed = Versjon {$version} er alt installert.
godot-executable-not-found = { -godot }-køyrberr fil vart ikkje funnen for versjon {$version}.
error-link-exists = Stigen {$path} finst allereie. Bruk --force for å overskrive.
error-link-symlink = Klarte ikkje å opprette lenkje frå {$link} til {$target}.
error-link-copy = Klarte ikkje å kopiere fil.

error-no-stable-releases-found = Ingen stabile utgivelser funne.

error-starting-godot = Kunne ikkje starte { -godot }.
confirm-yes = ja

default-set-success = Standardversjon {$version} er sett.
default-unset-success = Standardversjonen er fjerna.
provide-version-or-unset = Ver venleg og oppgjev ein versjon for å setja som standard eller «unset» for å fjerne standardversjonen.

error-open-zip = Kunne ikkje opne ZIP-fila { $path }.
error-read-zip = Kunne ikkje lesa ZIP-arkivet { $path }.
error-access-file = Kunne ikkje få tilgang til fila ved indeks { $index }.
error-reopen-zip = Kunne ikkje opne ZIP-fila på nytt { $path }.
error-invalid-file-name = Ugyldig filnamn i ZIP-arkivet
error-create-dir = Kunne ikkje opprette katalogen { $path }.
error-create-file = Kunne ikkje opprette fila { $path }.
error-read-zip-file = Kunne ikkje lesa frå ZIP-fila { $file }.
error-write-file = Kunne ikkje skrive til fila { $path }.
error-strip-prefix = Kunne ikkje fjerne prefiks.
error-set-permissions = Kunne ikkje setja løyve for { $path }.
error-create-symlink-windows = Kunne ikkje opprette symlink. Kontroller at {"\u001b"}]8;;ms-settings:developers{"\u001b"}\utviklarmodus{"\u001b"}]8;;{"\u001b"}\ er aktivert eller køyr som administrator.

help-upgrade = Oppgrader { -gdvm } til nyaste versjon
help-upgrade-major = Tillat oppgradering på tvers av hovudversjonar
help-upgrade-pre = Oppgrader til nyaste førehandsutgjeving
upgrade-not-needed = { -gdvm } er alt på siste versjon: { $version }.
upgrade-current-version-newer = Den noverande { -gdvm }-versjonen ({ $current }) er nyare enn den siste tilgjengelege versjonen ({ $latest }). Inga oppgradering nødvendig.
upgrade-install-dir-failed = Klarte ikkje å opprette installasjonskatalogen.
upgrade-rename-failed = Klarte ikkje å endre namnet på den noverande køyrberre fila.
upgrade-replace-failed = Klarte ikkje å erstatte den køyrberre fila med den nye.
upgrade-no-binary = Inga { -gdvm }-binærfil er tilgjengeleg for versjon { $version } og målet { $target }.
upgrade-checksum-required = Utgjevingsmanifestet inneheld ingen sjekksum for denne { -gdvm }-binærfila. Nektar å oppgradere.
error-fetching-gdvm-releases = Feil ved henting av { -gdvm }-utgjevingar.
error-parsing-gdvm-releases = Feil ved tolking av { -gdvm }-utgjevingar.
error-unsupported-gdvm-schema = Skjemaversjon for { -gdvm }-utgjevingsmanifestet er ikkje støtta: { $schema }. Prøv å oppgradere { -gdvm } manuelt.
upgrade-available = 💡 Ein ny versjon av { -gdvm } er tilgjengeleg: {$version}. Køyr «{ -gdvm } upgrade» for å oppdatere.
upgrade-available-major = 💡 Ei hovudversjonsoppdatering av { -gdvm } er tilgjengeleg: {$version}. Køyr «{ -gdvm } upgrade -m» for å oppdatere.
upgrade-available-both = 💡 Ein ny versjon av { -gdvm } er tilgjengeleg: {$minor_version}. Ei hovudversjonsoppdatering er òg tilgjengeleg: {$major_version}. Køyr «{ -gdvm } upgrade» for å oppdatere innan gjeldande hovudversjon, eller «{ -gdvm } upgrade -m» for å oppgradere til aller siste versjon.
upgrade-prerelease-available = 💡 Ei nyare førehandsutgjeving av { -gdvm } er tilgjengeleg. Køyr «{ -gdvm } upgrade --pre» for å installere ho.

help-pin = Fest ein versjon av { -godot } til gjeldande mappe.
help-pin-long = { help-pin }

    Dette vil opprette ei { -gdvm-toml }-fil i gjeldande mappe med den festa versjonen. Når du køyrer «{ -gdvm } run» i denne katalogen eller nokre av underkatalogane, vil den festa versjonen verta bruka i staden for standardversjonen.

    Dette er nyttig når du vil bruke ein spesifikk versjon av { -godot } for eit prosjekt utan å endre standardversjonen systemomfattande.

    Dette skriv førebels òg den eldre { -gdvmrc }-fila for bakoverkompatibilitet med eldre versjonar av { -gdvm }. Dette vil verta fjerna i ei framtidig utgjeving, so det er tilrådd å gå over til det nye { -gdvm-toml }-formatet og fjerne { -gdvmrc }-fila om ho finst.

    Du kan deaktivere skriving av ei { -gdvmrc }-fil med --no-legacy-flagget.
help-pin-version = Versjonen som skal festast
help-no-legacy = Ikkje skriv den eldre { -gdvmrc }-kompatibilitetsfila
pinned-success = Versjon {$version} vart festa i { -gdvm-toml }
error-pin-version-not-found = Kan ikkje feste versjon {$version}

error-file-not-found = Fil vart ikkje funnen. Ho finst kanskje ikkje på tenaren.
error-download-failed = Nedlasting feila med HTTP-status { $status }.
error-ensure-godot-binaries-failed = Kunne ikkje sikre { -godot }-køyrberre filer.

error-post-upgrade-action-failed = Trinnet { $id } mislukkast etter oppgraderinga.
    { -gdvm }-installasjonen din kan vera ufullstendig. Prøv å køyre { -gdvm } på nytt.

error-failed-reading-project-godot = Kunne ikkje lesa project.godot, kan ikkje automatisk bestemme prosjektversjonen.
warning-using-project-version = Brukar versjon { $version } definert i project.godot.
warning-gdvmrc-detected = Ei eigendefinert { -gdvmrc }-fil vart oppdaga. Støtte for { -gdvmrc }-filer er forelda og vil verta fjerna i ei framtidig utgjeving. Ver venleg og byt til den nye festefila som vert bruka av `{ -gdvm } pin`.

warning-project-version-mismatch =
    {"\u001b"}[33mÅtvaring: Versjonen definert i project.godot samsvarar ikkje med den { $pinned ->
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
             Ynskt versjon: { $requested_version }
    }

error-project-version-mismatch =
    {"\u001b"}[31m{ $pinned ->
        [1] Om du er sikker på at du vil køyre prosjektet med den festa versjonen, køyr {"\u001b"}[0m{ -gdvm } run --force{"\u001b"}[31m. Elles oppdater den festa versjonen i { -gdvmrc } for å samsvara med prosjektversjonen, eller fjern { -gdvmrc }-fila for å bruke prosjektversjonen.
        *[0] Om du er sikker på at du vil køyre prosjektet med den ynskte versjonen, køyr {"\u001b"}[0m{ -gdvm } run --force <version>{"\u001b"}[31m.
    }{"\u001b"}[0m

help-run-args = Tilleggsargument som skal sendast til { -godot }-køyrbar fil (t.d. -- path/to/project.godot).
help-run-force =
    Tving køyring av prosjektet med den ynskte eller festa versjonen sjølv om han ikkje samsvarar med prosjektversjonen.
help-run-force-long =
    { help-run-force }

    Viss du gjer dette, kan den ynskte eller festa versjonen av { -godot } overskrive prosjektfila. Viss du festar versjonar, er det tilrådd i staden å oppdatere den festa versjonen i { -gdvmrc } for å samsvara med prosjektversjonen, eller fjerne { -gdvmrc }-fila for å bruke prosjektversjonen.

help-config = Administrer { -gdvm }-konfigurasjon
help-format = Utdataformat: text (standard) eller json
help-info = Vis detaljert informasjon om ein installert versjon
info-default =
    { $value ->
        [1] { confirm-yes }
       *[0] { info-no }
    }
    .label = Standard:
info-executable = { $path }
    .label = Køyrberr fil:
info-install-path = { $path }
    .label = Installasjonsstig:
info-last-used = { $timestamp }
    .label = Sist bruka:
info-no = nei
info-registry = { $registry }
    .label = Register:
info-size = { size-display }
    .label = Storleik på disk:
info-variant = { $variant }
    .label = Variant:
info-version = { $version }
    .label = Versjon:
help-completions = Generer skript for skalfullføring
help-completions-shell = Skalet det skal genererast fullføringar for
help-config-get = Hent ein konfigurasjonsverdi
help-config-set = Set ein konfigurasjonsverdi
help-config-unset = Fjern ein konfigurasjonsverdi
help-config-list = List alle konfigurasjonsverdiar
help-config-key = Konfigurasjonsnykelen (t.d. prune.max-age-days)
help-config-value = Verdien som skal setjast for konfigurasjonsnykelen
help-config-unset-key = Konfigurasjonsnykelen som skal fjernast (t.d. prune.max-age-days)
help-config-show-sensitive = Vis sensitive konfigurasjonsverdiar i klårtekst
help-config-available = List alle tilgjengelege konfigurasjonsnyklar og verdiar, inkludert standardverdiar
warning-setting-sensitive = Du set ein sensitiv verdi som vil verta lagra i klårtekst i heimemappa di.
config-set-prompt = Ver venleg og oppgjev verdien for { $key }:
error-reading-input = Feil ved lesing av inndata
config-set-success = Konfigurasjonen vart oppdatert.
config-unset-success = Konfigurasjonsnykelen { $key } vart fjerna vellukka.
config-key-not-set = Konfigurasjonsnykel ikkje sett.
config-key-not-set-value = <ikkje sett>
error-unknown-config-key = Ukjend konfigurasjonsnykel.
error-invalid-config-subcommand = Ugyldig config-underkommando. Bruk «get», «set» eller «list».
error-parse-config = Kunne ikkje tolke konfigurasjonsfila.
error-parse-config-using-default = Brukar standard konfigurasjonsverdiar.

help-registry = Administrer register å installere { -godot }-bygg frå
help-registry-add = Legg til eit register
help-registry-remove = Fjern eit register
help-registry-list = List opp konfigurerte register
help-registry-refresh = Oppdater hurtigbufferen for eitt eller alle register
help-registry-name = Registernamnet
help-registry-url = Register-URL-en. Kan vera ein http(s)://- eller file://-URL.

registry-added = La til registeret { $registry } ({ $url }).
registry-removed = Fjerna registeret { $registry }.
registry-list-header = Konfigurerte register:
registry-tag-official = offisielt

error-invalid-registry-subcommand = Ugyldig register-underkommando. Bruk «add», «remove», «list» eller «refresh».
registry-trust-warning = { $registry } ({ $url }) er eit eigendefinert register, ikkje det offisielle. { -gdvm } sjekkar at nedlastingar stemmer med det registeret oppgjev, men kan ikkje vite om dei er trygge å køyre. Installer frå det berre om du stolar på dei som driv det.
registry-trust-prompt = Stoler du på dette registeret og vil halde fram? (ja/nei):
registry-trust-bypass = {"\u001b"}[1;31mHoppar over tillitssjekken for { $registry } ({ $url }) fordi du brukte --yes. { -gdvm } kan ikkje vite om filene er trygge å køyre. Tek ein kort pause; trykk Ctrl+C no for å stoppe.{"\u001b"}[0m
registry-trust-aborted = Avbrote: registeret er ikkje klarert.
registry-project-override-conflict = Prosjektet sin { -gdvm-toml } omdefinerer registeret { $registry } (konfigurasjonen din: { $machine_url }) som { $project_url }. Prosjektet sin definisjon har forrang.

help-registry-init = Initialiser ei ny registermappe
help-registry-add-build = Legg til eit bygg i eit register
help-registry-remove-build = Fjern eit bygg frå eit register
help-registry-validate = Valider ei registermappe
help-registry-dir = Registermappa
help-registry-init-name = Registernamnet. Standard er mappenamnet.

help-registry-build-version = Versjonsetiketten, t.d. 4.4-stable.
help-registry-build-variant = Variantnamnet. Standard er «default».
help-registry-build-platform = Plattformnøkkelen, t.d. linux-x86_64.
help-registry-build-file = Sti til byggarkivet som skal hashast
help-registry-build-store = Kopier arkivet inn i registeret og registrer ein relativ URL
help-registry-build-url = URL-en der arkivet skal serverast (når --store ikkje vert brukt)
help-registry-build-sha512 = SHA-512 til arkivet, i staden for å rekne det ut. Krev --size.
help-registry-build-size = Storleiken på arkivet i byte, i staden for å måle det. Krev --sha512.

registry-init-success = Initialiserte registeret { $name } i { $path }.
registry-build-added = La til bygget { $version } for { $platform }.
registry-build-removed = Fjerna bygget { $version }.
registry-build-warn-local-hash = Hashar den lokale fila og går ut frå at ho samsvarar med { $url }. { -gdvm } lastar ikkje ned URL-en for å stadfeste det.
registry-build-warn-unverified = Brukar SHA-512 og storleiken du oppgav utan å laste ned artefakten for å stadfeste dei. Sjå til at dei er rette.
registry-build-warn-explicit-store = Brukar SHA-512 og/eller storleiken du oppgav i staden for å måle det lagra arkivet.
registry-build-sha-mismatch = Oppgjeve SHA-512 ({ $expected }) samsvarar ikkje med artefakten ({ $actual }).
registry-build-size-mismatch = Oppgjeven storleik ({ $expected }) samsvarar ikkje med artefakten ({ $actual }).
registry-validate-ok =
    { $count ->
        [one] Registeret er gyldig ({ $count } artefakt kontrollert).
       *[other] Registeret er gyldig ({ $count } artefaktar kontrollerte).
    }
registry-validate-failed = Validering av registeret feila:
