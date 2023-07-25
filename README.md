# Kyberswap Exchange Rate Scraper Daemon

Scrape [Kyberswap](https://kyberswap.com) Classic pools' exchange rates and save as JSON to disk. Requires [Docker](https://www.docker.com) and an [Infura](https://infura.io) API key.

## Getting Started
Download source and build the image (this may take a while):
```bash
git clone https://github.com/ArielHorwitz/kersd.git
cd kersd
docker build -t kersd .
```

To configure our API key we make a copy of the template file and edit it:
```bash
cp ./dockerenv.template ~/.kersd-dockerenv
$EDITOR ~/.kersd-dockerenv
```

Run the container and pass our dockerenv file as an argument:
```bash
docker run --env-file ~/.kersd-dockerenv kersd
```

The daemon should be running in your terminal and printing log messages. We can stop the container in another terminal and copy data from the container to our local machine:
```bash
docker kill $(docker ps -ql)
rm -rf ~/.kersdb/
docker cp $(docker ps -qla):/app/db/ ~/.kersdb/
```

We saved the exchange rates JSON files in `~/.kersdb/`, where each subdirectory is a block number containing a file for each pool's exchange rate (in both directions). Token amounts are encoded in hex. Let's take a look at one of these as an example:
```bash
cat $(find ~/.kersdb/* | grep 0x | head -1) ; echo
```

The above command produces the contents of a random file in `~/.kersdb/`:
```json
{
  "block_number": 17770883,
  "pool": "0x6a4d5f8385ff6e7fc4ebf6f726e12a958daa1cba",
  "token0": "0xb9eefc4b0d472a44be93970254df4f4016569d27",
  "token1": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
  "sell0_buy1": {
    "sell_amount": "0x45f5",
    "buy_amount": "0x1",
    "exchange_rate": 0.00005583784689262382
  },
  "sell1_buy0": {
    "sell_amount": "0x5",
    "buy_amount": "0x186a0",
    "exchange_rate": 20000.0
  }
}
```

## Potential Improvements
- Real database
- Tests
