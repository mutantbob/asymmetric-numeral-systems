#!/bin/sh

in=/tmp/q-phases.txt
out=$1; shift

gnuplot <<EOF
set term png notransparent size 600,400
set output '$out'
set ylabel "Average bits per encoded stream"
set xlabel "uniform table accumulator initial phase"
plot '$in' with linespoints
EOF
