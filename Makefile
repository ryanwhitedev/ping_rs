install:
	cargo build --release
	mkdir -p ~/.local/bin
	cp ./target/release/ping_rs ~/.local/bin
	sudo setcap 'cap_net_raw+ep' ~/.local/bin/ping_rs

