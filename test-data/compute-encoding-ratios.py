#!/usr/bin/python

import sys
from math import *

class Gob:
    def __init__(self, fname, output_fname):
        self.f = open(fname, "r")
        self.of = open(output_fname, "w")

    def fetch_line(self):
        line = self.f.readline()
        if line is not None and len(line)>0:
            parts = line.split()
            if len(parts)<2:
                print("uhoh '%s'"%line)
            self.x = float(parts[0])
            self.y = float(parts[1])
        else:
            self.x = None
            self.y = None

def any_input_ready(streams):
    for stream in streams:
        if stream.x is not None:
            return True
    return False

def compute_range(streams):
    min_y=None
    max_y = None
    for stream in streams:
        if stream.y is None:
            continue
        if min_y is None or stream.y < min_y:
            min_y = stream.y
        if max_y is None or stream.y > max_y:
            max_y = stream.y
    return min_y, max_y

def index_with_min_x(streams):
    min_x = None
    answer = None
    for i in range(0,len(streams)):
        stream = streams[i]
        if stream.x is None:
            continue
        if min_x is None or stream.x < min_x:
            min_x = stream.x
            answer = i
    return answer

def safelog(x):
    return log(max(x,1))

def mission1(input_file_names, output_file_names):
    #print(input_file_names)
    streams = []
    for i in range(0,len(input_file_names)):
        streams.append( Gob(input_file_names[i], output_file_names[i]))

    #print(streams)

    for src in streams:
        src.fetch_line()

    old_center = 0
    while any_input_ready(streams):
        min_y,max_y = compute_range(streams)

        #print([ min_y, max_y])
        center = max(old_center, ( safelog(min_y) + safelog(max_y)) /2)

        idx = index_with_min_x(streams)

        dy = safelog(streams[idx].y) - center
        msg = "%f\t%f\n"%(streams[idx].x, dy)
        streams[idx].of.write(msg)
        streams[idx].fetch_line()

        old_center = center

def usage():
    msg = "Usage:\n %s in1 [ in2 ... ] out1 [ out2 ...] "%sys.argv[0]
    print(msg)
#
#
#

argv = sys.argv[1:]
n = len(argv)/2
if n != floor(n):
    usage()
    exit(1)
n = int(n)
inputs = argv[:n]
outputs = argv[n:]

if False:
    print(inputs)
    print(outputs)
    exit(1)

mission1(inputs, outputs)