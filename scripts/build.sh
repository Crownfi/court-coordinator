#!/usr/bin/env bash
set -e
projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)

# assuming .cargo/config has the following in the [alias] map
$projectPath/scripts/build_optimizer.sh
$projectPath/scripts/check_artifacts_size.sh

# Using a different version of the compiler may change the build output
rustc --version > $projectPath/artifacts/rustc_version.txt
