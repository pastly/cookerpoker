#!/usr/bin/env bash
set -eu
EXPECTED_FNAME=$1
ACTUAL_FNAME=$2

declare -a NEEDED_STRINGS=(
	"state Dealing"
	"player 1 bank 0.00"
	"player 2 bank 2000.00"
	"player 1 bet_status Folded"
	"player 2 bet_status Waiting"
)

for S in "${NEEDED_STRINGS[@]}"; do
	grep --quiet "$S" $ACTUAL_FNAME
	echo Found \"$S\" as expected.
done
