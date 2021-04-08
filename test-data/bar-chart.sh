#!/bin/sh

in=$1; shift
out=$1; shift

gnuplot <<EOF
set term png notransparent size 1200,500
set output '$out'
set boxwidth 0.5
set style fill solid
plot [-1:256] '$in' using 1:2 with boxes
EOF
