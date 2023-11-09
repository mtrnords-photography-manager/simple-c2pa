#!/bin/sh

MOBILE_DIR="$(pwd)/mobile"
APPLE_DIR="${MOBILE_DIR}/apple"
TARGET_DIR="${APPLE_DIR}/target"
OUT_DIR="${APPLE_DIR}/out"
BINDINGS_DIR="${MOBILE_DIR}/bindings"
BUILD_MODE="release"
NAME="simple_c2pa_mobile"
HEADER_PATH="${BINDINGS_DIR}/${NAME}FFI.h"
NEW_HEADER_DIR="${BINDINGS_DIR}/include"
STATIC_LIB_NAME="lib${NAME}.a"

rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim

# build the bindings
cargo build --manifest-path "${MOBILE_DIR}/Cargo.toml" --target-dir="${TARGET_DIR}"

# build the libraries
cargo build --target aarch64-apple-darwin --release
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
cargo build --target aarch64-apple-ios-sim --release

mkdir -p "${NEW_HEADER_DIR}"
cp "${HEADER_PATH}" "${NEW_HEADER_DIR}/"
cp "${BINDINGS_DIR}/${NAME}FFI.modulemap" "${NEW_HEADER_DIR}/module.modulemap"

mkdir -p ${OUT_DIR}
rm -rf "${OUT_DIR}/${NAME}_framework.xcframework"


xcrun lipo -create -output "${TARGET_DIR}/simulators.a" "${TARGET_DIR}/aarch64-apple-ios-sim/${BUILD_MODE}/${STATIC_LIB_NAME}" "${TARGET_DIR}/x86_64-apple-ios/${BUILD_MODE}/${STATIC_LIB_NAME}"

xcodebuild -create-xcframework \
	   -library "${TARGET_DIR}/aarch64-apple-ios/${BUILD_MODE}/${STATIC_LIB_NAME}" \
	   -headers "${NEW_HEADER_DIR}" \
	   -library "${TARGET_DIR}/simulators.a" \
	   -headers "${NEW_HEADER_DIR}" \
    -output "${OUT_DIR}/${NAME}_framework.xcframework"

cp "${BINDINGS_DIR}/${NAME}.swift" "${OUT_DIR}/"
