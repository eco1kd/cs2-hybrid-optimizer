<div align="center">

# CS2 Optimizer (Rust Edition)

### Высокопроизводительный нативный оптимизатор Counter-Strike 2

![GitHub release](https://img.shields.io/github/v/release/eco1kd/cs2-hybrid-optimizer?style=for-the-badge)
![Platform](https://img.shields.io/badge/Платформа-Windows%2010%20%7C%2011-blue?style=for-the-badge)
![Language](https://img.shields.io/badge/Язык-Rust%20🦀-orange?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)
![CS2](https://img.shields.io/badge/Counter--Strike%202-Оптимизировано-orange?style=for-the-badge)

### Устранение микрофризов - Стабильный фреймтайм - Идеальный Sub-Tick - Автономный файл ~1 МБ

| Выберите язык |
|:---:|
| **Русский** (текущий) &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; [**English**](./README.md) |

</div>

---

## О проекте

**CS2 Optimizer** — нативная, легковесная и полностью автономная Windows-утилита, переписанная с нуля на языке **Rust**.

Программа создана для устранения микрофризов, стабилизации времени кадра (0.1% и 1% Low FPS), минимизации сетевого инпут-лага и глубокой подготовки операционной системы к запуску **Counter-Strike 2**.

Благодаря переходу на Rust проект больше **не требует**:
- Установленного Python
- Сторонних DLL библиотек рядом с файлом
- Долгой распаковки через PyInstaller

Вы получаете **один чистый `.exe` файл размером ~1 МБ**, который стартует за 5 миллисекунд и не вызывает ложных срабатываний антивирусов.

---

## Архитектура

| Компонент | Назначение | Технологии |
| --- | --- | --- |
| Ядро программы | Безопасная работа с памятью и максимальная производительность | `Rust 2021`, `windows-sys`, `winreg`, `ntdll`, `kernel32` |
| Слой ядра NT | Сброс Standby List и блокировка высокоточного таймера ~0.5 мс | Нативный Windows NT API (`NtSetSystemInformation`, `NtSetTimerResolution`) |
| Сетевой уровень | Отключение алгоритма Нагла (`TcpAckFrequency = 1`, `TCPNoDelay = 1`) и сетевого троттлинга MMCSS | Реестр Windows (`Tcpip\Parameters\Interfaces`, `SystemProfile`) |
| Игровая интеграция | Поиск CS2 по всем дискам, Sub-Tick autoexec, исключения Defender | Парсинг Steam VDF, `steam.exe -applaunch 730`, Win32 Toolhelp32 |

---

## Возможности

### Кэш и память
- **Сброс GPU-кэша**: Очистка шейдеров D3DSCache, NVIDIA DXCache/GLCache/ComputeCache, AMD DxCache/DxcCache/OglCache/VkCache и Intel ShaderCache.
- **Очистка Steam Shader Cache**: Удаление устаревших данных кэша AppID 730 из `steamapps/shadercache/730`.
- **Сброс RAM Standby List**: Вызов системного вызова NT с правами `SeProfileSingleProcessPrivilege` для освобождения памяти в standby-листе.
- **Высокоточный системный таймер**: Запрос и удержание максимальной точности таймера ~0.5мс (вместо 15.6мс по умолчанию) на протяжении всей игровой сессии.

### Система и сеть
- **Оптимизация задержки TCP**: Отключение алгоритма Нагла на всех сетевых адаптерах для мгновенной отправки пакетов в Sub-Tick.
- **Профиль MMCSS**: Установка `SystemResponsiveness = 0` (100% CPU для игры) и `NetworkThrottlingIndex = 0xffffffff` (снятие сетевых ограничений).
- **Win32PrioritySeparation**: Настройка значения `0x26` для максимального приоритета активного окна.
- **Схема питания**: Блокировка режима "Высокая производительность" для исключения троттлинга процессора.
- **Fullscreen Optimizations**: Отключение DWM-оптимизации полного экрана для снижения инпут-лага.
- **Сброс DNS**: Очистка DNS-кэша перед поиском матча.

### Запуск CS2 и фоновый бустер
- **Поиск на всех дисках**: Автоматическое обнаружение игры на любом SSD (D:, E: и т.д.) через реестр и `libraryfolders.vdf`.
- **Sub-Tick `autoexec.cfg`**: Создание актуального конфига (`engine_low_latency_sleep_after_client_tick true`, `cl_net_buffer_ticks 0`, рейты) с **автоматическим бэкапом** старого файла.
- **Запуск через Steam AppLaunch**: Запуск через команду `steam.exe -applaunch 730` с гарантированной передачей параметров в движок.
- **Нативный C/Rust процесс-бустер**: Фоновый поток отслеживает запуск `cs2.exe` и сразу выставляет ему класс приоритета `HIGH_PRIORITY_CLASS`.

---

## Сборка программы

### Требования
* Установленный [Rust](https://rustup.rs/) (cargo и rustc)

### Сборка в один клик (Windows)
Запустите `build.bat` или выполните команду:
```cmd
cargo build --release
```
Готовый оптимизированный исполняемый файл появится в:
```text
target/release/cs2-optimizer.exe
```

---

## Инструкция по использованию

1. Скачайте `cs2-optimizer.exe` из раздела [Releases](../../releases).
2. Запустите от имени администратора.
3. Выберите язык (English / Русский).
4. Ознакомьтесь с отчётом очистки и нажмите `Y` или `Enter` для запуска Counter-Strike 2.
5. Не закрывайте окно консоли во время матча, чтобы поддерживать активность высокоточного таймера ~0.5мс.

---

## Безопасность

Проект:
- **Не внедряется** в память процесса `cs2.exe`
- **Не изменяет** исполняемые файлы игры
- **Не перехватывает** функции VAC
- Полностью безопасен и совместим с Valve Anti-Cheat.

---

## Лицензия

MIT License
