#!/usr/bin/env zsh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

RUNTIMES_DIR="${SPINE2D_UPSTREAM_RUNTIMES_DIR:-}"
if [[ -z "${RUNTIMES_DIR}" ]]; then
  for cand in \
    "${ROOT_DIR}/.cache/spine-runtimes" \
    "${ROOT_DIR}/third_party/spine-runtimes" \
  ; do
    if [[ -d "${cand}" ]]; then
      RUNTIMES_DIR="${cand}"
      break
    fi
  done
fi

if [[ ! -d "${RUNTIMES_DIR}" ]]; then
  echo "Missing upstream runtimes dir. Set SPINE2D_UPSTREAM_RUNTIMES_DIR to a spine-runtimes checkout." >&2
  exit 2
fi

SPINE_C_INCLUDE="${RUNTIMES_DIR}/spine-c/include"
SPINE_C_SRC="${RUNTIMES_DIR}/spine-c/src"
if [[ ! -f "${SPINE_C_INCLUDE}/spine-c.h" || ! -f "${SPINE_C_SRC}/extensions.cpp" ]]; then
  echo "Missing spine-c sources under: ${RUNTIMES_DIR}/spine-c" >&2
  exit 2
fi

SPINE_CPP_INCLUDE=""
SPINE_CPP_SRC=""
if [[ -d "${RUNTIMES_DIR}/spine-cpp/include" && -d "${RUNTIMES_DIR}/spine-cpp/src/spine" ]]; then
  SPINE_CPP_INCLUDE="${RUNTIMES_DIR}/spine-cpp/include"
  SPINE_CPP_SRC="${RUNTIMES_DIR}/spine-cpp/src/spine"
elif [[ -d "${RUNTIMES_DIR}/spine-cpp/spine-cpp/include" && -d "${RUNTIMES_DIR}/spine-cpp/spine-cpp/src/spine" ]]; then
  SPINE_CPP_INCLUDE="${RUNTIMES_DIR}/spine-cpp/spine-cpp/include"
  SPINE_CPP_SRC="${RUNTIMES_DIR}/spine-cpp/spine-cpp/src/spine"
else
  echo "Missing spine-cpp sources under: ${RUNTIMES_DIR}/spine-cpp" >&2
  exit 2
fi

BUILD_DIR="${ROOT_DIR}/.cache/spine2d-oracle"
mkdir -p "${BUILD_DIR}"
OUT="${BUILD_DIR}/spine_cpp_lite_dump_constraints"

clang++ -std=c++17 -O2 \
  -I"${SPINE_C_INCLUDE}" \
  -I"${SPINE_C_SRC}" \
  -I"${SPINE_CPP_INCLUDE}" \
  "${ROOT_DIR}/scripts/spine_cpp_lite_dump_constraints.cpp" \
  "${SPINE_C_SRC}/extensions.cpp" \
  "${SPINE_C_SRC}/generated/"*.cpp \
  "${SPINE_CPP_SRC}/"*.cpp \
  -o "${OUT}"

exec "${OUT}" "$@"
