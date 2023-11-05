SOCKET := /tmp/.unixdomain.sock
PID_FILE := /var/run/server.pid

GARBAGE := *VBox*.log

RESET := \033[0m
RED := \033[1m\033[31m

define rm
@if [ -e "$(1)" ]; then \
	sudo rm -rf "$(1)"; \
	echo "$(RED)[X] $(1) removed.$(RESET)"; \
fi
endef

revagrant: fclean vagrant

vagrant:
	vagrant up --provision
	vagrant ssh

server:
	$(call rm,$(SOCKET))
	sudo cargo run --manifest-path taskmasterd/Cargo.toml

debug:
	$(call rm,$(SOCKET))
	sudo cargo run --manifest-path taskmasterd/Cargo.toml -- --debug

stop:
	-@sudo kill -TERM $$(sudo cat $(PID_FILE))
	$(call rm,$(PID_FILE))

client:
	sudo python3 taskmasterctl/taskmasterctl.py

clean:  stop
	$(call sudo,rm,$(SOCKET))
	@rm -rf $(GARBAGE)
	$(call rm,target)

fclean: clean
	vagrant destroy -f
	$(call rm,.vagrant)
