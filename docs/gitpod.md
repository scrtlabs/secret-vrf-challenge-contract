### Gitpod Usage

The environment will automatically start with localsecret, and it exposes the ports for application development.
Specifically -

* 26657 - RPC
* 1317 - LCD/Rest Gateway
* 9090 - GRPC
* 9091 - GRPC-Web
* 5000 - LocalSecret faucet

To connect, prepend the port number with the gitpod url. e.g. if my workspace is at `https://scrtlabs-gitpoddevenv-shqyv12iyrv.ws-eu54.gitpod.io` then I would be able
to connect to the LCD service at `https://1317-scrtlabs-gitpoddevenv-shqyv12iyrv.ws-eu54.gitpod.io`.

To configure SecretCLI you'll want to use:
```
secretcli config node `https://26657-scrtlabs-gitpoddevenv-your-workspace-url.gitpod.io`
secretcli config chain-id secret-testnet-1
```

We also recommend for ease of use while testing
```
secretcli config keyring-backend test
secretcli config output json
```

This environment also comes with all the dependencies you need to develop Secret Contracts, and SecretCLI.

To make the most out of the environment, you can switch between two views:

* LocalSecret will provide the logs from the running secretd node, which can be helpful when testing and debugging contracts.
* Terminal can allow you to execute secretcli commands or clone a repository.

Switching between the views can be done here:

![image](https://user-images.githubusercontent.com/16579705/182580806-f43563ed-ab36-4403-89b3-435d7e7fc4da.png)
