
MEASURE=cargo run --manifest-path=../measure/Cargo.toml


all: out/atmm-freq.png out/atmm.bin \
	out/dq-freq.png out/dq.bin \
	out/mpeg-freq.png out/mpeg.bin \
	out/encoder-phase.png \
	out/table-variations.png out/table-variations2.png out/table-variations3.png

out/atmm.txt:
	$(MEASURE) at-the-mountains-of-madness.html -O $@

out/atmm.bin:
	$(MEASURE) at-the-mountains-of-madness.html -o $@

out/dq.txt:
	$(MEASURE) dream-quest.html -O $@

out/dq.bin:
	$(MEASURE) dream-quest.html -o $@

out/mpeg.bin:
	$(MEASURE) iso13818-2.pdf -o $@

out/mpeg.txt:
	$(MEASURE) iso13818-2.pdf -O $@

out/%-freq.png: out/%.txt
	./bar-chart.sh $< $@

out/encoder-phase.png: /tmp/q-phases.txt
	./graph-encoder-phase.sh $@

RELATIVES=/tmp/ra.txt /tmp/rb.txt /tmp/rc.txt /tmp/re.txt /tmp/rf.txt

out/table-variations3.png: $(RELATIVES)

out/table-variations.png out/table-variations2.png: $(ABSOLUTES)
	./graph-table-variations.sh out/table-variations.png out/table-variations2.png

out/table-variations3.png:
	./graph-table-variations-4.sh out/table-variations3.png

ABSOLUTES=/tmp/qa.txt /tmp/qb.txt /tmp/qc.txt /tmp/qe.txt /tmp/qf.txt
$(RELATIVES): $(ABSOLUTES)
	python3 ./compute-encoding-ratios.py $(ABSOLUTES) $(RELATIVES)
