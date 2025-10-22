# P4 IP Forwarding Project Makefile

P4C = p4c
BMV2_SWITCH = simple_switch
BMV2_CLI = simple_switch_CLI

# P4 program files
P4_SRC = src/ip_forwarding.p4
P4_JSON = build/ip_forwarding.json

# Control plane files
CONTROL_PLANE = control_plane/controller.py
ROUTING_TABLE = control_plane/routing_table.json

# Build directory
BUILD_DIR = build
LOGS_DIR = logs

.PHONY: all clean compile run stop test

all: compile

# Create necessary directories
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(LOGS_DIR):
	mkdir -p $(LOGS_DIR)

# Compile P4 program
compile: $(BUILD_DIR) $(P4_JSON)

$(P4_JSON): $(P4_SRC)
	$(P4C) --target bmv2 --arch v1model $(P4_SRC) -o $(P4_JSON)

# Run the switch
run: $(P4_JSON) $(LOGS_DIR)
	$(BMV2_SWITCH) --interface 0@veth0 --interface 1@veth2 \
		--log-console --thrift-port 9090 $(P4_JSON) \
		> $(LOGS_DIR)/switch.log 2>&1 &

# Stop the switch
stop:
	pkill -f $(BMV2_SWITCH) || true

# Run control plane
control: $(CONTROL_PLANE)
	python3 $(CONTROL_PLANE)

# Setup virtual interfaces for testing
setup-interfaces:
	sudo ip link add veth0 type veth peer name veth1
	sudo ip link add veth2 type veth peer name veth3
	sudo ip link set veth0 up
	sudo ip link set veth1 up
	sudo ip link set veth2 up
	sudo ip link set veth3 up

# Clean up virtual interfaces
clean-interfaces:
	sudo ip link delete veth0 || true
	sudo ip link delete veth2 || true

# Test connectivity
test: run
	sleep 2
	python3 tests/test_forwarding.py

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR) $(LOGS_DIR)

# Install dependencies
install-deps:
	sudo apt-get update
	sudo apt-get install -y p4c bmv2 python3-pip
	pip3 install -r requirements.txt

help:
	@echo "Available targets:"
	@echo "  all          - Compile P4 program"
	@echo "  compile      - Compile P4 program to JSON"
	@echo "  run          - Run BMv2 switch"
	@echo "  stop         - Stop BMv2 switch"
	@echo "  control      - Run control plane"
	@echo "  setup-interfaces - Setup virtual interfaces"
	@echo "  clean-interfaces - Clean up virtual interfaces"
	@echo "  test         - Run forwarding tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  install-deps - Install dependencies"
