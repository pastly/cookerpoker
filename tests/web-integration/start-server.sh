#/bin/sh
# Purposefully sh, not bash. That's what make uses by default, and I don't see
# a reason to rely on bash at this time.


cargo run --bin poker-server > poker-server.log 2>&1 &
echo $! > poker-server.pid

BAIL_TS=$((10+$(date +%s)))
while ! grep --quiet "Rocket has launched from" poker-server.log
do
    echo "Waiting for server to start ..."
    sleep 1
    [ "$BAIL_TS" -le $(date +%s) ] && exit 10
done
exit 0
