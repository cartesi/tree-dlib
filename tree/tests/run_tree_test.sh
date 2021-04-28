# Start geth
geth --dev --http --http.api eth,net,web3 --ws --ws.api eth,net,web3 >>/dev/null 2>&1 &
pid=$!
sleep 3

# Create second account in geth
echo '{"jsonrpc":"2.0","method":"personal_newAccount","params":[""],"id":67}' | socat /tmp/geth.ipc -

# Deploy contracts to localhost
rm TestTree.address
npx hardhat deploy --network localhost --reset
npx hardhat run --network localhost ../../scripts/deploy_test_tree.ts
sleep 3

# Run tests flagged with ingored
cargo test -p tree --test tree_test -- --nocapture --ignored

# kill geth
kill "$pid"
sleep 5
