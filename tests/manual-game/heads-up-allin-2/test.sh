#!/usr/bin/env bash
set -eu
EXPECTED_FNAME=$1
ACTUAL_FNAME=$2

declare -a NEEDED_STRINGS=(
	"state NotStarted"
	"player 2 bank 0"
	"player 1 bank 200000"
	"player 2 bet_status Waiting"
	"player 1 bet_status Waiting"
)

for S in "${NEEDED_STRINGS[@]}"; do
	grep --quiet "$S" $ACTUAL_FNAME
	echo Found \"$S\" as expected.
done
