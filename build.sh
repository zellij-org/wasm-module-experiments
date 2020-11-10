#!/bin/bash

cd loader
cargo build
cd ../module
cargo build
#cd ../asmscript
#npm run build