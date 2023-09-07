all: build install

build:
	cargo build --release

install:
	sudo cp target/release/cfnupd /usr/local/bin
	sudo chmod 775 /usr/local/bin/cfnupd

