#!/bin/bash

cd ..
cargo build --release
rm -rf /usr/bin/dynamic-dns-keeper
cp target/release/dynamic-dns-keeper /usr/bin/

systemctl stop dynamic-dns-keeper
systemctl start dynamic-dns-keeper
