
MEASURE=cargo run --manifest-path=../measure/Cargo.toml


all: out/atmm-freq.png out/atmm.bin \
	out/dq-freq.png out/dq.bin \
	out/mpeg-freq.png out/mpeg.bin

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
