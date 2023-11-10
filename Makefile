SOCK_FILE := /tmp/taskmaster.sock
PID_FILE := /tmp/taskmasterd.pid
LOG_FILE := /tmp/taskmasterd.log

GARBAGE := *VBox*.log taskmasterctl/__pycache__

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

tmptaskmaster:
	@mkdir -p /tmp/taskmaster
	@chmod 777 /tmp/taskmaster

daemon: tmptaskmaster
	cargo run --manifest-path taskmasterd/Cargo.toml -- $(CONFIG)

nodaemon: tmptaskmaster
	cargo run --manifest-path taskmasterd/Cargo.toml -- --debug $(CONFIG)

stop:
	-@kill -TERM $$(cat $(PID_FILE) 2>/dev/null) 2>/dev/null
	$(call rm,$(PID_FILE))

client:
	@python3 taskmasterctl/taskmasterctl.py

clean: stop
	@rm -rf $(GARBAGE)
	@rm -rf $(SOCK_FILE)
	@rm -rf $(LOG_FILE)
	$(call rm,taskmasterd/target)
	@rm -rf /tmp/taskmaster/*
