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

-gdvm =
    { $case ->
       *[nominative] gdvm
        [definite] gdvm-ը
        [genitive] gdvm-ի
    }
-gdvm-toml =
    { $case ->
       *[nominative] gdvm.toml
        [definite] gdvm.toml-ը
        [locative] gdvm.toml-ում
    }
-gdvmrc =
    { $case ->
       *[nominative] .gdvmrc
        [definite] .gdvmrc-ը
        [locative] .gdvmrc-ում
    }
-godot =
    { $case ->
       *[nominative] Godot
        [definite] Godot-ը
        [genitive] Godot-ի
    }
size-display =
    { $unit ->
        [b] { NUMBER($value, maximumFractionDigits: 0) } Բ
        [kib] { NUMBER($value, maximumFractionDigits: 1) } ԿիԲ
        [mib] { NUMBER($value, maximumFractionDigits: 1) } ՄիԲ
        [gib] { NUMBER($value, maximumFractionDigits: 1) } ԳիԲ
       *[tib] { NUMBER($value, maximumFractionDigits: 1) } ՏիԲ
    }

help-about = { -godot } տարբերակների կառավարիչ
help-help = Ցուցադրել օգնություն (տես ամփոփում '-h'-ով)
help-gdvm-version = Ցուցադրել { -godot } տարբերակների կառավարչի տարբերակը

help-install = Տեղադրել նոր { -godot } տարբերակ
help-run = Գործարկել որոշակի { -godot } տարբերակ
help-show = Ցույց տալ { -godot(case: "genitive") } նշված տարբերակի գործարկվողի ուղին
help-cache-path = Ցույց տալ նշված { -godot } տարբերակի ներբեռնման պահված արխիվի ուղին
help-link = Կապել { -godot(case: "genitive") } որոշակի տարբերակի գործարկվողը նշված ուղու հետ
help-list = Ցուցադրել բոլոր տեղադրված { -godot } տարբերակները
help-remove = Հեռացնել տեղադրված { -godot } տարբերակը
help-csharp = [հնացած] Օգտագործել { -godot } տարբերակը C# աջակցության համար: Փոխարենը օգտագործեք «csharp» տարբերակի ցուցիչը (օրինակ՝ csharp:4.4):
help-run-csharp-long = { help-csharp }
help-version = Տեղադրվող տարբերակը (օրինակ՝ 4, csharp:4.4, stable, latest):
help-version-long =
    { help-version }

    Ձևաչափ՝ [տարբերակ:]տարբերակ_կամ_բանալի_բառ

    Եթե առկա է վերջում դրված *, այն կհամապատասխանի նույն նախածանցով ամենանոր կառուցմանը, օրինակ՝ «4.7-dev*»-ը համապատասխանում է 4.7-dev1, 4.7-dev2 և այլն։

    Բանալի բառեր։ «latest» համապատասխանում է ամենանոր տարբերակին: Լռելյայն ներառվում են միայն կայուն թողարկումները, սակայն նախնական թողարկումները կարելի է ներառել --pre դրոշակով:

    Տարբերակներ՝ նախածանցեք տարբերակի անունով և երկու կետով, օրինակ՝ «csharp:4.4» C# տարբերակի համար:

    Օրինակներ՝ 4.4-ը կտեղադրի { -godot } 4.4-ի վերջին կայուն թողարկումը: Եթե միայն նախնական թողարկումներ կան, ապա կտեղադրվի վերջին նախնական թողարկումը: 4.3-rc*-ը կտեղադրի { -godot } 4.3-ի վերջին թողարկման թեկնածուն և այլն:
help-version-installed = Տեղադրված տարբերակը (օրինակ՝ 4.2 կամ 4.2-stable):

help-search = Ցուցադրել մատյանի հասանելի թողարկումները
help-filter = Ընտրովի տող թողարկման պիտակները ֆիլտրելու համար
help-filter-deprecated = [հնացած] Ընտրովի տող թողարկման պիտակները ֆիլտրելու համար: Փոխարենը օգտագործեք ֆիլտրի դիրքային արգումենտը։
help-include-pre = Ներառել նախնական թողարկումները (rc, beta, dev)
help-cache-only = Օգտագործել միայն պահված թողարկման տեղեկատվությունը առանց գրանցամատյանին հարցում անելու
help-limit = Ցուցադրվող թողարկումների քանակը, լռելյայն 10. Օգտագործեք 0՝ բոլորի ցուցադրման համար
help-clear-cache = Մաքրել թողարկումների քեշը
help-refresh = Թարմացնել թողարկումների քեշը գրանցամատյանից
help-refresh-flag = Թարմացնել թողարկումների քեշը հրամանը գործարկելուց առաջ

help-prune = Հեռացնել տեղադրումները և քեշավորված արխիվները, որոնք այլևս չեն օգտագործվում
help-prune-long = { help-prune }

    Լռելյայն prune-ը հեռացնում է այն տեղադրումները, որոնք երկար ժամանակ չեն օգտագործվել, ինչպես նաև հնացած քեշավորված ներբեռնման արխիվները՝ պահպանելով յուրաքանչյուր տեղադրում, որին դեռ ցույց է տալիս ակտիվ հղում։ Որպես լռելյայն սահմանված տեղադրումը երբեք չի հեռացվում՝ անկախ տրված դրոշակներից։ Հնության շեմը կարգավորելի է « { -gdvm } config set prune.max-age-days <օրեր> » հրամանով (լռելյայն՝ { $default_days } օր)։
help-prune-all = Հեռացնել բոլոր տեղադրումներն ու քեշավորված արխիվները՝ անկախ հնությունից։ Ակտիվ հղում ունեցող տեղադրումները պահպանվում են, եթե նաև --force տրված չէ։
help-prune-force = Անտեսել հղումները, որպեսզի միայն հղումով հղվող տեղադրումները նույնպես հնարավոր լինի հեռացնել։
help-prune-dry-run = Ցույց տալ, թե ինչ կհեռացվեր՝ առանց որևէ բան ջնջելու։
prune-nothing-dry-run = Ոչինչ չէր հեռացվի։
prune-nothing-removed = Հեռացնելու բան չկա. ամեն ինչ օգտագործվում է կամ հնության շեմի սահմաններում է։
prune-preserved-by-link =
    { $count ->
        [one] Պահպանվեց { $count } տեղադրում, որը դեռ հղվում է հղումով։
       *[other] Պահպանվեց { $count } տեղադրում, որոնք դեռ հղվում են հղումով։
    }
warning-broken-install-reinstalling = Տեղադրված { $version }-ը չունի իր գործարկվող ֆայլը. այն վերատեղադրվում է։

help-force = Ստիպել վերատեղադրումը, նույնիսկ եթե տարբերակը արդեն տեղադրված է:
help-redownload = Նորից ներբեռնել տարբերակը, նույնիսկ եթե այն արդեն տեղադրված է:
help-yes = Բաց թողնել հեռացման հաստատման հուշումը
help-remove-yes-deprecated = [հնացած] Այս դրոշակը անօգուտ է և կհեռացվի ապագա թողարկումներում:
help-link-version = Այն տարբերակը, որը պետք է կապվի։ Եթե այն չի տրվում, տարբերակը որոշվում է ընթացիկ պանակի կամ լռելյայն տարբերակի հիման վրա։
help-link-path = Ուղին, որտեղ կստեղծվի հղումը կամ պատճենը, օրինակ «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }»։
help-link-force = Վերագրել գոյություն ունեցող հղումը, եթե կա
help-link-copy = Պատճենել գործարկվողը հղման փոխարեն
no-cache-files-found = Պահված թողարկումներ չեն գտնվել։
no-cache-metadata-found = Պահված թողարկումների ինդեքսը չի գտնվել։
gdvm-toml-malformed = { $path }-ի { -gdvm-toml(case: "definite") } անտեսվում է, քանի որ հնարավոր չէ վերլուծել. { $error }

help-console = { -godot(case: "definite") } կատարողական կոնսոլով: լռելյայն է Windows-ում, այլ համակարգչներում ակտիվացվում է:

help-default = Կառավարել լռելյայն տարբերակը
help-default-version = Տարբերակը, որը պետք է տեղադրվի լռելյայն (օրինակ՝ 4.2 կամ 4.2-stable):
no-default-set = Լռելյայն տարբերակը սահմանված չէ: Գործարկեք "{ -gdvm } use <version>"՝ լռելյայն տարբերակը համակարգային մակարդակով սահմանելու համար, կամ "{ -gdvm } pin <version>"՝ լռելյայն տարբերակը ընթացիկ պանակի համար սահմանելու համար:

warning-prerelease = Դուք տեղադրում եք նախնական թողարկում ({$branch}).
warning-deprecated-csharp-flag = --csharp դրոշակը հնացած է։ Փոխարենը օգտագործեք "csharp" տարբերակի ցուցիչը (օրինակ՝ csharp:4.4):

label-error = Սխալ։
label-note = Նշում։
label-warning = Զգուշացում։
progress-rate = { size-display }/վրկ
progress-eta-remaining = մնաց { $time }
progress-fraction = { $done }/{ $total }
status-downloading = Ներբեռնում
status-extracting = Բացահայտում
status-fetching = Բերում
status-installed = Տեղադրվեց
status-installing = Տեղադրում
status-removed = Հեռացված է
prune-item-detail = { $label } ({ size-display })
status-freed = Ազատվեց
status-pruned = Մաքրվեց
status-would-free = Կազատվի
status-would-prune = Կմաքրվի
status-removing = Հեռացվում է
status-running = Գործարկում
status-cleared = Մաքրվեց
status-refreshed = Թարմացվեց
status-skipped = Բաց թողնվեց
status-upgraded = Թարմացվեց
status-upgrading = Թարմացում
status-verifying = Ստուգում
subject-cached-archive = պահված արխիվ
subject-cache = քեշ
subject-cache-files = քեշավորված ֆայլեր
subject-cache-metadata = քեշի մետատվյալներ
subject-releases = թողարկումներ
subject-update-manifest = թարմացման մանիֆեստ
upgrade-target = { -gdvm } { $version }

auto-installing-version = Ինքնաշխատ տեղադրում { $version } տարբերակի

no-versions-installed = Տեղադրված տարբերակներ չկան:
installed-versions = Տեղադրված { -godot } տարբերակներ:
progress-eta =
    { $magnitude ->
        [seconds] { $secs }վ
        [minutes] { $mins }ր { $secs }վ
       *[hours] { $hours }ժ { $mins }ր
    }

unsupported-platform = Չաջակցվող համակարգ
unsupported-architecture = Չաջակցվող ստեղծածություն
error-checksum-mismatch = Ստորակետի չափազանելը չհամընկեց ֆայլի { $file }
error-invalid-sha-length = Անվավեր SHA երկարություն { $length }
error-size-mismatch = Չափի անհամապատասխանություն { $file } ֆայլի համար. սպասվում էր { $expected } բայթ, ստացվել է { $actual } բայթ։
error-insecure-url = Մերժվում է { $url }-ի բեռնումը չգաղտնագրված կապով։ Թույլատրվում են միայն https:// և file:// URL-ները։ Սահմանեք GDVM_ALLOW_INSECURE_URLS միջավայրի փոփոխականը՝ չգաղտնագրված http:// URL-ները թույլատրելու համար։
error-insecure-redirect = Մերժվում է վերաուղղորդումը https://-ից չգաղտնագրված http:// URL-ի։ Սահմանեք GDVM_ALLOW_INSECURE_URLS միջավայրի փոփոխականը՝ չգաղտնագրված http:// URL-ները թույլատրելու համար։
error-response-not-utf8 = { $url }-ի պատասխանը վավեր UTF-8 չէ։
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
error-publish-already-initialized = Ռեեստրը արդեն սկզբնավորված է { $path } հասցեում
error-publish-archive-not-found = Արխիվը չի գտնվել՝ { $path }
error-publish-no-such-platform = { $platform } հարթակ չկա { $variant } տարբերակի համար
error-publish-no-such-variant = { $variant } տարբերակ չկա
error-publish-invalid-segment = Անվավեր { $what }՝ { $value }
error-registry-fetch-failed = Չհաջողվեց բեռնել { $url }. HTTP { $status }
error-registry-fetch-release-failed = Չհաջողվեց բեռնել թողարկման մետատվյալները
error-registry-invalid-name = Անվավեր ռեգիստրի անուն. { $name }
error-registry-missing-index = « { $name } » ռեգիստրում բացակայում է index.json-ը
error-registry-missing-manifest = « { $name } » ռեգիստրում բացակայում է registry.json-ը
error-registry-not-configured = « { $name } » ռեգիստրը կարգավորված չէ
error-registry-parse-index = Չհաջողվեց վերլուծել « { $name } »-ի ինդեքսը։
error-registry-parse-manifest = Չհաջողվեց վերլուծել « { $name } »-ի մանիֆեստը։
error-registry-unknown = Անհայտ ռեգիստր « { $name } »
error-registry-unsupported-url-scheme = Ռեգիստրի URL-ի չաջակցվող սխեմա. { $url }
error-spec-empty-registry = Դատարկ ռեգիստրի անուն « { $input } »-ում
error-spec-empty-variant = Դատարկ տարբերակի անուն « { $input } »-ում
error-spec-empty-version = Դատարկ տարբերակ « { $input } »-ում
error-system-time = Համակարգի ժամանակը UNIX դարաշրջանից առաջ է
error-unrecognized-version-format = Տարբերակի չճանաչված ձևաչափ. { $input }
error-non-interactive-trust = Հնարավոր չէ հարցնել «{ $registry }» ռեգիստրին ({ $url }) վստահելու մասին ոչ ինտերակտիվ նստաշրջանում։ Փոխանցեք --yes՝ դրան բացահայտորեն վստահելու համար։
error-non-interactive-value = Հնարավոր չէ հարցնել «{ $key }»-ի արժեքը ոչ ինտերակտիվ նստաշրջանում։ Փոխարենը փոխանցեք արժեքը որպես արգումենտ։
error-registry-unsupported-schema = «{ $registry }» ռեգիստրը հայտարարում է չաջակցվող սխեմայի տարբերակ { $schema }։
label-caused-by = Պատճառը.
label-error-coded = Սխալ { $code }.
error-wildcard-position = Ունիվերսալ նիշը (*) կարող է հայտնվել միայն թողարկման պիտակի վերջում, օրինակ՝ 4.7-dev* (ստացվել է { $input })։
hint-try-wildcard = { $requested } պիտակով թողարկում չկա, բայց կան նմանատիպ պիտակներ, որոնցից ամենանորը { $newest }-ն է։ Փորձեք { $suggestion }՝ դրանց համապատասխանելու համար։
download-retrying = Ներբեռնումն ընդհատվեց, կրկին փորձ ({ $attempt } { $max }-ից)...
download-resuming = Ընդհատված ներբեռնումը վերսկսվում է (արդեն ներբեռնված է { size-display }).
warning-resume-verification-failed = Վերսկսված ներբեռնումը չի համապատասխանում սպասվող ստուգիչ գումարին. այն նորից ամբողջությամբ ներբեռնվում է։
lock-waiting = Սպասում է { -gdvm(case: "genitive") } մեկ այլ գործընթացի ավարտին (կողպեք՝ { $resource })...
prune-skipped-error = { $item }-ը բաց է թողնվում. { $error }
prune-skipped-in-use = { $item }-ը բաց է թողնվում. այն օգտագործվում է { -gdvm(case: "genitive") } մեկ այլ գործընթացի կողմից։

error-find-user-dirs = Չհաջողվեց գտնել օգտագործողի պանակները։
warning-fetching-releases-using-cache = Սխալ թողարկումների ստացման ժամանակ՝ { $error }։ Օգտագործվում են պահված թողարկումները։

error-version-not-found = Տարբերակը չի գտնվել:
error-archive-not-cached = {$version}-ի համար պահված արխիվ չի գտնվել։ Նախ տեղադրեք այն՝ քեշը լրացնելու համար։
error-multiple-versions-found = Մի քանի տարբերակներ են համընկնում ձեր հարցմանը:
    {$list}
link-created = {$version}-ը կապվեց {$path}-ին
copy-created = {$version} տարբերակը պատճենվեց {$path} ուղու վրա
no-matching-releases = Համապատասխան թողարկումներ չեն գտնվել:
available-releases = Հասանելի թողարկումներ:

version-already-installed = Տարբերակը արդեն տեղադրված է։ օգտագործեք {$version}:
godot-executable-not-found = { -godot(case: "genitive") } գործարկվող ֆայլը չի գտնվել {$version} տարբերակի համար:
error-link-exists = {$path} ուղին արդեն գոյություն ունի։ Օգտագործեք --force՝ վերագրելու համար։
error-link-symlink = Չհաջողվեց ստեղծել հղումը {$link}-ից դեպի {$target}։
error-link-copy = Չհաջողվեց պատճենել ֆայլը։

error-no-stable-releases-found = Կայուն թողարկումներ չեն գտնվել:

error-starting-godot = Չհաջողվեց գործարկել { -godot(case: "definite") }։
confirm-yes = այո

default-set-success = Հաջողությամբ սահմանվել է {$version} որպես լռելյայն { -godot } տարբերակը։
default-unset-success = Հաջողությամբ հեռացվեց լռելյայն { -godot } տարբերակը։
provide-version-or-unset = Խնդրում ենք տրամադրել տարբերակ՝ սահմանելու համար լռելյայն կամ 'unset'՝ լռելյայն տարբերակը հեռացնելու համար։

error-open-zip = Չհաջողվեց բացել ZIP ֆայլը { $path }։
error-read-zip = Չհաջողվեց կարդալ ZIP արխիվը { $path }։
error-access-file = Չհաջողվեց մուտք գործել ֆայլը ինդեքսով { $index }։
error-reopen-zip = Չհաջողվեց կրկին բացել ZIP ֆայլը { $path }։
error-invalid-file-name = Անվավեր ֆայլի անուն ZIP արխիվում
error-create-dir = Չհաջողվեց ստեղծել գրացուցակը { $path }։
error-create-file = Չհաջողվեց ստեղծել ֆայլը { $path }։
error-read-zip-file = Չհաջողվեց կարդալ ZIP ֆայլից { $file }։
error-write-file = Չհաջողվեց գրել ֆայլը { $path }։
error-strip-prefix = Սխալ է հանել նախաբառը։
error-set-permissions = Չհաջողվեց սահմանել արտոնայինությունները { $path }։
error-create-symlink-windows = Չհաջողվեց ստեղծել symlink։ Ստուգեք, արդյոք {"\u001b"}]8;;ms-settings:developers{"\u001b"}\developer mode-ը{"\u001b"}]8;;{"\u001b"}\ միացած է կամ գործարկեք ադմինի իրավունքներով:

help-upgrade = Թարմացնել { -gdvm(case: "definite") } վերջին տարբերակին
help-upgrade-major = Թույլատրել թարմացումը հիմնական տարբերակների միջև
help-upgrade-pre = Թարմացնել մինչև վերջին նախնական թողարկումը
upgrade-not-needed = { -gdvm(case: "definite") } արդեն վերջին տարբերակն է՝ { $version }։
upgrade-current-version-newer = Ներկայիս { -gdvm } տարբերակը ({ $current }) ավելի նոր է, քան վերջին հասանելի տարբերակը ({ $latest })։ Թարմացումը անհրաժեշտ չէ։
upgrade-install-dir-failed = Չհաջողվեց ստեղծել տեղադրման ծառարանները։
upgrade-rename-failed = Չհաջողվեց փոխանակել գործող գործարկվող ֆայլի անունը։
upgrade-replace-failed = Չհաջողվեց փոխարինել գործող գործարկվող ֆայլը նորով։
upgrade-no-binary = { -gdvm(case: "genitive") } երկուական ֆայլ հասանելի չէ { $version } տարբերակի և { $target } թիրախի համար։
upgrade-checksum-required = Թողարկման մանիֆեստը չի պարունակում ստուգիչ գումար այս { -gdvm } երկուական ֆայլի համար։ Թարմացումը մերժվում է։
error-fetching-gdvm-releases = { -gdvm(case: "genitive") } թողարկումները բերելու սխալ։
error-parsing-gdvm-releases = { -gdvm(case: "genitive") } թողարկումները վերլուծելու սխալ։
error-unsupported-gdvm-schema = { -gdvm(case: "genitive") } թողարկումների մանիֆեստի սխեմայի չաջակցվող տարբերակ․ { $schema }։ Փորձեք թարմացնել { -gdvm(case: "definite") } ձեռքով։
upgrade-available = 💡 { -gdvm(case: "genitive") } նոր տարբերակ է հասանելի՝ {$version}: Գործարկեք «{ -gdvm } upgrade» թարմացնելու համար:
upgrade-available-major = 💡 { -gdvm(case: "genitive") } հիմնական տարբերակի թարմացում է հասանելի՝ {$version}: Գործարկեք «{ -gdvm } upgrade -m» թարմացնելու համար:
upgrade-available-both = 💡 { -gdvm(case: "genitive") } նոր տարբերակ է հասանելի՝ {$minor_version}: Հիմնական տարբերակի թարմացում նույնպես հասանելի է՝ {$major_version}: Գործարկեք «{ -gdvm } upgrade» ընթացիկ հիմնական տարբերակի շրջանակում թարմացնելու համար, կամ «{ -gdvm } upgrade -m» վերջին տարբերակին թարմացնելու համար:
upgrade-prerelease-available = 💡 { -gdvm(case: "genitive") } նոր նախնական թողարկում հասանելի է։ Այն տեղադրելու համար գործարկեք «{ -gdvm } upgrade --pre»։

help-pin = «Գամել» { -godot(case: "genitive") } տարբերակը ընթացիկ պանակում:
help-pin-long = { help-pin }

    Սա կստեղծի { -gdvm-toml } ֆայլ ընթացիկ պանակում գամված տարբերակով: Երբ դուք կգործարկեք "{ -gdvm } run" այս պանակում կամ դրա ենթապանակներում, կօգտագործվի գամված տարբերակը լռելյայն տարբերակի փոխարեն:

    Սա օգտակար է, երբ ցանկանում եք օգտագործել որոշակի { -godot } տարբերակ նախագծի համար, առանց փոխելու լռելյայն տարբերակը ամբողջ համակարգում:

    Ներկայումս սա գրում է նաև հնացած { -gdvmrc } ֆայլը։ { -gdvm(case: "genitive") } հին տարբերակների հետ համատեղելիության համար: Այն կհեռացվի ապագա թողարկումում, ուստի խորհուրդ է տրվում անցնել նոր { -gdvm-toml } ձևաչափին և հեռացնել { -gdvmrc } ֆայլը, եթե այն կա:

    Դուք կարող եք անջատել { -gdvmrc } ֆայլի գրումը --no-legacy դրոշակով:
help-pin-version = Տարբերակը, որը պետք է գամել
help-no-legacy = Չգրել հնացած { -gdvmrc } համատեղելիության ֆայլը
pinned-success = {$version} տարբերակը հաջողությամբ գամվեց { -gdvm-toml(case: "locative") }
error-pin-version-not-found = Չհաջողվեց գամել {$version} տարբերակը

error-file-not-found = Ֆայլը չի գտնվել։ Հնարավոր է, որ այն գոյություն չունի սերվերի վրա։
error-download-failed = Ներբեռնումը ձախողվեց HTTP կարգավիճակով { $status }։
error-ensure-godot-binaries-failed = Չհաջողվեց ապահովել { -godot(case: "genitive") } գործարկվող ֆայլերը։

error-post-upgrade-action-failed = { $id } քայլը ձախողվեց թարմացումից հետո։
    Ձեր { -gdvm(case: "genitive") } տեղակայումը կարող է թերի լինել: Վերսկսեք { -gdvm(case: "definite") }:

error-failed-reading-project-godot = Չհաջողվեց կարդալ project.godot ֆայլը, հնարավոր չէ ինքնուրույն որոշել նախագծի տարբերակը:
warning-using-project-version = Օգտագործվում է project.godot-ում սահմանված տարբերակը ({ $version }).
warning-gdvmrc-detected = Հայտնաբերվել է անհատական  { -gdvmrc } ֆայլ:  { -gdvmrc } ֆայլերի աջակցությունը հնացած է և կհեռացվի ապագա թողարկումում: Խնդրում ենք անցնել նոր գամման ֆայլին, որն օգտագործվում է `{ -gdvm } pin`-ի կողմից:

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
        [1] Եթե համոզված եք, որ ցանկանում եք նախագծը գործարկել սահմանված (pinned) տարբերակով, ապա գործարկեք {"\u001b"}[0m{ -gdvm } run --force{"\u001b"}[31m: Հակառակ դեպքում թարմացրեք { -gdvmrc(case: "locative") } սահմանված տարբերակը, որպեսզի այն համապատասխանի նախագծի տարբերակին, կամ հեռացրեք { -gdvmrc(case: "definite") }, որպեսզի օգտագործվի նախագծի տարբերակը:
        *[0] Եթե համոզված եք, որ ցանկանում եք նախագծը գործարկել պահանջված (requested) տարբերակով, ապա գործարկեք {"\u001b"}[0m{ -gdvm } run --force <version>{"\u001b"}[31m:
    }{"\u001b"}[0m

help-run-args = Լրացուցիչ արգումենտներ, որոնք պետք է փոխանցվեն { -godot(case: "genitive") } գործարկվող ֆայլին (օրինակ՝ -- path/to/project.godot)։
help-run-force =
    Պարտադրել նախագծի գործարկումը պահանջված կամ սահմանված տարբերակով, նույնիսկ եթե այն չի համընկնում նախագծի տարբերակին:
help-run-force-long =
    { help-run-force }

    Եթե դուք այս գործողությունը կատարեք, { -godot(case: "genitive") } պահանջված կամ սահմանված տարբերակը կարող է վերագրանցել նախագծի ֆայլը։ Եթե տարբերակը սահմանված է { -gdvmrc(case: "locative") }, խորհուրդ է տրվում թարմացնել այն, որպեսզի այն համապատասխանի նախագծի տարբերակին, կամ հեռացնել { -gdvmrc } ֆայլը, որպեսզի օգտագործվի նախագծի տարբերակը:

help-config = Կառավարել { -gdvm(case: "genitive") } կարգավորումները
help-format = Արտածման ձևաչափ. text (լռելյայն) կամ json
help-info = Ցուցադրել տեղադրված տարբերակի մանրամասն տեղեկությունները
info-default =
    { $value ->
        [1] { confirm-yes }
       *[0] { info-no }
    }
    .label = Լռելյայն.
info-executable = { $path }
    .label = Գործարկվող ֆայլ.
info-install-path = { $path }
    .label = Տեղադրման ուղի.
info-last-used = { $timestamp }
    .label = Վերջին օգտագործումը.
info-no = ոչ
info-registry = { $registry }
    .label = Ռեգիստր.
info-size = { size-display }
    .label = Չափը սկավառակի վրա.
info-variant = { $variant }
    .label = Տարբերակ.
info-version = { $version }
    .label = Տարբերակ.
help-completions = Գեներացնել shell-ի ավտոլրացման սկրիպտներ
help-completions-shell = Shell-ը, որի համար գեներացվում են ավտոլրացումները
help-config-get = Ստանալ կարգավորման արժեքը
help-config-set = Սահմանել կարգավորման արժեքը
help-config-unset = Հեռացնել կարգավորման արժեքը
help-config-list = Ցուցադրել բոլոր կարգավորումների արժեքները
help-config-key = Կարգավորման բանալին (օր․՝ prune.max-age-days)
help-config-value = Կարգավորման բանալու համար սահմանվող արժեքը
help-config-unset-key = Կարգավորման բանալին, որը պետք է հեռացնել (օր․՝ prune.max-age-days)
help-config-show-sensitive = Ցույց տալ զգայուն կարգավորումների արժեքները բաց տեքստով
help-config-available = Ցուցադրել բոլոր հասանելի կարգավորման բանալիները և դրանց արժեքները, ներառյալ լռելյայնները
warning-setting-sensitive = Դուք սահմանում եք զգայուն արժեք, որը կպահպանվի բաց տեքստով ձեր գլխավոր թղթապանակում։
config-set-prompt = Խնդրում ենք մուտքագրել { $key } արժեքը:
error-reading-input = Մուտքագրված արժեքը սխալ է։ Խնդրում ենք փորձել նորից։
config-set-success = Կարգավորումը հաջողությամբ թարմացվեց։
config-unset-success = Կարգավորման բանալին { $key } հաջողությամբ հեռացվեց։
config-key-not-set = Կարգավորման բանալին սահմանված չէ։
config-key-not-set-value = <սահմանված չէ>
error-unknown-config-key = Անհայտ կարգավորման բանալի։
error-invalid-config-subcommand = Անվավեր ենթահրաման config-ի համար։ Օգտագործեք «get», «set» կամ «list»։
error-parse-config = Չհաջողվեց վերլուծել կարգավորման ֆայլը։
error-parse-config-using-default = Օգտագործվում են կարգավորման լռելյայն արժեքները։

help-registry = Կառավարել ռեեստրները, որոնցից տեղադրվում են { -godot(case: "genitive") } կառուցումները
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

error-invalid-registry-subcommand = Անվավեր ռեեստրի ենթահրաման: Օգտագործեք «add», «remove», «list» կամ «refresh»:
registry-trust-warning = { $registry } ({ $url })-ը հատուկ ռեեստր է, ոչ թե պաշտոնականը: { -gdvm(case: "definite") } ստուգում է, որ ներբեռնումները համապատասխանում են ռեեստրի նշածին, բայց չի կարող ջանաչել, թե արդյոք դրանք անվտանգ են գործարկելու համար: Տեղադրեք դրանից միայն այն դեպքում, եթե վստահում եք նրան, ով այն կառավարում է:
registry-trust-prompt = Վստահո՞ւմ եք այս ռեեստրին և ցանկանում եք շարունակել: (այո/ոչ).
registry-trust-bypass = {"\u001b"}[1;31m{ $registry } ({ $url })-ի վստահության ստուգումը բաց է թողնվում, քանը որ օգտագործեցիք --yes: { -gdvm(case: "definite") } չի կարող ստուգել՝ արդյոք դրա ֆայլերն անվտանգ են գործարկելու համար: Կարճ դադար. սեղմեք Ctrl+C հիմա՝ դադարեցնելու համար:{"\u001b"}[0m
registry-trust-aborted = Ընդհատվեց. ռեեստրը վստահելի չէ:
registry-project-override-conflict = Նախագծի { -gdvm-toml(case: "definite") } վերասահմանում է { $registry } ռեեստրը (ձեր կարգավորումը՝ { $machine_url }) որպես { $project_url }: Նախագծի սահմանումն ունի առաջնահերթություն:

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
registry-build-warn-local-hash = Հաշվարկվում է տեղական ֆայլի հեշը՝ ենթադրելով, որ այն համապատասխանում է { $url }-ին: { -gdvm(case: "definite") } URL-ը չի ներբեռնում՝ ստուգելու համար:
registry-build-warn-unverified = Օգտագործվում են ձեր տրամադրած SHA-512-ը և չափը՝ առանց արտեֆակտը ներբեռնելու դրանք ստուգելու: Համոզվեք, որ դրանք ճիշտ են:
registry-build-warn-explicit-store = Օգտագործվում են ձեր տրամադրած SHA-512-ը և/կամ չափը՝ պահված արխիվը չափելու փոխարեն:
registry-build-sha-mismatch = Տրամադրված SHA-512-ը ({ $expected }) չի համապատասխանում արտեֆակտին ({ $actual }):
registry-build-size-mismatch = Տրամադրված չափը ({ $expected }) չի համապատասխանում արտեֆակտին ({ $actual }):
registry-validate-ok =
    { $count ->
        [one] Ռեեստրը վավեր է (ստուգվեց { $count } արտեֆակտ):
       *[other] Ռեեստրը վավեր է (ստուգվեց { $count } արտեֆակտ):
    }
registry-validate-failed = Ռեեստրի վավերացումը ձախողվեց.
