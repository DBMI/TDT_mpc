import sys

f1 = open(sys.argv[1], 'r')
f2 = open(sys.argv[2], 'r')
input_f = open(sys.argv[3], 'r')
mp = []
input_f.readline()
for line in input_f:
    mp.append(line.strip().split(' ')[0])
n = 0
k = 0
mx = 0
decimal_points = 0
for line in f1:
    if line.strip().split(' ')[0] == "n":
        n = int(line.strip().split(' ')[1])
    if line.strip().split(' ')[0] == "k":
        k = int(line.strip().split(' ')[1])
    if line.strip().split(' ')[0] == "max":
        mx = int(line.strip().split(' ')[1])
    if line.strip().split(' ')[0] == "decimal_points":
        decimal_points = int(line.strip().split(' ')[1])
id_bits = 1
while 2**id_bits < n:
    id_bits += 1

shifts = 0
while 2**shifts < 10**decimal_points:
    shifts += 1
bin_output = []
for line in f2:
    if line.strip().split(' ')[0] == "(binary)":
        bin_output = line.strip().split(' ')[1:]
        break
if len(bin_output) == 0:
    exit(0)
tdt_bits = len(bin_output)/k-id_bits
print "SNP", "\t", "TDT Score"
print "====================="
for i in range(k):
    snp = int(''.join(reversed(
        bin_output[i*(id_bits+tdt_bits): i*(id_bits+tdt_bits) + id_bits])), 2)
    tdt = int(''.join(reversed(
        bin_output[i*(id_bits+tdt_bits) + id_bits: (i+1)*(id_bits+tdt_bits)])),
        2)/float(2**shifts)
    if tdt == 0:
        break
    print mp[snp], "\t",
    print "%.*f" % (decimal_points, tdt)
