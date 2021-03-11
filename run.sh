#!/bin/sh

BINARY_LOCATION=./target/release/tcp
cargo build --release
sudo setcap cap_net_admin=eip $BINARY_LOCATION
./$BINARY_LOCATION &
PID=$!
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0

wait $PID
