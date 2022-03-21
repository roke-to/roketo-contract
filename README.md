# Roke.to contract [tmp]

Direct X-to-Y tokens streaming with no 3rd parties. Forget about paydays and boost control of your payment flows on a new level.

For a business — truly automatize payrolls with repetitive actions needed, give grants to projects smoothy without step-by-step payouts, send money easily to community members for each repeatable task. Now you can deal with it without any hassle.

For a freelancer, artist, community member, new project, or any other earner — own your funds at any moment. Don't be deluded with payroll, all your finances become predictable and accessible. Withdraw funds at any moment without locks, penalties and fees.

Developed by [Kikimora Labs](https://kikimora.ch/).

# Repository structure

- [streaming](#streaming)
- [finance](#finance)
- [examples](#examples)
- [tests](#tests)

## Streaming

All source code is located at `streaming/src` folder. Files `calls.rs` and `views.rs` are about interacting with the contract.
Other files contains of helpers and primitives.

# Build

Run `sh build.sh`. It will put the compiled contracts into `res` folder named as `*.wasm`.

# Tests

Run `sh test.sh`. It will build the contract followed by running simulation tests from `tests/main.rs`.

# Links

- [Landing](https://www.roke.to/) (webflow, outside repo)
- [dApp](test.app-v2.roke.to) (testnet)
- [Docs](https://www.notion.so/kikimora-labs/Roketo-2056455fdcf4452f9e690601cc49d7a4)
- [API docs](/streaming/README.md)
