#!/bin/bash

MOBILE_DIR="mobile"
ANDROID_DIR="${MOBILE_DIR}/android"
TARGET_DIR="${ANDROID_DIR}/target"
OUT_DIR="${ANDROID_DIR}/out"
BINDINGS_DIR="${MOBILE_DIR}/bindings"
JNILIBS_DIR="${OUT_DIR}/jniLibs"
BUILD_MODE="release"
NAME="simple_c2pa_mobile"
LIB_NAME="lib${NAME}.so"
FIXED_LIB_NAME="libuniffi_${NAME}.so"

# build the bindings
cargo build --manifest-path "${MOBILE_DIR}/Cargo.toml" --target-dir="${TARGET_DIR}"
mkdir -p ${OUT_DIR}
cp -r ${BINDINGS_DIR}/uniffi ${OUT_DIR}/

# build the libraries
declare -A arch_dir_map=(
    ["x86_64-linux-android"]="x86_64" #,
#    ["i686-linux-android"]="x86",
#    ["armv7-linux-androideabi"]="armeabi-v7a",
#    ["aarch64-linux-android"]="arm64-v8a"
)
for architecture in "${!arch_dir_map[@]}"; do
    TARGET_DIR="$(pwd)/${architecture}"
    echo ${TARGET_DIR}}
    if [ "${BUILD_MODE}" == "release" ]; then
        cross build --manifest-path "${MOBILE_DIR}/Cargo.toml" --target "$architecture" --target-dir="${TARGET_DIR}" --release
    else
        cross build --manifest-path "${MOBILE_DIR}/Cargo.toml" --target "$architecture" --target-dir="${TARGET_DIR}"
    fi

    DESTINATION_DIR="${JNILIBS_DIR}/${arch_dir_map[$architecture]}"
    mkdir -p ${DESTINATION_DIR}
    cp "${TARGET_DIR}/${BUILD_MODE}/${LIB_NAME}" "${DESTINATION_DIR}/${FIXED_LIB_NAME}"
done


