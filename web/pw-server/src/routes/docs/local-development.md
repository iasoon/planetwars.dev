# Local development

Besides using the web editor, it is also possible to develop a bot in your own development environment.

Using the `planetwars-client` you can play matches remotely, with your bot running on your computer.
This is similar to using the "Play" button in the web editor.

You can then submit your bot to the server as a docker container.

This way, you can author bots in any language or tool you want - as long as you can dockerize it.

## Running your bot locally

You can use the `planetwars-client` to play matches locally.

Currently, no binaries are available, so you'll have to build the client from source.

### Building the binary

If you do not have a rust compiler installed already, obtain one through https://rustup.rs/.

1. Clone the repository:  
   `git clone https://github.com/iasoon/planetwars.dev.git`
2. Build and install the client:  
   `cargo install --path planetwars.dev/planetwars-client`

### Create a bot config

The bot config file specifies how to run your bot. Create a file `mybot.toml` with contents like so:

```toml
# Comand to run when starting the bot.
# Argv style also supported: ["python", "simplebot.py"]
command = "python simplebot.py"

# Directory in which to run the command.
# It is recommended to use an absolute path here.
working_directory = "/home/user/simplebot"
```

### Playing a match

Run `planetwars-client path/to/mybot.toml opponent_name`

Try `planetwars-client --help` for more options.

## Publishing your bot as a docker container

Once you are happy with your bot, you can push it to the planetwars server as a docker container.

First, we will containerize our bot.

### Containerizing your bot

Our project directory looks like this:

```
simplebot/
├── Dockerfile
└── simplebot.py
```

We used this basic dockerfile. You can reuse this for simple python-based bots.

```Dockerfile
FROM python:3.10.1-slim-buster
WORKDIR /app
COPY simplebot.py simplebot.py
CMD python simplebot.py
```

Refer to https://docs.docker.com for guides on how to write your own dockerfile.

In the directory that contains your `Dockerfile`, run the following command:

```bash
docker build -t my-bot-name .
```

If all went well, your docker daemon now holds a container tagged as `my-bot-name`.

### Publishing the bot

1. **Create a bot**:  
   Before you can publish your container, you will first need to create a bot on planetwars.dev.  
   You can create a new bot by clicking the "New bot" button on your user profile page.  
   If you have an existing bot that you wish to overwrite, you can use that instead.
2. **Log in to the planetwars docker registry**:  
   `docker login registry.planetwars.dev`  
   Authenticate using your planetwars.dev credentials.
3. **Tag your bot**:  
   `docker tag my-bot-name registry.planetwars.dev/my-bot-name`
4. **Push your bot**:  
   `docker push registry.planetwars.dev/my-bot-name`  
   This will upload the container to planetwars.dev, and automatically create a new bot version.

That was it! If all went well, you should be able to see the new version on your bot page.
