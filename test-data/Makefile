
MEASURE=cargo run --manifest-path=../measure/Cargo.toml


all: out/atmm-freq.png out/dq-freq.png

out/atmm.txt:
	$(MEASURE) at-the-mountains-of-madness.html -O $@

out/dq.txt:
	$(MEASURE) dream-quest.html -O $@

out/%-freq.png: out/%.txt
	./bar-chart.sh $< $@
