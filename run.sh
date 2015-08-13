#!/bin/bash
PARAMETERS=$1
INPUT=$2
echo "Generating circuit and encoding input..."
./bin/release/mpc_framework input $1 $2
./bin/circtool.exe -tb input/circ.txt input/circ.bin
echo "Done!"

