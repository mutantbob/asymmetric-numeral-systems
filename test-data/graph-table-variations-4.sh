#!/bin/sh

out=$1; shift

gnuplot <<EOF
set term png notransparent size 1600,900 font ",24"
set xlabel "all possible messages, sorted by result"

set output '$out'
set ylabel "variance in logarithm of encoded result"
plot '/tmp/ra.txt' with lines title 'uniform',\
'/tmp/re.txt' with lines title 'old uniform',\
'/tmp/rc.txt' with lines title 'ranged by prevalence',\
'/tmp/rb.txt' with lines title 'ranged backwards',\
'/tmp/rf.txt' with lines title 'weird'

EOF
