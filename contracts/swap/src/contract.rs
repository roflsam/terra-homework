#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult, WasmMsg, BankMsg, WasmQuery, Uint128, CosmosMsg, to_binary, attr, Coin, Addr, QueryRequest, StdError
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use shared::oracle::{QueryMsg as oracle_query, PriceResponse};
use shared::querier::query_balance;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:swap";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        token_address: deps.api.addr_validate(&msg.token_address)?,
        oracle_address: deps.api.addr_validate(&msg.oracle_address)?,

    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  
    match msg {
        ExecuteMsg::BuyLemons {} => try_buy_lemons(deps, info),
        ExecuteMsg::WithdrawLuna {amount} => try_withdraw_luna(deps, info, amount, env),
    }
 
}

pub fn try_buy_lemons(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // find fund in the contract
    let payment = info
    .funds
    .iter()
    .find(|x| x.denom == String::from("uluna") && x.amount > Uint128::zero())
    .ok_or_else(|| {
        StdError::generic_err(format!("No {} assets are provided to bond", String::from("uluna")))
    })?;
    let state = STATE.load(deps.storage)?;


    // get price of luna and lemon, figure out how much lemon to give for luna
    let price_lemons = query_oracle(deps.as_ref(), state.oracle_address)?;

    let lemons_to_sell = Uint128::from(1u64).multiply_ratio(payment.amount, Uint128::from(price_lemons));

    // now send lemon by minting
    let mint_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.token_address.into(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.into(),
            amount: lemons_to_sell,
        })?,
    });

    Ok(Response::new()
    .add_attributes(vec![attr("action", "buy_lemons")])
    .add_message(mint_msg))
}

pub fn try_withdraw_luna(deps: DepsMut, info: MessageInfo,  amount:i32, env: Env) -> Result<Response, ContractError> {
    //priv check
    let state: State = STATE.load(deps.storage)?;

    if state.owner != info.sender {
        return Err(ContractError::Unauthorized{});
    }

    //valid amount check
    if amount <= 0i32{
        return Err(ContractError::InvalidQuantity{});
    }

    //sketchy convert
    let amount: u64 = amount as u64;

    //check balance
    let balance: Uint128 = query_balance(&deps.querier, &env.contract.address, String::from("uluna"))?;

    if balance < amount.into(){
        return Err(ContractError::InvalidQuantity{});
    }

    //pay out uluna
    let bank_msg = CosmosMsg::Bank(BankMsg::Send{
        to_address: info.sender.to_string(),
        amount: vec![
            Coin{
                denom: String::from("uluna"),
                amount: amount.into(),
            }],
    });
    let res = Response::new()
        .add_attributes(vec![attr("action", "withdraw_luna")])
        .add_message(bank_msg);

    Ok(res)
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    // TODO
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => Ok(to_binary(&query_state(deps)?)?),
    }
}
pub fn query_state(deps: Deps) -> StdResult<State> {
    let state: State = STATE.load(deps.storage)?;
    Ok(state)
}
pub fn query_oracle(deps: Deps, oracle_address: Addr) -> StdResult<u64> {
    // load price form the oracle
    let price_response: PriceResponse =
        deps.querier.query(
            &QueryRequest::Wasm(WasmQuery::Smart {contract_addr: oracle_address.to_string(),
            msg: to_binary(&oracle_query::QueryPrice { })?,}
        ))?;

    Ok(price_response.price)
}
#[cfg(test)]
mod tests {
    #[test]
    fn proper_initialization() {

        //TODO
    }
}
