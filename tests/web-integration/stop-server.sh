#!/bin/sh
kill $(cat server.pid) || /bin/true
