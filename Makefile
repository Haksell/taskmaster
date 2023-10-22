SOCKET := /tmp/.unixdomain.sock

GARBAGE := *VBox*.log

RESET := \033[0m
RED := \033[1m\033[31m

define rm
@if [ -e "$(1)" ]; then \
	rm -rf "$(1)"; \
	echo "$(RED)[X] $(1) removed.$(RESET)"; \
fi
endef

revagrant: fclean vagrant

vagrant:
	vagrant up --provision
	vagrant ssh

server:
	$(call rm,$(SOCKET))
	cargo run --bin server

client:
	cargo run --bin client

clean:
	$(call rm,$(SOCKET))
	$(call rm,$(GARBAGE))
	$(call rm,target)

fclean: clean
	vagrant destroy -f
	$(call rm,.vagrant)