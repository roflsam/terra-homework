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
from terra_sdk.core.coin import Coin
from terra_sdk.core.coins import Coins

################################################
# parse configs
################################################


################################################
# terra objects
################################################

# terra = LocalTerra()
# wallet = terra.wallets["test3"]
terra = LCDClient(url="https://bombay-lcd.terra.dev/", chain_id="bombay-12")

wallet = terra.wallet(MnemonicKey(mnemonic='avocado record cook clog home hello degree pulse wedding box secret civil panel often swing accident critic inspire junk resist dilemma remain stadium slam'))
friend_wallet = terra.wallet(MnemonicKey(mnemonic='jungle aware blue opinion alcohol dog session theme tower eager away bind cat false addict robot dune guilt wrong avoid exclude repeat open seminar'))

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



# deploy
deploy_local_wasm('artifacts/cw20_token.wasm', wallet, terra)
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

lemon_result = init_contract(67482, message, wallet, terra)
lemon_address = lemon_result.logs[0].events_by_type["instantiate_contract"]["contract_address"][0]
# check friend's address
terra.wasm.contract_query(lemon_address, {'balance':{'address': friend_wallet.key.acc_address}})

# now send some lemon to friend
msg = {
  "mint":{
    'recipient': friend_wallet.key.acc_address,
    "amount": "1000000",
  }
}
execute_msg('terra1l40arwn0lehwuay48pzxxu9h8x298twk3s7a4f', msg, wallet, terra)

# now check balance
terra.wasm.contract_query(lemon_address, {'balance':{'address': friend_wallet.key.acc_address}})


# Notes
var_result.to_data().keys()

var_result_address = var_result.logs[0].events_by_type["instantiate_contract"]["contract_address"][0]

terra.wasm.contract_query(var_result_address, {"get_count":{}})

execute_msg('terra1mcjuqmckylgcyqmu4nameelv2p796va429mfcg', {'deposit_stable':{}}, wallet, terra_local, coins=Coins.from_str(f"100000000uusd"))
