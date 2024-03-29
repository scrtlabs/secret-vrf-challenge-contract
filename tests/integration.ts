import axios from "axios";
import {SecretNetworkClient, TxResponse, Wallet} from "secretjs";
import fs from "fs";
import assert from "assert";

function localsecretUrl() {
  return process.env.LOCALSECRET || "http://localhost";
}

// Returns a client with which we can interact with secret network
const initializeClient = async (endpoint: string, chainId: string) => {
  const wallet = new Wallet(); // Use default constructor of wallet to generate random mnemonic.
  const accAddress = wallet.address;
  const client = new SecretNetworkClient({
    // Create a client to interact with the network
    url: endpoint,
    chainId: chainId,
    wallet: wallet,
    walletAddress: accAddress,
  });

  console.log(`Initialized client with wallet address: ${accAddress}`);
  return client;
};

// Stores and instantiates a new contract in our network
const initializeContract = async (
  client: SecretNetworkClient,
  contractPath: string
) => {
  const wasmCode = fs.readFileSync(contractPath);
  console.log("Uploading contract");

  const uploadReceipt = await client.tx.compute.storeCode(
    {
      wasm_byte_code: wasmCode,
      sender: client.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );

  if (uploadReceipt.code !== 0) {
    console.log(
      `Failed to get code id: ${JSON.stringify(uploadReceipt.rawLog)}`
    );
    throw new Error(`Failed to upload contract`);
  }

  const codeIdKv = uploadReceipt.jsonLog![0].events[0].attributes.find(
    (a: any) => {
      return a.key === "code_id";
    }
  );

  const codeId = Number(codeIdKv!.value);
  console.log("Contract codeId: ", codeId);

  const contractCodeHash = (await client.query.compute.codeHashByCodeId({code_id: String(codeId)})).code_hash;

  if (contractCodeHash === undefined) {
    throw new Error(`Failed to get code hash`);
  }

  console.log(`Contract hash: ${contractCodeHash}`);

  const contract = await client.tx.compute.instantiateContract(
    {
      sender: client.address,
      code_id: codeId,
      init_msg: {
        min_bet: 5,
        max_bet: 1000,
        max_total: 1_000_000,
        supported_denoms: ["uscrt"]
      },
      code_hash: contractCodeHash,
      label: "My contract" + Math.ceil(Math.random() * 10000), // The label should be unique for every contract, add random string in order to maintain uniqueness
    },
    {
      gasLimit: 1000000,
    }
  );

  if (contract.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${contract.rawLog}`
    );
  }

  const contractAddress = contract.arrayLog!.find(
    (log) => log.type === "message" && log.key === "contract_address"
  )!.value;

  console.log(`Contract address: ${contractAddress}`);

  const contractInfo: [string, string] = [contractCodeHash, contractAddress];
  return contractInfo;
};


const getFromFaucet = async (address: string) => {
  await axios.get(`${localsecretUrl()}:5000/faucet?address=${address}`);
};

async function getScrtBalance(userCli: SecretNetworkClient): Promise<string> {
  let balanceResponse = await userCli.query.bank.balance({
    address: userCli.address,
    denom: "uscrt",
  });

  if (balanceResponse?.balance?.amount === undefined) {
    throw new Error(`Failed to get balance for address: ${userCli.address}`)
  }

  return balanceResponse.balance.amount;
}

async function fillUpFromFaucet(
  client: SecretNetworkClient,
  targetBalance: Number
) {
  let balance = await getScrtBalance(client);
  while (Number(balance) < targetBalance) {
    try {
      await getFromFaucet(client.address);
    } catch (e) {
      console.error(`failed to get tokens from faucet: ${e}`);
    }
    balance = await getScrtBalance(client);
  }
  console.error(`got tokens from faucet: ${balance}`);
}

// Initialization procedure
async function initializeAndUploadContract() {
  let endpoint = `${localsecretUrl()}:1317`;
  let chainId =  process.env.CHAINID || "secretdev-1";

  const client = await initializeClient(endpoint, chainId);
  // we'll need 2 players for the game
  const client2 = await initializeClient(endpoint, chainId);

  await fillUpFromFaucet(client, 100_000_000);
  await fillUpFromFaucet(client2, 100_000_000);

  const [contractHash, contractAddress] = await initializeContract(
    client,
    "contract.wasm"
  );

  const clientInfo: [SecretNetworkClient, SecretNetworkClient, string, string] = [
    client,
    client2,
    contractHash,
    contractAddress,
  ];
  return clientInfo;
}

async function initializeGame(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddess: string
): Promise<TxResponse> {
  const tx: TxResponse = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddess,
      code_hash: contractHash,
      msg: {bet: {bets: [{result: {exact: {num: 31}}, amount: {denom: "uscrt", amount: "1000"}}]}},
      sent_funds: [{denom: "uscrt", amount: "1000"}],
    },
    {
      gasLimit: 200000,
    }
  );

  console.log(`initializeGame TX used ${tx.gasUsed} gas`);

  return tx;
}

async function test_run_game(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
) {
  let tx: TxResponse = await initializeGame(client, contractHash, contractAddress);

  let rouletteResult = ""
  for (const k in tx.jsonLog[0].events) {
    console.log(tx.jsonLog[0].events[k].type)
    if (tx.jsonLog[0].events[k].type.toLocaleLowerCase() === 'wasm-wasm-roulette_result') {
      rouletteResult = tx.jsonLog[0].events[k].attributes[1].value;
    }
  }

  console.log(`Got result: ${rouletteResult}`)

  assert(
      typeof Number(rouletteResult) === "number",
    `result returned something that isn't a number: ${JSON.stringify(tx)}`
  );
}

async function test_gas_limits() {
  // There is no accurate way to measue gas limits but it is actually very recommended to make sure that the gas that
  // is used by a specific tx makes sense
}

async function runTestFunction<R>(tester: CallableFunction): Promise<R> {
  // @ts-ignore
  console.log(`Testing ${tester.name}`);
  let resp: R = await tester();
  // @ts-ignore
  console.log(`[SUCCESS] ${tester.name}`);

  return resp;
}

(async () => {

  console.log(`Initializing contract and deploying to ${localsecretUrl()}`);

  const [client, client2, contractHash, contractAddress] =
    await initializeAndUploadContract();


  await runTestFunction<string>(
      test_run_game.bind(this, client, contractHash, contractAddress),
  );

})();
