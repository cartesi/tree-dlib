# Start geth
geth --dev --http --http.api eth,net,web3 --ws --ws.api eth,net,web3 >>/dev/null 2>&1 &
pid=$!
sleep 3

# Deploy contracts to localhost
npx hardhat deploy --network localhost

# Run tests flagged with ignore
cargo test -p tree --test tree_test -- --nocapture --ignored

# kill geth
kill "$pid"
sleep 5
