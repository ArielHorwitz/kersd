# Kyberswap Exchange Rate Scraper Daemon

Scrape [Kyberswap](https://kyberswap.com) Classic pools' exchange rates and save as JSON to disk. Requires [Docker](https://www.docker.com) and an [Infura](https://infura.io) API key.

## Getting Started
Download source:
```bash
git clone git@github.com:ArielHorwitz/kersd.git
cd kersd
```

To configure our API key we make a copy of the template file and edit it:
```bash
cp ./dockerenv.template ~/.kersd-dockerenv
$EDITOR ~/.kersd-dockerenv
```

Build the image (this may take a while):
```bash
docker build -t kersd .
```

Run the container and pass our dockerenv file as an argument:
```bash
docker run kersd --env-file ~/.kersd-dockerenv
```

Our terminal should show new block numbers as they are found, as well as any errors. In another terminal we can stop the container and copy data from the container to our local machine:
```bash
docker kill $(docker ps -ql)
rm -rf ~/.kersdb/
docker cp $(docker ps -qla):/app/db/ ~/.kersdb/
```

The exchange rates are saved as JSON files in `~/.kersdb/`, where each subdirectory is a block number containing a file for each pool's exchange rate. Token amounts are saved in hex. Let's take a look at one of these as an example:
```bash
# Print contents of a random file in ~/.kersdb/
cat $(find ~/.kersdb/* | grep 0x | head -1) ; echo
```

## Potential Improvements
- More detailed exchange rates with a range of buy/sell amounts in both directions
- Tests
- Logging
