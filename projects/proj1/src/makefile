.PHONY: all clean

all:
	cargo build --release

bench:
	python3 benchmark.py

plot:
	python3 draw_plots.py

clean:
	cargo clean
	rm -rf files/*
	rm -rf plots/*
	rm -rf *.csv
