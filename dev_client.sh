#!/bin/bash
RUST_LOG=debug cargo run --bin client -- --ip "192.168.1.110" --port 4545 --file "./test_files/test.txt"