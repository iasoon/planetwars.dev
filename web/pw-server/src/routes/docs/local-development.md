## Local development

Besides using the web editor, it is also possible to develop a bot in your own development environment.  
Using the `planetwars-client` you can play test matches remotely, with your bot running on your computer.  
You can then submit your bot to the server as a docker container.  

### Playing matches with a local bot

You can use the `planetwars-client` to play matches locally.

Currently, no binaries are available, so you'll have to build the client from source.

#### Building the binary
If you do not have a rust compiler installed already, obtain one through https://rustup.rs/.
1. Clone the repository:  
   `git clone https://github.com/iasoon/planetwars.dev.git`
2. Build and install the client:  
   `cargo install --path planetwars.dev/planetwars-client`

#### Create a bot config
The bot config file specifies how to run your bot. Create a file `mybot.toml` with contents like so:

```toml
# Comand to run when starting the bot.
# Argv style also supported: ["python", "simplebot.py"]
command = "python simplebot.py"

# Directory in which to run the command.
# It is recommended to use an absolute path here.
working_directory = "/home/user/simplebot"
```

#### Playing a match
Run `planetwars-client /path/to/mybot.toml opponent_name`

Try `planetwars-client --help` for more options.


### Publishing your bot as a docker container
Once you are happy with your bot, you can push it to the planetwars server as a docker container.

1. **Create a bot.**   
   Before you can publish a new bot version, you will first need a registered bot name.
   You can use an existing name, or you can create one by using the "New bot" button on your user profile page (you can get there by clicking your name in the navbar).
2. Log in to the planetwars docker registry:  
   `docker login registry.planetwars.dev`  
3. Tag and push your bot to `registry.planetwars.dev/my-bot-name:latest`.
4. Your bot should be up and running now! Feel free to launch a game against it to test whether all is working well.
   Shortly, your bot should show up in the rankings.
