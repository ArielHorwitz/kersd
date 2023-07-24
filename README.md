# Kyberswap Exchange Rate Scraper Daemon

Scrape [Kyberswap](https://kyberswap.com) Classic pools' exchange rates and save as JSON to disk. Requires [Docker](https://www.docker.com) and an [Infura](https://infura.io) API key.

## Getting Started
Download source:
```bash
git clone git@github.com:ArielHorwitz/kersd.git
cd kersd
```

Configure API key (replace `YOUR_API_KEY`):
```bash
printf YOUR_API_KEY > ./APIKEY
```

Build the image and run a container:
```bash
docker build -t kersd .  # This may take a while
docker run kersd
```

Our terminal should show new block numbers as they are found, as well as any errors. In another terminal we can stop the container and copy data from the container to our local machine:
```bash
docker kill $(docker ps -ql)
rm -rf ~/.kersd-db/
docker cp $(docker ps -qla):/src/db/ ~/.kersd-db/
```

The exchange rates are saved as JSON files to `~/.kersd-db/`, where each subdirectory is a block number which contains a file for each pool's exchange rate. Token amounts are saved in hex. Let's take a look at one of these as an example:
```bash
# Print contents of a random file in ~/.kersd-db/
cat $(find ~/.kersd-db//* | grep 0x | head -1) ; echo
```

## Potential Improvements
- More detailed exchange rates with a range of buy/sell amounts in both directions
- Remove API key from image and pass to container
- Command line argument parsing for custom configuration
- Tests
- Logging
