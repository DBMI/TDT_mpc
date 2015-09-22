#!/bin/bash 
### This shell script is expected to run from 'docker run' command within a Docker Container.
###  Added to https://github.com/DBMI/TDT_mpc forked from https://github.com/ndokmai/TDT_mpc

# copy configuration files from host drive to container drive
cp /opt/mydata/addresses.txt  /opt/TDT_mpc/input
cp /opt/mydata/parameters.txt /opt/TDT_mpc/input

# change a working directory
cd /opt/TDT_mpc

# prepare secure multi-party computation (MPC)
./gen_circuit.sh /opt/mydata/mydata.input

# Sequentially, run multi-party computation (MPC)
./run_mpc.sh