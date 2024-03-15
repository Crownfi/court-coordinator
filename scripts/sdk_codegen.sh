#!/usr/bin/env bash
set -e
projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)
cd ../packages/cargo/court-coordinator-sdk-maker
cargo run -- ../../npm/court-coordinator-sdk/src/base
