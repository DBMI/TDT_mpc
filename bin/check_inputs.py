import sys
from operator import itemgetter

config = open(sys.argv[1], 'r')
input0 = open(sys.argv[2], 'r')
input1 = open(sys.argv[3], 'r')
input2 = open(sys.argv[4], 'r')

n = 0
k = 0
threshold = 0.0
decimal_points = 0
for line in config:
    if line.strip().split(' ')[0] == "n":
        n = int(line.strip().split(' ')[1])
    if line.strip().split(' ')[0] == "k":
        k = int(line.strip().split(' ')[1])
    if line.strip().split(' ')[0] == "decimal_points":
        decimal_points = int(line.strip().split(' ')[1])
    if line.strip().split(' ')[0] == "threshold":
        threshold = float(line.strip().split(' ')[1])

print n, k, threshold
bs = []
cs = []
input0.readline()
for (i, line) in list(enumerate(input0)):
    if i >= n:
        break
    b = int(line.strip().split(' ')[1])
    c = int(line.strip().split(' ')[2])
    bs.append(b)
    cs.append(c)

input1.readline()
for (i, line) in list(enumerate(input1)):
    if i >= n:
        break
    bs[i] += int(line.strip().split(' ')[1])
    cs[i] += int(line.strip().split(' ')[2])

input2.readline()
for (i, line) in list(enumerate(input2)):
    if i >= n:
        break
    bs[i] += int(line.strip().split(' ')[1])
    cs[i] += int(line.strip().split(' ')[2])

tdt = []
for (i, (b, c)) in list(enumerate(zip(bs, cs)))[:n]:
    tdt.append((i, (b-c)**2/float(b+c)))
tdt = sorted(tdt, key=itemgetter(1), reverse=True)
for i in tdt[:k]:
    if i[1] > threshold:
        print i[0], "\t", i[1]
    else:
        print 0, "\t", 0.0
