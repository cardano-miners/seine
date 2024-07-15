# Fortuna block indexer middleware

This is a synchronization middleware that powers the
Fortuna [block-explorer](https://minefortuna.com/explorer).

## Usage

> Really only relevant for the Fortuna website maintainers.

```shell
touch .env
```

Add the following to the `.env` file:

```shell
DOLOS_ENDPOINT="<fill in with your dolos endpoint>"
```

Then run the following:

```shell
cargo run
```
