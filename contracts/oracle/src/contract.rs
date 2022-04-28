#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, attr
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PriceResponse};
use shared::oracle::QueryMsg;
use crate::state::{State, STATE};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // instantiate with price
    let state = State {
        price: msg.price,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
    .add_attribute("method", "instantiate")
    .add_attribute("owner", info.sender)
    .add_attribute("price", msg.price.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdatePrice { price} => try_update_price(deps, info, price),
    }
    //TODO: execute try_update_price
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_price(
    deps: DepsMut,
    info: MessageInfo,
    price: u64,
) -> Result<Response, ContractError> {
    let mut state: State  = STATE.load(deps.storage)?;

    //priv check
    if state.owner != info.sender {
        return Err(ContractError::Unauthorized{});
    }

    //valid price check
    if price <= 0u64{
        return Err(ContractError::PriceInstantiationError{});
    }

    state.price = price;

    STATE.save(deps.storage, &state)?;

    let res = Response::new()
        .add_attributes(vec![attr("action", "update_price")]);

    Ok(res)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // TODO
    match msg {
        QueryMsg::QueryPrice {} => to_binary(&query_price(deps)?),
    }

}

fn query_price(deps: Deps) -> StdResult<PriceResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(PriceResponse { price: state.price })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { price: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res : PriceResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::QueryPrice {}).unwrap()).unwrap();
        //assert_eq!(res, Err(StdError::generic_err("not implemented")));
        assert_eq!(17, res.price);
    }

    #[test]
    fn increment() {}
}
