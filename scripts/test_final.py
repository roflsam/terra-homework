################################################
# imports
################################################

import pandas as pd
import os
import yaml
from terra_sdk.client.lcd import LCDClient
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
from terra_sdk.core.fee import Fee
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.client.lcd.api.tx import CreateTxOptions
from terra_sdk.client.localterra import LocalTerra
import base64
import json
import pendulum
import subprocess
import argparse
from terra_sdk.core.coins import Coins, Coin

################################################
# parse configs
################################################

contracts_df = pd.read_csv("/repos/metadata69/contracts.tsv", sep="\t")

################################################
# terra objects
################################################

terra = LocalTerra()

wallet = terra.wallets["test7"]

friend_wallet = terra.wallets["test2"]

################################################
# deploy func
################################################

def deploy_local_wasm(file_path, wallet, terra):
  with open(file_path, "rb") as fp:
    file_bytes = base64.b64encode(fp.read()).decode()
    store_code_msg = MsgStoreCode(wallet.key.acc_address, file_bytes)
    store_code_tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[store_code_msg], fee=Fee(6900000, "1000000uluna")))
    store_code_result = terra.tx.broadcast(store_code_tx)

  #persist code_id
  deployed_code_id = store_code_result.logs[0].events_by_type["store_code"]["code_id"][0]

  return deployed_code_id

def init_contract(code_id, init_msg, wallet, terra):

  #invoke contract instantiate
  instantiate_msg = MsgInstantiateContract(
    wallet.key.acc_address,
    wallet.key.acc_address,
    code_id,
    init_msg,
    {"uluna": 1000000, "uusd": 1000000},
  )

  #there is a fixed UST fee component now, so it's easier to pay fee in UST
  instantiate_tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[instantiate_msg], fee=Fee(690000, "1000000uusd")))
  instantiate_tx_result = terra.tx.broadcast(instantiate_tx)

  return instantiate_tx_result


def execute_msg(address, msg, wallet, terra, coins=None):

  execute_msg = MsgExecuteContract(
    sender=wallet.key.acc_address,
    contract=address,
    execute_msg=msg,
    coins=coins 
  )

  #there is a fixed UST fee component now, so it's easier to pay fee in UST
  tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[execute_msg], fee=Fee(2000000, "10000000uusd")))
  tx_result = terra.tx.broadcast(tx)

  return tx_result


################################################
# deploy code id
################################################

troll_bridge_code_id = deploy_local_wasm("/repos/terra-homework/artifacts/troll_bridge.wasm", wallet, terra)

################################################
# lemon token exercises
################################################

#deploy lemon cw20 contract

cw20_code_id = contracts_df[(contracts_df["name"]=="token") & (contracts_df["protocol"]=="terraswap")]["code_id"].values[0]

message = {
  "name": "Lemon token",
  "symbol": "LEMON",
  "decimals": 6,
  "initial_balances":[
    {
      "address": wallet.key.acc_address,
      "amount": "69000000",
    }
  ],
  "mint":{
    "minter": wallet.key.acc_address,
  }
}

lemon_token_result = init_contract(cw20_code_id, message, wallet, terra)
lemon_token_address = lemon_token_result.logs[0].events_by_type["instantiate_contract"]["contract_address"][0]

#mint some lemon for friend

message = {
  "mint":{
    "recipient": friend_wallet.key.acc_address,
    "amount": "420000000",
  }
}

mint_result = execute_msg(lemon_token_address, message, wallet, terra)

#confirm balances

print(f"friend_balance: {terra.wasm.contract_query(lemon_token_address, {'balance':{'address': friend_wallet.key.acc_address}})}")
print(f"minter_balance: {terra.wasm.contract_query(lemon_token_address, {'balance':{'address': wallet.key.acc_address}})}")