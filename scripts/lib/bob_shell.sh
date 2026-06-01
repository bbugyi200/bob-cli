#!/bin/bash

if [[ -z "${SCRIPTNAME:-}" ]]; then
  SCRIPTNAME="$(basename "$0")"
  readonly SCRIPTNAME
fi

if [[ -z "${DEBUG+x}" ]]; then
  DEBUG=false
fi

if [[ -z "${VERBOSE+x}" ]]; then
  VERBOSE=0
fi

function _bob_verbose_enabled() {
  [[ "${DEBUG:-false}" == true ]] && return 0
  [[ "${VERBOSE:-0}" =~ ^[0-9]+$ ]] && [[ "${VERBOSE:-0}" -gt 0 ]]
}

function _bob_log() {
  local level="$1"
  shift

  while [[ "$#" -gt 0 ]]; do
    case "$1" in
    --up | -u)
      shift
      [[ "$#" -gt 0 ]] && shift
      ;;
    --)
      shift
      break
      ;;
    *)
      break
      ;;
    esac
  done

  local message="${1:-}"
  if [[ "$#" -gt 0 ]]; then
    shift
  fi

  printf '%s: %s: ' "${SCRIPTNAME}" "${level}" >&2
  if [[ "$#" -gt 0 ]]; then
    printf "${message}" "$@" >&2
  else
    printf '%s' "${message}" >&2
  fi
  printf '\n' >&2
}

function log::debug() {
  _bob_verbose_enabled || return 0
  _bob_log "debug" "$@"
}

function log::error() {
  _bob_log "error" "$@"
}

function log::info() {
  _bob_log "info" "$@"
}

function log::warn() {
  _bob_log "warning" "$@"
}

function die() {
  local exit_code=1

  if [[ "${1:-}" == "-x" ]]; then
    exit_code="$2"
    shift 2
  fi

  local message="${1:-}"
  if [[ "$#" -gt 0 ]]; then
    shift
  fi

  log::error --up 1 "${message}" "$@"
  exit "${exit_code}"
}

function usage() {
  local patterns=()
  if declare -p USAGE_GRAMMAR >/dev/null 2>&1; then
    patterns=("${USAGE_GRAMMAR[@]}")
  fi

  printf 'usage: '

  if [[ "${#patterns[@]}" -eq 0 ]]; then
    printf '%s\n' "${SCRIPTNAME}"
    return 0
  fi

  local pattern
  local first=true
  for pattern in "${patterns[@]}"; do
    if [[ "${first}" == true ]]; then
      printf '%s %s\n' "${SCRIPTNAME}" "${pattern}"
      first=false
    else
      printf '       %s %s\n' "${SCRIPTNAME}" "${pattern}"
    fi
  done
}
