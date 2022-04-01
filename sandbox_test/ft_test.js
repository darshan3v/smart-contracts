const nearAPI = require("near-api-js");
const BN = require("bn.js");
const { base_decode } = require("near-api-js/lib/utils/serialize");
const fs = require("fs").promises;
const assert = require("assert").strict;

function getConfig(env) {
  switch (env) {
    case "sandbox":
    case "local":
      return {
        networkId: "sandbox",
        nodeUrl: "http://localhost:3030",
        masterAccount: "test.near",
        contractAccount: "ft.test.near",
        keyPath: "/tmp/near-sandbox/validator_key.json",
      };
  }
}

const contractMethods = {
  viewMethods: ["storage_balance_bounds","storage_balance_of","ft_metadata","ft_total_supply","ft_balance_of"],
  changeMethods: ["storage_deposit","storage_withdraw","storage_unregister","ft_transfer","ft_transfer_call","mint","ft_transfer_player_reward","new"],
};
let config;
let masterAccount;
let masterKey;
let pubKey;
let keyStore;
let near;

async function initNear() {
  config = getConfig(process.env.NEAR_ENV || "sandbox");
  const keyFile = require(config.keyPath);
  masterKey = nearAPI.utils.KeyPair.fromString(
    keyFile.secret_key || keyFile.private_key
  );
  pubKey = masterKey.getPublicKey();
  keyStore = new nearAPI.keyStores.InMemoryKeyStore();
  keyStore.setKey(config.networkId, config.masterAccount, masterKey);
  near = await nearAPI.connect({
    deps: {
      keyStore,
    },
    networkId: config.networkId,
    nodeUrl: config.nodeUrl,
  });
  masterAccount = new nearAPI.Account(near.connection, config.masterAccount);
  console.log("Finish init NEAR");
}

async function createContractUser(
  accountPrefix,
  contractAccountId,
  contractMethods
) {
  let accountId = accountPrefix + "." + config.masterAccount;
  await masterAccount.createAccount(
    accountId,
    pubKey,
    new BN(10).pow(new BN(25))
  );
  keyStore.setKey(config.networkId, accountId, masterKey);
  const account = new nearAPI.Account(near.connection, accountId);
  const accountUseContract = new nearAPI.Contract(
    account,
    contractAccountId,
    contractMethods
  );
  return accountUseContract;
}

async function initTest() {
  const contract = await fs.readFile("/home/darshan/projects/near/catch/contracts/ft/target/wasm32-unknown-unknown/release/ft.wasm");
  const _contractAccount = await masterAccount.createAndDeployContract(
    config.contractAccount,
    pubKey,
    contract,
    new BN(10).pow(new BN(26))
  );

  const aliceUseContract = await createContractUser(
    "alice",
    config.contractAccount,
    contractMethods
  );

  const bobUseContract = await createContractUser(
    "bob",
    config.contractAccount,
    contractMethods
  );
  console.log("Finish deploy contracts and create test accounts");
  return { aliceUseContract, bobUseContract };
}

async function test() {
  // 1. Creates testing accounts and deploys a contract

  await initNear();
  const { aliceUseContract, bobUseContract } = await initTest();

  // 2. Initialising Contract

  let owner_id = "alice.test.near";
  let total_supply = "10000000000000000000000"; // Greater than 2^53
  let metadata = {
    "spec": "1.1.0",
    "name": "CAT Token",
    "symbol": "CAT",
    "icon": "C-A-T-C-H",
    "reference": "https://github.com/near/core-contracts/tree/master/w-near-141",
    "reference_hash": "AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M=",
    "decimals": 0,
  };

 await aliceUseContract.new({args: {
    owner_id, total_supply, metadata 
  }})

  /****************************/
  /*  TEST - STORAGE MANAGER  */
  /****************************/

  // Test storage_balance_bounds()

  let exp_storage_balance_bounds = {
    "min": "1250000000000000000000",
    "max": "1250000000000000000000"
  };

  let storage_balance_bounds = await aliceUseContract.storage_balance_bounds();
  assert.equal(storage_balance_bounds,exp_storage_balance_bounds)

  // Test storage_balance_of()

  let exp_storage_balance_of = {
    "total": "1250000000000000000000",
    "available": "0"
  }

  let storage_balance_of_alice = await bobUseContract.storage_balance_of("alice.test.near");
  assert.equal(storage_balance_of_alice,exp_storage_balance_of)

  let storage_balance_of_bob = await bobUseContract.storage_balance_of("bob.test.near");
  assert.equal(storage_balance_of_bob,exp_storage_balance_of)
}

test();
