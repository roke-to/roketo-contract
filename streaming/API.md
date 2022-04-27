# Roketo contract documentation

- [x] Calls and views
- [x] How to use section
- [ ] Roketo-sdk examples
- [ ] Rrrors details

## Content
- How to
    - [near cli](#near-cli)
    - [near-api-js](#near-api-js)
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


## How to use the contract
Roketo’s smart contract is a regular NEAR contract. To find better understanding about principles of NEAR protocol, please read the [nomicon](https://nomicon.io). A several method’s of calls are described there, so you would be able to guide yourself with the docs mentioned.

### roketo-sdk
> Todo

### near cli
At first, install node. Then type `npm install -g near-cli` in the console ([read more](https://github.com/near/near-cli)).
```bash
near login
```
In examples below repleace `yourname.testnet` to your NEAR testnet login. As an example try the call view method [get_stats](#getstats):
```bash
near view streaming-roketo.dcversus.testnet get_stats --accountId yourname.testnet
```
After the call you might get the response like [this in explorer as response](https://explorer.testnet.near.org/transactions/29nT6VWaSpXugWeAp2kBmoYy2J13WRa8SMA58gzcN7K8).

#### Stream creation
Roketo’s streaming contract works with NEP-141 tokens. The easiest way to obtain NEP-141 tokens is to wrap NEAR tokens with the contract `wrap.near/wrap.testnet` to receive wNEAR. Lets deposit: ([response](https://explorer.testnet.near.org/transactions/8XkzRbeMQJykiN9EwJykKvH9fmLASChYQXwWwaQvF5Ex))
```bash
near call wrap.testnet near_deposit --deposit 1 --accountId yourname.testnet
```
To create a stream, you should send the initial amount of tokens and bring the instructions of stream processing to the contract. The process of stream creation is optimized to be executed with the only transaction, based on NEP-141 feature `ft_transfer_call`. The idea is to send tokens with the proper message that will be interpreted by roketo streaming contract as instructions.
To succeed, we need to call the token contract to send tokens into roketo’s streaming contract with msg with [CreateRequest](#create) structure packed to JSON. Example below: ([response](https://explorer.testnet.near.org/transactions/4nCQoP6i57obfgkzLgwaUDw97ZC18eZWyyVvRc7AiGF9))
```bash
near call wrap.testnet ft_transfer_call '{"amount": "1000000000000000000000000","receiver_id": "streaming-roketo.dcversus.testnet", "memo": "test", "msg": "{\"Create\":{\"request\":{\"owner_id\":\"yourname.testnet\",\"receiver_id\":\"dcversus.testnet\",\"tokens_per_sec\":385802469135802500}}}"}' --depositYocto 1 --gas 200000000000000 --accountId yourname.testnet
```
Now let’s try to pause our stream ([response](https://explorer.testnet.near.org/transactions/GWoSGiCwioMbjwegNp9N5EESYHzsjRSgjg4TnVTzdL7w))
```bash
near call streaming-roketo.dcversus.testnet pause_stream '{"stream_id": "CUr4BNXQgXqPWCJVvxu5v6jarGnV8y9s6iVWTK2g6fkt"}'  --depositYocto 1 --gas 200000000000000 --accountId yourname.testnet
```
You are awesome <3

### near-api-js
Start from the installation of the package and connecting to the wallet ([see quick reference](https://github.com/near/near-api-js/blob/master/examples/quick-reference.md)). After login we are able to create stream  ([see signature](#create)):
```ts
const ftContract = new Contract(account, 'wrap.testnet', {
    changeMethods: ['ft_transfer_call', 'near_deposit'],
});

// only for NEAR, we need deposit NEAR on wrap 
await ftContract.near_deposit({}, 200000000000000, 1000000000000000000000000);

await ftContract.ft_transfer_call({
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
```

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
- **Initialized** - stream is created with `is_auto_start_enabled` = false, tokens are not sending until the stream will be started
- **Active** - tokens are in process of streaming
- **Paused** - streamed tokens are withdrawn to receiver and sending paused
- **Finished** - stream is over. Receiver is able to withdraw the balance.

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
    // otherwise only receiver can do it.
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
Several view methods of the Roketo’s contract.

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
Return list of incoming (or outgoing) streams to account. Request:
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
Return `[Token, TokenStats]` (see [Token](#token), [TokenStats](#tokenstats)) and request:
```json
{
    "token_account_id": "string"
}
```
#### `get_account_ft`
Response
account token stats (numbers) `[total_incoming, total_outgoing, total_received]`, request: 
```json
{
    "account_id": "AccountId",
    "token_account_id": "AccountId"
}
```

## Roketo calls

Modifying methods of contract requires a deposit. Some of methods should be called through NEP-141 FT [(ft_on_transfer)](#ftontransfer)

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
The action will create users and stream with the transferred payload. The deposit attached will be a transfering amount (commission will be deducted automatically).

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

Starts initialized or paused stream. It might be executed only by the owner of the stream in case of initialization or the receiver too (if stream was paused). Expects one yocto as deposit. Signature:
```json
{
    "stream_id": "StreamId"
}
```

#### `pause_stream`
Pauses the stream and transfer streamed tokens to the receiver. Stream can be paused only if it was in the active state. Might be executed only by the owner or the receiver of the stream. Expects one yocto as deposit. Signature:
```json
{
    "stream_id": "StreamId"
}
```

#### `stop_stream`
Finishing the stream. Finished streams can’t be restarted. All remaining amount of deposit go’s back to the owner — all streamed deposit will be sent to receiver. Can be executed only by the owner of the stream. Expects one yocto as deposit. Signature:
```json
{
    "stream_id": "StreamId"
}
```

#### `withdraw`
Transfer streamed tokens to the receiver. If stream deposit was streamed, then the stream will finish. Can be executed only by the receiver of the stream. Expects one yocto as deposit Signature: 
```json
{
    "stream_ids": ["StreamId"]
}
```

### Other calls
These methods are not essential for the functioning of the main task of the contract, but may be useful

#### `change_receiver`
sets a new receiver for the stream. Can be executed only by the receiver of the stream. Expects `storage_balance_needed` from token as deposit. Call signature:
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
