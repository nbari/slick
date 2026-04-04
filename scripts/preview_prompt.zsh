#!/usr/bin/env zsh

set -euo pipefail

ROOT_DIR=${0:A:h:h}
SLICK_BIN=${SLICK_PREVIEW_BIN:-$ROOT_DIR/target/debug/slick}
SLICK_PREVIEW_BRANCH=${SLICK_PREVIEW_BRANCH:-develop}
SLICK_PREVIEW_STATUS=${SLICK_PREVIEW_STATUS:-M 10}
SLICK_PREVIEW_ELAPSED=${SLICK_PREVIEW_ELAPSED:-0}
SLICK_PREVIEW_INTERVAL=${SLICK_PREVIEW_INTERVAL:-1}
SLICK_PREVIEW_TOOLBOX_NAME=${SLICK_PREVIEW_TOOLBOX_NAME:-codex}
SLICK_PREVIEW_DEVPOD_NAME=${SLICK_PREVIEW_DEVPOD_NAME:-hfile}
SLICK_PREVIEW_AWS_PROFILE=${SLICK_PREVIEW_AWS_PROFILE:-${SLICK_PREVIEW_AWS_LABEL:-prod}}
SLICK_PREVIEW_AWS_REGION=${SLICK_PREVIEW_AWS_REGION:-eu-central-1}
SLICK_PREVIEW_KUBECONFIG_PRIMARY=${SLICK_PREVIEW_KUBECONFIG_PRIMARY:-${SLICK_PREVIEW_KUBECONFIG:-/tmp/dev-cluster}}
SLICK_PREVIEW_KUBECONFIG_SECONDARY=${SLICK_PREVIEW_KUBECONFIG_SECONDARY:-/tmp/prod-cluster}
SLICK_PREVIEW_PYTHON_ENV=${SLICK_PREVIEW_PYTHON_ENV:-venv}

PROMPT_DATA=$(printf '{"action":"","auth_failed":false,"branch":"%s","remote":[],"staged":false,"status":"%s","u_name":""}' \
  "$SLICK_PREVIEW_BRANCH" "$SLICK_PREVIEW_STATUS")

TOOLBOX_TMP=''

cleanup() {
  if [[ -n ${TOOLBOX_TMP} && -d ${TOOLBOX_TMP} ]]; then
    rm -rf -- "$TOOLBOX_TMP"
  fi
}
trap cleanup EXIT

usage() {
  cat <<'USAGE'
Usage: zsh scripts/preview_prompt.zsh [--watch]

Renders prompt previews for the common context markers using the current
SLICK_PROMPT_* environment variables plus simulated Toolbx, DevPod, AWS, k8s,
and Python contexts.

Optional environment overrides:
  SLICK_PREVIEW_BRANCH=main
  SLICK_PREVIEW_STATUS="M 2"
  SLICK_PREVIEW_TOOLBOX_NAME=toolbox
  SLICK_PREVIEW_DEVPOD_NAME=workspace
  SLICK_PREVIEW_AWS_PROFILE=prod
  SLICK_PREVIEW_AWS_REGION=eu-west-1
  SLICK_PREVIEW_KUBECONFIG_PRIMARY=/tmp/dev-cluster
  SLICK_PREVIEW_KUBECONFIG_SECONDARY=/tmp/prod-cluster
  SLICK_PREVIEW_PYTHON_ENV=.venv
  SLICK_PREVIEW_INTERVAL=2

Compatibility aliases still supported:
  SLICK_PREVIEW_AWS_LABEL
  SLICK_PREVIEW_KUBECONFIG
USAGE
}

ensure_binary() {
  if [[ -x $SLICK_BIN ]]; then
    return
  fi

  cargo build --quiet --manifest-path "$ROOT_DIR/Cargo.toml"
}

setup_toolbox_fixture() {
  TOOLBOX_TMP=$(mktemp -d)
  : > "$TOOLBOX_TMP/.toolboxenv"
  printf 'engine="podman"\nname="%s"\nid="preview"\n' "$SLICK_PREVIEW_TOOLBOX_NAME" > "$TOOLBOX_TMP/.containerenv"
}

preview_kubeconfig() {
  if [[ -n $SLICK_PREVIEW_KUBECONFIG_SECONDARY ]]; then
    printf '%s:%s' "$SLICK_PREVIEW_KUBECONFIG_PRIMARY" "$SLICK_PREVIEW_KUBECONFIG_SECONDARY"
  else
    printf '%s' "$SLICK_PREVIEW_KUBECONFIG_PRIMARY"
  fi
}

render_prompt() {
  local -a env_args
  env_args=(
    env
    -u AWS_ACCESS_KEY_ID
    -u AWS_DEFAULT_REGION
    -u AWS_PROFILE
    -u AWS_REGION
    -u AWS_SECRET_ACCESS_KEY
    -u AWS_SESSION_TOKEN
    -u DEVPOD
    -u DEVPOD_WORKSPACE_ID
    -u KUBECONFIG
    -u VIRTUAL_ENV
    -u VIRTUAL_ENV_PROMPT
    -u PYENV_VERSION
    -u PIPENV_ACTIVE
    -u PIPENV_ACTIVE_COLOR
    -u SLICK_TEST_TOOLBOXENV_PATH
    -u SLICK_TEST_CONTAINERENV_PATH
    "$@"
    "$SLICK_BIN"
    prompt
    -e "$SLICK_PREVIEW_ELAPSED"
    -r 0
    -k main
    -d "$PROMPT_DATA"
  )

  "${env_args[@]}"
}

show_example() {
  local title=$1
  shift
  local rendered

  print -P -- "%B${title}%b"
  rendered=$(render_prompt "$@")
  print -P -- "$rendered"
  print
}

show_examples() {
  local kubeconfig
  kubeconfig=$(preview_kubeconfig)

  print -P -- "%BPrompt Preview%b"
  print

  show_example "Base"
  show_example "Toolbx" \
    SLICK_TEST_TOOLBOXENV_PATH="$TOOLBOX_TMP/.toolboxenv" \
    SLICK_TEST_CONTAINERENV_PATH="$TOOLBOX_TMP/.containerenv"
  show_example "DevPod" \
    DEVPOD=true \
    DEVPOD_WORKSPACE_ID="$SLICK_PREVIEW_DEVPOD_NAME"
  show_example "AWS (profile)" \
    AWS_PROFILE="$SLICK_PREVIEW_AWS_PROFILE"
  show_example "AWS (region fallback)" \
    AWS_REGION="$SLICK_PREVIEW_AWS_REGION"
  show_example "Kubernetes" \
    KUBECONFIG="$SLICK_PREVIEW_KUBECONFIG_PRIMARY"
  show_example "Kubernetes (multi-file)" \
    KUBECONFIG="$kubeconfig"
  show_example "Toolbx + Python" \
    SLICK_TEST_TOOLBOXENV_PATH="$TOOLBOX_TMP/.toolboxenv" \
    SLICK_TEST_CONTAINERENV_PATH="$TOOLBOX_TMP/.containerenv" \
    VIRTUAL_ENV="/tmp/venvs/$SLICK_PREVIEW_PYTHON_ENV"
  show_example "DevPod + AWS + Kubernetes" \
    DEVPOD=true \
    DEVPOD_WORKSPACE_ID="$SLICK_PREVIEW_DEVPOD_NAME" \
    AWS_REGION="$SLICK_PREVIEW_AWS_REGION" \
    KUBECONFIG="$kubeconfig"
  show_example "Toolbx + AWS + Kubernetes + Python" \
    SLICK_TEST_TOOLBOXENV_PATH="$TOOLBOX_TMP/.toolboxenv" \
    SLICK_TEST_CONTAINERENV_PATH="$TOOLBOX_TMP/.containerenv" \
    AWS_PROFILE="$SLICK_PREVIEW_AWS_PROFILE" \
    KUBECONFIG="$kubeconfig" \
    VIRTUAL_ENV="/tmp/venvs/$SLICK_PREVIEW_PYTHON_ENV"
}

main() {
  local mode=${1:-once}

  case "$mode" in
    once)
      ;;
    --watch)
      ;;
    -h|--help)
      usage
      return 0
      ;;
    *)
      usage >&2
      return 1
      ;;
  esac

  cd "$ROOT_DIR"
  ensure_binary
  setup_toolbox_fixture

  if [[ "$mode" == --watch ]]; then
    while true; do
      clear
      show_examples
      print -r -- "Refreshing every ${SLICK_PREVIEW_INTERVAL}s. Press Ctrl-C to exit."
      sleep "$SLICK_PREVIEW_INTERVAL"
    done
  else
    show_examples
  fi
}

main "$@"
