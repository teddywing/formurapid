VERSION := $(shell egrep '^version = ' Cargo.toml | awk -F '"' '{ print $$2 }')
TOOLCHAIN := $(shell fgrep default_host_triple $(HOME)/.rustup/settings.toml | awk -F '"' '{ print $$2 }')

SOURCES := $(shell find src -name '*.rs')
RELEASE_PRODUCT := target/release/formurapid

MAN_PAGE := doc/formurapid.1

DIST := $(abspath dist)
DIST_PRODUCT := $(DIST)/bin/formurapid
DIST_MAN_PAGE := $(DIST)/share/man/man1/formurapid.1


$(RELEASE_PRODUCT): $(SOURCES)
	cargo build --release


.PHONY: doc
doc: $(MAN_PAGE)

$(MAN_PAGE): $(MAN_PAGE).txt
	a2x --no-xmllint --format manpage $<


.PHONY: dist
dist: $(DIST_PRODUCT) $(DIST_MAN_PAGE)

$(DIST):
	mkdir -p $@

$(DIST)/bin: | $(DIST)
	mkdir -p $@

$(DIST)/share/man/man1: | $(DIST)
	mkdir -p $@

$(DIST_PRODUCT): $(RELEASE_PRODUCT) | $(DIST)/bin
	cp $< $@

$(DIST_MAN_PAGE): $(MAN_PAGE) | $(DIST)/share/man/man1
	cp $< $@


.PHONY: pkg
pkg: formurapid_$(VERSION)_$(TOOLCHAIN).tar.bz2

formurapid_$(VERSION)_$(TOOLCHAIN).tar.bz2: dist
	tar cjv -s /dist/formurapid_$(VERSION)_$(TOOLCHAIN)/ -f $@ dist
