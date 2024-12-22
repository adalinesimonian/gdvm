hello = Привет, мир!

help-about = Менеджер версий Godot
help-help = Показать справку (см. краткое описание с '-h')
help-help-command = Показать это сообщение или справку по указанным подкомандам
help-gdvm-version = Показать версию менеджера версий Godot

help-install = Установить новую версию Godot
help-run = Запустить определенную версию Godot
help-list = Список всех установленных версий Godot
help-remove = Удалить установленную версию Godot

help-branch = Ветка (stable, beta, alpha или custom).
help-csharp = Использовать версию Godot с поддержкой C#.
help-version = Версия для установки (например, 4), или "stable" для последней стабильной версии.
help-version-long =
    Версия для установки (например, 4), или "stable" для последней стабильной версии.

    Примеры: 4.4 установит последнюю стабильную версию Godot 4.4. Если существуют только
    предварительные версии, в этом случае будет установлена последняя предварительная версия.
    4.3-rc установит последний релиз-кандидат Godot 4.3 и т.д.

help-search = Список удаленных релизов из godot-builds
help-filter = Необязательная строка для фильтрации тегов релизов
help-include-pre = Включить предварительные версии (rc, beta, dev)
help-cache-only = Использовать только кэшированную информацию о релизах без запроса к GitHub API
help-limit = Количество релизов для отображения, по умолчанию 10. Используйте 0, чтобы отобразить все
help-clear-cache = Очистить кэш релизов gdvm
help-version-installed = Установленная версия (например, 4.2 или 4.2-stable).

help-force = Принудительная переустановка, даже если версия уже установлена.
help-redownload = Повторно загрузить версию, даже если она уже загружена в кэше.

using-cached-zip = Используется кэшированный архив релиза, пропускается загрузка.
cached-zip-stored = Архив релиза Godot сохранен в кэше.

help-default = Управление версией по умолчанию
help-default-version = Версия для установки по умолчанию (например, 4.2 или 4.2-stable).
no-default-set = Версия по умолчанию не установлена.

installing-version = Установка версии {$version}
installed-success = Версия {$version} успешно установлена

warning-prerelease = Внимание: Вы устанавливаете предварительную версию ({$branch}).

no-versions-installed = Версии не установлены.
installed-versions = Установленные версии Godot:
removed-version = Версия {$version} удалена

force-reinstalling-version = Принудительная переустановка версии {$version}.

removing-version = Удаление версии {$version}

error-version-not-found = Версия не найдена.
error-multiple-versions-found = Найдено несколько версий, соответствующих запросу:

error-invalid-godot-version = Неверный формат версии Godot. Ожидаемые форматы: x, x.y, x.y.z, x.y.z.w или x.y.z-тег.
error-invalid-remote-version = Неверный формат удаленной версии Godot. Ожидаемые форматы: x, x.y, x.y.z, x.y.z.w, x.y.z-тег или "stable".

running-version = Запуск версии {$version}
no-matching-releases = Соответствующие релизы не найдены.
available-releases = Доступные релизы:
cache-cleared = Кэш успешно очищен.

version-already-installed = Версия уже установлена.
version-not-found = Версия не найдена.
godot-executable-not-found = Исполняемый файл Godot не найден для версии {$version}.
no-cache-file-found = Файл кэша не найден.
warning-cache-metadata-reset = Индекс кэша релизов недействителен или поврежден. Сброс.
cache-files-removed = Файлы кэша успешно удалены.
cache-metadata-removed = Метаданные кэша успешно удалены.
error-cache-metadata-empty = Ошибка: Метаданные кэша пусты, необходимо сначала получить релизы.
no-cache-files-found = Файлы кэша не найдены.
no-cache-metadata-found = Метаданные кэша не найдены.

help-yes = Пропустить подтверждение удаления

confirm-remove = Вы уверены, что хотите удалить эту версию? (да/нет):
confirm-yes = да
remove-cancelled = Удаление отменено.

help-console = Запустить Godot с консольным окном. По умолчанию false в Windows, true на других платформах.

default-set-success = Успешно установлено {$version} как версия Godot по умолчанию.
default-unset-success = Успешно удалено значение версии Godot по умолчанию.
provide-version-or-unset = Пожалуйста, укажите версию для установки по умолчанию или 'unset' для удаления версии по умолчанию.

error-no-stable-releases-found = Стабильные версии не найдены.

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

error-find-user-dirs = Не удалось найти пользовательские каталоги.

fetching-releases = Получение релизов...
releases-fetched = Релизы получены.

error-starting-godot = Не удалось запустить Godot: { $error }

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
