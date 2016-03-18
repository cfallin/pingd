PREFIX=/

.PHONY: all
all:
	cargo build --release

.PHONY: install
install: all
	cp target/release/pingd $(PREFIX)/usr/local/bin/pingd
	cp scripts/pingd.sh $(PREFIX)/usr/local/bin/pingd.sh
	chmod +x $(PREFIX)/usr/local/bin/pingd.sh
	cp scripts/pingd.conf $(PREFIX)/etc/pingd.conf
	bash -c "if [ -d /etc/init ]; then cp scripts/upstart/pingd.conf /etc/init/pingd.conf; fi"
