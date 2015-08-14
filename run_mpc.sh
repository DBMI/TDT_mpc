#!/bin/bash
cd input
echo "Running multi-party computation..."
../bin/mpc.exe config.txt | tee output.txt
rm config.txt input.txt circ.bin circ.txt
echo "Done!"
