#!/bin/bash

#set -e

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <bin-name> <final-name> <version>"
    return 1
fi

BIN_NAME=$1
FINAL_NAME=$2
VERSION=$3
TARGET_DIR="versions"
TARGET_FILE="hhz-${FINAL_NAME}-${VERSION}"

echo "Building binary '${BIN_NAME}' with version '${VERSION}' and name '${FINAL_NAME}'..."

# Set the environment variable for the build. The build.rs script will read this.
export HHZ_ENGINE_NAME="${TARGET_FILE}"
cargo build --bin="${BIN_NAME}" -F="${BIN_NAME}" --release

# Good practice to unset the variable afterwards.
unset HHZ_ENGINE_NAME

# Create the target directory if it doesn't exist
mkdir -p "${TARGET_DIR}"

echo "Moving 'target/release/${BIN_NAME}' to '${TARGET_DIR}/${TARGET_FILE}'"
mv "target/release/${BIN_NAME}" "${TARGET_DIR}/${TARGET_FILE}"

echo "Build and move complete."