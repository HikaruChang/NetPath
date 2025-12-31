.PHONY: all build install clean test

# Rust target for OpenWrt (cross-compilation)
RUST_TARGET ?= x86_64-unknown-linux-musl
CARGO ?= cargo
INSTALL ?= install
DESTDIR ?= /tmp/netpath-install

# Directories
PREFIX ?= /usr
BINDIR = $(PREFIX)/bin
SBINDIR = $(PREFIX)/sbin
SYSCONFDIR = /etc
DATADIR = $(PREFIX)/share/netpath

all: build

build:
	$(CARGO) build --release --target $(RUST_TARGET)

build-debug:
	$(CARGO) build --target $(RUST_TARGET)

install: build
	$(INSTALL) -d $(DESTDIR)$(SBINDIR)
	$(INSTALL) -m 0755 target/$(RUST_TARGET)/release/netpath $(DESTDIR)$(SBINDIR)/
	
	$(INSTALL) -d $(DESTDIR)$(SYSCONFDIR)/netpath
	$(INSTALL) -m 0644 examples/config.toml $(DESTDIR)$(SYSCONFDIR)/netpath/
	
	$(INSTALL) -d $(DESTDIR)$(SYSCONFDIR)/init.d
	$(INSTALL) -m 0755 files/netpath.init $(DESTDIR)$(SYSCONFDIR)/init.d/netpath
	
	$(INSTALL) -d $(DESTDIR)$(SYSCONFDIR)/config
	$(INSTALL) -m 0644 files/netpath.uci $(DESTDIR)$(SYSCONFDIR)/config/netpath
	
	$(INSTALL) -d $(DESTDIR)$(SYSCONFDIR)/hotplug.d/iface
	$(INSTALL) -m 0755 files/netpath.hotplug $(DESTDIR)$(SYSCONFDIR)/hotplug.d/iface/15-netpath

clean:
	$(CARGO) clean

test:
	$(CARGO) test

check:
	$(CARGO) check
	$(CARGO) clippy

fmt:
	$(CARGO) fmt
