#!/bin/bash
# purposefully sh, not bash. That's what make uses by default, and I don't see
# a reason to rely on bash at this time.
export PYTHONUNBUFFERED=1
$PYTHON3 $MANAGE_PY runserver "127.0.0.1:$PORT" >server.stdout.txt 2>server.stderr.txt &
echo $! >server.pid

BAIL_TS=$((3+$(date +%s)))
while ! grep --quiet "Starting development server at" server.stdout.txt
do
	echo "Waiting for server to start ..."
	sleep 1
	[ "$BAIL_TS" -le $(date +%s) ] && exit 10
done

exit 0
