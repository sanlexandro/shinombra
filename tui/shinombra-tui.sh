#!/usr/bin/env bash
#
# shinombra-tui
#

set -euo pipefail

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/shinombra"
MANIFEST="$CONFIG_DIR/manifest.toml"

# === Цвета ===
C_PRIMARY=212
C_ACCENT=99
C_OK=46
C_WARN=214
C_ERR=196
C_DIM=245

# -------------------------------------------------------------
# Утилиты вывода
# -------------------------------------------------------------

info()  { gum style --foreground "$C_ACCENT" "$@"; }
ok()    { gum style --foreground "$C_OK" --bold "✔ $*"; }
warn()  { gum style --foreground "$C_WARN" "⚠ $*"; }
err()   { gum style --foreground "$C_ERR" --bold "✘ $*" >&2; }

# Печатает сообщение об ошибке и завершает скрипт с кодом 1.
die() {
  err "$*"
  exit 1
}

# -------------------------------------------------------------
# Проверка зависимостей
# -------------------------------------------------------------

check_dependencies() {
  local missing=()

  if ! command -v gum >/dev/null 2>&1; then
    missing+=("gum (https://github.com/charmbracelet/gum)")
  fi
  if ! command -v yq >/dev/null 2>&1; then
    missing+=("yq - the Go version by mikefarah (https://github.com/mikefarah/yq), not the Python jq-wrapper")
  fi

  if [ ${#missing[@]} -gt 0 ]; then
    echo "Error: missing required dependencies:" >&2
    for dep in "${missing[@]}"; do
      echo "  - $dep" >&2
    done
    exit 1
  fi
}

# -------------------------------------------------------------
# Манифест: первый запуск / чтение
# -------------------------------------------------------------

# Создаёт манифест с полным набором секций-заглушек, чтобы
# последующие yq-запросы (.paths.*, .daemon_settings.*, .settings.*)
# никогда не падали на отсутствующем ключе.
create_default_manifest() {
  mkdir -p "$CONFIG_DIR"
  cat > "$MANIFEST" <<'EOF'
[paths]
config = ""
overlays = []

[daemon_settings]
log_level = "Info"
pipewire_conversion = false
save_token = true

[settings]
EOF
}

ensure_manifest_exists() {
  if [[ -f "$MANIFEST" ]]; then
    return
  fi

  warn "Manifest not found: $MANIFEST"
  if gum confirm "Create a new manifest.toml now?"; then
    create_default_manifest
    ok "Created $MANIFEST"
  else
    die "Cannot continue without a manifest."
  fi
}

# Проверяет, что манифест - валидный TOML, читаемый yq.
# Без этого любая последующая yq-команда просто тихо упадёт с невнятной ошибкой.
validate_manifest() {
  if ! yq -o toml '.' "$MANIFEST" >/dev/null 2>/tmp/shinombra_yq_err; then
    err "Manifest is not valid TOML: $MANIFEST"
    echo "--- yq error ---" >&2
    cat /tmp/shinombra_yq_err >&2
    rm -f /tmp/shinombra_yq_err
    die "Fix the file manually or delete it to regenerate a default one."
  fi
  rm -f /tmp/shinombra_yq_err
}

# Читает .paths.config (пустая строка, если не задано)
read_current_config() {
  yq -r '.paths.config // ""' "$MANIFEST"
}

# Читает .paths.overlays как bash-массив в переменную по ссылке.
# Пустой/отсутствующий массив в TOML корректно даёт пустой bash-массив
# (а не массив из одной пустой строки).
read_current_overlays() {
  local -n out_ref=$1
  out_ref=()
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && out_ref+=("$line")
  done < <(yq -r '(.paths.overlays // [])[]' "$MANIFEST" 2>/dev/null || true)
}

# -------------------------------------------------------------
# Форматирование списков для вывода
# -------------------------------------------------------------

# Печатает элементы массива через ", " или "<none>", если массив пуст.
join_or_none() {
  local -n arr_ref=$1
  if [ ${#arr_ref[@]} -eq 0 ]; then
    echo "<none>"
    return
  fi
  local out="" item
  for item in "${arr_ref[@]}"; do
    if [[ -z "$out" ]]; then
      out="$item"
    else
      out="$out, $item"
    fi
  done
  echo "$out"
}

# -------------------------------------------------------------
# Отображение состояния
# -------------------------------------------------------------

show_header() {
  clear
  gum style \
    --foreground "$C_PRIMARY" --border-foreground "$C_PRIMARY" --border double \
    --align center --width 40 --margin "1 2" --padding "1 2" \
    "◆ Shinombra TUI ◆" "config manager"
}

# $1 = заголовок блока, $2 = значение config, имя ссылки на массив overlays = $3
show_state_block() {
  local title="$1" config_val="$2"
  local overlays_str
  overlays_str=$(join_or_none "$3")

  info --bold "$title"
  gum style --foreground "$C_DIM" --border rounded --border-foreground "$C_DIM" \
    --padding "0 1" --margin "0 0 1 0" \
    "Config    : ${config_val:-<not set>}" \
    "Overlays  : $overlays_str"
}

# -------------------------------------------------------------
# Файловые списки
# -------------------------------------------------------------

list_files() {
  local dir="$1"
  find "$dir" -maxdepth 1 -type f -printf "%f\n" 2>/dev/null | sort
}

# Приводит путь из манифеста (относительный к CONFIG_DIR, "~/...", или уже
# абсолютный) к единому абсолютному пути. Используется везде, где нужно
# сравнить "указывает ли путь X на тот же файл, что и файл Y" - так что
# неважно, в каком виде путь записан в манифесте.
# realpath -m нормализует ".." и повторяющиеся "/", не требуя, чтобы файл
# реально существовал (-m = "missing okay").
resolve_path() {
  local p="$1" raw
  case "$p" in
    /*)
      raw="$p"
      ;;
    "~"|"~/"*)
      raw="${HOME}${p#\~}"
      ;;
    *)
      raw="$CONFIG_DIR/$p"
      ;;
  esac
  realpath -m -- "$raw"
}

# Проверяет, что каждый путь вида "configs/x.toml" / "overlays/y.toml"
# реально существует относительно CONFIG_DIR. Печатает предупреждение и
# возвращает не-ноль, если что-то отсутствует, но не прерывает скрипт -
# решение остаётся за вызывающим кодом.
validate_paths_exist() {
  local -n paths_ref=$1
  local missing=()
  local p resolved
  for p in "${paths_ref[@]:-}"; do
    [[ -n "$p" ]] || continue
    resolved=$(resolve_path "$p")
    if [[ ! -f "$resolved" ]]; then
      missing+=("$resolved")
    fi
  done
  if [ ${#missing[@]} -gt 0 ]; then
    warn "These files no longer exist on disk:"
    local m
    for m in "${missing[@]}"; do
      echo "    - $m"
    done
    return 1
  fi
  return 0
}

# -------------------------------------------------------------
# Выбор конфига
# -------------------------------------------------------------

# Устанавливает глобальную FINAL_CONFIG.
select_config() {
  local current_config="$1"
  local -n files_ref=$2

  if [ ${#files_ref[@]} -eq 0 ]; then
    warn "No config files found in $CONFIG_DIR/configs - keeping current config."
    FINAL_CONFIG="$current_config"
    return
  fi

  # Находим, какой пункт списка (basename) соответствует текущему конфигу,
  # сравнивая по абсолютному пути - так что неважно, записан ли текущий
  # путь в манифесте как "configs/x.toml", "~/.../x.toml" или "/abs/.../x.toml".
  local current_resolved
  current_resolved=$(resolve_path "$current_config")
  local current_basename=""
  local f
  for f in "${files_ref[@]}"; do
    if [[ "$(resolve_path "configs/$f")" == "$current_resolved" ]]; then
      current_basename="$f"
      break
    fi
  done

  local selected
  selected=$(gum choose \
    --header="◇ Select main config" --cursor="▶ " \
    --selected="$current_basename" \
    "${files_ref[@]}")

  if [[ -z "$selected" ]]; then
    # Esc / пустой выбор - не теряем текущее значение
    FINAL_CONFIG="$current_config"
  else
    FINAL_CONFIG="configs/$selected"
  fi
}

# -------------------------------------------------------------
# Выбор оверлеев
# -------------------------------------------------------------

# Устанавливает глобальный массив FINAL_OVERLAYS.
select_overlays() {
  local -n current_ref=$1
  local -n files_ref=$2

  FINAL_OVERLAYS=()

  if [ ${#files_ref[@]} -eq 0 ]; then
    return
  fi

  # Резолвим текущие оверлеи в абсолютные пути один раз, чтобы дальше
  # сравнивать с каждым пунктом списка независимо от того, как путь
  # записан в манифесте (relative/tilde/absolute).
  local current_resolved=()
  local o
  for o in "${current_ref[@]}"; do
    current_resolved+=("$(resolve_path "$o")")
  done

  local current_names=()
  local f r
  for f in "${files_ref[@]}"; do
    r=$(resolve_path "overlays/$f")
    for cr in "${current_resolved[@]}"; do
      if [[ "$r" == "$cr" ]]; then
        current_names+=("$f")
        break
      fi
    done
  done

  local selected_joined
  selected_joined=$(IFS=,; echo "${current_names[*]}")

  local selected_lines=()
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && selected_lines+=("$line")
  done < <(gum choose --no-limit \
    --header="◇ Select active overlays (space to toggle, enter to confirm)" --cursor="▶ " \
    --selected="$selected_joined" \
    --selected-prefix="✔ " --unselected-prefix="  " \
    "${files_ref[@]}")

  for line in "${selected_lines[@]:-}"; do
    [[ -n "$line" ]] || continue
    FINAL_OVERLAYS+=("overlays/$line")
  done
}

# -------------------------------------------------------------
# Сохранение
# -------------------------------------------------------------

save_manifest() {
  local config="$1"
  local -n overlays_ref=$2

  local backup="${MANIFEST}.bak"
  if ! cp -- "$MANIFEST" "$backup"; then
    err "Could not create backup before saving - aborting to be safe."
    return 1
  fi

  export FINAL_CONFIG="$config"
  if ! yq -o toml -i '.paths.config = strenv(FINAL_CONFIG)' "$MANIFEST" 2>/tmp/shinombra_yq_err; then
    unset FINAL_CONFIG
    err "Failed to write config path to manifest."
    cat /tmp/shinombra_yq_err >&2
    rm -f /tmp/shinombra_yq_err
    cp -- "$backup" "$MANIFEST"
    err "Restored the manifest from backup - no changes were kept."
    return 1
  fi
  unset FINAL_CONFIG

  if ! yq -o toml -i '.paths.overlays = []' "$MANIFEST" 2>/tmp/shinombra_yq_err; then
    err "Failed to reset overlays list in manifest."
    cat /tmp/shinombra_yq_err >&2
    rm -f /tmp/shinombra_yq_err
    cp -- "$backup" "$MANIFEST"
    err "Restored the manifest from backup - no changes were kept."
    return 1
  fi

  local overlay
  for overlay in "${overlays_ref[@]:-}"; do
    [[ -n "$overlay" ]] || continue
    export OVERLAY="$overlay"
    if ! yq -o toml -i '.paths.overlays += [strenv(OVERLAY)]' "$MANIFEST" 2>/tmp/shinombra_yq_err; then
      unset OVERLAY
      err "Failed to add overlay '$overlay' to manifest."
      cat /tmp/shinombra_yq_err >&2
      rm -f /tmp/shinombra_yq_err
      cp -- "$backup" "$MANIFEST"
      err "Restored the manifest from backup - no changes were kept."
      return 1
    fi
    unset OVERLAY
  done

  rm -f /tmp/shinombra_yq_err
  rm -f -- "$backup"
  return 0
}

maybe_restart_service() {
  if gum confirm "Restart shinombra service now?"; then
    if gum spin --spinner dot --title "Restarting shinombra..." -- systemctl --user restart shinombra; then
      ok "Service restarted."
    else
      err "systemctl failed to restart the service. Check 'systemctl --user status shinombra' for details."
    fi
  else
    gum style --foreground "$C_DIM" "→ Restart manually: systemctl --user restart shinombra"
  fi
}

# -------------------------------------------------------------
# Проверка и очистка service.env перед перезапуском
# -------------------------------------------------------------

ENV_FILE="$CONFIG_DIR/service.env"

# -------------------------------------------------------------
# Проверка и очистка service.env перед перезапуском
# -------------------------------------------------------------

ENV_FILE="$CONFIG_DIR/service.env"

check_and_clean_env_config_flag() {
  # Если env-файла нет - ничего не делаем
  [[ -f "$ENV_FILE" ]] || return 0

  # Ищем --config с последующим пробелом
  if grep -qE -- '--config[[:space:]]' "$ENV_FILE"; then
    echo
    warn "A '--config' flag was detected in $ENV_FILE!"
    gum style --foreground "$C_DIM" \
      "Your manifest already defines config/overlays." \
      "Having '--config' in service.env sets 'simple mode' and overrides manifest logic."
    echo

    if gum confirm "Remove '--config' flag from service.env now?"; then
      cp -- "$ENV_FILE" "${ENV_FILE}.bak"

      # Удаляем --config и следующий за ним путь
      # Работает для: --config /path/to/file
      sed -i -E 's/--config[[:space:]]+[^[:space:]"]+//g' "$ENV_FILE"
      
      # Убираем двойные пробелы, если они образовались в строке
      sed -i -E 's/[[:space:]]{2,}/ /g' "$ENV_FILE"

      rm -f -- "${ENV_FILE}.bak"
      ok "Removed '--config' flag from $ENV_FILE"
    else
      warn "Kept '--config' in service.env. Note that it will override manifest paths!"
    fi
  fi
}

# -------------------------------------------------------------
# Main
# -------------------------------------------------------------

main() {
  check_dependencies
  show_header

  ensure_manifest_exists
  validate_manifest

  local current_config
  current_config=$(read_current_config)
  local current_overlays=()
  read_current_overlays current_overlays

  show_state_block "Current state" "$current_config" current_overlays

  local config_files=()
  local overlay_files=()
  mapfile -t config_files  < <(list_files "$CONFIG_DIR/configs")
  mapfile -t overlay_files < <(list_files "$CONFIG_DIR/overlays")

  # Мягкое предупреждение, если то, что уже в манифесте, не существует на диске -
  # не блокирует работу, просто информирует до того, как пользователь начнёт выбирать.
  local current_all=("$current_config" "${current_overlays[@]:-}")
  validate_paths_exist current_all || true
  echo

  select_config "$current_config" config_files
  select_overlays current_overlays overlay_files

  echo
  show_state_block "◆ Preview" "$FINAL_CONFIG" FINAL_OVERLAYS

  if ! gum confirm "Save these changes?"; then
    warn "Cancelled."
    exit 0
  fi

  if ! save_manifest "$FINAL_CONFIG" FINAL_OVERLAYS; then
    die "Manifest was not saved. Your original file should be intact - check the error above."
  fi

  ok "Manifest saved successfully."

  check_and_clean_env_config_flag
  
  maybe_restart_service
}

main "$@"
