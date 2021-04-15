#!/bin/sh

out=$1; shift
out2=$1; shift

gnuplot <<EOF
set term png notransparent size 1000,1000 font ",24"
set output '$out'
set ylabel "numerical encoding result" 
set xlabel "all possible messages, sorted by result" 
set logscale y

plot [] [1e5:1e9] '/tmp/qa.txt' with lines title "uniform", '/tmp/qc.txt' with lines title "ranged by prevalence", '/tmp/qb.txt' with lines title "ranged backwards"

set output '$out2'
plot [] [1e5:1e9] '/tmp/qa.txt' with lines title "uniform", '/tmp/qe.txt' with lines title "old uniform", '/tmp/qf.txt' with lines title "weird", '/tmp/qb.txt' with lines title "ranged backwards", 4**10 title "unencoded"
EOF
