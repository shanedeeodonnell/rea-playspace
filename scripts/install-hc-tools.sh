#!/bin/bash

# install wasm32 compilation target
rustup target install wasm32-unknown-unknown

# install `hc` cli tool
# use the version from June 8th, 2022 to match version
# 0.0.143 of holochain 
# 0.0.136 of hdk
# KEEP THIS IN SYNC
cargo install holochain_cli --version 0.0.41