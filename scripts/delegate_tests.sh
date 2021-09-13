#!/bin/bash

npx hardhat node --no-deploy >> /dev/null 2>&1 &
hardhat_pid=$!
sleep 3

## testing tree delegate

./tree/target/debug/tree_server_main >> /dev/null 2>&1 &
server_pid=$!
sleep 3

DELEGATE_TEST=1 npx hardhat test test/test_tree.ts --network localhost

# kill tree server
kill "$server_pid"

## end testing tree delegate

kill "$hardhat_pid"
