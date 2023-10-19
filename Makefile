SOCKET := /tmp/.unixdomain.sock

RESET := \033[0m
RED := \033[1m\033[31m

define rm
@if [ -e "$(1)" ]; then \
	rm -rf "$(1)"; \
	echo "$(RED)[X] $(1) removed.$(RESET)"; \
fi
endef

server:
	$(call rm,$(SOCKET))
	cargo run --bin server

client:
	cargo run --bin client

clean:
	$(call rm,$(SOCKET))
	$(call rm,target)

fclean: clean
	vagrant destroy
	$(call rm,.vagrant)