SOCKET := /tmp/.unixdomain.sock

RESET := \033[0m
RED := \033[1m\033[31m

define remove_target
@if [ -e "$(1)" ]; then \
	rm -rf "$(1)"; \
	echo "$(RED)[X] $(1) removed.$(RESET)"; \
fi
endef

server:
	$(call remove_target,$(SOCKET))
	cargo run --bin server

client:
	cargo run --bin client

clean:
	$(call remove_target,$(SOCKET))
	$(call remove_target,target)

fclean: clean
	$(call remove_target,.vagrant)