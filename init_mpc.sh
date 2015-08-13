#!/bin/bash
cd input
echo "Running multi-party computation..."
../bin/mpc.exe config.txt > output.txt
echo "Done!"
