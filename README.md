# Secret Roulette

This repo contains the secret contract implementation of a roulette game. 

It's fairly straightforward if you know your way around a CosmWasm contract, and is mainly intended to be used as an example of some tricks and usages of common libraries or other scenarios contract developers may encounter.

You will find:

* Usage of random numbers using the new network RNG service
* Usage of cw-storage-plus
* Sending native tokens
* Storing and reading to/from state
* Create a powerful CI with Github Actions

## Random Numbers in Secret Network

To learn more about how to use random numbers in Secret Network apps, see [our documentation](https://github.com/scrtlabs/SecretNetwork/blob/master/docs/random-usage.md)

## Gitpod

Can't run your local secret environment because you're on M1, or too lazy to install docker? We got your back!

For more details on how to use see [usage](docs/gitpod.md)

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/scrtlabs/rps)
