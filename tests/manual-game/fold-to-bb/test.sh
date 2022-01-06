#!/usr/bin/env bash
set -eu
EXPECTED_FNAME=$1
ACTUAL_FNAME=$2

#grep --quiet "player 3 bank 999.90" $ACTUAL_FNAME
grep --quiet "player 3 bank 1000.05" $ACTUAL_FNAME
