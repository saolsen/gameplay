# Gameplay

Programs that play games!
Game play is a collection of games (just connect4 so far) and agents that play
those games. You can develop your own agent(s) to play the game(s) and battle
other agents.
Eventually we'll add more games and set up tournaments and all kinds of fun stuff.

## Getting Started

### Dependencies

Install Rust (see [rustup](https://rustup.rs/) for installation) and
Docker (see [docker](https://www.docker.com)) and clone the repo.

### Build

`cargo build --release`

### Play connect4 locally (against yourself)

`target/release/gameplay connect4 play`

### Play against a local agent (best for developing an agent)

`target/release/gameplay connect4 play --player1-url http://localhost:8000`

You can also have your local agent play itself.

`target/release/gameplay connect4 play --player0-url http://localhost:8000 --player1-url http://localhost:8000`

### Play against other agents

Agents can be written in any language, so they all have their own dependencies.
To avoid everyone having to install every agent's dependencies each agent has
a dockerfile that runs it. (See [how agents work](#how-agents-work))

Build all the agents.

`docker compose build`

Then you can play against them! For example here is how you could play against
my [mcts agent](https://www.steveindusteves.com/p/connect4-mcts)

`docker compose run gameplay gameplay connect4 --player1-url http://saolsen_connect4_mcts`

Or to have two agents play eachother. In this case my rand agent vs my mcts agent.

`docker compose run gameplay gameplay connect4 --player0-url http://saolsen_connect4_rand --player1-url http://saolsen_connect4_mcts`

## How agents work

The interface for an agent is an HTTP endpoint.

On the agents turn, it gets POSTed a JSON version of the gamestate and
must respond with a JSON version of the action it wishes to take.
This interface was chosen for maximum compatibility. All programming languages
can speak http and handle json (or at least have a library you can use)
so you are welcome to write agents in any language you want!

You can develop your agent locally and play against it yourself or try it
against other builtin agents. Once your agent works you can add it to the repo
so that other people can play against it, and it can play in tournaments.

This is where docker comes in. Since agents can be written in any language docker
lets each agent manage whatever dependencies they need.

If you can make an http service that speaks json you can write an agent. And if
you can make a docker container that runs it you can add it to the repo.

## Writing an agent

Create a new directory for yourself in
`agents/your_github_username/connect4/your_agent_name`.
Then treat that as the root of your project and set up whatever you need to.
You must create a web service with a single endpoint that accepts a POST request.
This endpoint will be what is passed in as `--playerN-url` and will be hit
on every turn.

The specific request depends on the game. Right now the only game is connect4 so
the json will look like this.

```
{
    "board": [
        null,null,null,null,null,null,
        null,null,null,null,null,null,
        1   ,null,null,null,null,null,
        0   ,1   ,null,null,null,null,
        null,null,null,null,null,null,
        null,null,null,null,null,null,
        null,null,null,null,null,null
    ],
    "next_player": 0
}
```

The board is a single array. Each slot is a space in the grid. If it is `null`
then that slot is empty. Otherwise, if it is `0`, player 1 (blue) has a chip there.

If it's `1` player 2 (red) has a chip there.
Each 6 elements are a column, starting from the bottom and growing towards the top.
You can imagine taking a picture of the array above and turning it 90 degrees to the left
and that is what the connect4 board would look like.

If you think of the column as 0-6 (7 columns) and each row as 0-5 (6 rows) where
row 0 is the bottom row, row 5 is the top row, column 0 is the leftmost column
and column 6 is the rightmost column. Then you would index the array as
`board[col * 6 + row]`

The `next_player` is the index of the player whose turn it is.

Then you must reply with a json action that looks like this. It is the column
that you wish to drop your chip into (0-6).

```
{"column": 3}
```

There are also a number of headers that are passed with the request.

* `Gameplay-Game` says which game is being played. Right now it will always be `connect4`.
* `Gameplay-Match-ID` is a unique id for this match. Since an agent service could be
playing multiple games at once this lets you keep track of which game is which so if you
have any internal state you can keep it separate.
* `Gameplay-Player` is the index of the player that you are playing as. `0` or `1`.
* `Gameplay-Match-Status` is the status of the match. It will be `InProgress`
is still going or `Over` if it is over. This final `Over` request with the final state of
the game lets the agent know the match is over, so it can clean up any state it has.

You can run your service locally and test it by passing its url as either
`--player0-url` or `--player1-url` (or both). For example.

`target/release/gameplay connect4 play --player1-url http://localhost:8000`

## Packaging an agent

To make the agent easy for everyone to play against without having to install
all it's dependencies we need to make a docker container for it. Look at
`docker/saolsen_connect4_rand.Dockerfile` for an example. Then there needs to
be an entry for it in compose.yaml. This isn't really super easy right now
unless you are familiar with docker so come ask in discord if you have questions.

## TODO

This is still in a very early state and here's a rough list of things I want to
do to make it easier.

### Tournaments

Competitions where all the agents play each-other, and we can see which ones are
the best. Maybe we can get some sponsors and have prizes.

### Language libraries

I'd like to make some libraries for common languages that handle the http and
json parts of an agent, so you can easily write one as just a single function.
This will be a far easier way to make agents for supported languages, we'll
handle all the docker parts too. Should make it way easier for people.

### HTTP level test suite for agents.

An easy-to-use test command, eg `gameplay connect4 test http://localhost:8000`
that can be used during development to help agent authors.

### CI tests

Want to allow anyone to add agents but also want to make sure they all keep
working.

## Links

No website yet. Coming soon (hopefully with tournaments).

https://discord.gg/3c9w2AqygD
