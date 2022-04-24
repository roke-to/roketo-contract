# Contract documentation 

## How to use
The rocket contract is a regular Near contract. You need to have a general understanding of working with NEAR before. Suggested to read [nomicon](https://nomicon.io).

### roketo-sdk
> Todo

### near-api-js


### cli


## Views
View methods of contract.

### `get_stream`
Returns the data about the stream.

### `get_account`
Returns the account requested. Props: 
```json
{
    "account_id": "AccountId"
}
```
Response
```json
{
    "active_incoming_streams": "number",
    "active_outgoing_streams": "number",
    "inactive_incoming_streams": "number",
    "inactive_outgoing_streams": "number",

    "total_incoming": { "AccountId": "number" },
    "total_outgoing": { "AccountId": "number" },
    "total_received": { "AccountId": "number" },

    "deposit": "string",

    "stake": "string",

    "last_created_stream": "StreamId",
    "is_cron_allowed": "boolean"
}
```

### `get_account_incoming_streams`

### `get_account_outgoing_streams`

### Other views

- `get_stats`
- `get_dao`
- `get_token`
- `get_account_ft`

## Calls

Modifying methods of contract, require a deposit. Some methods should be called through NEP-141 FT with [ft_on_transfer](#ft_on_transfer) its a [Create](#create), [Push](#push), [Deposit](#deposit), [Stake](#stake). Main contract methods: [start_stream](#start_stream), [pause_stream](#pause_stream), [stop_stream](#stop_stream), [withdraw](#withdraw) and [other calls](#other-calls), [dao calls](#dao-calls), [oracle calls](#oracle-calls)

### `ft_on_transfer`
**Don't make this call directly.** You should call `ft_transfer_call` at any compatible NEP-141 FT with deposit and payload:
```jsonc
{
    "receiver_id": "ROKETO_ACCOUNT_ID",
    "memo": "string", // Can be any string
    "msg": "TRANSFER_PAYLOAD" // Different for call types, stringified json
}
```

Where `ROKETO_ACCOUNT_ID` is a `streaming-roketo.dcversus.testnet` in testnet and `todo` in mainnet. `TRANSFER_PAYLOAD` are different for actions. Details below.

#### `Create`
The action will create users and stream with the transferred payload. Attached deposit will be a transfering amount (commission will be deducted). **You should call `Push` through `ft_on_transfer` for work!**

```jsonc
{
    "Create": {
        "request": {
            "owner_id": "AccountId",
            "receiver_id": "AccountId",
            "tokens_per_sec": "string", 
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

### `start_stream` 

Starts initialized or paused stream. Can be executed only by the owner of the stream in case of initialization or the receiver too if stream was paused. Expects one yocto as deposit[^1]. Signature: 
```json
{
    "stream_id": "StreamId"
}
```

### `pause_stream`
Pauses the stream and transfer streamed tokens to the reciever. Stream can be paused only if it was in the active state. Can be executed only by the owner or the receiver of the stream. Expects one yocto as deposit[^1]. Signature: 
```json
{
    "stream_id": "StreamId"
}
```

### `stop_stream`
Finishing the stream. Finished streams cannot be restarted. All remaining deposit sends back to the owner, all streamed deposit will be send to reciever. Can be executed only by the owner of the stream. Expects one yocto as deposit[^1]. Signature: 
```json
{
    "stream_id": "StreamId"
}
```

### `withdraw`
Transfer streamed tokens to the reciever. If stream deposit was streamed, then the stream will finish. Can be executed only by the receiver of the stream. Expects one yocto as deposit[^1] Signature: 
```json
{
    "stream_ids": ["StreamId"]
}
```

### Other calls
These methods are not essential for the functioning of the main task of the contract, but may be useful

- `change_receiver` sets a new reciever for the stream. Can be executed only by the receiver of the stream. Expects `storage_balance_needed` from token as deposit[^2]. Call signature:
```jsonc
{
    "stream_id": "StreamId",
    "receiver_id": "AccountId" // New receiver 
}
```
- `account_update_cron_flag` update user property `is_cron_allowed`. If true anyone can call `withdraw` for income streams, otherwise only reciever can do it. Property and call are needed for internal purposes. Expects one yocto as deposit[^1]. Signature:
```json
{
    "is_cron_allowed": "boolean"
}
```
- `account_unstake` send staked `utility_token` to your account. Expects one yocto as deposit[^1].
```json
{
    "amount": "string"
}
```
- `account_deposit_near` add a near deposit to your account. No props, need only attached deposit[^1].

### Dao calls
Methods can be executed only by dao account. 

- `dao_update_token` add or update token configration.[^3]
```jsonc
{
    "token": {
        "account_id": "AccountId", // token account id
        "is_listed": "boolean", // if true we apply commission otherwise apply commission_unlisted
        "commission_on_create": "number", // taken in current fts in case of listed token
        "commission_coef": "SafeFloat", // percentage of tokens taken for commission
        "storage_balance_needed": "string", 
        "gas_for_ft_transfer": "Gas", // gas settings for UI purposes
        "gas_for_storage_deposit": "Gas", // gas settings for UI purposes
    }
}
```

- `dao_change_owner` sets a new dao account
```json
{
    "new_dao_id": "AccountId"
}
```
- `dao_update_commission_unlisted` sets a commission for unlisted tokens
```json
{
    "commission_unlisted": "number"
}
```
- `dao_withdraw_ft` withdraw collected FT commission
```json
{

    "token_account_id": "AccountId",
    "receiver_id": "AccountId",
    "amount": "number",
}
```
- `dao_withdraw_near` withdraw collected NEAR commission
```json
{

    "receiver_id": "AccountId",
    "amount": "number",
}
```
- `dao_add_oracle` add new oracle
```json
{
    "new_oracle_id": "AccountId"
}
```
- `dao_remove_oracle` remove oracle from list
```json
{
    "oracle_id": "AccountId"
}
```

### Oracle calls
The oracle is an external contract that we register as Dao (see [dao calls](#dao-calls)). Oracle purpose is a update `commission_on_create` for tokens.

- `oracle_update_commission_on_create` update `commission_on_create` for specified token.
```json
{
    "token_account_id": "AccountId",
    "commission_on_create": "number",
}
```
- `oracle_update_eth_near_ratio` update dao `eth_near_ratio` (used only for Aurora calls)


[^1]: one yocto
[^2]: storage_balance_needed
[^3]: token listing
