#!/bin/bash

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <engine1_name> <engine2_name> [time_control] [rounds]"
    echo "Example: $0 hhz-v7 hhz-v8 40/60 100"
    return 1
fi

set -x

ENGINE1_NAME=$1
ENGINE2_NAME=$2
ENGINE1="./versions/${ENGINE1_NAME}"
ENGINE2="./versions/${ENGINE2_NAME}"

# Default time control: 40 moves in 60 seconds, with a 10-second increment
TC=${3:-"40/60+10"}
# Default rounds: 2
ROUNDS=${4:-"20"}

PGN_DIR="games"
PGN_FILE="${PGN_DIR}/${ENGINE1_NAME}-vs-${ENGINE2_NAME}-R${ROUNDS}.pgn"

# Create the games directory if it doesn't exist
mkdir -p "${PGN_DIR}"

echo "Starting tournament..."
echo "Engine 1: ${ENGINE1} (as ${ENGINE1_NAME})"
echo "Engine 2: ${ENGINE2} (as ${ENGINE2_NAME})"
echo "Time Control: ${TC}"
echo "Rounds: ${ROUNDS}"
echo "PGN Output: ${PGN_FILE}"

cutechess-cli -engine cmd="${ENGINE1}" name="${ENGINE1_NAME}" \
-engine cmd="${ENGINE2}" name="${ENGINE2_NAME}" \
-each proto=uci tc="${TC}" -rounds "${ROUNDS}" -pgnout "${PGN_FILE}" \
-openings file=openings/silver-suite.txt format=pgn

echo "Tournament finished."