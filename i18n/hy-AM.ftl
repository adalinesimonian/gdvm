hello = Բարեւ, Աշխարհ:

help-about = Godot տարբերակների կառավարիչ
help-help = Ցուցադրել օգնություն (տես ամփոփում '-h'-ով)
help-help-command = Ցուցադրել այս հաղորդագրությունը կամ օգնությունը նշված ենթահրամանների համար
help-gdvm-version = Ցուցադրել Godot տարբերակների կառավարչի տարբերակը

help-install = Տեղադրել նոր Godot տարբերակ
help-run = Գործարկել որոշակի Godot տարբերակ
help-list = Ցուցադրել բոլոր տեղադրված Godot տարբերակները
help-remove = Հեռացնել տեղադրված Godot տարբերակը

help-branch = Ճյուղը (կայուն, բետա, ալֆա կամ հարմարեցված):
help-csharp = Օգտագործել Godot տարբերակը C# աջակցության համար:
help-run-csharp-long = { help-csharp }

    Եթե տրված է, արժեքը անվերջակետում է լռելյայն տարբերակի համար, այլապես լռելյայն տարբերակը կօգտագործվի: Այսպիսի դեպքում, եթե դուք սահմանել եք լռելյայն տարբերակը "use --csharp"-ով, դուք կկարողանաք փորձել գործարկել նույն տարբերակը, բայց առանց C# աջակցության "run --csharp false"-ով: Միայն այն դեպքում է աշխատելու ակնարկություն, եթե աջակցության առանց տեղադրված տարբերակը չի տեղադրված: (Միայն գործարկեք "տեղադրել" այնտեղ:)
help-version = Տեղադրվող տարբերակը (օրինակ՝ 4), կամ "stable" վերջին կայուն տարբերակի համար:
help-version-long =
    { help-version }

    Օրինակներ՝ 4.4-ը կտեղադրի Godot 4.4-ի վերջին կայուն թողարկումը: Եթե միայն նախնական թողարկումներ կան, ապա կտեղադրվի վերջին նախնական թողարկումը: 4.3-rc-ը կտեղադրի Godot 4.3-ի վերջին թողարկման թեկնածուն և այլն:
help-version-installed = Տեղադրված տարբերակը (օրինակ՝ 4.2 կամ 4.2-stable):

help-search = Ցուցադրել հեռավոր թողարկումները godot-builds-ից
help-filter = Ընտրովի տող թողարկման պիտակները ֆիլտրելու համար
help-include-pre = Ներառել նախնական թողարկումները (rc, beta, dev)
help-cache-only = Օգտագործել միայն պահված թողարկման տեղեկատվությունը առանց GitHub API հարցման
help-limit = Ցուցադրվող թողարկումների քանակը, լռելյայն 10. Օգտագործեք 0՝ բոլորի ցուցադրման համար
help-clear-cache = Մաքրել gdvm թողարկման քեշը

help-yes = Բաց թողնել հեռացման հաստատման հուշումը

help-default = Կառավարել լռելյայն տարբերակը
help-default-version = Տարբերակը, որը պետք է տեղադրվի լռելյայն (օրինակ՝ 4.2 կամ 4.2-stable):
no-default-set = Լռելյայն տարբերակը սահմանված չէ: Գործարկեք "gdvm use <version>"՝ լռելյայն տարբերակը համակարգային մակարդակով սահմանելու համար, կամ "gdvm pin <version>"՝ լռելյայն տարբերակը ընթացիկ պանակի համար սահմանելու համար:

help-upgrade = Թարմացնել gdvm-ը վերջին տարբերակին

help-pin = «Գամել» Godot-ի տարբերակը ընթացիկ պանակում:
help-pin-long = { help-pin }

    Սա կստեղծի .gdvmrc ֆայլ ընթացիկ պանակում գամված տարբերակով: Երբ դուք կգործարկեք "gdvm run" այս պանակում կամ դրա ենթապանակներում, կօգտագործվի գամված տարբերակը լռելյայն տարբերակի փոխարեն:

    Սա օգտակար է, երբ ցանկանում եք օգտագործել որոշակի Godot տարբերակ նախագծի համար, առանց փոխելու լռելյայն տարբերակը ամբողջ համակարգում:
help-pin-version = Տարբերակը, որը պետք է գամել
pinned-success = {$version} տարբերակը հաջողությամբ գամվեց .gdvmrc-ում
error-pin-version-not-found = Չհաջողվեց գամել {$version} տարբերակը
pin-subcommand-description = Սահմանում կամ թարմացնում է .gdvmrc ֆայլը նշված տարբերակով

installing-version = Տեղադրվում է {$version} տարբերակը
installed-success = {$version} տարբերակը հաջողությամբ տեղադրվեց

warning-prerelease = Զգուշացում: Դուք տեղադրում եք նախնական թողարկում ({$branch}).

no-versions-installed = Տեղադրված տարբերակներ չկան:
installed-versions = Տեղադրված Godot տարբերակներ:
removed-version = {$version} տարբերակը հեռացված է

removing-version = Հեռացվում է {$version} տարբերակը

confirm-remove = Վստա՞հ եք, որ ցանկանում եք հեռացնել այս տարբերակը: (այո/ոչ):
confirm-yes = այո
remove-cancelled = Հեռացումը չեղարկված է:

error-version-not-found = Տարբերակը չի գտնվել:
error-multiple-versions-found = Մի քանի տարբերակներ են համընկնում ձեր հարցմանը:

error-invalid-godot-version = Անվավեր Godot տարբերակի ձևաչափ: սպասվող ձևաչափեր՝ x, x.y, x.y.z, x.y.z.w կամ x.y.z-tag.
error-invalid-remote-version = Անվավեր հեռավոր Godot տարբերակի ձևաչափ: սպասվող ձևաչափեր՝ x, x.y, x.y.z, x.y.z.w, x.y.z-tag կամ "stable".

running-version = Գործարկվում է {$version} տարբերակը
no-matching-releases = Համապատասխան թողարկումներ չեն գտնվել:
available-releases = Հասանելի թողարկումներ:
cache-cleared = Քեշը հաջողությամբ մաքրվել է:

version-already-installed = Տարբերակը արդեն տեղադրված է։ օգտագործեք {$version}:
godot-executable-not-found = Godot-ի գործարկվող ֆայլը չի գտնվել {$version} տարբերակի համար:

unsupported-platform = Չաջակցվող համակարգ
unsupported-architecture = Չաջակցվող ստեղծածություն

verifying-checksum = Ստուգվում է ստորակետը...
checksum-verified = Ստորակետը ստուգված է:
error-checksum-mismatch = Ստորակետի չափազանելը չհամընկեց ֆայլի { $file }

error-find-user-dirs = Չհաջողվեց գտնել օգտագործողի պանակները։

fetching-releases = Թողարկումները բերվում են...
releases-fetched = Թողարկումները բերվել են։

error-starting-godot = Չհաջողվեց գործարկել Godot-ը՝ { $error }

help-console = Godot-ը կատարողական կոնսոլով: լռելյայն է Windows-ում, այլ համակարգչներում ակտիվացվում է:

default-set-success = Հաջողությամբ սահմանվել է {$version} որպես լռելյայն Godot տարբերակը։
default-unset-success = Հաջողությամբ հեռացվեց լռելյայն Godot տարբերակը։
provide-version-or-unset = Խնդրում ենք տրամադրել տարբերակ՝ սահմանելու համար լռելյայն կամ 'unset'՝ լռելյայն տարբերակը հեռացնելու համար։

error-no-stable-releases-found = Կայուն թողարկումներ չեն գտնվել:
force-reinstalling-version = Բարդ ընդկարեն տեղադրությունը տարբերակ {$version}.

help-force = Ստիպել վերատեղադրումը, նույնիսկ եթե տարբերակը արդեն տեղադրված է:
help-redownload = Նորից ներբեռնել տարբերակը, նույնիսկ եթե այն արդեն տեղադրված է:

cached-zip-stored = Պահված է Godot թողարկումը արխիվում։
using-cached-zip = Օգտագործել պահված թողարկումը, ներբեռնումը անջատված է։
warning-cache-metadata-reset = Պահված թողարկումների ինդեքսը անվավեր է կամ կորուպտավորված է։ Վերսկսելու համար։
cache-files-removed = Պահված թողարկումները հեռացվել են։
cache-metadata-removed = Պահված թողարկումների ինդեքսը հեռացվել է։
error-cache-metadata-empty = Սխալ՝ պահված թողարկումների ինդեքսը դատարկ է, անհրաժեշտ է ստանալ թողարկումները առաջին համար։
no-cache-files-found = Պահված թողարկումներ չեն գտնվել։
no-cache-metadata-found = Պահված թողարկումների ինդեքսը չի գտնվել։

force-redownload = Տեղադրման վերանեղմումը տարբերակի {$version} համար։
operation-downloading-url = Ներբեռնում Ձեզ URL-ից {$url}...
operation-download-complete = Ամրագրումը ավարտվել է.
operation-extracting = Բացահայտում...
operation-extract-complete = Բացահայտումը ավարտված է.

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

auto-installing-version = Ինքնաշխատ տեղադրում { $version } տարբերակի
warning-sha-sums-missing = Չհաջողվեց գտնել ստուգանակոչումները այս թողարկման համար։ Թողարկման հաստատումը հապողվեց։

upgrade-starting = Սկսվում է gdvm-ի թարմացումը...
upgrade-downloading-latest = Ներբեռնում է gdvm-ի վերջին տարբերակը...
upgrade-complete = gdvm-ը հաջողությամբ թարմացվեց!
upgrade-failed = Թարմացումը ձախողվեց՝ { $error }
upgrade-download-failed = Թարմացման բեռնումը ձախողվեց: { $error }
upgrade-file-create-failed = Չհաջողվեց ստեղծել թարմացման ֆայլը: { $error }
upgrade-file-write-failed = Չհաջողվեց գրելու տվյալները թարմացման ֆայլում: { $error }
upgrade-install-dir-failed = Չհաջողվեց ստեղծել տեղադրման ծառարանները: { $error }
upgrade-rename-failed = Չհաջողվեց փոխանակել գործող գործարկվող ֆայլի անունը: { $error }
upgrade-replace-failed = Չհաջողվեց փոխարինել գործող գործարկվող ֆայլը նորով: { $error }

upgrade-available = 💡 Նոր gdvm տարբերակը հասանելի է: {$version}։ Գործարկեք «gdvm upgrade» թարմացման համար։

error-file-not-found = Ֆայլը չի գտնվել։ Հնարավոր է, որ այն գոյություն չունի սերվերի վրա։
error-download-failed = Ներբեռնումը ձախողվեց անակնկալ սխալի պատճառով։ { $error }

checking-updates = Ստուգվում են թարմացումները gdvm-ի համար...
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
