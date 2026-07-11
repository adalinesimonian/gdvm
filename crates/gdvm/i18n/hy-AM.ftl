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

hello = Բարեւ, Աշխարհ:

help-about = Godot տարբերակների կառավարիչ
help-help = Ցուցադրել օգնություն (տես ամփոփում '-h'-ով)
help-help-command = Ցուցադրել այս հաղորդագրությունը կամ օգնությունը նշված ենթահրամանների համար
help-gdvm-version = Ցուցադրել Godot տարբերակների կառավարչի տարբերակը

help-install = Տեղադրել նոր Godot տարբերակ
help-run = Գործարկել որոշակի Godot տարբերակ
help-show = Ցույց տալ Godot-ի նշված տարբերակի գործարկվողի ուղին
help-cache-path = Ցույց տալ նշված Godot տարբերակի ներբեռնման պահված արխիվի ուղին
help-link = Կապել Godot-ի որոշակի տարբերակի գործարկվողը նշված ուղու հետ
help-list = Ցուցադրել բոլոր տեղադրված Godot տարբերակները
help-remove = Հեռացնել տեղադրված Godot տարբերակը

help-branch = Ճյուղը (կայուն, բետա, ալֆա կամ հարմարեցված):
help-csharp = [հնացած] Օգտագործել Godot տարբերակը C# աջակցության համար: Փոխարենը օգտագործեք «csharp» տարբերակի ցուցիչը (օրինակ՝ csharp:4.4):
help-run-csharp-long = { help-csharp }
help-version = Տեղադրվող տարբերակը (օրինակ՝ 4, csharp:4.4, stable, latest):
help-version-long =
    { help-version }

    Ձևաչափ՝ [տարբերակ:]տարբերակ_կամ_բանալի_բառ

    Բանալի բառեր։ «latest» համապատասխանում է ամենանոր տարբերակին: Լռելյայն ներառվում են միայն կայուն թողարկումները, սակայն նախնական թողարկումները կարելի է ներառել --pre դրոշակով:

    Տարբերակներ՝ նախածանցեք տարբերակի անունով և երկու կետով, օրինակ՝ «csharp:4.4» C# տարբերակի համար:

    Օրինակներ՝ 4.4-ը կտեղադրի Godot 4.4-ի վերջին կայուն թողարկումը: Եթե միայն նախնական թողարկումներ կան, ապա կտեղադրվի վերջին նախնական թողարկումը: 4.3-rc-ը կտեղադրի Godot 4.3-ի վերջին թողարկման թեկնածուն և այլն:
help-version-installed = Տեղադրված տարբերակը (օրինակ՝ 4.2 կամ 4.2-stable):

help-search = Ցուցադրել մատյանի հասանելի թողարկումները
help-filter = Ընտրովի տող թողարկման պիտակները ֆիլտրելու համար
help-include-pre = Ներառել նախնական թողարկումները (rc, beta, dev)
help-cache-only = Օգտագործել միայն պահված թողարկման տեղեկատվությունը առանց գրանցամատյանին հարցում անելու
help-limit = Ցուցադրվող թողարկումների քանակը, լռելյայն 10. Օգտագործեք 0՝ բոլորի ցուցադրման համար
help-clear-cache = Մաքրել թողարկումների քեշը
help-refresh = Թարմացնել թողարկումների քեշը գրանցամատյանից
help-refresh-flag = Թարմացնել թողարկումների քեշը հրամանը գործարկելուց առաջ

help-prune = Հեռացնել տեղադրումները և քեշավորված արխիվները, որոնք այլևս չեն օգտագործվում
help-prune-long = { help-prune }

    Լռելյայն prune-ը հեռացնում է այն տեղադրումները, որոնք երկար ժամանակ չեն օգտագործվել, ինչպես նաև հնացած քեշավորված ներբեռնման արխիվները՝ պահպանելով յուրաքանչյուր տեղադրում, որին դեռ ցույց է տալիս ակտիվ հղում։ Որպես լռելյայն սահմանված տեղադրումը երբեք չի հեռացվում՝ անկախ տրված դրոշակներից։ Հնության շեմը կարգավորելի է « gdvm config set prune.max-age-days <օրեր> » հրամանով (լռելյայն՝ { $default_days } օր)։
help-prune-all = Հեռացնել բոլոր տեղադրումներն ու քեշավորված արխիվները՝ անկախ հնությունից։ Ակտիվ հղում ունեցող տեղադրումները պահպանվում են, եթե նաև --force տրված չէ։
help-prune-force = Անտեսել հղումները, որպեսզի միայն հղումով հղվող տեղադրումները նույնպես հնարավոր լինի հեռացնել։
help-prune-dry-run = Ցույց տալ, թե ինչ կհեռացվեր՝ առանց որևէ բան ջնջելու։

prune-dry-run-header = Հետևյալը կհեռացվեր (փորձնական գործարկում)՝
prune-removed-header = Հեռացվեց հետևյալը՝
prune-installs-header = Տեղադրումներ՝
prune-archives-header = Քեշավորված արխիվներ՝
prune-nothing-dry-run = Ոչինչ չէր հեռացվի։
prune-nothing-removed = Հեռացնելու բան չկա. ամեն ինչ օգտագործվում է կամ հնության շեմի սահմաններում է։
prune-preserved-by-link = Պահպանվեց { $count } տեղադրում, որը դեռ հղվում է հղումով։
prune-freed = Ազատվեց մոտավորապես { $size }։
prune-would-free = Կազատվեր մոտավորապես { $size }։

help-force = Ստիպել վերատեղադրումը, նույնիսկ եթե տարբերակը արդեն տեղադրված է:
help-redownload = Նորից ներբեռնել տարբերակը, նույնիսկ եթե այն արդեն տեղադրված է:
help-yes = Բաց թողնել հեռացման հաստատման հուշումը
help-link-version = Այն տարբերակը, որը պետք է կապվի։ Եթե այն չի տրվում, տարբերակը որոշվում է ընթացիկ պանակի կամ լռելյայն տարբերակի հիման վրա։
help-link-path = Ուղին, որտեղ կստեղծվի հղումը կամ պատճենը, օրինակ «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }»։
help-link-force = Վերագրել գոյություն ունեցող հղումը, եթե կա
help-link-copy = Պատճենել գործարկվողը հղման փոխարեն

cached-zip-stored = Պահված է Godot թողարկումը արխիվում։
using-cached-zip = Օգտագործել պահված թողարկումը, ներբեռնումը անջատված է։
warning-cache-metadata-reset = Պահված թողարկումների ինդեքսը անվավեր է կամ կորուպտավորված է։ Վերսկսելու համար։
cache-files-removed = Պահված թողարկումները հեռացվել են։
cache-metadata-removed = Պահված թողարկումների ինդեքսը հեռացվել է։
error-cache-metadata-empty = Սխալ՝ պահված թողարկումների ինդեքսը դատարկ է, անհրաժեշտ է ստանալ թողարկումները առաջին համար։
no-cache-files-found = Պահված թողարկումներ չեն գտնվել։
no-cache-metadata-found = Պահված թողարկումների ինդեքսը չի գտնվել։
gdvm-toml-malformed = {"\u001b"}[33mԶգուշացում. { $path }-ի gdvm.toml-ն անտեսվում է, քանի որ հնարավոր չէ վերլուծել. { $error }{"\u001b"}[0m

help-console = Godot-ը կատարողական կոնսոլով: լռելյայն է Windows-ում, այլ համակարգչներում ակտիվացվում է:

help-default = Կառավարել լռելյայն տարբերակը
help-default-version = Տարբերակը, որը պետք է տեղադրվի լռելյայն (օրինակ՝ 4.2 կամ 4.2-stable):
no-default-set = Լռելյայն տարբերակը սահմանված չէ: Գործարկեք "gdvm use <version>"՝ լռելյայն տարբերակը համակարգային մակարդակով սահմանելու համար, կամ "gdvm pin <version>"՝ լռելյայն տարբերակը ընթացիկ պանակի համար սահմանելու համար:

installing-version = Տեղադրվում է {$version} տարբերակը
installed-success = {$version} տարբերակը հաջողությամբ տեղադրվեց

warning-prerelease = {"\u001b"}[33mԶգուշացում: Դուք տեղադրում եք նախնական թողարկում ({$branch}).{"\u001b"}[0m
warning-deprecated-csharp-flag = {"\u001b"}[33mԶգուշացում: --csharp դրոշակը հնացած է։ Փոխարենը օգտագործեք "csharp" տարբերակի ցուցիչը (օրինակ՝ csharp:4.4):{"\u001b"}[0m

force-reinstalling-version = Բարդ ընդկարեն տեղադրությունը տարբերակ {$version}.

auto-installing-version = Ինքնաշխատ տեղադրում { $version } տարբերակի

no-versions-installed = Տեղադրված տարբերակներ չկան:
installed-versions = Տեղադրված Godot տարբերակներ:
removed-version = {$version} տարբերակը հեռացված է
removing-version = Հեռացվում է {$version} տարբերակը

force-redownload = Տեղադրման վերանեղմումը տարբերակի {$version} համար։
operation-downloading-url = Ներբեռնում Ձեզ URL-ից {$url}...
operation-download-complete = Ամրագրումը ավարտվել է.
operation-extracting = Բացահայտում...
operation-extract-complete = Բացահայտումը ավարտված է.

unsupported-platform = Չաջակցվող համակարգ
unsupported-architecture = Չաջակցվող ստեղծածություն

verifying-checksum = Ստուգվում է ստորակետը...
checksum-verified = Ստորակետը ստուգված է:
error-checksum-mismatch = Ստորակետի չափազանելը չհամընկեց ֆայլի { $file }
error-invalid-sha-length = Անվավեր SHA երկարություն { $length }
error-size-mismatch = Չափի անհամապատասխանություն { $file } ֆայլի համար. սպասվում էր { $expected } բայթ, ստացվել է { $actual } բայթ։
error-insecure-url = Մերժվում է { $url }-ի բեռնումը չգաղտնագրված կապով։ Թույլատրվում են միայն https:// և file:// URL-ները։ Սահմանեք GDVM_ALLOW_INSECURE_URLS միջավայրի փոփոխականը՝ չգաղտնագրված http:// URL-ները թույլատրելու համար։
error-insecure-redirect = Մերժվում է վերաուղղորդումը https://-ից չգաղտնագրված http:// URL-ի։ Սահմանեք GDVM_ALLOW_INSECURE_URLS միջավայրի փոփոխականը՝ չգաղտնագրված http:// URL-ները թույլատրելու համար։
error-response-not-utf8 = { $url }-ի պատասխանը վավեր UTF-8 չէ․ { $error }
error-response-too-large = { $url }-ի պատասխանը գերազանցում է թույլատրված առավելագույն չափը՝ { $limit } բայթ։
error-too-many-redirects = Չափազանց շատ վերաուղղորդումներ։
error-config-invalid-number = Անվավեր արժեք { $key }-ի համար. { $value } (սպասվում էր թիվ)
error-config-unknown-key = Անհայտ կարգավորման բանալի. { $key }
error-invalid-path = Անվավեր ուղի. { $path }
error-publish-missing-manifest = registry.json-ը բացակայում է
error-publish-no-such-version = այդպիսի տարբերակ չկա. { $version }
error-publish-store-or-url-required = պետք է տրամադրվի --store կամ --url
error-publish-store-requires-file = --store-ը պահանջում է տեղական --file
error-publish-url-requires-integrity = --url-ը պահանջում է կամ տեղական --file, կամ բացահայտ --sha512 և --size
error-registry-fetch-failed = Չհաջողվեց բեռնել { $url }. HTTP { $status }
error-registry-fetch-release-failed = Չհաջողվեց բեռնել թողարկման մետատվյալները
error-registry-invalid-name = Անվավեր ռեգիստրի անուն. { $name }
error-registry-missing-index = « { $name } » ռեգիստրում բացակայում է index.json-ը
error-registry-missing-manifest = « { $name } » ռեգիստրում բացակայում է registry.json-ը
error-registry-not-configured = « { $name } » ռեգիստրը կարգավորված չէ
error-registry-parse-index = Չհաջողվեց վերլուծել « { $name } »-ի ինդեքսը. { $error }
error-registry-parse-manifest = Չհաջողվեց վերլուծել « { $name } »-ի մանիֆեստը. { $error }
error-registry-unknown = Անհայտ ռեգիստր « { $name } »
error-registry-unsupported-url-scheme = Ռեգիստրի URL-ի չաջակցվող սխեմա. { $url }
error-spec-empty-registry = Դատարկ ռեգիստրի անուն « { $input } »-ում
error-spec-empty-variant = Դատարկ տարբերակի անուն « { $input } »-ում
error-spec-empty-version = Դատարկ տարբերակ « { $input } »-ում
error-system-time = Համակարգի ժամանակը UNIX դարաշրջանից առաջ է
error-unrecognized-version-format = Տարբերակի չճանաչված ձևաչափ. { $input }

error-find-user-dirs = Չհաջողվեց գտնել օգտագործողի պանակները։

fetching-releases = Թողարկումները բերվում են...
releases-fetched = Թողարկումները բերվել են։
error-fetching-releases = Սխալ թողարկումների ստացման ժամանակ՝ { $error }
warning-fetching-releases-using-cache = Սխալ թողարկումների ստացման ժամանակ՝ { $error }։ Օգտագործվում են պահված թողարկումները։

error-version-not-found = Տարբերակը չի գտնվել:
error-archive-not-cached = {$version}-ի համար պահված արխիվ չի գտնվել։ Նախ տեղադրեք այն՝ քեշը լրացնելու համար։
error-multiple-versions-found = Մի քանի տարբերակներ են համընկնում ձեր հարցմանը:

running-version = Գործարկվում է {$version} տարբերակը
link-created = {$version}-ը կապվեց {$path}-ին
copy-created = {$version} տարբերակը պատճենվեց {$path} ուղու վրա
no-matching-releases = Համապատասխան թողարկումներ չեն գտնվել:
available-releases = Հասանելի թողարկումներ:
cache-cleared = Քեշը հաջողությամբ մաքրվել է:
cache-refreshed = Քեշը հաջողությամբ թարմացվել է:

version-already-installed = Տարբերակը արդեն տեղադրված է։ օգտագործեք {$version}:
godot-executable-not-found = Godot-ի գործարկվող ֆայլը չի գտնվել {$version} տարբերակի համար:
error-link-exists = {$path} ուղին արդեն գոյություն ունի։ Օգտագործեք --force՝ վերագրելու համար։
error-link-symlink = Չհաջողվեց ստեղծել հղումը {$link}-ից դեպի {$target}։ {$error}
error-link-copy = Կատարվողը պատճենելու ձախողում՝ {$error}

error-no-stable-releases-found = Կայուն թողարկումներ չեն գտնվել:

error-starting-godot = Չհաջողվեց գործարկել Godot-ը՝ { $error }

confirm-remove = Վստա՞հ եք, որ ցանկանում եք հեռացնել այս տարբերակը: (այո/ոչ):
confirm-yes = այո
remove-cancelled = Հեռացումը չեղարկված է:

default-set-success = Հաջողությամբ սահմանվել է {$version} որպես լռելյայն Godot տարբերակը։
default-unset-success = Հաջողությամբ հեռացվեց լռելյայն Godot տարբերակը։
provide-version-or-unset = Խնդրում ենք տրամադրել տարբերակ՝ սահմանելու համար լռելյայն կամ 'unset'՝ լռելյայն տարբերակը հեռացնելու համար։

error-open-zip = Չհաջողվեց բացել ZIP ֆայլը { $path }: { $error }
error-read-zip = Չհաջողվեց կարդալ ZIP արխիվը { $path }: { $error }
error-access-file = Չհաջողվեց մուտք գործել ֆայլը ինդեքսով { $index }: { $error }
error-reopen-zip = Չհաջողվեց կրկին բացել ZIP ֆայլը { $path }: { $error }
error-invalid-file-name = Անվավեր ֆայլի անուն ZIP արխիվում
error-create-dir = Չհաջողվեց ստեղծել գրացուցակը { $path }: { $error }
error-create-file = Չհաջողվեց ստեղծել ֆայլը { $path }: { $error }
error-read-zip-file = Չհաջողվեց կարդալ ZIP ֆայլից { $file }: { $error }
error-write-file = Չհաջողվեց գրել ֆայլը { $path }: { $error }
error-strip-prefix = Սխալ է հանել նախաբառը՝ { $error }
error-set-permissions = Չհաջողվեց սահմանել արտոնայինությունները { $path }: { $error }
error-create-symlink-windows = Չհաջողվեց ստեղծել symlink։ Ստուգեք, արդյոք {"\u001b"}]8;;ms-settings:developers{"\u001b"}\developer mode-ը{"\u001b"}]8;;{"\u001b"}\ միացած է կամ գործարկեք ադմինի իրավունքներով:

help-upgrade = Թարմացնել gdvm-ը վերջին տարբերակին
help-upgrade-major = Թույլատրել թարմացումը հիմնական տարբերակների միջև
help-upgrade-pre = Թարմացնել մինչև վերջին նախնական թողարկումը
upgrade-starting = Սկսվում է gdvm-ի թարմացումը...
upgrade-downloading-latest = Ներբեռնում է gdvm-ի վերջին տարբերակը...
upgrade-complete = gdvm-ը հաջողությամբ թարմացվեց!
upgrade-not-needed = gdvm-ը արդեն վերջին տարբերակն է՝ { $version }։
upgrade-current-version-newer = Ներկայիս gdvm տարբերակը ({ $current }) ավելի նոր է, քան վերջին հասանելի տարբերակը ({ $latest })։ Թարմացումը անհրաժեշտ չէ։
upgrade-failed = Թարմացումը ձախողվեց՝ { $error }
upgrade-download-failed = Թարմացման բեռնումը ձախողվեց: { $error }
upgrade-file-create-failed = Չհաջողվեց ստեղծել թարմացման ֆայլը: { $error }
upgrade-file-write-failed = Չհաջողվեց գրելու տվյալները թարմացման ֆայլում: { $error }
upgrade-install-dir-failed = Չհաջողվեց ստեղծել տեղադրման ծառարանները: { $error }
upgrade-rename-failed = Չհաջողվեց փոխանակել գործող գործարկվող ֆայլի անունը: { $error }
upgrade-replace-failed = Չհաջողվեց փոխարինել գործող գործարկվող ֆայլը նորով: { $error }
upgrade-no-binary = gdvm-ի երկուական ֆայլ հասանելի չէ { $version } տարբերակի և { $target } թիրախի համար։
upgrade-checksum-required = Թողարկման մանիֆեստը չի պարունակում ստուգիչ գումար այս gdvm երկուական ֆայլի համար։ Թարմացումը մերժվում է։
error-fetching-gdvm-releases = gdvm-ի թողարկումները բերելու սխալ․ { $error }
error-parsing-gdvm-releases = gdvm-ի թողարկումները վերլուծելու սխալ․ { $error }
error-unsupported-gdvm-schema = gdvm-ի թողարկումների մանիֆեստի սխեմայի չաջակցվող տարբերակ․ { $schema }։ Փորձեք թարմացնել gdvm-ը ձեռքով։
checking-updates = Ստուգվում են թարմացումները gdvm-ի համար...
upgrade-available = 💡 gdvm-ի նոր տարբերակ է հասանելի՝ {$version}: Գործարկեք «gdvm upgrade» թարմացնելու համար:
upgrade-available-major = 💡 gdvm-ի հիմնական տարբերակի թարմացում է հասանելի՝ {$version}: Գործարկեք «gdvm upgrade -m» թարմացնելու համար:
upgrade-available-both = 💡 gdvm-ի նոր տարբերակ է հասանելի՝ {$minor_version}: Հիմնական տարբերակի թարմացում նույնպես հասանելի է՝ {$major_version}: Գործարկեք «gdvm upgrade» ընթացիկ հիմնական տարբերակի շրջանակում թարմացնելու համար, կամ «gdvm upgrade -m» վերջին տարբերակին թարմացնելու համար:
upgrade-prerelease-available = 💡 gdvm-ի նոր նախնական թողարկում հասանելի է։ Այն տեղադրելու համար գործարկեք «gdvm upgrade --pre»։

help-pin = «Գամել» Godot-ի տարբերակը ընթացիկ պանակում:
help-pin-long = { help-pin }

    Սա կստեղծի gdvm.toml ֆայլ ընթացիկ պանակում գամված տարբերակով: Երբ դուք կգործարկեք "gdvm run" այս պանակում կամ դրա ենթապանակներում, կօգտագործվի գամված տարբերակը լռելյայն տարբերակի փոխարեն:

    Սա օգտակար է, երբ ցանկանում եք օգտագործել որոշակի Godot տարբերակ նախագծի համար, առանց փոխելու լռելյայն տարբերակը ամբողջ համակարգում:

    Ներկայումս սա գրում է նաև հնացած .gdvmrc ֆայլը։ gdvm-ի հին տարբերակների հետ համատեղելիության համար: Այն կհեռացվի ապագա թողարկումում, ուստի խորհուրդ է տրվում անցնել նոր gdvm.toml ձևաչափին և հեռացնել .gdvmrc ֆայլը, եթե այն կա:

    Դուք կարող եք անջատել .gdvmrc ֆայլի գրումը --no-legacy դրոշակով:
help-pin-version = Տարբերակը, որը պետք է գամել
help-no-legacy = Չգրել հնացած .gdvmrc համատեղելիության ֆայլը
pinned-success = {$version} տարբերակը հաջողությամբ գամվեց gdvm.toml-ում
error-pin-version-not-found = Չհաջողվեց գամել {$version} տարբերակը
pin-subcommand-description = Սահմանում կամ թարմացնում է gdvm.toml ֆայլը նշված տարբերակով

error-file-not-found = Ֆայլը չի գտնվել։ Հնարավոր է, որ այն գոյություն չունի սերվերի վրա։
error-download-failed = Ներբեռնումը ձախողվեց անակնկալ սխալի պատճառով։ { $error }
error-ensure-godot-binaries-failed = Չհաջողվեց ապահովել Godot-ի գործարկվող ֆայլերը։
    Սխալ: { $error }։
    Փորձեք ջնջել { $path } ֆայլը և վերսկսեք gdvm-ը:

error-failed-reading-project-godot = Չհաջողվեց կարդալ project.godot ֆայլը, հնարավոր չէ ինքնուրույն որոշել նախագծի տարբերակը:
warning-using-project-version = Օգտագործվում է project.godot-ում սահմանված տարբերակը ({ $version }).

warning-project-version-mismatch =
    {"\u001b"}[33mԶգուշացում. project.godot-ում սահմանված տարբերակը չի համընկնում { $pinned ->
        [1] սահմանված (pinned)
        *[0] պահանջված (requested)
    } տարբերակին: Նախագիծը բացելը { $pinned ->
        [1] սահմանված (pinned)
        *[0] պահանջված (requested)
    } տարբերակով կարող է վերագրանցել նախագծի ֆայլը:{"\u001b"}[0m

    { $pinned ->
        [1] Նախագծի տարբերակը:   { $project_version }
            Սահմանված տարբերակը: { $requested_version }
        *[0] Նախագծի տարբերակը:   { $project_version }
             Պահանջված տարբերակը: { $requested_version }
    }

error-project-version-mismatch = {"\u001b"}[31m{ $pinned ->
        [1] Եթե համոզված եք, որ ցանկանում եք նախագծը գործարկել սահմանված (pinned) տարբերակով, ապա գործարկեք {"\u001b"}[0mgdvm run --force{"\u001b"}[31m: Հակառակ դեպքում թարմացրեք .gdvmrc-ում սահմանված տարբերակը, որպեսզի այն համապատասխանի նախագծի տարբերակին, կամ հեռացրեք .gdvmrc-ը, որպեսզի օգտագործվի նախագծի տարբերակը:
        *[0] Եթե համոզված եք, որ ցանկանում եք նախագծը գործարկել պահանջված (requested) տարբերակով, ապա գործարկեք {"\u001b"}[0mgdvm run --force <version>{"\u001b"}[31m:
    }{"\u001b"}[0m
warning-project-version-mismatch-force = {"\u001b"}[33mԲաց թողնել հաստատման հարցումը և շարունակել { $pinned ->
        [1] սահմանված (pinned)
        *[0] պահանջված (requested)
    } տարբերակով {"\u001b"}[0m({ $requested_version }){"\u001b"}[33m։{"\u001b"}[0m

help-run-args = Լրացուցիչ արգումենտներ, որոնք պետք է փոխանցվեն Godot-ի գործարկվող ֆայլին (օրինակ՝ -- path/to/project.godot)։
help-run-force =
    Պարտադրել նախագծի գործարկումը պահանջված կամ սահմանված տարբերակով, նույնիսկ եթե այն չի համընկնում նախագծի տարբերակին:
help-run-force-long =
    { help-run-force }

    Եթե դուք այս գործողությունը կատարեք, Godot-ի պահանջված կամ սահմանված տարբերակը կարող է վերագրանցել նախագծի ֆայլը։ Եթե տարբերակը սահմանված է .gdvmrc-ում, խորհուրդ է տրվում թարմացնել այն, որպեսզի այն համապատասխանի նախագծի տարբերակին, կամ հեռացնել .gdvmrc ֆայլը, որպեսզի օգտագործվի նախագծի տարբերակը:

help-config = Կառավարել gdvm-ի կարգավորումները
help-config-get = Ստանալ կարգավորման արժեքը
help-config-set = Սահմանել կարգավորման արժեքը
help-config-unset = Հեռացնել կարգավորման արժեքը
help-config-list = Ցուցադրել բոլոր կարգավորումների արժեքները
help-config-key = Կարգավորման բանալին (օր․՝ prune.max-age-days)
help-config-value = Կարգավորման բանալու համար սահմանվող արժեքը
help-config-unset-key = Կարգավորման բանալին, որը պետք է հեռացնել (օր․՝ prune.max-age-days)
help-config-show-sensitive = Ցույց տալ զգայուն կարգավորումների արժեքները բաց տեքստով
help-config-available = Ցուցադրել բոլոր հասանելի կարգավորման բանալիները և դրանց արժեքները, ներառյալ լռելյայնները
warning-setting-sensitive = {"\u001b"}[33mԶգուշացում․ Դուք սահմանում եք զգայուն արժեք, որը կպահպանվի բաց տեքստով ձեր գլխավոր թղթապանակում։{"\u001b"}[0m
config-set-prompt = Խնդրում ենք մուտքագրել { $key } արժեքը:
error-reading-input = Մուտքագրված արժեքը սխալ է։ Խնդրում ենք փորձել նորից։
config-set-success = Կարգավորումը հաջողությամբ թարմացվեց։
config-unset-success = Կարգավորման բանալին { $key } հաջողությամբ հեռացվեց։
config-key-not-set = Կարգավորման բանալին սահմանված չէ։
error-unknown-config-key = Անհայտ կարգավորման բանալի։
error-invalid-config-value = Անվավեր արժեք { $key } կարգավորման բանալու համար։
error-invalid-config-subcommand = Անվավեր ենթահրաման config-ի համար։ Օգտագործեք «get», «set» կամ «list»։
error-parse-config = Չհաջողվեց վերլուծել կարգավորման ֆայլը․ { $error }
error-parse-config-using-default = {"\u001b"}[33mՕգտագործվում են կարգավորման լռելյայն արժեքները։{"\u001b"}[0m

help-registry = Կառավարել ռեեստրները, որոնցից տեղադրվում են Godot-ի կառուցումները
help-registry-add = Ավելացնել ռեեստր
help-registry-remove = Հեռացնել ռեեստր
help-registry-list = Ցուցադրել կարգավորված ռեեստրները
help-registry-refresh = Թարմացնել մեկ կամ բոլոր ռեեստրների քեշը
help-registry-name = Ռեեստրի անունը
help-registry-url = Ռեեստրի URL-ը։ Կարող է լինել http(s):// կամ file:// URL։

registry-added = Ավելացվեց { $registry } ռեեստրը ({ $url }):
registry-removed = Հեռացվեց { $registry } ռեեստրը:
registry-list-header = Կարգավորված ռեեստրներ.
registry-tag-official = պաշտոնական
registry-error = Ռեեստրի սխալ. { $error }

error-invalid-registry-subcommand = Անվավեր ռեեստրի ենթահրաման: Օգտագործեք «add», «remove», «list» կամ «refresh»:
registry-trust-warning = {"\u001b"}[33m{ $registry } ({ $url })-ը հատուկ ռեեստր է, ոչ թե պաշտոնականը: gdvm-ը ստուգում է, որ ներբեռնումները համապատասխանում են ռեեստրի նշածին, բայց չի կարող ջանաչել, թե արդյոք դրանք անվտանգ են գործարկելու համար: Տեղադրեք դրանից միայն այն դեպքում, եթե վստահում եք նրան, ով այն կառավարում է:{"\u001b"}[0m
registry-trust-prompt = Վստահո՞ւմ եք այս ռեեստրին և ցանկանում եք շարունակել: (այո/ոչ).
registry-trust-bypass = {"\u001b"}[1;31m{ $registry } ({ $url })-ի վստահության ստուգումը բաց է թողնվում, քանը որ օգտագործեցիք --yes: gdvm-ը չի կարող ստուգել՝ արդյոք դրա ֆայլերն անվտանգ են գործարկելու համար: Կարճ դադար. սեղմեք Ctrl+C հիմա՝ դադարեցնելու համար:{"\u001b"}[0m
registry-trust-aborted = Ընդհատվեց. ռեեստրը վստահելի չէ:
registry-project-override-conflict = {"\u001b"}[33mՆախագծի gdvm.toml-ը վերասահմանում է { $registry } ռեեստրը (ձեր կարգավորումը՝ { $machine_url }) որպես { $project_url }: Նախագծի սահմանումն ունի առաջնահերթություն:{"\u001b"}[0m

help-registry-init = Նախաստորագրել նոր ռեեստրի թղթապանակ
help-registry-add-build = Ավելացնել կառուցում ռեեստրում
help-registry-remove-build = Հեռացնել կառուցումը ռեեստրից
help-registry-validate = Ստուգել ռեեստրի թղթապանակը
help-registry-dir = Ռեեստրի թղթապանակը
help-registry-init-name = Ռեեստրի անունը (լռելյայն՝ թղթապանակի անունը)

help-registry-build-version = Տարբերակի պիտակը, օր.՝ 4.4-stable։
help-registry-build-variant = Տարբերակի անունը։ Լռելյայն՝ «default»։
help-registry-build-platform = Հարթակի բանալին, օր.՝ linux-x86_64։
help-registry-build-file = Կառուցման արխիվի ուղին հեշավորելու համար
help-registry-build-store = Պատճենել արխիվը ռեեստր և գրանցել հարաբերական URL
help-registry-build-url = URL-ը, որտեղ արխիվը կմատուցվի (երբ --store-ը չի օգտագործվում)
help-registry-build-sha512 = Արխիվի SHA-512-ը՝ այն հաշվարկելու փոխարեն: Պահանջում է --size:
help-registry-build-size = Արխիվի չափը բայթերով՝ այն չափելու փոխարեն: Պահանջում է --sha512:

registry-init-success = { $name } ռեեստրը նախաստորագրվեց { $path }-ում:
registry-build-added = Ավելացվեց { $version } կառուցումը { $platform }-ի համար:
registry-build-removed = Հեռացվեց { $version } կառուցումը:
registry-build-downloading = { $url }-ի ներբեռնում՝ չափը և SHA-512-ը հաշվարկելու համար…
registry-build-warn-local-hash = {"\u001b"}[33mՀաշվարկվում է տեղական ֆայլի հեշը՝ ենթադրելով, որ այն համապատասխանում է { $url }-ին: gdvm-ը URL-ը չի ներբեռնում՝ ստուգելու համար:{"\u001b"}[0m
registry-build-warn-unverified = {"\u001b"}[33mՕգտագործվում են ձեր տրամադրած SHA-512-ը և չափը՝ առանց արտեֆակտը ներբեռնելու դրանք ստուգելու: Համոզվեք, որ դրանք ճիշտ են:{"\u001b"}[0m
registry-build-warn-explicit-store = {"\u001b"}[33mՕգտագործվում են ձեր տրամադրած SHA-512-ը և/կամ չափը՝ պահված արխիվը չափելու փոխարեն:{"\u001b"}[0m
registry-build-sha-mismatch = Տրամադրված SHA-512-ը ({ $expected }) չի համապատասխանում արտեֆակտին ({ $actual }):
registry-build-size-mismatch = Տրամադրված չափը ({ $expected }) չի համապատասխանում արտեֆակտին ({ $actual }):
registry-validate-ok = Ռեեստրը վավեր է (ստուգվեց { $count } արտեֆակտ):
registry-validate-failed = Ռեեստրի վավերացումը ձախողվեց.
