cd ~/nearcore
rm -rf /tmp/near-sandbox
target/debug/neard-sandbox --home /tmp/near-sandbox init
target/debug/neard-sandbox --home /tmp/near-sandbox run