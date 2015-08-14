#!/bin/bash
cd gmw_backend
make clean
cd ../gmw_frontend
cargo clean
cd ../input/
rm output.txt
rm ../bin/*.exe -rf
rm ../bin/release -rf
