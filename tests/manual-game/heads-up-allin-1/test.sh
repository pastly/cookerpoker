#!/usr/bin/env bash
set -eu
EXPECTED_FNAME=$1
ACTUAL_FNAME=$2

declare -a NEEDED_STRINGS=(
	"state Betting(PreFlop)"
	"player 1 bank 999.90"
	"player 2 bank 999.95"

)

for S in "${NEEDED_STRINGS[@]}"; do
	grep --quiet "$S" $ACTUAL_FNAME
	echo Found \"$S\" as expected.
done
