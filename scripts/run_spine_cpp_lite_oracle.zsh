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

# spine-c is a thin wrapper around spine-cpp; the directory layout differs between upstream branches.
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
OUT="${BUILD_DIR}/spine_cpp_lite_oracle"

PATCHED_DIR="${BUILD_DIR}/patched-spine-cpp"
PATCHED_SLIDER_CPP="${PATCHED_DIR}/Slider.cpp"

python3 "${ROOT_DIR}/scripts/patch_spine_runtimes_oracle.py" \
  --in "${SPINE_CPP_SRC}/Slider.cpp" \
  --out "${PATCHED_SLIDER_CPP}"

SPINE_CPP_SOURCES=("${SPINE_CPP_SRC}/"*.cpp)
SPINE_CPP_SOURCES=(${SPINE_CPP_SOURCES:#${SPINE_CPP_SRC}/Slider.cpp})

ORACLE_CXXFLAGS=(-std=c++11 -O2 -fno-exceptions -fno-rtti)
ORACLE_LDFLAGS=()
if [[ "${SPINE2D_ORACLE_DEBUG:-0}" == "1" ]]; then
  ORACLE_CXXFLAGS=(-std=c++11 -O0 -g -fno-omit-frame-pointer -fno-exceptions -fno-rtti)
fi
if [[ "${SPINE2D_ORACLE_ASAN:-0}" == "1" ]]; then
  ORACLE_CXXFLAGS+=(-fsanitize=address -fno-omit-frame-pointer)
  ORACLE_LDFLAGS+=(-fsanitize=address)
fi

if [[ ! -x "${OUT}" || "${SPINE2D_ORACLE_REBUILD:-0}" == "1" || "${ROOT_DIR}/scripts/spine_cpp_lite_oracle.cpp" -nt "${OUT}" ]]; then
  clang++ "${ORACLE_CXXFLAGS[@]}" \
    -I"${SPINE_C_INCLUDE}" \
    -I"${SPINE_C_SRC}" \
    -I"${SPINE_CPP_INCLUDE}" \
    "${ROOT_DIR}/scripts/spine_cpp_lite_oracle.cpp" \
    "${SPINE_C_SRC}/extensions.cpp" \
    "${SPINE_C_SRC}/generated/"*.cpp \
    "${SPINE_CPP_SOURCES[@]}" \
    "${PATCHED_SLIDER_CPP}" \
    "${ORACLE_LDFLAGS[@]}" \
    -o "${OUT}"
fi

exec "${OUT}" "$@"
