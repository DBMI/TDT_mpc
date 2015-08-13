#!/bin/bash
cd gmw_backend 
make
cp *.exe ../bin
cd ../gmw_frontend
cargo build --release
cp target/release/ ../bin/ -r
