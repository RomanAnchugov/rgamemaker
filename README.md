# RGM — клон Game Maker 8.1 на Rust

Полноценный клон классического Game Maker 8.1: IDE, язык GML, игровой движок
и рантайм — всё на Rust. Проект организован как workspace из 6 крейтов.

```
cargo run -p demo-gen                    # собрать демо → demo.gmproj
cargo run -p gmruntime -- demo.gmproj    # запустить (F1 — комната-галерея)
cargo run -p gmide                       # открыть IDE
cargo test --workspace                   # 35 тестов (язык + движок)
```

## Крейты

| Крейт        | Что это |
|--------------|---------|
| `gml`        | Язык GML: лексер, парсер (AST), интерпретатор с семантикой GM8 |
| `gmformat`   | Формат проекта `.gmproj` (JSON + base64-ресурсы) |
| `gmengine`   | Игровой движок: комнаты, инстансы, события, коллизии, ~260 функций GML |
| `gmruntime`  | Рантайм на macroquad: окно, рендер, TTF-шрифты, звук, ввод |
| `gmide`      | IDE на egui: редакторы всех 9 типов ресурсов, кнопка Run |
| `demo-gen`   | Генератор демо-игры (спрайты и тайлсет рисуются кодом) |

## Язык GML (семантика GM8)

- Реализовано: `if/else`, `while`, `do..until`, `for`, `repeat`, `switch`
  **без fall-through**, `with` (включая `other`), `break`, `exit`, `var`,
  `globalvar`, `global.`, `obj.var` (чтение из первого / запись во все),
  `id.var`, `self/other/all/noone`, массивы 1D/2D с авто-ростом,
  `+= -= *= /=`, `++/--`, `? :`, `div/mod/and/or/xor/not`, hex-литералы `$FF`,
  оба вида кавычек, комментарии `//` и `/* */`, перенос строк после оператора,
  `execute_string`/`execute_file`. Идентификаторы регистронезависимы.
- Скрипты возвращают значение последнего выражения (как в GM8), аргументы:
  `argument0..15`, `argument[]`, `argument_count`.
- Нет короткого замыкания `&&/||` (как в GM8), деление на 0 — ошибка,
  числа форматируются максимум с 2 знаками (`string(1/3) == "0.33"`).
- Защита от бесконечных циклов (лимит операций на событие).

## Движок

- **События**: Create, Destroy, Alarm 0–11, Step (begin/normal/end),
  Collision (с наследованием объектов и семантикой `solid`), Keyboard /
  Key Press / Key Release (vk-коды Windows), Mouse (локальные/глобальные,
  колесо, enter/leave, no button), Draw, Other (animation end, outside room,
  boundary, game/room start/end, no more lives/health), `event_perform`.
- **Порядок шага** как в GM8: begin step → alarm → keyboard → мышь → step →
  **таймлайны** → движение (friction/gravity/пути/анимация) → коллизии →
  **частицы** → end step → outside/boundary → уборка → смена комнаты.
- **Переменные**: `x, y, hspeed, vspeed, speed, direction, friction, gravity,
  sprite_index, image_*, sprite_width/height/xoffset/yoffset, alarm[]`,
  путь (`path_index, path_position, path_speed, path_endaction, path_absolute`),
  таймлайн (`timeline_index/running/speed/position/loop`), глобальные
  `score, lives, health, mouse_x/y, room*, fps, keyboard_string,
  keyboard_lastchar, view_*` и т.д.
- **Функции** (~260): математика, строки (включая `string_format`), цвета,
  даты (`date_*`), инстансы (`instance_*`, `place_*`, `move_*`, `collision_*`,
  `mp_linear_step`, `mp_potential_step`), комнаты, рисование (`draw_sprite`,
  `_ext`, `_part`, `_stretched`, `draw_text`, `_ext`, `_transformed`,
  `_color`, `draw_set_font/halign/valign`, `draw_healthbar`, `draw_arrow`,
  `draw_triangle`, `draw_roundrect`), звук (`sound_*`), структуры данных
  (`ds_list/map/stack`), `event_perform`, random с seed.
- **Коллизии**: bbox + попиксельные маски (precise), наследование объектов,
  solid-семантика GM8, deactivate/activate.
- **Комнаты**: фоновые слои (tiled), цвет фона, creation code,
  persistent-инстансы, **тайлы** (рисуются по depth вместе с инстансами,
  `tile_add`/`tile_get_*`/`tile_layer_*`), **8 видов** со слежением,
  границами и скоростью (`view_visible[0..7]`, `view_xview[0..7]`, …),
  `room_speed` (фиксированный шаг), `room_goto*`, `game_restart/end`.
- **Пути**: ресурс с точками и скоростями сегментов, `path_start`/`path_end`,
  endaction (стоп/цикл/реверс), absolute/relative, `path_get_*`, `path_draw`.
- **Таймлайны**: ресурс «момент → код», переменные инстанса
  `timeline_*`, моменты выполняются по `timeline_position` с учётом
  `timeline_speed` и `timeline_loop`.
- **Частицы**: системы, типы (14 форм `pt_shape_*` или спрайт, размер,
  масштаб, цвет (RGB/HSV/микс), альфа-кривая, скорость, направление,
  гравитация, жизнь, ориентация, blend, step/death-спавн), эмиттеры
  (прямоугольник/эллипс/ромб/линия, 3 распределения, stream/burst),
  `part_*` полный набор, глубина системы, авто-обновление/отрисовка.
- **Шрифты**: ресурсы (TTF или встроенный), `draw_set_font`,
  `string_width` учитывает шрифт, кастомный размер.
- **Сохранения**: `game_save(file)` / `game_load(file)` — полное состояние
  (комната, инстансы с переменными, счёт/жизни/здоровье, глобальные).
- Идентификаторы инстансов с 100001, `vk_*`-коды совпадают с GM8.

## IDE

Стиль Windows XP / Delphi / Game Maker 8: серые панели, острые углы,
системный Tahoma для интерфейса и Courier New для кода (на Windows/Linux
берутся аналогичные системные шрифты), пиксельные иконки тулбара и дерева
ресурсов (ничего не отображается «квадратиками»), меню File/Edit/Resources/Run,
горячие клавиши **F5** (запуск), **Ctrl+S** (сохранение), **Ctrl+O** (открытие),
**F7** (проверка кода), цветной вывод консоли, статус-бар.

Редактор кода: белым по чёрному в стиле GM8 — ключевые слова синим, вызовы
функций тёмно-синим, константы (`c_*`, `vk_*`, `pt_*`…) фиолетовым, строки и
числа красным/оранжевым, комментарии зелёным. **Автодополнение**: начинает
показывать варианты при наборе (от 3 символов) или по **Ctrl+Space**; **Tab**
вставляет выбранное, ↑/↓ листают; в списке — функции, ключевые слова,
константы, переменные инстанса и имена ресурсов проекта с сигнатурами
популярных функций.

Дерево ресурсов и редактор комнат показывают **реальные спрайты**: в списке —
превью кадра 0 (у объектов — их спрайт, без спрайта — цветная метка), в комнате —
спрайты инстансов с учётом origin, один в один с тем, как их отрисует игра.

Редакторы всех типов ресурсов GM8: **спрайты** (пиксельный холст:
карандаш/ластик/заливка/линия/прямоугольник/пипетка, кадры, origin, bbox,
precise, импорт PNG), **звуки** (wav/ogg + прослушивание), **фоны**,
**пути** (точки на сетке, ЛКМ добавить/ПКМ удалить, скорости точек,
closed), **скрипты**, **таймлайны** (моменты с шагом и кодом),
**шрифты** (TTF + размер), **объекты** (спрайт/solid/persistent/depth/маска/
родитель, все события, редактор кода с подсветкой GML), **комнаты**
(параметры, 8 видов, фоновые слои, расстановка инстансов и **тайлов** из
тайлсета, creation code). Кнопки **Check** (компиляция всего кода) и
**▶ Run** (сохраняет проект и запускает `gmruntime`).

## Экспорт игры

**Из IDE**: `File → Export game…` — задаёте имя и папку, IDE в фоне собирает
релизный бинарник с **проектом, вшитым внутрь** (`RGM_EMBED`), и кладёт рядом
`README.txt`. Результат — один самодостаточный файл (`mygame` / `mygame.exe`),
который запускается без установки и без внешних файлов; прогресс сборки виден
в панели Output.

**Вручную** (на целевой ОС):
```bash
RGM_EMBED=mygame.gmproj cargo build --release -p gmruntime
# → target/release/gmruntime — автономная игра
```
Рантайм по-прежнему принимает и путь к проекту аргументом.

**Кросс-платформенная сборка** — workflow `.github/workflows/builds.yml`:
GitHub Actions собирает под Windows / macOS / Linux артефакты `gmruntime-*`
(универсальный ран) и `rgm-demo-*` (демо-игра со встроенным проектом); по тегу
`v*` публикует Release. Стек полностью кроссплатформенный (macroquad/miniquad),
платформозависимого кода в движке нет.

## Демо-игра

Комната 1 — «Арканоид» на GML (коллизии, отскоки, счёт, жизни) с
**частицами** при разрушении кирпичей. Комната 2 (клавиша **F1**) — «галерея
возможностей»: объект едет по **пути**, **таймлайн** выводит сообщения,
пол и потолок выложены **тайлами**. Управление: ←/→ или мышь, `Space` —
запуск мяча, `F2` — рестарт.


Формат: `- [ ]` — не сделано, `- [x]` — сделано. Действия сгруппированы по вкладкам
библиотек GM8. Номера lib/id сверяем со спекой GMK (LateralGM) при имплементации —
они должны совпадать 1:1 для будущего импорта .gmk.

## A. События (диалог Add Event)

- [x] Create
- [x] Destroy
- [x] Alarm 0–11 (12 шт.)
- [x] Step: Step / Begin step / End step (3)
- [x] Collision with <каждый объект>
- [x] Keyboard (held): <no key>, <any key>, A–Z (26), 0–9 (10), спец-клавиши (см. ниже)
- [x] Key press: тот же набор клавиш
- [x] Key release: тот же набор клавиш
- [x] Mouse: no button, left/right/middle button, mouse enter, mouse leave, wheel up, wheel down
- [x] Mouse pressed: left/right/middle (над объектом)
- [x] Mouse released: left/right/middle (над объектом)
- [x] Global mouse: left/right/middle button (held)
- [x] Global mouse pressed: left/right/middle
- [x] Global mouse released: left/right/middle
- [x] Other: outside room, intersect boundary, game start, game end, room start, room end, no more lives, no more health, animation end
- [x] Other: User defined 0–15 (16 шт.)
- [x] Draw
- [ ] (8.1, опционально) Trigger events

Спец-клавиши: Enter, Escape, Space, Backspace, Tab, Shift, Control, Alt,
Left/Right/Up/Down, PageUp, PageDown, End, Home, Delete, Insert, F1–F12,
Numpad 0–9, Multiply, Add, Subtract, Divide, Decimal.

## B. Действия по вкладкам

### Move (lib 1)
- [x] Start moving in a direction
- [x] Set direction and speed of motion
- [x] Move towards a point
- [x] Speed horizontal
- [x] Speed vertical
- [x] Set gravity
- [x] Reverse horizontal direction
- [x] Reverse vertical direction
- [x] Set friction

### main1 (lib 2)
- [x] Create instance
- [x] Create moving
- [x] Create random
- [x] Change instance
- [x] Destroy instance
- [x] Destroy at position
- [x] Change sprite
- [x] Transform sprite
- [x] Set sprite (collision mask)
- [x] Play a sound
- [x] Stop a sound
- [x] If a sound is playing (question)
- [x] Go to next room
- [x] Go to previous room
- [x] Go to a different room
- [x] Restart the current room
- [x] Sleep some milliseconds
- [x] Set the mouse cursor

### main2 (lib 3)
- [x] Set the value of an alarm clock
- [x] Use a path
- [x] Position in path
- [x] Path position
- [x] Path speed
- [x] Bounce against objects
- [ ] (сверить со спекой) прочие main2: timeline-позиция/скорость и пр.
- [ ] Отдельная пиксельная иконка на каждое действие (сейчас — цветная плитка библиотеки + маркер)

### control (lib 4)
- [x] Execute a piece of code
- [x] Comment
- [x] Set the value of a variable (relative)
- [x] If a variable has a value (question)
- [x] If an expression is true (question)
- [x] Check empty (question)
- [x] Check collision (question)
- [x] Check object (question)
- [x] Test instance number (question)
- [x] Test chance (question)
- [x] Check question (question, да/нет)
- [x] Test expression (question)
- [x] Check mouse (question)
- [x] Check grid / aligned to grid (question)
- [x] Start of block
- [x] End of block
- [x] Else
- [x] Exit this event
- [x] Repeat next action (n times)
- [x] While / Until маркеры (в формате gmk есть; в UI GM8 отсутствуют — опционально)

### score (lib 5)
- [x] Set the score (relative)
- [x] If score is equal to (question)
- [x] Draw the value of score
- [x] Set the number of lives (relative)
- [x] If lives is equal to (question)
- [x] Draw the number of lives
- [x] Set the health (relative)
- [x] If health less than (question)
- [x] Draw the health bar
- [x] Clear the highscore table
- [x] If the score is among the best (question)

### extra (lib 6)
- [x] Open a web page
- [x] Show the game information
- [x] Show a video
- [x] Play/pause/resume/stop CD (4 действия)
- [x] Replace sprite from file
- [x] Replace background from file
- [x] (сверить со спекой) прочие extra

### draw (lib 7)
- [x] Draw a sprite image
- [x] Draw a background image
- [x] Draw a text
- [x] Draw a transformed text
- [x] Draw a rectangle
- [x] Draw a horizontal gradient
- [x] Draw a vertical gradient
- [x] Set the color (used for drawing)
- [x] Set the font
- [x] Draw the value of a variable

### particles (lib 8)
- [x] Create/destroy particle system
- [ ] Clear particle system (действием)
- [x] Create particle type (+ настройка)
- [ ] Destroy particle type
- [ ] Create/destroy emitter
- [ ] Set emitter region
- [ ] Emitter stream / burst

## C. Инфраструктура

- [x] Модель `DndAction` + `EventDef::Actions(...)` в gmformat (+ serde)
- [x] Таблица `ACTION_DEFS`/`LIB_DEFS` — 75 действий (id RGM-внутренние; таблица соответствия с id GM8 — на этапе импорта .gmk)
- [x] Компилятор Actions → GML (блоки/else/вопросы/applies_to/relative/not) + unit-тесты
- [x] UI: 3 колонки (события с иконками типов | действия | библиотеки-вкладки + сетка)
- [x] Drag-and-drop: сетка → список (вставка по позиции), reorder внутри списка
- [x] Диалоги параметров (генерируемые поля; ресурсы — списки; выражения с подсветкой)
- [x] Контекстное меню действия: параметры / вырезать / копировать / вставить / удалить
- [x] Двойной клик = параметры; подпись действия = текст из текущих параметров
- [ ] Иконки событий по типам в списке событий
- [ ] Демо: события obj_paddle на D&D
- [ ] Импорт .gmk (финальный этап): lib/id/params маппятся 1:1

## D. Порядок работ
1. Модель + таблица определений + иконки
2. Компилятор Actions → GML + тесты
3. UI (3 колонки, DnD, диалоги)
4. Интеграция движка + демо
5. Импорт .gmk

## Отличия от оригинала / TODO

- Свой формат `.gmproj` (JSON) вместо бинарного `.gm8/.gmk` — импорт старых
  проектов не поддерживается.
- Рендерится первый видимый view в окно (данные и логика — все 8);
  D&D-действия (только код), 3D, attractors/destroyers/deflectors/changers
  частиц, `mp_potential_settings`.
- `show_message` пишет в консоль рантайма (без модальных окон).
