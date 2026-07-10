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

hello = Привет, мир!

help-about = Менеджер версий Godot
help-help = Показать справку (см. краткое описание с '-h')
help-help-command = Показать это сообщение или справку по указанным подкомандам
help-gdvm-version = Показать версию менеджера версий Godot

help-install = Установить новую версию Godot
help-run = Запустить определенную версию Godot
help-show = Показать путь к исполняемому файлу указанной версии Godot
help-cache-path = Показать путь к кэшированному архиву загрузки для указанной версии Godot
help-link = Создать ссылку на исполняемый файл версии Godot по указанному пути
help-list = Список всех установленных версий Godot
help-remove = Удалить установленную версию Godot

help-branch = Ветка (stable, beta, alpha или custom).
help-csharp = [устарело] Использовать версию Godot с поддержкой C#. Используйте спецификатор варианта «csharp» (например, csharp:4.4).
help-run-csharp-long = { help-csharp }
help-version = Версия для установки (например, 4, csharp:4.4, stable, latest).
help-version-long =
    { help-version }

    Формат: [вариант:]версия_или_ключевое_слово

    Ключевые слова: «latest» соответствует самой новой версии. По умолчанию включаются только стабильные версии, но предварительные версии можно включить с помощью флага --pre.

    Варианты: добавьте имя варианта и двоеточие, например «csharp:4.4» для C#-версии.

    Примеры: 4.4 установит последнюю стабильную версию Godot 4.4. Если существуют только предварительные версии, будет установлена последняя предварительная версия. 4.3-rc установит последний релиз-кандидат Godot 4.3 и т.д.
help-version-installed = Установленная версия (например, 4.2 или 4.2-stable).

help-search = Список доступных релизов из реестра
help-filter = Необязательная строка для фильтрации тегов релизов
help-include-pre = Включить предварительные версии (rc, beta, dev)
help-cache-only = Использовать только кэшированную информацию о релизах без запроса к реестру
help-limit = Количество релизов для отображения, по умолчанию 10. Используйте 0, чтобы отобразить все
help-clear-cache = Очистить кэш релизов
help-refresh = Обновить кэш релизов из реестра
help-refresh-flag = Обновить кэш релизов перед выполнением этой команды

help-prune = Удалить установки и кэшированные архивы, которые больше не используются
help-prune-long = { help-prune }

    По умолчанию prune удаляет установки, которые давно не использовались, и устаревшие кэшированные архивы загрузок, сохраняя при этом любую установку, на которую всё ещё указывает ссылка. Установка, заданная как стандартная, никогда не удаляется, независимо от переданных флагов. Порог давности настраивается командой «gdvm config set prune.max-age-days <дни>» (по умолчанию { $default_days } дн.).
help-prune-all = Удалить все установки и кэшированные архивы независимо от давности. Установки, на которые всё ещё указывает активная ссылка, сохраняются, если не указан также --force.
help-prune-force = Игнорировать ссылки, чтобы установки, на которые ссылается только ссылка, тоже могли быть удалены.
help-prune-dry-run = Показать, что было бы удалено, ничего не удаляя.

prune-dry-run-header = Будет удалено следующее (пробный запуск):
prune-removed-header = Удалено следующее:
prune-installs-header = Установки:
prune-archives-header = Кэшированные архивы:
prune-nothing-dry-run = Ничего не было бы удалено.
prune-nothing-removed = Удалять нечего; всё используется или в пределах порога давности.
prune-preserved-by-link = Сохранено установок, на которые всё ещё ссылается ссылка: { $count }.
prune-freed = Освобождено примерно { $size }.
prune-would-free = Было бы освобождено примерно { $size }.

help-force = Принудительная переустановка, даже если версия уже установлена.
help-redownload = Повторно загрузить версию, даже если она уже загружена в кэше.
help-yes = Пропустить подтверждение удаления
help-link-version = Версия для ссылки. Если не указана, версия определяется на основе текущего каталога или версии по умолчанию.
help-link-path = Путь, по которому будет создана ссылка или копия, например «{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }».
help-link-force = Перезаписать существующую ссылку, если она есть
help-link-copy = Копировать исполняемый файл вместо создания ссылки

cached-zip-stored = Архив релиза Godot сохранен в кэше.
using-cached-zip = Используется кэшированный архив релиза, пропускается загрузка.
warning-cache-metadata-reset = Индекс кэша релизов недействителен или поврежден. Сброс.
cache-files-removed = Файлы кэша успешно удалены.
cache-metadata-removed = Метаданные кэша успешно удалены.
error-cache-metadata-empty = Ошибка: Метаданные кэша пусты, необходимо сначала получить релизы.
no-cache-files-found = Файлы кэша не найдены.
no-cache-metadata-found = Метаданные кэша не найдены.
gdvm-toml-malformed = {"\u001b"}[33mПредупреждение: файл gdvm.toml в { $path } игнорируется, так как его не удалось разобрать: { $error }{"\u001b"}[0m

help-console = Запустить Godot с консольным окном. По умолчанию false в Windows, true на других платформах.

help-default = Управление версией по умолчанию
help-default-version = Версия для установки по умолчанию (например, 4.2 или 4.2-stable).
no-default-set =Локальная версия не установлена. Запустите "gdvm use <version>", чтобы установить
    локальную версию для всей системы, или "gdvm pin <version>", чтобы установить
    локальную версию для текущего каталога.

installing-version = Установка версии {$version}
installed-success = Версия {$version} успешно установлена

warning-prerelease = {"\u001b"}[33mВнимание: Вы устанавливаете предварительную версию ({$branch}).{"\u001b"}[0m
warning-deprecated-csharp-flag = {"\u001b"}[33mВнимание: Флаг --csharp устарел. Используйте спецификатор варианта "csharp" вместо него (например, csharp:4.4).{"\u001b"}[0m

force-reinstalling-version = Принудительная переустановка версии {$version}.

auto-installing-version = Автоматическая установка версии {$version}

no-versions-installed = Версии не установлены.
installed-versions = Установленные версии Godot:
removed-version = Версия {$version} удалена
removing-version = Удаление версии {$version}

force-redownload = Принудительная повторная загрузка версии {$version}.
operation-downloading-url = Загружается {$url}...
operation-download-complete = Загрузка завершена.
operation-extracting = Извлечение...
operation-extract-complete = Извлечение завершено.

unsupported-platform = Неподдерживаемая платформа
unsupported-architecture = Неподдерживаемая архитектура

verifying-checksum = Проверка контрольной суммы...
checksum-verified = Контрольная сумма проверена.
error-checksum-mismatch = Несоответствие контрольной суммы для файла { $file }
error-invalid-sha-length = Неверная длина SHA { $length }
error-size-mismatch = Несоответствие размера для файла { $file }: ожидалось { $expected } байт, получено { $actual } байт.
error-insecure-url = Отказ в получении { $url } через незашифрованное соединение. Разрешены только URL-адреса https:// и file://. Установите переменную окружения GDVM_ALLOW_INSECURE_URLS, чтобы разрешить незашифрованные URL-адреса http://.
error-insecure-redirect = Отказ в переходе по перенаправлению с https:// на незашифрованный URL-адрес http://. Установите переменную окружения GDVM_ALLOW_INSECURE_URLS, чтобы разрешить незашифрованные URL-адреса http://.
error-response-not-utf8 = Ответ от { $url } не является корректным UTF-8: { $error }
error-response-too-large = Ответ от { $url } превышает максимально допустимый размер { $limit } байт.
error-too-many-redirects = Слишком много перенаправлений.

error-find-user-dirs = Не удалось найти пользовательские каталоги.

fetching-releases = Получение релизов...
releases-fetched = Релизы получены.
error-fetching-releases = Ошибка при получении релизов: { $error }
warning-fetching-releases-using-cache = Ошибка при получении релизов: { $error }. Используются кэшированные релизы.

error-version-not-found = Версия не найдена.
error-archive-not-cached = Кэшированный архив для {$version} не найден. Сначала установите его, чтобы заполнить кэш.
error-multiple-versions-found = Найдено несколько версий, соответствующих запросу:

running-version = Запуск версии {$version}
link-created = Создана ссылка: {$version} -> {$path}
copy-created = Создана копия версии {$version} по пути {$path}
no-matching-releases = Соответствующие релизы не найдены.
available-releases = Доступные релизы:
cache-cleared = Кэш успешно очищен.
cache-refreshed = Кэш успешно обновлен.

version-already-installed = Версия {$version} уже установлена.
godot-executable-not-found = Исполняемый файл Godot не найден для версии {$version}.
error-link-exists = Путь {$path} уже существует. Используйте --force для перезаписи.
error-link-symlink = Не удалось создать ссылку из {$link} в {$target}: {$error}
error-link-copy = Не удалось скопировать исполняемый файл: {$error}

error-no-stable-releases-found = Стабильные версии не найдены.

error-starting-godot = Не удалось запустить Godot: { $error }

confirm-remove = Вы уверены, что хотите удалить эту версию? (да/нет):
confirm-yes = да
remove-cancelled = Удаление отменено.

default-set-success = Успешно установлено {$version} как версия Godot по умолчанию.
default-unset-success = Успешно удалено значение версии Godot по умолчанию.
provide-version-or-unset = Пожалуйста, укажите версию для установки по умолчанию или 'unset' для удаления версии по умолчанию.

error-open-zip = Не удалось открыть ZIP файл { $path }: { $error }
error-read-zip = Не удалось прочитать ZIP архив { $path }: { $error }
error-access-file = Не удалось получить доступ к файлу по индексу { $index }: { $error }
error-reopen-zip = Не удалось повторно открыть ZIP файл { $path }: { $error }
error-invalid-file-name = Недопустимое имя файла в ZIP архиве
error-create-dir = Не удалось создать каталог { $path }: { $error }
error-create-file = Не удалось создать файл { $path }: { $error }
error-read-zip-file = Не удалось прочитать из ZIP файла { $file }: { $error }
error-write-file = Не удалось записать в файл { $path }: { $error }
error-strip-prefix = Ошибка удаления префикса: { $error }
error-set-permissions = Не удалось установить разрешения для { $path }: { $error }
error-create-symlink-windows = Не удалось создать символьную ссылку. Убедитесь, что {"\u001b"}]8;;ms-settings:developers{"\u001b"}\режим разработчика{"\u001b"}]8;;{"\u001b"}\ включен или запустите от имени администратора.

help-upgrade = Обновить gdvm до последней версии
help-upgrade-major = Разрешить обновление через основные версии
help-upgrade-pre = Обновить до последнего предварительного выпуска
upgrade-starting = Начинается обновление gdvm...
upgrade-downloading-latest = Загрузка последней версии gdvm...
upgrade-complete = gdvm успешно обновлён!
upgrade-not-needed = gdvm уже на последней версии: { $version }.
upgrade-current-version-newer = Текущая версия gdvm ({ $current }) новее, чем последняя доступная версия ({ $latest }). Обновление не требуется.
upgrade-failed = Ошибка обновления: { $error }
upgrade-download-failed = Не удалось загрузить обновление: { $error }
upgrade-file-create-failed = Не удалось создать файл обновления: { $error }
upgrade-file-write-failed = Не удалось записать данные в файл обновления: { $error }
upgrade-install-dir-failed = Не удалось создать директорию установки: { $error }
upgrade-rename-failed = Не удалось переименовать текущий исполняемый файл: { $error }
upgrade-replace-failed = Не удалось заменить исполняемый файл на новый: { $error }
upgrade-no-binary = Нет доступного двоичного файла gdvm для версии { $version } и цели { $target }.
upgrade-checksum-required = Манифест выпуска не содержит контрольную сумму для этого двоичного файла gdvm. Обновление отклонено.
error-fetching-gdvm-releases = Ошибка получения релизов gdvm: { $error }
error-parsing-gdvm-releases = Ошибка разбора релизов gdvm: { $error }
error-unsupported-gdvm-schema = Неподдерживаемая версия схемы манифеста релизов gdvm: { $schema }. Попробуйте обновить gdvm вручную.
checking-updates = Проверка обновлений для gdvm...
upgrade-available = 💡 Доступна новая версия gdvm: {$version}. Запустите «gdvm upgrade», чтобы обновить.
upgrade-available-major = 💡 Доступно обновление основной версии gdvm: {$version}. Запустите «gdvm upgrade -m», чтобы обновить.
upgrade-available-both = 💡 Доступна новая версия gdvm: {$minor_version}. Также доступно обновление основной версии: {$major_version}. Запустите «gdvm upgrade» для обновления в рамках текущей основной версии или «gdvm upgrade -m» для обновления до последней версии.
upgrade-prerelease-available = 💡 Доступен новый предварительный выпуск gdvm. Выполните «gdvm upgrade --pre», чтобы установить его.

help-pin = Закрепить версию Godot в текущем каталоге.
help-pin-long = { help-pin }

    Это создаст файл gdvm.toml в текущем каталоге с закреплённой версией. Когда вы запустите "gdvm run" в этом каталоге или любом из его подкаталогов, будет использоваться закреплённая версия вместо версии по умолчанию.

    Это полезно, когда вы хотите использовать определённую версию Godot для проекта, не изменяя версию по умолчанию для всей системы.

    В настоящее время также записывается устаревший файл .gdvmrc для совместимости со старыми версиями gdvm. Он будет удалён в будущем выпуске, поэтому рекомендуется перейти на новый формат gdvm.toml и удалить файл .gdvmrc, если он существует.

    Вы можете отключить запись файла .gdvmrc с помощью флага --no-legacy.
help-pin-version = Укажите версию для закрепления
help-no-legacy = Не записывать устаревший файл совместимости .gdvmrc
pinned-success = Версия {$version} успешно закреплена в gdvm.toml
error-pin-version-not-found = Невозможно закрепить версию {$version}
pin-subcommand-description = Устанавливает или обновляет gdvm.toml с заданной версией

error-file-not-found = Файл не найден. Возможно, он не существует на сервере.
error-download-failed = Загрузка не удалась из-за непредвиденной ошибки: { $error }
error-ensure-godot-binaries-failed = Не удалось гарантировать исполняемые файлы Godot.
    Ошибка: { $error }.
    Попробуйте удалить { $path } и запустить gdvm снова.

error-failed-reading-project-godot = Не удалось прочитать project.godot, невозможно автоматически определить версию проекта.
warning-using-project-version = Используется версия { $version }, указанная в project.godot.

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
        [1] Если вы уверены, что хотите запустить проект с закреплённой версией, выполните {"\u001b"}[0mgdvm run --force{"\u001b"}[31m. В противном случае обновите закреплённую версию в .gdvmrc, чтобы она соответствовала версии проекта, или удалите файл .gdvmrc, чтобы использовать версию проекта.
        *[0] Если вы уверены, что хотите запустить проект с запрошенной версией, выполните {"\u001b"}[0mgdvm run --force <version>{"\u001b"}[31m.
    }{"\u001b"}[0m
warning-project-version-mismatch-force = {"\u001b"}[33mПропуск запроса подтверждения и продолжение с { $pinned ->
        [1] закреплённой
        *[0] запрошенной
    } версией {"\u001b"}[0m({ $requested_version }){"\u001b"}[33m.{"\u001b"}[0m

help-run-args = Дополнительные аргументы для передачи исполняемому файлу Godot (например, -- путь/к/project.godot).
help-run-force =
    Принудительный запуск проекта с запрошенной или закреплённой версией, даже если она не совпадает с версией проекта.
help-run-force-long =
    { help-run-force }

    Если вы сделаете это, запрошенная или закреплённая версия Godot может перезаписать файл проекта. Если вы закрепляете версии, рекомендуется вместо этого обновить закреплённую версию в .gdvmrc, чтобы она соответствовала версии проекта, или удалить файл .gdvmrc, чтобы использовать версию проекта.

help-config = Управление конфигурацией gdvm
help-config-get = Получить значение параметра конфигурации
help-config-set = Установить значение параметра конфигурации
help-config-unset = Удалить значение параметра конфигурации
help-config-list = Показать все параметры конфигурации
help-config-key = Ключ конфигурации (например, prune.max-age-days)
help-config-value = Значение для установки ключа конфигурации
help-config-unset-key = Ключ конфигурации для удаления (например, prune.max-age-days)
help-config-show-sensitive = Показать чувствительные параметры конфигурации в открытом виде
help-config-available = Показать все доступные ключи конфигурации и их значения, включая значения по умолчанию
warning-setting-sensitive = {"\u001b"}[33mПредупреждение: Вы устанавливаете чувствительное значение, которое будет сохранено в открытом виде в вашем домашнем каталоге.{"\u001b"}[0m
config-set-prompt = Пожалуйста, введите значение для { $key }:
error-reading-input = Ошибка чтения ввода
config-set-success = Конфигурация успешно обновлена.
config-unset-success = Ключ конфигурации { $key } успешно удалён.
config-key-not-set = Ключ конфигурации не установлен.
error-unknown-config-key = Неизвестный ключ конфигурации.
error-invalid-config-value = Недопустимое значение для ключа конфигурации { $key }.
error-invalid-config-subcommand = Недопустимая подкоманда config. Используйте "get", "set" или "list".
error-parse-config = Не удалось разобрать файл конфигурации: { $error }
error-parse-config-using-default = {"\u001b"}[33mИспользуются значения конфигурации по умолчанию.{"\u001b"}[0m

help-registry = Управление реестрами для установки сборок Godot
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
registry-error = Ошибка реестра: { $error }

error-invalid-registry-subcommand = Недопустимая подкоманда реестра. Используйте «add», «remove», «list» или «refresh».
registry-trust-warning = {"\u001b"}[33m{ $registry } ({ $url }) — это пользовательский реестр, а не официальный. gdvm проверяет, что загрузки соответствуют тому, что указывает реестр, но не может определить, безопасно ли их запускать. Устанавливайте из него, только если доверяете тому, кто им управляет.{"\u001b"}[0m
registry-trust-prompt = Доверяете ли вы этому реестру и хотите продолжить? (да/нет):
registry-trust-bypass = {"\u001b"}[1;31mПроверка доверия для { $registry } ({ $url }) пропущена, потому что вы использовали --yes. gdvm не может определить, безопасно ли запускать его файлы. Небольшая пауза; нажмите Ctrl+C сейчас, чтобы остановить.{"\u001b"}[0m
registry-trust-aborted = Прервано: реестр не является доверенным.
registry-project-override-conflict = {"\u001b"}[33mФайл gdvm.toml проекта переопределяет реестр { $registry } (ваша конфигурация: { $machine_url }) как { $project_url }. Определение проекта имеет приоритет.{"\u001b"}[0m

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
registry-build-downloading = Загрузка { $url } для вычисления размера и SHA-512…
registry-build-warn-local-hash = {"\u001b"}[33mХеширование локального файла в предположении, что он соответствует { $url }. gdvm не загружает URL для проверки.{"\u001b"}[0m
registry-build-warn-unverified = {"\u001b"}[33mИспользуются указанные вами SHA-512 и размер без загрузки артефакта для их проверки. Убедитесь, что они верны.{"\u001b"}[0m
registry-build-warn-explicit-store = {"\u001b"}[33mИспользуются указанные вами SHA-512 и/или размер вместо измерения сохранённого архива.{"\u001b"}[0m
registry-build-sha-mismatch = Указанный SHA-512 ({ $expected }) не соответствует артефакту ({ $actual }).
registry-build-size-mismatch = Указанный размер ({ $expected }) не соответствует артефакту ({ $actual }).
registry-validate-ok = Реестр корректен (проверено артефактов: { $count }).
registry-validate-failed = Проверка реестра не пройдена:
