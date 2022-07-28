# GitpodDevEnv

### LocalSecret as a Service

Can't run your local secret environment because you're on M1, or too lazy to install docker? We got your back!

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/scrtlabs/GitpodDevEnv)

This will create an environment that automatically starts localsecret, and exposes the ports for application development. To connect,
prepend the port number with the gitpod url. e.g. if my workspace is at `https://scrtlabs-gitpoddevenv-shqyv12iyrv.ws-eu54.gitpod.io` then I would be able
to connect to the LCD service at `https://1317-scrtlabs-gitpoddevenv-shqyv12iyrv.ws-eu54.gitpod.io`.

This repo also comes with all the dependencies you need to develop Secret Contracts, and SecretCLI. 

### Millionaire's Problem

The code in this repo solves the millionaire's problem in a rather naive way.

This is part of the [Getting Started Guide](https://example.com). It is meant as an easy to understand first contract example.
Therefore, it is expected to value readability over optimized code and idiomatic Rust.