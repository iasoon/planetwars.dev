# planetwars-client

`planetwars-client` can be used to play a match with your bot running on your own machine.

## Usage

First, create a config `mybot.toml`:

```toml
# Comand to run when starting the bot.
# Argv style also supported: ["python", "simplebot.py"]
command = "python simplebot.py"

# Directory in which to run the command.
# It is recommended to use an absolute path here.
working_directory = "/home/user/simplebot"
```

Then play a match:  `planetwars-client /path/to/mybot.toml opponent_name`

## Building
- Obtain rust compiler through https://rustup.rs/ or your package manager
- Checkout this repository
- Run `cargo install --path .` in the `planetwars-client` directory