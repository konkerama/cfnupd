
build:
	cargo build --release

copy:
	sudo cp target/release/cfnupd /usr/local/bin
	sudo chmod 775 /usr/local/bin/cfnupd

install: build copy
