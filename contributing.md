# Git Workflow Standards

Этот документ описывает стандарты работы с git в проекте Ambient Lighting.

## Именование веток

Используйте префиксы для типов задач:

- `feature/<short-topic>` — новая функциональность
- `fix/<short-topic>` — исправление багов
- `refactor/<short-topic>` — рефакторинг без изменения функциональности
- `chore/<short-topic>` — техническая работа (документация, настройка окружения и т.д.)
- `docs/<short-topic>` — чисто документация

**Примеры:**
- `feature/dma-frame-transfer`
- `fix/null-buffer-handling`
- `chore/contributors-readme`

## Принцип одной задачи

**Один PR/ветка = одна задача.**

Не смешивайте в одной ветке:
- Документацию + новую фичу
- Рефакторинг + исправление бага
- Несколько несвязанных задач

Это упрощает code review и откат изменений при необходимости.

## Формат коммитов

Используйте [Conventional Commits](https://www.conventionalcommits.org/) формат:

```
<type>(<scope>): <subject>

[optional body]
```

### Типы коммитов

- `feat` — новая функциональность
- `fix` — исправление бага
- `refactor` — рефакторинг кода
- `docs` — документация
- `chore` — техническая работа (сборка, зависимости)
- `test` — тесты
- `style` — форматирование, отступы (не влияет на логику)

### Scope (опционально)

Указывает, какой модуль затронут:
- `c-worker` — C-модуль захвата
- `core` — Rust-ядро
- `ui` — пользовательский интерфейс
- `hard-device` — аппаратная часть (ESP32)

### Примеры коммитов

```
feat(c-worker): add DMA pointer transfer API and module README
fix(c-worker): handle null frame buffer in on_process
refactor(core): separate capture and processing responsibilities
docs: add git workflow standards to contributors.md
chore: update PipeWire dependency to 0.3.65
```

## Workflow

### 1. Создание ветки

```bash
git checkout main
git pull origin main
git checkout -b feature/my-new-feature
```

### 2. Работа в ветке

Коммитьте свободно, но с правильными префиксами:

```bash
git add .
git commit -m "feat(c-worker): implement frame capture"
git commit -m "fix(c-worker): handle edge case"
git commit -m "docs(c-worker): update README"
```

### 3. Подготовка к merge

Перед вливанием в `main` приведите историю к чистому виду через squash:

```bash
git checkout main
git pull origin main
git checkout feature/my-new-feature
git rebase -i origin/main
# В редакторе: оставить первый 'pick', остальные -> 's' (squash)
```

Или используйте `git merge --squash` на этапе вливания.

### 4. Merge в main

```bash
git checkout main
git merge --squash feature/my-new-feature
git commit -m "feat(c-worker): implement frame capture with DMA transfer"
git push origin main
```

### 5. Cleanup

```bash
git branch -d feature/my-new-feature
git push origin --delete feature/my-new-feature
```

## Правила для main

- **main всегда в рабочем состоянии** — не пушьте напрямую WIP-коммиты
- **Линейная история** — используйте squash или rebase для чистоты
- **Один коммит = одна завершённая задача** — после merge в main должен быть понятный commit message

## Code Review (опционально)

Для командной работы:
1. Запушьте feature-ветку в origin
2. Откройте Pull Request
3. После ревью используйте "Squash and merge"
4. GitHub автоматически удалит ветку после merge (если настроено)

## Полезные alias

Добавьте в `~/.gitconfig`:

```ini
[alias]
    st = status -sb
    co = checkout
    br = branch -vv
    lg = log --oneline --decorate --graph -n 20
    sync = !git checkout main && git pull origin main
    newf = !sh -c 'git checkout -b feature/$1' -
    cleanup = !git branch --merged main | grep -v 'main' | xargs -r git branch -d
```

Использование:
```bash
git sync              # обновить main
git newf my-feature   # создать feature/my-feature
git cleanup           # удалить слитые ветки
```
