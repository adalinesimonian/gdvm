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
        [b] { NUMBER($value, maximumFractionDigits: 0) } Б
        [kib] { NUMBER($value, maximumFractionDigits: 1) } КиБ
        [mib] { NUMBER($value, maximumFractionDigits: 1) } МиБ
        [gib] { NUMBER($value, maximumFractionDigits: 1) } ГиБ
       *[tib] { NUMBER($value, maximumFractionDigits: 1) } ТиБ
    }

help-about = Менеджер версий { -godot }
help-help = Показать справку (см. краткое описание с '-h')
help-gdvm-version = Показать версию менеджера версий { -godot }

help-install = Установить новую версию { -godot }
help-run = Запустить определенную версию { -godot }
help-show = Показать путь к исполняемому файлу указанной версии { -godot }
help-cache-path = Показать путь к кэшированному архиву загрузки для указанной версии { -godot }
help-link = Создать ссылку на исполняемый файл версии { -godot } по указанному пути
help-list = Список всех установленных версий { -godot }
help-remove = Удалить установленную версию { -godot }
help-csharp = [устарело] Использовать версию { -godot } с поддержкой C#. Используйте спецификатор варианта «csharp» (например, csharp:4.4).
help-run-csharp-long = { help-csharp }
help-version = Версия для установки (например, 4, csharp:4.4, stable, latest).
help-version-long =
    { help-version }

    Формат: [вариант:]версия_или_ключевое_слово

    Если в конце присутствует *, он будет соответствовать самой новой сборке с тем же префиксом, например «4.7-dev*» соответствует 4.7-dev1, 4.7-dev2 и т. д.

    Ключевые слова: «latest» соответствует самой новой версии. По умолчанию включаются только стабильные версии, но предварительные версии можно включить с помощью флага --pre.

    Варианты: добавьте имя варианта и двоеточие, например «csharp:4.4» для C#-версии.

    Примеры: 4.4 установит последнюю стабильную версию { -godot } 4.4. Если существуют только предварительные версии, будет установлена последняя предварительная версия. 4.3-rc* установит последний релиз-кандидат { -godot } 4.3 и т.д.
help-version-installed = Установленная версия (например, 4.2 или 4.2-stable).

help-search = Список доступных релизов из реестра
help-filter = Необязательная строка для фильтрации тегов релизов
help-filter-deprecated = [устарело] Необязательная строка для фильтрации тегов релизов. Вместо этого используйте позиционный аргумент фильтра.
help-include-pre = Включить предварительные версии (rc, beta, dev)
help-cache-only = Использовать только кэшированную информацию о релизах без запроса к реестру
help-limit = Количество релизов для отображения, по умолчанию 10. Используйте 0, чтобы отобразить все
help-clear-cache = Очистить кэш релизов
help-refresh = Обновить кэш релизов из реестра
help-refresh-flag = Обновить кэш релизов перед выполнением этой команды

help-prune = Удалить установки и кэшированные архивы, которые больше не используются
help-prune-long = { help-prune }

    По умолчанию prune удаляет установки, которые давно не использовались, и устаревшие кэшированные архивы загрузок, сохраняя при этом любую установку, на которую всё ещё указывает ссылка. Установка, заданная как стандартная, никогда не удаляется, независимо от переданных флагов. Порог давности настраивается командой «{ -gdvm } config set prune.max-age-days <дни>» (по умолчанию { $default_days } дн.).
help-prune-all = Удалить все установки и кэшированные архивы независимо от давности. Установки, на которые всё ещё указывает активная ссылка, сохраняются, если не указан также --force.
help-prune-force = Игнорировать ссылки, чтобы установки, на которые ссылается только ссылка, тоже могли быть удалены.
help-prune-dry-run = Показать, что было бы удалено, ничего не удаляя.
prune-nothing-dry-run = Ничего не было бы удалено.
prune-nothing-removed = Удалять нечего; всё используется или в пределах порога давности.
prune-preserved-by-link =
    { $count ->
        [one] Сохранена { $count } установка, на которую всё ещё ссылается ссылка.
        [few] Сохранены { $count } установки, на которые всё ещё ссылается ссылка.
       *[many] Сохранено { $count } установок, на которые всё ещё ссылается ссылка.
    }
warning-broken-install-reinstalling = У установленной версии { $version } отсутствует исполняемый файл, она переустанавливается.

help-force = Принудительная переустановка, даже если версия уже установлена.
help-redownload = Повторно загрузить версию, даже если она уже загружена в кэше.
help-yes = Пропустить подтверждение удаления
help-remove-yes-deprecated = [устарело] Этот флаг не выполняет никаких действий и будет удален в будущем выпуске.
help-link-version = Версия для ссылки. Если не указана, версия определяется на основе текущего каталога или версии по умолчанию.
help-link-path = Путь, по которому будет создана ссылка или копия, например «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }».
help-link-force = Перезаписать существующую ссылку, если она есть
help-link-copy = Копировать исполняемый файл вместо создания ссылки
no-cache-files-found = Файлы кэша не найдены.
no-cache-metadata-found = Метаданные кэша не найдены.
gdvm-toml-malformed = файл { -gdvm-toml } в { $path } игнорируется, так как его не удалось разобрать: { $error }

help-diagnose = Проверить установку и сообщить о её состоянии.
diagnose-base-dir = Каталог { -gdvm }: { $path }
diagnose-healthy = Проблем не обнаружено.
diagnose-install-broken = У { $version } отсутствует исполняемый файл. Выполните «{ -gdvm } install» для этой версии, чтобы переустановить её.
diagnose-install-ok = { $version } может запускаться.
diagnose-partial-downloads =
    { $count ->
        [one] В кэше { $count } прерванная загрузка; она возобновляется автоматически, либо «{ -gdvm } prune» удаляет её.
        [few] В кэше { $count } прерванные загрузки; они возобновляются автоматически, либо «{ -gdvm } prune» удаляет их.
       *[many] В кэше { $count } прерванных загрузок; они возобновляются автоматически, либо «{ -gdvm } prune» удаляет их.
    }
diagnose-path-missing = { $path } отсутствует в PATH; шим godot не будет доступен по имени.
diagnose-path-ok = Каталог bin находится в PATH.
diagnose-shim-missing = Шим «{ $name }» отсутствует или не исполняем. Переустановка { -gdvm } перезаписывает его.
diagnose-shim-ok = Шим «{ $name }» установлен и исполняем.

help-console = Запустить { -godot } с консольным окном. По умолчанию false в Windows, true на других платформах.

help-default = Управление версией по умолчанию
help-default-version = Версия для установки по умолчанию (например, 4.2 или 4.2-stable).
no-default-set =Локальная версия не установлена. Запустите "{ -gdvm } use <version>", чтобы установить
    локальную версию для всей системы, или "{ -gdvm } pin <version>", чтобы установить
    локальную версию для текущего каталога.

warning-prerelease = Вы устанавливаете предварительную версию ({$branch}).
warning-deprecated-csharp-flag = Флаг --csharp устарел. Используйте спецификатор варианта "csharp" вместо него (например, csharp:4.4).

label-error = Ошибка:
label-note = Примечание:
label-warning = Внимание:
progress-rate = { size-display }/с
progress-eta-remaining = { $time } осталось
progress-fraction = { $done }/{ $total }
status-downloading = Загрузка
status-extracting = Извлечение
status-fetching = Получение
status-installed = Установлено
status-installing = Установка
status-removed = Удалено
status-healthy = Исправно
status-ok = OK
prune-item-detail = { $label } ({ size-display })
status-freed = Освобождено
status-pruned = Очищено
status-would-free = Будет освобождено
status-would-prune = Будет очищено
status-removing = Удаление
status-running = Запуск
status-cleared = Очищен
status-refreshed = Обновлен
status-skipped = Пропущено
status-upgraded = Обновлено
status-upgrading = Обновление
status-verifying = Проверка
subject-cached-archive = кэшированный архив
subject-cache = кэш
subject-cache-files = файлы кэша
subject-cache-metadata = метаданные кэша
subject-releases = релизы
subject-update-manifest = манифест обновления
upgrade-target = { -gdvm } { $version }

auto-installing-version = Автоматическая установка версии {$version}

no-versions-installed = Версии не установлены.
installed-versions = Установленные версии { -godot }:
progress-eta =
    { $magnitude ->
        [seconds] { $secs } с
        [minutes] { $mins } мин { $secs } с
       *[hours] { $hours } ч { $mins } мин
    }

unsupported-platform = Неподдерживаемая платформа
unsupported-architecture = Неподдерживаемая архитектура
error-checksum-mismatch = Несоответствие контрольной суммы для файла { $file }
error-invalid-sha-length = Неверная длина SHA { $length }
error-size-mismatch = Несоответствие размера для файла { $file }: ожидалось { $expected } байт, получено { $actual } байт.
error-insecure-url = Отказ в получении { $url } через незашифрованное соединение. Разрешены только URL-адреса https:// и file://. Установите переменную окружения GDVM_ALLOW_INSECURE_URLS, чтобы разрешить незашифрованные URL-адреса http://.
error-insecure-redirect = Отказ в переходе по перенаправлению с https:// на незашифрованный URL-адрес http://. Установите переменную окружения GDVM_ALLOW_INSECURE_URLS, чтобы разрешить незашифрованные URL-адреса http://.
error-response-not-utf8 = Ответ от { $url } не является корректным UTF-8.
error-response-too-large = Ответ от { $url } превышает максимально допустимый размер { $limit } байт.
error-too-many-redirects = Слишком много перенаправлений.
error-config-invalid-number = Недопустимое значение для { $key }: { $value } (ожидалось число)
error-config-unknown-key = Неизвестный ключ конфигурации: { $key }
error-config-path-empty = Путь не может быть пустым.
error-config-path-file = Путь указывает на файл, а не на каталог: { $path }
error-config-path-reserved = Путь зарезервирован для внутренних нужд gdvm: { $path }
error-config-path-overlap = Настроенные пути не должны пересекаться: { $key }
error-invalid-path = Недопустимый путь: { $path }
error-publish-missing-manifest = отсутствует registry.json
error-publish-no-such-version = нет такой версии: { $version }
error-publish-store-or-url-required = необходимо указать --store или --url
error-publish-store-requires-file = --store требует локальный --file
error-publish-url-requires-integrity = --url требует либо локальный --file, либо явные --sha512 и --size
error-publish-already-initialized = Реестр уже инициализирован в { $path }
error-publish-archive-not-found = Архив не найден: { $path }
error-publish-no-such-platform = Платформа { $platform } для варианта { $variant } не найдена
error-publish-no-such-variant = Вариант { $variant } не найден
error-publish-invalid-segment = Некорректный { $what }: { $value }
error-registry-fetch-failed = Не удалось получить { $url }: HTTP { $status }
error-registry-fetch-release-failed = Не удалось получить метаданные выпуска
error-registry-invalid-name = Недопустимое имя реестра: { $name }
error-registry-missing-index = В реестре «{ $name }» отсутствует index.json
error-registry-missing-manifest = В реестре «{ $name }» отсутствует registry.json
error-registry-not-configured = Реестр «{ $name }» не настроен
error-registry-parse-index = Не удалось разобрать индекс реестра «{ $name }».
error-registry-parse-manifest = Не удалось разобрать манифест реестра «{ $name }».
error-registry-unknown = Неизвестный реестр «{ $name }»
error-registry-unsupported-url-scheme = Неподдерживаемая схема URL реестра: { $url }
error-spec-empty-registry = Пустое имя реестра в «{ $input }»
error-spec-empty-variant = Пустое имя варианта в «{ $input }»
error-spec-empty-version = Пустая версия в «{ $input }»
error-system-time = Системное время раньше эпохи UNIX
error-unrecognized-version-format = Нераспознанный формат версии: { $input }
error-diagnose-problems =
    { $count ->
        [one] Обнаружена { $count } проблема.
        [few] Обнаружено { $count } проблемы.
       *[many] Обнаружено { $count } проблем.
    }
error-non-interactive-trust = Невозможно запросить доверие к реестру «{ $registry }» ({ $url }) в неинтерактивном сеансе. Передайте --yes, чтобы явно доверять ему.
error-non-interactive-value = Невозможно запросить значение для «{ $key }» в неинтерактивном сеансе. Вместо этого передайте значение аргументом.
error-registry-unsupported-schema = Реестр «{ $registry }» объявляет неподдерживаемую версию схемы { $schema }.
label-caused-by = Причина:
label-error-coded = Ошибка { $code }:
error-wildcard-position = Подстановочный знак (*) может стоять только в конце тега выпуска, например 4.7-dev* (получено { $input }).
hint-try-wildcard = Выпуска с тегом { $requested } нет, но есть похожие теги, самый новый из которых — { $newest }. Попробуйте { $suggestion }, чтобы найти их.
download-retrying = Загрузка прервана, повторная попытка ({ $attempt } из { $max })...
download-resuming = Возобновляется прерванная загрузка (уже загружено { size-display }).
warning-resume-verification-failed = Возобновлённая загрузка не совпала с ожидаемой контрольной суммой, она загружается заново целиком.
lock-waiting = Ожидание завершения другого процесса { -gdvm } (блокировка: { $resource })...
prune-skipped-error = Пропуск { $item }: { $error }
prune-skipped-in-use = Пропуск { $item }: он используется другим процессом { -gdvm }.

error-find-user-dirs = Не удалось найти пользовательские каталоги.
warning-fetching-releases-using-cache = Ошибка при получении релизов: { $error }. Используются кэшированные релизы.

error-version-not-found = Версия не найдена.
error-archive-not-cached = Кэшированный архив для {$version} не найден. Сначала установите его, чтобы заполнить кэш.
error-multiple-versions-found = Найдено несколько версий, соответствующих запросу:
    {$list}
link-created = Создана ссылка: {$version} -> {$path}
copy-created = Создана копия версии {$version} по пути {$path}
no-matching-releases = Соответствующие релизы не найдены.
available-releases = Доступные релизы:

version-already-installed = Версия {$version} уже установлена.
godot-executable-not-found = Исполняемый файл { -godot } не найден для версии {$version}.
error-link-exists = Путь {$path} уже существует. Используйте --force для перезаписи.
error-link-symlink = Не удалось создать ссылку из {$link} в {$target}.
error-link-copy = Не удалось скопировать файл.

error-no-stable-releases-found = Стабильные версии не найдены.

error-starting-godot = Не удалось запустить { -godot }.
confirm-yes = да

default-set-success = Успешно установлено {$version} как версия { -godot } по умолчанию.
default-unset-success = Успешно удалено значение версии { -godot } по умолчанию.
provide-version-or-unset = Пожалуйста, укажите версию для установки по умолчанию или 'unset' для удаления версии по умолчанию.

error-open-zip = Не удалось открыть ZIP файл { $path }.
error-read-zip = Не удалось прочитать ZIP архив { $path }.
error-access-file = Не удалось получить доступ к файлу по индексу { $index }.
error-reopen-zip = Не удалось повторно открыть ZIP файл { $path }.
error-invalid-file-name = Недопустимое имя файла в ZIP архиве
error-create-dir = Не удалось создать каталог { $path }.
error-create-file = Не удалось создать файл { $path }.
error-read-zip-file = Не удалось прочитать из ZIP файла { $file }.
error-write-file = Не удалось записать в файл { $path }.
error-strip-prefix = Ошибка удаления префикса.
error-set-permissions = Не удалось установить разрешения для { $path }.
error-create-symlink-windows = Не удалось создать символьную ссылку. Убедитесь, что {"\u001b"}]8;;ms-settings:developers{"\u001b"}\режим разработчика{"\u001b"}]8;;{"\u001b"}\ включен или запустите от имени администратора.

help-upgrade = Обновить { -gdvm } до последней версии
help-upgrade-major = Разрешить обновление через основные версии
help-upgrade-pre = Обновить до последнего предварительного выпуска
upgrade-not-needed = { -gdvm } уже на последней версии: { $version }.
upgrade-current-version-newer = Текущая версия { -gdvm } ({ $current }) новее, чем последняя доступная версия ({ $latest }). Обновление не требуется.
upgrade-install-dir-failed = Не удалось создать директорию установки.
upgrade-rename-failed = Не удалось переименовать текущий исполняемый файл.
upgrade-replace-failed = Не удалось заменить исполняемый файл на новый.
upgrade-no-binary = Нет доступного двоичного файла { -gdvm } для версии { $version } и цели { $target }.
upgrade-checksum-required = Манифест выпуска не содержит контрольную сумму для этого двоичного файла { -gdvm }. Обновление отклонено.
error-fetching-gdvm-releases = Ошибка получения релизов { -gdvm }.
error-parsing-gdvm-releases = Ошибка разбора релизов { -gdvm }.
error-unsupported-gdvm-schema = Неподдерживаемая версия схемы манифеста релизов { -gdvm }: { $schema }. Попробуйте обновить { -gdvm } вручную.
upgrade-available = 💡 Доступна новая версия { -gdvm }: {$version}. Запустите «{ -gdvm } upgrade», чтобы обновить.
upgrade-available-major = 💡 Доступно обновление основной версии { -gdvm }: {$version}. Запустите «{ -gdvm } upgrade -m», чтобы обновить.
upgrade-available-both = 💡 Доступна новая версия { -gdvm }: {$minor_version}. Также доступно обновление основной версии: {$major_version}. Запустите «{ -gdvm } upgrade» для обновления в рамках текущей основной версии или «{ -gdvm } upgrade -m» для обновления до последней версии.
upgrade-prerelease-available = 💡 Доступен новый предварительный выпуск { -gdvm }. Выполните «{ -gdvm } upgrade --pre», чтобы установить его.

help-pin = Закрепить версию { -godot } в текущем каталоге.
help-pin-long = { help-pin }

    Это создаст файл { -gdvm-toml } в текущем каталоге с закреплённой версией. Когда вы запустите "{ -gdvm } run" в этом каталоге или любом из его подкаталогов, будет использоваться закреплённая версия вместо версии по умолчанию.

    Это полезно, когда вы хотите использовать определённую версию { -godot } для проекта, не изменяя версию по умолчанию для всей системы.

    В настоящее время также записывается устаревший файл { -gdvmrc } для совместимости со старыми версиями { -gdvm }. Он будет удалён в будущем выпуске, поэтому рекомендуется перейти на новый формат { -gdvm-toml } и удалить файл { -gdvmrc }, если он существует.

    Вы можете отключить запись файла { -gdvmrc } с помощью флага --no-legacy.
help-pin-version = Укажите версию для закрепления
help-no-legacy = Не записывать устаревший файл совместимости { -gdvmrc }
pinned-success = Версия {$version} успешно закреплена в { -gdvm-toml }
error-pin-version-not-found = Невозможно закрепить версию {$version}

error-file-not-found = Файл не найден. Возможно, он не существует на сервере.
error-download-failed = Загрузка не удалась из-за ошибки HTTP-статуса { $status }.
error-ensure-godot-binaries-failed = Не удалось гарантировать исполняемые файлы { -godot }.

error-post-upgrade-action-failed = Шаг { $id } завершился ошибкой после обновления.
    Установка { -gdvm } может быть неполной. Попробуйте запустить { -gdvm } снова.

error-failed-reading-project-godot = Не удалось прочитать project.godot, невозможно автоматически определить версию проекта.
warning-using-project-version = Используется версия { $version }, указанная в project.godot.
warning-gdvmrc-detected = Был обнаружен пользовательский файл { -gdvmrc }. Поддержка файлов { -gdvmrc } устарела и будет удалена в будущем выпуске. Пожалуйста, перейдите на новый формат закрепления файлов, используемый командой `{ -gdvm } pin`.

warning-project-version-mismatch =
    {"\u001b"}[33mПредупреждение: версия, указанная в project.godot, не совпадает с { $pinned ->
        [1] закреплённой
        *[0] запрошенной
    } версией. Открытие проекта с { $pinned ->
        [1] закреплённой
        *[0] запрошенной
    } версией может перезаписать файл проекта.{"\u001b"}[0m

    { $pinned ->
        [1] Версия проекта:      { $project_version }
            Закреплённая версия: { $requested_version }
        *[0] Версия проекта:       { $project_version }
             Запрошенная версия:   { $requested_version }
    }

error-project-version-mismatch = {"\u001b"}[31m{ $pinned ->
        [1] Если вы уверены, что хотите запустить проект с закреплённой версией, выполните {"\u001b"}[0m{ -gdvm } run --force{"\u001b"}[31m. В противном случае обновите закреплённую версию в { -gdvmrc }, чтобы она соответствовала версии проекта, или удалите файл { -gdvmrc }, чтобы использовать версию проекта.
        *[0] Если вы уверены, что хотите запустить проект с запрошенной версией, выполните {"\u001b"}[0m{ -gdvm } run --force <version>{"\u001b"}[31m.
    }{"\u001b"}[0m

help-run-args = Дополнительные аргументы для передачи исполняемому файлу { -godot } (например, -- путь/к/project.godot).
help-run-force =
    Принудительный запуск проекта с запрошенной или закреплённой версией, даже если она не совпадает с версией проекта.
help-run-force-long =
    { help-run-force }

    Если вы сделаете это, запрошенная или закреплённая версия { -godot } может перезаписать файл проекта. Если вы закрепляете версии, рекомендуется вместо этого обновить закреплённую версию в { -gdvmrc }, чтобы она соответствовала версии проекта, или удалить файл { -gdvmrc }, чтобы использовать версию проекта.

help-config = Управление конфигурацией { -gdvm }
help-format = Формат вывода: text (по умолчанию) или json
help-info = Показать подробную информацию об установленной версии
info-default =
    { $value ->
        [1] { confirm-yes }
       *[0] { info-no }
    }
    .label = По умолчанию:
info-executable = { $path }
    .label = Исполняемый файл:
info-install-path = { $path }
    .label = Путь установки:
info-last-used = { $timestamp }
    .label = Последнее использование:
info-no = нет
info-registry = { $registry }
    .label = Реестр:
info-size = { size-display }
    .label = Размер на диске:
info-variant = { $variant }
    .label = Вариант:
info-version = { $version }
    .label = Версия:
help-completions = Сгенерировать скрипты автодополнения для оболочки
help-completions-shell = Оболочка, для которой генерируются автодополнения
help-config-get = Получить значение параметра конфигурации
help-config-set = Установить значение параметра конфигурации
help-config-unset = Удалить значение параметра конфигурации
help-config-list = Показать все параметры конфигурации
help-config-key = Ключ конфигурации (например, prune.max-age-days)
help-config-value = Значение для установки ключа конфигурации
help-config-unset-key = Ключ конфигурации для удаления (например, prune.max-age-days)
help-config-show-sensitive = Показать чувствительные параметры конфигурации в открытом виде
help-config-available = Показать все доступные ключи конфигурации и их значения, включая значения по умолчанию
warning-setting-sensitive = Вы устанавливаете чувствительное значение, которое будет сохранено в открытом виде в вашем домашнем каталоге.
config-set-prompt = Пожалуйста, введите значение для { $key }:
error-reading-input = Ошибка чтения ввода
config-set-success = Конфигурация успешно обновлена.
config-unset-success = Ключ конфигурации { $key } успешно удалён.
config-key-not-set = Ключ конфигурации не установлен.
config-key-not-set-value = <не установлено>
error-unknown-config-key = Неизвестный ключ конфигурации.
error-invalid-config-subcommand = Недопустимая подкоманда config. Используйте "get", "set" или "list".
error-parse-config = Не удалось разобрать файл конфигурации.
error-parse-config-using-default = Используются значения конфигурации по умолчанию.

help-registry = Управление реестрами для установки сборок { -godot }
help-registry-add = Добавить реестр
help-registry-remove = Удалить реестр
help-registry-list = Показать настроенные реестры
help-registry-refresh = Обновить кэш одного или всех реестров
help-registry-name = Имя реестра
help-registry-url = URL реестра. Может быть http(s):// или file:// URL.

registry-added = Реестр { $registry } добавлен ({ $url }).
registry-removed = Реестр { $registry } удалён.
registry-list-header = Настроенные реестры:
registry-tag-official = официальный

error-invalid-registry-subcommand = Недопустимая подкоманда реестра. Используйте «add», «remove», «list» или «refresh».
registry-trust-warning = { $registry } ({ $url }) — это пользовательский реестр, а не официальный. { -gdvm } проверяет, что загрузки соответствуют тому, что указывает реестр, но не может определить, безопасно ли их запускать. Устанавливайте из него, только если доверяете тому, кто им управляет.
registry-trust-prompt = Доверяете ли вы этому реестру и хотите продолжить? (да/нет):
registry-trust-bypass = {"\u001b"}[1;31mПроверка доверия для { $registry } ({ $url }) пропущена, потому что вы использовали --yes. { -gdvm } не может определить, безопасно ли запускать его файлы. Небольшая пауза; нажмите Ctrl+C сейчас, чтобы остановить.{"\u001b"}[0m
registry-trust-aborted = Прервано: реестр не является доверенным.
registry-project-override-conflict = Файл { -gdvm-toml } проекта переопределяет реестр { $registry } (ваша конфигурация: { $machine_url }) как { $project_url }. Определение проекта имеет приоритет.

help-registry-init = Инициализировать новый каталог реестра
help-registry-add-build = Добавить сборку в реестр
help-registry-remove-build = Удалить сборку из реестра
help-registry-validate = Проверить каталог реестра
help-registry-dir = Каталог реестра
help-registry-init-name = Имя реестра. По умолчанию имя каталога.

help-registry-build-version = Тег версии, например, 4.4-stable.
help-registry-build-variant = Имя варианта. По умолчанию «default».
help-registry-build-platform = Ключ платформы, например, linux-x86_64.
help-registry-build-file = Путь к архиву сборки для хеширования
help-registry-build-store = Скопировать архив в реестр и записать относительный URL
help-registry-build-url = URL, по которому будет раздаваться архив (если не используется --store)
help-registry-build-sha512 = SHA-512 архива вместо его вычисления. Требует --size.
help-registry-build-size = Размер архива в байтах вместо его измерения. Требует --sha512.

registry-init-success = Реестр { $name } инициализирован в { $path }.
registry-build-added = Сборка { $version } добавлена для { $platform }.
registry-build-removed = Сборка { $version } удалена.
registry-build-warn-local-hash = Хеширование локального файла в предположении, что он соответствует { $url }. { -gdvm } не загружает URL для проверки.
registry-build-warn-unverified = Используются указанные вами SHA-512 и размер без загрузки артефакта для их проверки. Убедитесь, что они верны.
registry-build-warn-explicit-store = Используются указанные вами SHA-512 и/или размер вместо измерения сохранённого архива.
registry-build-sha-mismatch = Указанный SHA-512 ({ $expected }) не соответствует артефакту ({ $actual }).
registry-build-size-mismatch = Указанный размер ({ $expected }) не соответствует артефакту ({ $actual }).
registry-validate-ok =
    { $count ->
        [one] Реестр корректен (проверен { $count } артефакт).
        [few] Реестр корректен (проверено { $count } артефакта).
       *[many] Реестр корректен (проверено { $count } артефактов).
    }
registry-validate-failed = Проверка реестра не пройдена:
