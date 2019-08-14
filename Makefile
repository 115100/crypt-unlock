PREFIX = $(DESTDIR)/usr/lib/dracut/modules.d/60ssh-unlock
DRACUT_SCRIPTS = $(wildcard dracut/60ssh-unlock/*.sh)
DRACUT_DATA_FILES = dracut/60ssh-unlock/sshd_config

install: target/release/crypt-unlock $(DRACUT_SCRIPTS) $(DRACUT_DATA_FILES)
	mkdir -p $(PREFIX)
	install -m 0755 $< $(PREFIX)
	install -m 0755 $(DRACUT_SCRIPTS) $(PREFIX)
	install -m 0644 $(DRACUT_DATA_FILES) $(PREFIX)
.PHONY: install

clean:
	rm -rf bin/ target/
.PHONY: clean
