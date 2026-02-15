EXAMPLES = examples/square examples/cardioid examples/guitar examples/band examples/move-the-line

.PHONY: examples clean

examples: $(addsuffix .html,$(EXAMPLES))

examples/square.html: examples/square.yml examples/square-config.yml
	cargo run -- points examples/square.yml

examples/cardioid.html: examples/cardioid.yml examples/cardioid-config.yml
	cargo run -- points examples/cardioid.yml

examples/guitar.html: examples/guitar.yml examples/guitar-config.yml
	cargo run -- points examples/guitar.yml

examples/band.html: examples/band.svg examples/band-config.yml
	cargo run -- svg examples/band.svg

examples/move-the-line.html: examples/move-the-line-config.yml
	cargo run -- text --font AkayaTelivigala-Regular 'Move The Line' --config examples/move-the-line-config.yml -o examples/move-the-line

clean:
	rm -f $(addsuffix .html,$(EXAMPLES)) $(addsuffix -embed.html,$(EXAMPLES))
