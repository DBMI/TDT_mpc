PARAMETERS=$1
INPUT=$2

cd input
python ../bin/interpret_result.py $PARAMETERS output.txt $INPUT
