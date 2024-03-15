#!/bin/sh
set -e
echo "optimizer_workarounds active: arg1=$1";

# Following 2 lines are needed to get private git dependencies to work
apk add --no-cache git openssh-client 
export CARGO_NET_GIT_FETCH_WITH_CLI=true;

# Run the "actual" entrypoint
optimize.sh .

# Update file permissions so they aren't root
chown -R "$1" /code/artifacts
