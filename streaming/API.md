# Roketo contract documentation

- [x] Calls and views
- [x] How to use section
- [ ] Roketo-sdk examples
- [ ] Rrrors details

## Content
- How to
    - [near-api-js](#near-api-js)
    - [near cli](#near-cli)
- Structures
    - [Basic structures](#basic-structures)
    - [Stream](#stream)
    - [Account](#account)
    - [Token](#token)
    - [Token Stats](#tokenstats)
- Views
    - Main views
        - [get_stream](#get_stream)
        - [get_account](#get_account)
        - [get_account_incoming_streams](#get_account_incoming_streams)
        - [get_account_outgoing_streams](#get_account_outgoing_streams)
    - [Other views](#other-views)
- Calls
    - Through NEP-141 FT ([ft_on_transfer](#ft_on_transfer))
        - [Create](#create)
        - [Push](#push)
        - [Deposit](#deposit)
        - [Stake](#stake)
    - Main calls
        - [start_stream](#start_stream)
        - [pause_stream](#pause_stream)
        - [stop_stream](#stop_stream)
        - [withdraw](#withdraw)
    - [Other calls](#other-calls)
    - [Dao calls](#dao-calls)
    - [Oracle calls](#oracle-calls)


## How to use contract
The rocketo contract is a regular Near contract. You need to have a general understanding of working with NEAR before. Suggested to read [nomicon](https://nomicon.io). You can doing call through several methdos.  

### roketo-sdk
> Todo

### near-api-js
Lets start from install package and connecting to wallet ([see quick reference](https://github.com/near/near-api-js/blob/master/examples/quick-reference.md)). After login we can create stream ([see signature](#create)):
```ts
const fTcontract = new Contract(account, 'wrap.testnet', {
    changeMethods: ['ft_transfer_call', 'near_deposit'],
});

// only for NEAR, we need deposit NEAR on wrap 
await fTcontract.near_deposit({}, 200000000000000, 1000000000000000000000000);

await fTcontract.ft_transfer_call({
    receiver_id: 'streaming-roketo.dcversus.testnet'
    amount: '1000000000000000000000000', // 1 NEAR
    memo: 'Roketo transfer',
    msg: JSON.stringify({
        Create: {
            request: {
                "owner_id": "yourname.testnet",
                "receiver_id": "dcversus.testnet",
                "tokens_per_sec": 385802469135802469, // 1 month for 1 NEAR
            }
        }
    }),
}, 200000000000000);

await fTcontract.ft_transfer_call({
    receiver_id: 'streaming-roketo.dcversus.testnet'
    amount: '1', // 1 yocto
    memo: 'Roketo transfer',
    msg: '"Push"',
}, 200000000000000);
```

### near cli
Install node and do `npm install -g near-cli` in console ([read more](https://github.com/near/near-cli)).
```bash
near login
```
and after as example lets call [get_stats](#getstats):
```bash
near call streaming-roketo.dcversus.testnet get_stats --accountId yourname.testnet
```
After calling you will got something like [this in explorer](https://explorer.testnet.near.org/transactions/29nT6VWaSpXugWeAp2kBmoYy2J13WRa8SMA58gzcN7K8).

#### Stream creation
For stream creation in NEAR we need NEAR->wNEAR before ([explorer](https://explorer.testnet.near.org/transactions/8XkzRbeMQJykiN9EwJykKvH9fmLASChYQXwWwaQvF5Ex))
```bash
near call wrap.testnet near_deposit --deposit 1 --accountId dcversus.testnet
```
stream creation through ft_transfer_call ([explorer](https://explorer.testnet.near.org/transactions/4nCQoP6i57obfgkzLgwaUDw97ZC18eZWyyVvRc7AiGF9))
```bash
near call wrap.testnet ft_transfer_call '{"amount": "1000000000000000000000000","receiver_id": "streaming-roketo.dcversus.testnet", "memo": "test", "msg": "{\"Create\":{\"request\":{\"owner_id\":\"dcversus.testnet\",\"receiver_id\":\"lebedev.testnet\",\"tokens_per_sec\":385802469135802500}}}"}' --depositYocto 1 --gas 200000000000000 --accountId dcversus.testnet
```
dont forget push! ([explorer](https://explorer.testnet.near.org/transactions/Cckoemb4haAL43AGe699w3nREqANqKLsd6g1bHEbRA8D))
```bash
near call wrap.testnet ft_transfer_call '{"amount": "1", "receiver_id": "streaming-roketo.dcversus.testnet", "memo": "test", "msg": "\"Push\""}' --depositYocto 1 --gas 200000000000000 --accountId dcversus.testnet
```
lets pause our stream ([explorer](https://explorer.testnet.near.org/transactions/GWoSGiCwioMbjwegNp9N5EESYHzsjRSgjg4TnVTzdL7w))
```bash
near call streaming-roketo.dcversus.testnet pause_stream '{"stream_id": "CUr4BNXQgXqPWCJVvxu5v6jarGnV8y9s6iVWTK2g6fkt"}'  --depositYocto 1 --gas 200000000000000 --accountId dcversus.testnet
```
You are awesome <3


## Roketo structures

### Basic structures
- `AccountId` near account id eg 'test.near', `string`
- `StreamId` near sdk CryptoHash eg '4q7BJuzBBcuv72bbLYAVXiL21mic3Cn3WPpWxMtS6tyP', `string`
- `Timestamp` is a `string` representation of unix time in nanosec
- `SafeFloat`
```json
{ "val": "number", "pow": "number" }
```
Examples
```jsonc
[
    { "val": 1, "pow": -3 }, // 0.1%
    { "val": 2, "pow": -4 }  // 0.02%
]
```

### Stream
```jsonc
{
    "id": "StreamId", // stream id, is a CryptoHash
    "description": "?string", // optional text description of the stream, max 255 symbols
    "creator_id": "AccountId", // stream creator
    "owner_id": "AccountId", // is an owner (or sender) of the stream
    "receiver_id": "AccountId", // receiver
    "token_account_id": "AccountId", // NEP-141 token account id

    "timestamp_created": "Timestamp", // is a timestamp when the stream has been created
    "last_action": "Timestamp", // is a timestamp of the last update called

    "balance": "string", // remaining tokens to stream in yocto
    "tokens_per_sec": "string", // stream speed, values in yocto

    "status": "string", // StreamStatus, see details below
    "tokens_total_withdrawn": "string", // amount of withdrawn tokens

    "cliff": "?Timestamp", // optional, when is will be available to withdraw

    "amount_to_push": "string", // tokens amount what should be pushed to finance contract

    "is_expirable": "boolean", // if true, owner can add deposit before stream finished
    "is_locked": "boolean", //  if true, any actions (stop, start etc are forbidden)
}
```
#### Stream status
- **Initialized** - stream created with `is_auto_start_enabled` = false, tokens not sending before stream will be started
- **Active** - tokens are in process of streaming
- **Paused** - streamed tokens are withdrawn to reciever and sending paused
- **Finished** - stream over, can not 

### Account
```jsonc
{
    // stream counters (like statistics)
    "active_incoming_streams": "number",
    "active_outgoing_streams": "number",
    "inactive_incoming_streams": "number",
    "inactive_outgoing_streams": "number",

    // hashmaps with calculation of total token amount by token account id
    "total_incoming": { "AccountId": "number" },
    "total_outgoing": { "AccountId": "number" },
    "total_received": { "AccountId": "number" },

    "deposit": "string", // near deposited on account

    "stake": "string", // stacked count on account

    "last_created_stream": "StreamId", // last created stream id

    // boolean, if true anyone can call `withdraw` for income streams,
    // otherwise only reciever can do it.
    // Property and call are needed for internal purposes. 
    "is_cron_allowed": "boolean"
}
```

### Token
```jsonc
{
    "account_id": "AccountId",
    "is_listed": "bool",
    "collected_commission": "string",

    "commission_on_create": "string", // taken in current fts in case of listed token
    "commission_coef": SafeFloat, // percentage of tokens taken for commission

    "storage_balance_needed": "string",
    "gas_for_ft_transfer": "string",
    "gas_for_storage_deposit": "string",
}
```

### TokenStats
```jsonc
{
    "total_deposit": "string",
    "tvl": "string",
    "transferred": "string",
    "refunded": "string",
    "total_commission_collected": "string",

    "streams": "number",
    "active_streams": "number",

    "last_update_time": "Timestamp",
}
```

## Roketo views
View methods of contract.

### Main views

#### `get_stream`
Returns the data about the stream. Request:
```json
{
    "stream_id": "StreamId"
}
```

Response are [stream representation](#stream)

#### `get_account`
Returns the account requested. Request: 
```json
{
    "account_id": "AccountId"
}
```
Response are [account representation](#account)


#### `get_account_incoming_streams` 
#### `get_account_outgoing_streams`
Return list of incoming (or outgoing) streams for account. Request:
```json
{
    "account_id": "AccountId",
    "from": "number",
    "limit": "number",
}
```
Response are array of [stream representation](#stream)

### Other views

#### `get_stats`
Return contract stats:
```jsonc
{
    "dao_tokens": { // Hashmap of listed tokens and it stats
        "AccountId": TokenStats // object, see roketo structures
    },

    "total_accounts": "number",
    "total_streams": "number",
    "total_dao_tokens": "number",
    "total_active_streams": "number",
    "total_aurora_streams": "number",
    "total_streams_unlisted": "number", // of unlisted tokens

    "total_account_deposit_near": "string",
    "total_account_deposit_eth": "string",

    "last_update_time": "Timestamp"
}
```

#### `get_dao`
Return contract 'settings'
```jsonc
{
    "dao_id": "AccountId",
    "tokens": { // Hashmap of listed tokens and it configs
        "AccountId": Token, // object, see roketo structures 
    },
    "commission_unlisted": "string", // Balance

    "utility_token_id": "AccountId",
    "utility_token_decimals": "number",

    "eth_near_ratio": SafeFloat, // object, related to charges in Aurora

    "oracles": [ "AccountId" ], // Hashset of account ids
}
```
#### `get_token`
return `[Token, TokenStats]` (see [Token](#token), [TokenStats](#tokenstats)) and request:
```json
{
    "token_account_id": "string"
}
```
#### `get_account_ft`
response account token stats (numbers) `[total_incoming, total_outgoing, total_received]`, request: 
```json
{
    "account_id": "AccountId",
    "token_account_id": "AccountId"
}
```

## Roketo calls

Modifying methods of contract, require a deposit. Some methods should be called through NEP-141 FT [(ft_on_transfer)](#ftontransfer)

### `ft_on_transfer`
**Don't make this call directly.** You should call `ft_transfer_call` at any compatible NEP-141 FT with deposit and payload:
```jsonc
{
    "receiver_id": "ROKETO_ACCOUNT_ID",
    "memo": "string", // Can be any string
    "msg": "TRANSFER_PAYLOAD" // Different for call types, stringified json
}
```

Where `ROKETO_ACCOUNT_ID` is a `streaming-roketo.dcversus.testnet` in testnet and `streaming.r-v2.near` in mainnet. `TRANSFER_PAYLOAD` are different for actions. Details below.

#### `Create`
The action will create users and stream with the transferred payload. Attached deposit will be a transfering amount (commission will be deducted). **You should call `Push` through `ft_on_transfer` for work after create!**

```jsonc
{
    "Create": {
        "request": {
            "owner_id": "AccountId",
            "receiver_id": "AccountId",
            "tokens_per_sec": "number",  // Attention! it should be a BigInt and you need serialize by right way
            "description": "string?",
            "cliff_period_sec": "number?",
            "is_auto_start_enabled": "boolean?",
            "is_expirable": "boolean?",
            "is_locked": "boolean?"
        }
    }
}
```

- `owner_id` account id, is an owner (or sender) of the stream
- `receiver_id` account id, is a receiver of the stream. Must not be the same as the owner
- `tokens_per_sec` stream speed, values in yocto
- `description` optional text description of the stream, max 255 symbols
- `cliff_period_sec` optional, time in sec when is unavailable to withdraw
- `is_auto_start_enabled` optional bool, if false, stream will be inactive before owner call start_stream
- `is_expirable` optional bool, if true, owner can add deposit before stream finished
- `is_locked` optional bool, if true, any actions (stop, start etc will be forbidden)

#### `Push`
Send tokens from streaming contract to finance contract. Should be called after `Create` for internal purposes.

```json
 "Push"
```

#### `Deposit`
Add attached deposit to the stream.

```json
{
    "Deposit": {
        "stream_id": "StreamId"
    }
}
```

#### `Stake`
Expect only `utility_token`! Stake attached deposit to account.

```json
"Stake"
```

### Main calls

#### `start_stream` 

Starts initialized or paused stream. Can be executed only by the owner of the stream in case of initialization or the receiver too if stream was paused. Expects one yocto as deposit. Signature: 
```json
{
    "stream_id": "StreamId"
}
```

#### `pause_stream`
Pauses the stream and transfer streamed tokens to the reciever. Stream can be paused only if it was in the active state. Can be executed only by the owner or the receiver of the stream. Expects one yocto as deposit. Signature: 
```json
{
    "stream_id": "StreamId"
}
```

#### `stop_stream`
Finishing the stream. Finished streams cannot be restarted. All remaining deposit sends back to the owner, all streamed deposit will be send to reciever. Can be executed only by the owner of the stream. Expects one yocto as deposit. Signature: 
```json
{
    "stream_id": "StreamId"
}
```

#### `withdraw`
Transfer streamed tokens to the reciever. If stream deposit was streamed, then the stream will finish. Can be executed only by the receiver of the stream. Expects one yocto as deposit Signature: 
```json
{
    "stream_ids": ["StreamId"]
}
```

### Other calls
These methods are not essential for the functioning of the main task of the contract, but may be useful

#### `change_receiver`
sets a new reciever for the stream. Can be executed only by the receiver of the stream. Expects `storage_balance_needed` from token as deposit. Call signature:
```jsonc
{
    "stream_id": "StreamId",
    "receiver_id": "AccountId" // New receiver 
}
```
#### `account_update_cron_flag`
update user property `is_cron_allowed`. [See more](#get_account) Expects one yocto as deposit. Signature:
```json
{
    "is_cron_allowed": "boolean"
}
```
#### `account_unstake`
send staked `utility_token` to your account. Expects one yocto as deposit.
```json
{
    "amount": "string"
}
```
#### `account_deposit_near`
add a near deposit to your account. No props, need only attached deposit.

### Dao calls
Methods can be executed only by dao account. 

- `dao_update_token` add or update token configration.
```jsonc
{
    "token": {
        "account_id": "AccountId", // token account id
        "is_listed": "boolean", // if true we apply commission otherwise apply commission_unlisted
        "commission_on_create": "number", // taken in current fts in case of listed token
        "commission_coef": SafeFloat, //  percentage of tokens taken for commission
        "storage_balance_needed": "string", 
        "gas_for_ft_transfer": "string", // gas settings for UI purposes
        "gas_for_storage_deposit": "string", // gas settings for UI purposes
    }
}
```

#### `dao_change_owner`
sets a new dao account
```json
{
    "new_dao_id": "AccountId"
}
```
#### `dao_update_commission_unlisted`
sets a commission for unlisted tokens
```json
{
    "commission_unlisted": "number"
}
```
#### `dao_withdraw_ft`
withdraw collected FT commission
```json
{

    "token_account_id": "AccountId",
    "receiver_id": "AccountId",
    "amount": "number",
}
```
#### `dao_withdraw_near`
withdraw collected NEAR commission
```json
{

    "receiver_id": "AccountId",
    "amount": "number",
}
```
#### `dao_add_oracle`
add new oracle
```json
{
    "new_oracle_id": "AccountId"
}
```
#### `dao_remove_oracle`
remove oracle from list
```json
{
    "oracle_id": "AccountId"
}
```

### Oracle calls
The oracle is an external contract that we register as Dao (see [dao calls](#dao-calls)). Oracle purpose is a update `commission_on_create` for tokens.

#### `oracle_update_commission_on_create`
update `commission_on_create` for specified token.
```json
{
    "token_account_id": "AccountId",
    "commission_on_create": "number",
}
```
#### `oracle_update_eth_near_ratio`
update dao `eth_near_ratio` (used only for Aurora calls)
```jsonc
{
    "ratio": SafeFloat
}
```
