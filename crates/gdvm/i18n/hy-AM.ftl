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
warning-sha-sums-missing = Չհաջողվեց գտնել ստուգանակոչումները այս թողարկման համար։ Թողարկման հաստատումը հապողվեց։

error-find-user-dirs = Չհաջողվեց գտնել օգտագործողի պանակները։

fetching-releases = Թողարկումները բերվում են...
releases-fetched = Թողարկումները բերվել են։
error-fetching-releases = Սխալ թողարկումների ստացման ժամանակ՝ { $error }
warning-fetching-releases-using-cache = Սխալ թողարկումների ստացման ժամանակ՝ { $error }։ Օգտագործվում են պահված թողարկումները։

error-version-not-found = Տարբերակը չի գտնվել:
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
error-link-godotsharp-target = Հնարավոր չէ որոշել GodotSharp թիրախի ուղին։
error-link-godotsharp-missing = GodotSharp պանակը բացակայում է գործարկվողի կողքին։

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
checking-updates = Ստուգվում են թարմացումները gdvm-ի համար...
upgrade-available = 💡 gdvm-ի նոր տարբերակ է հասանելի՝ {$version}: Գործարկեք «gdvm upgrade» թարմացնելու համար:
upgrade-available-major = 💡 gdvm-ի հիմնական տարբերակի թարմացում է հասանելի՝ {$version}: Գործարկեք «gdvm upgrade -m» թարմացնելու համար:
upgrade-available-both = 💡 gdvm-ի նոր տարբերակ է հասանելի՝ {$minor_version}: Հիմնական տարբերակի թարմացում նույնպես հասանելի է՝ {$major_version}: Գործարկեք «gdvm upgrade» ընթացիկ հիմնական տարբերակի շրջանակում թարմացնելու համար, կամ «gdvm upgrade -m» վերջին տարբերակին թարմացնելու համար:

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
help-config-key = Կարգավորման բանալին (օր․՝ github.token)
help-config-value = Կարգավորման բանալու համար սահմանվող արժեքը
help-config-unset-key = Կարգավորման բանալին, որը պետք է հեռացնել (օր․՝ github.token)
help-config-show-sensitive = Ցույց տալ զգայուն կարգավորումների արժեքները բաց տեքստով
help-config-available = Ցուցադրել բոլոր հասանելի կարգավորման բանալիները և դրանց արժեքները, ներառյալ լռելյայնները
warning-setting-sensitive = {"\u001b"}[33mԶգուշացում․ Դուք սահմանում եք զգայուն արժեք, որը կպահպանվի բաց տեքստով ձեր գլխավոր թղթապանակում։{"\u001b"}[0m
config-set-prompt = Խնդրում ենք մուտքագրել { $key } արժեքը:
error-reading-input = Մուտքագրված արժեքը սխալ է։ Խնդրում ենք փորձել նորից։
config-set-success = Կարգավորումը հաջողությամբ թարմացվեց։
config-unset-success = Կարգավորման բանալին { $key } հաջողությամբ հեռացվեց։
config-key-not-set = Կարգավորման բանալին սահմանված չէ։
error-unknown-config-key = Անհայտ կարգավորման բանալի։
error-invalid-config-subcommand = Անվավեր ենթահրաման config-ի համար։ Օգտագործեք «get», «set» կամ «list»։
error-parse-config = Չհաջողվեց վերլուծել կարգավորման ֆայլը․ { $error }
error-parse-config-using-default = {"\u001b"}[33mՕգտագործվում են կարգավորման լռելյայն արժեքները։{"\u001b"}[0m
error-github-api = GitHub API սխալ․ { $error }
error-github-rate-limit = GitHub API-ի սահմանափակումը գերազանցվել է։

  Խնդիրը լուծելու համար ստեղծեք անձնական հասանելիության բանալի GitHub-ում՝ այցելելով https://github.com/settings/tokens։

  Սեղմեք «Generate new token», ընտրեք միայն անհրաժեշտ նվազագույն թույլտվությունները (օրինակ՝ public_repo), ապա սահմանեք բանալին GITHUB_TOKEN միջավայրային փոփոխականով կամ գործարկելով հետևյալ հրամանը․

    gdvm config set github.token

  Ուշադրություն․ բանալին կպահպանվի բաց տեքստով ձեր գլխավոր թղթապանակում։ Համոզվեք, որ այն պահում եք անվտանգ։
  Անվտանգության համար խորհուրդ է տրվում պարբերաբար ստուգել և թարմացնել ձեր բանալիները։

error-copy-file-failed = Ֆայլը չհաջողվեց պատճենել՝ { $error }
error-move-file-failed = Ֆայլը չհաջողվեց տեղափոխել՝ { $error }
error-user-dir-not-found = Չհաջողվեց ստեղծել կարճուղի. օգտատիրոջ գրացուցակը չի գտնվել
error-desktop-not-found = Չհաջողվեց ստեղծել դյուրանցում. աշխատասեղանի գրացուցակը չի գտնվել
warning-shortcut-macos-not-supported = Այս պահին MacOS-ում կարճ ստեղներ ստեղծելը չի ​​աջակցվում, ուստի կարճ ստեղներ չեն ստեղծվի։