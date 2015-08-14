#!/bin/bash
INPUT=$1
echo "Generating circuit and encoding input..."
./bin/release/mpc_framework input parameters.txt $INPUT
./bin/circtool.exe -tb input/circ.txt input/circ.bin
echo "Done!"

