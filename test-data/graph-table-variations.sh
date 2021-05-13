#!/bin/sh

out=$1; shift
out2=$1; shift

gnuplot <<EOF
set term png notransparent size 1000,1000 font ",24"
set xlabel "all possible messages, sorted by result"
set ylabel "numerical encoding result"
set logscale y

set output '$out'
plot [] '/tmp/qa.txt' with lines title "uniform", '/tmp/qc.txt' with lines title "ranged by prevalence", '/tmp/qb.txt' with lines title "ranged backwards", 4**10 title "unencoded"

set output '$out2'
plot [] '/tmp/qa.txt' with lines title "uniform", '/tmp/qe.txt' with lines title "old uniform", '/tmp/qf.txt' with lines title "weird", '/tmp/qb.txt' with lines title "ranged backwards", 4**10 title "unencoded"

EOF
