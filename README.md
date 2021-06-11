# Idavoll Network

Idavoll Network is an decentralized organization platform that provides infrastructure and services to users of the Idavoll Network and Polkadot ParaChains.

## Getting Started

### Rust Setup

Setup instructions for working with the [Rust](https://www.rust-lang.org/) programming language can
be found at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started). Follow those
steps to install [`rustup`](https://rustup.rs/) and configure the Rust toolchain to default to the
latest stable version.

### Makefile

This project uses a [Makefile](Makefile) to document helpful commands and make it easier to execute
them. Get started by running these [`make`](https://www.gnu.org/software/make/manual/make.html)
targets:

1. `make init` - Run the [init script](scripts/init.sh) to configure the Rust toolchain for
   [WebAssembly compilation](https://substrate.dev/docs/en/knowledgebase/getting-started/#webassembly-compilation).
1. `make run` - Build and launch this project in development mode.

The init script and Makefile both specify the version of the
[Rust nightly compiler](https://substrate.dev/docs/en/knowledgebase/getting-started/#rust-nightly-toolchain)
that this project depends on.

### Build

The `make run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
make build
```
or you and `cargo build` or `cargo build --release` to build it. and you can run `cargo test` to run the tests.
```
 cargo build 
 or 
 cargo test
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/idavoll-node -h
```

## Run

The `make run` command will launch a temporary node and its state will be discarded after you
terminate the process. After the project has been built, there are other ways to launch the node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/idavoll-node --dev
```

Purge the development chain's state:

```bash
./target/release/idavoll-node purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/idavoll-node -lruntime=debug --dev
```

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).




## Usage
A simple way to use IDOVALL-NETWORK to create and use DAO organization,you can run local node for use it with `./target/release/idavoll-node --dev --tmp --ws-external`, and use the [Polkadot JS UI](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer), you may need the [types](https://github.com/idavollnetwork/idavoll/blob/main/types.json) with the UI.then we can create a `DAO` organization and make it usefull.

```
1. create organization
2. add members into organization
3. deposit local asset
4. create proposal
5. vote to decision
6. view the result
```


### Create Organization
we can create organization with the inherent user `alice` and submit an extrinsicz with `idavoll.create_organization` function.
1. `origin`: the owner of the organization,on this,it's `alice`.
2. `total`: the issuance of the new token, When a user creates an organization, a new token is automatically created for voting.
3. `info`: the details of the new organization,we can use the default value of `OrgInfo`.

### Add Members and assign the token
There is a simple way to add member to `DAO` organization,submit an extrinsicz with `idavoll.add_member_and_assign_token` function,In fact, all members of the organization have the right to add members and assign token to the new member, not just the rights that are unique to the owner of the organization. If a member of the organization wants to participate in the voting of proposals in the organization, it needs to have the unique token of the organization. Created when the organization is created, the token needs to be distributed by the owner or distributed by other members who own the token.

1. `origin`: the owner(`alice`) of the organization or other member in the organization.
2. `target`: the new account.
3. `id`: Ordinal number created by the organization，it mapped whit the organization id.
4. `assigned_value`: the amount of the token,the `origin`(`alice`) will transfer token to the new account.

### Deposit
After the organization is created, any member can deposit assets(Local asset[`IDV`]) to the organization as an organization asset, submit an extrinsicz with `idavoll.deposit_to_organization` function.

1. `origin`: the owner(`alice`) of the organization or other member in the organization.
2. `id`: Ordinal number created by the organization，it mapped whit the organization id.
3. `value`: the amount of the local asset(IDV).

### Create Proposal
Now we can submit an extrinsicz with `idavoll.create_proposal` to create proposals to spend the assets of the organization,then all members in the organization can voting on the proposal.

1. `origin`: any member in the organization.
2. `id`: Ordinal number created by the organization，it mapped whit the organization id.
3. `length`: the block number(length) as the proposal lift time, if the current block number more than the `length`, than the proposal is expired.
4. `sub_param`: the vote rule, it was satisfied with the organization's rule.
5. `Call`: `Call::IdavollModule(IdavallCall::transfer(RECEIVER.clone(),value))` like [this](https://github.com/idavollnetwork/idavoll/blob/main/pallets/idavoll/src/mock.rs#L150)

### Vote
We can use `idavoll.vote_proposal` to participate in the voting of the proposal and process the result of the vote, all members in the organization can voting on proposal with the token values.

1. `origin`: any member in the organization.
2. `pid`: the proposal id of the proposal return by create_proposal.
3. `value`: the weight of vote power,it is the token amount of the token in the organization.
4. `yesorno`: the user approve or against the proposal(`yes` or `no`).

### Result
Finally, after a proposal has been voted and passed, the content of the proposal will be automatically processed (that is, the call of `Call` in the proposal is executed), if the proposal is not passed, it will be closed, and the execution result can be directly viewed after the proposal is passed. (Such as `Balance::free_balance`).


