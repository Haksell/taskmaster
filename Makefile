SOCKET := /tmp/.taskmaster.sock
PID_FILE := /var/run/server.pid

GARBAGE := *VBox*.log

RESET := \033[0m
RED := \033[1m\033[31m

CONFIG ?= config_files/main.yml

define rm
@if [ -e "$(1)" ]; then \
	rm -rf "$(1)"; \
	echo "$(RED)[X] $(1) removed.$(RESET)"; \
fi
endef

revagrant: cleanvagrant vagrant

vagrant:
	vagrant up --provision
	vagrant ssh

cleanvagrant:
	vagrant destroy -f
	$(call rm,.vagrant)

daemon:
	$(call rm,$(SOCKET))
	cargo run --manifest-path taskmasterd/Cargo.toml -- $(CONFIG)

nodaemon:
	$(call rm,$(SOCKET))
	cargo run --manifest-path taskmasterd/Cargo.toml -- --no-daemonize $(CONFIG)

stop:
	-@kill -TERM $$(cat $(PID_FILE) 2>/dev/null) 2>/dev/null
	$(call rm,$(PID_FILE))

client:
	@python3 taskmasterctl/taskmasterctl.py

clean: stop
	$(call rm,$(SOCKET))
	@rm -rf $(GARBAGE)
	$(call rm,target)
